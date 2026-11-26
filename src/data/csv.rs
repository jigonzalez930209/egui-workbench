//! CSV and TSV parsing into a typed dataset.

use std::path::Path;

use super::column::Column;
use super::dataset::Dataset;
use super::source::{ColumnSchema, DataError, DataSource};

/// CSV options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CsvOptions {
    /// Field delimiter.
    pub delimiter: u8,
    /// Treat the first row as the column names.
    pub header: bool,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            header: true,
        }
    }
}

impl CsvOptions {
    /// Comma-separated.
    pub fn csv() -> Self {
        Self::default()
    }

    /// Tab-separated.
    pub fn tsv() -> Self {
        Self {
            delimiter: b'\t',
            ..Self::default()
        }
    }
}

/// A parsed CSV/TSV source.
#[derive(Clone, Debug, PartialEq)]
pub struct CsvSource {
    name: String,
    data: Dataset,
    schema: Vec<ColumnSchema>,
}

impl CsvSource {
    /// Parse UTF-8 delimited text.
    ///
    /// # Errors
    /// When the text has no rows or a row has more fields than the header.
    pub fn parse(name: impl Into<String>, text: &str, options: CsvOptions) -> Result<Self, DataError> {
        let name = name.into();
        let data = parse_csv(&name, text.as_bytes(), options)?;
        let schema = data.columns().iter().map(ColumnSchema::from_column).collect();
        Ok(Self { name, data, schema })
    }

    /// Open a file from disk.
    ///
    /// # Errors
    /// When the file cannot be read or parsed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = path.as_ref();
        let bytes = super::source::read_limited(path)?;
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("data.csv");
        let options = if name.ends_with(".tsv") || name.ends_with(".tab") {
            CsvOptions::tsv()
        } else {
            CsvOptions::csv()
        };
        let data = parse_csv(name, &bytes, options)?;
        let schema = data.columns().iter().map(ColumnSchema::from_column).collect();
        Ok(Self {
            name: name.to_owned(),
            data,
            schema,
        })
    }

    /// The file name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The parsed data.
    pub fn data(&self) -> &Dataset {
        &self.data
    }
}

impl DataSource for CsvSource {
    fn schema(&self) -> &[ColumnSchema] {
        &self.schema
    }

    fn row_count(&self) -> usize {
        self.data.row_count()
    }

    fn read_range(&self, range: std::ops::Range<usize>) -> Result<Dataset, DataError> {
        super::source::check_range(&range, self.row_count())?;
        Ok(self.data.slice(range))
    }
}

/// Parse bytes as a delimited table, inferring int, float, bool, and text columns.
///
/// # Errors
/// When there is no header and no rows, or a row has more fields than the header.
pub fn parse_csv(name: &str, bytes: &[u8], options: CsvOptions) -> Result<Dataset, DataError> {
    let text = std::str::from_utf8(bytes).map_err(|error| DataError::Connector(error.to_string()))?;
    // A byte-order mark would stay glued to the first header name.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut rows = parse_rows(text, options.delimiter);
    if rows.is_empty() {
        return Ok(Dataset::new().with_name(name));
    }
    let headers = if options.header {
        rows.remove(0)
    } else {
        (0..rows[0].len()).map(|index| format!("column_{index}")).collect()
    };
    if rows.iter().any(|row| row.len() > headers.len()) {
        return Err(DataError::Connector("a row has more fields than the header".to_owned()));
    }
    let mut columns = Vec::with_capacity(headers.len());
    for (index, header) in headers.iter().enumerate() {
        let values: Vec<&str> = rows
            .iter()
            .map(|row| row.get(index).map_or("", String::as_str))
            .collect();
        columns.push(infer_column(header, &values));
    }
    let mut dataset = Dataset::from_columns(columns);
    dataset.set_metadata("source", name);
    Ok(dataset)
}

trait NamedDataset {
    fn with_name(self, name: &str) -> Self;
}

impl NamedDataset for Dataset {
    fn with_name(mut self, name: &str) -> Self {
        self.set_metadata("source", name);
        self
    }
}

fn infer_column(name: &str, values: &[&str]) -> Column {
    let non_empty: Vec<&str> = values.iter().copied().filter(|value| !value.is_empty()).collect();
    if non_empty.is_empty() {
        return Column::text(name, values.to_vec()).with_validity(values.iter().map(|value| !value.is_empty()));
    }
    if non_empty
        .iter()
        .all(|value| value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false"))
    {
        return Column::bools(
            name,
            values
                .iter()
                .map(|value| value.eq_ignore_ascii_case("true"))
                .collect::<Vec<_>>(),
        )
        .with_validity(values.iter().map(|value| !value.is_empty()));
    }
    if non_empty.iter().all(|value| value.parse::<i64>().is_ok()) {
        return Column::ints(
            name,
            values
                .iter()
                .map(|value| value.parse::<i64>().unwrap_or(0))
                .collect::<Vec<_>>(),
        )
        .with_validity(values.iter().map(|value| !value.is_empty()));
    }
    if non_empty.iter().all(|value| value.parse::<f64>().is_ok()) {
        return Column::floats(
            name,
            values
                .iter()
                .map(|value| value.parse::<f64>().unwrap_or(f64::NAN))
                .collect::<Vec<_>>(),
        )
        .with_validity(values.iter().map(|value| !value.is_empty()));
    }
    Column::text(name, values.to_vec()).with_validity(values.iter().map(|value| !value.is_empty()))
}

fn parse_rows(text: &str, delimiter: u8) -> Vec<Vec<String>> {
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
            character if character == delimiter => row.push(std::mem::take(&mut field)),
            // A blank line is not a row of one empty field.
            '\n' if row.is_empty() && field.is_empty() => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            '\r' => {}
            _ => field.push(character),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::CellValue;

    #[test]
    fn csv_infers_types_and_handles_quotes() {
        let text = "name,voltage,cycle,ok\n\"coil, a\",1.5,2,true\ncoil b,2.0,3,false\n";
        let source = CsvSource::parse("test.csv", text, CsvOptions::csv()).expect("csv");
        assert_eq!(source.row_count(), 2);
        assert_eq!(source.schema()[0].kind, crate::data::ColumnType::Text);
        assert_eq!(source.schema()[1].kind, crate::data::ColumnType::Float);
        assert_eq!(source.schema()[2].kind, crate::data::ColumnType::Int);
        assert_eq!(source.schema()[3].kind, crate::data::ColumnType::Bool);
        let page = source.read_range(0..2).expect("page");
        assert_eq!(page.cell(0, 0), CellValue::Text("coil, a"));
        assert_eq!(page.cell(1, 1), CellValue::Float(2.0));
    }

    #[test]
    fn tsv_and_empty_cells() {
        let source = CsvSource::parse("test.tsv", "a\tb\n1\t\n2\tx\n", CsvOptions::tsv()).expect("tsv");
        assert_eq!(source.row_count(), 2);
        assert_eq!(source.data().cell(0, 1), CellValue::Null);
        assert_eq!(source.data().cell(1, 1), CellValue::Text("x"));
    }
}
