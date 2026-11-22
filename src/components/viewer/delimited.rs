//! RFC 4180 delimited text: quoted fields, doubled quotes, embedded newlines, and CRLF.

use super::table::{Cell, CellKind};

/// Parse `text` with `delimiter` into typed cells.
pub(super) fn parse(text: &str, delimiter: u8) -> Vec<Vec<Cell>> {
    let delimiter = char::from(delimiter);
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if quoted {
            match character {
                '"' if chars.peek() == Some(&'"') => {
                    chars.next();
                    field.push('"');
                }
                '"' => quoted = false,
                _ => field.push(character),
            }
            continue;
        }
        match character {
            '"' if field.is_empty() => quoted = true,
            c if c == delimiter => {
                row.push(cell(&field));
                field.clear();
            }
            '\n' => {
                row.push(cell(&field));
                rows.push(std::mem::take(&mut row));
                field.clear();
            }
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                row.push(cell(&field));
                rows.push(std::mem::take(&mut row));
                field.clear();
            }
            _ => field.push(character),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(cell(&field));
        rows.push(row);
    }
    // A trailing newline is a row terminator, not an empty last row.
    while rows.last().is_some_and(|row| row.len() == 1 && row[0].is_empty()) {
        rows.pop();
    }
    rows
}

fn cell(text: &str) -> Cell {
    if text.is_empty() {
        return Cell::empty();
    }
    let trimmed = text.trim();
    if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("false") {
        return Cell::boolean(trimmed.eq_ignore_ascii_case("true"));
    }
    if trimmed.parse::<f64>().is_ok() {
        return Cell {
            text: text.to_owned(),
            kind: CellKind::Number,
        };
    }
    Cell::text(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(rows: &[Vec<Cell>]) -> Vec<Vec<&str>> {
        rows.iter()
            .map(|row| row.iter().map(|cell| cell.text.as_str()).collect())
            .collect()
    }

    #[test]
    fn quoted_fields_keep_delimiters_and_newlines() {
        let rows = parse("name,note\n\"Ada, A.\",\"line one\nline two\"\n", b',');
        assert_eq!(texts(&rows), [["name", "note"], ["Ada, A.", "line one\nline two"],]);
    }

    #[test]
    fn doubled_quotes_are_unescaped() {
        let rows = parse("\"He said \"\"hi\"\"\",1\r\n", b',');
        assert_eq!(texts(&rows), [["He said \"hi\"", "1"]]);
    }

    #[test]
    fn tsv_and_numbers_are_typed() {
        let rows = parse("x\ty\n1.5\t-2\n", b'\t');
        assert_eq!(rows[1][0].kind, CellKind::Number);
        assert_eq!(rows[1][1].kind, CellKind::Number);
        assert_eq!(rows[1][1].text, "-2");
    }

    #[test]
    fn blank_lines_do_not_create_rows() {
        let rows = parse("a,b\n\n,\n", b',');
        assert_eq!(rows.len(), 3);
    }
}
