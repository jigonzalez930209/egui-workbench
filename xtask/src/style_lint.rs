//! `cargo xtask style-lint`: no literal radii or font sizes outside the style module.
//!
//! Every component reads its shape and type scale from `egui_workbench::style`, so a
//! custom `WorkbenchStyle` reaches every pixel. A line that really needs a fixed value
//! (a 2 pt indicator, a value the user configured) ends with `// style: fixed`.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Files that own the literal values.
const EXEMPT: [&str; 2] = ["src/foundation/style.rs", "src/components/plot/scene.rs"];

/// Calls whose first argument must not be a non-zero number literal.
const FIRST_ARGUMENT: [&str; 5] = [
    "CornerRadius::same(",
    ".corner_radius(",
    "FontId::proportional(",
    "FontId::monospace(",
    "FontId::new(",
];

/// Painter calls whose second argument (the radius) must not be a non-zero literal.
const SECOND_ARGUMENT: [&str; 3] = [".rect_filled(", ".rect_stroke(", ".rect("];

/// Scan `root/src` and return every violation as `path:line: message`.
pub fn violations(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files = Vec::new();
    collect(&root.join("src"), &mut files)?;
    files.sort();
    let mut found = Vec::new();
    for file in files {
        let relative = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        if EXEMPT.contains(&relative.as_str()) {
            continue;
        }
        let text = fs::read_to_string(&file)?;
        for (line, message) in check_source(&text) {
            found.push(format!("{relative}:{line}: {message}"));
        }
    }
    Ok(found)
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

/// Every violation in a source file as `(line, message)`. Calls may span lines;
/// comments, `// style: fixed` lines, and the test module are skipped.
fn check_source(text: &str) -> Vec<(usize, String)> {
    // Blank out comments and exempt lines (keeping the newlines) so offsets map to lines.
    let mut code = String::with_capacity(text.len());
    for line in text.lines() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        let skip = line.trim_start().starts_with("//") || line.ends_with("// style: fixed");
        if !skip {
            code.push_str(line.split(" // ").next().unwrap_or(line));
        }
        code.push('\n');
    }
    let line_of = |offset: usize| code[..offset].matches('\n').count() + 1;
    let mut found = Vec::new();
    for call in FIRST_ARGUMENT {
        for (start, _) in code.match_indices(call) {
            if nonzero_literal(code[start + call.len()..].trim_start()) {
                found.push((
                    line_of(start),
                    format!("literal in `{call}…)`: read it from `style::get(ctx)`"),
                ));
            }
        }
    }
    for call in SECOND_ARGUMENT {
        for (start, _) in code.match_indices(call) {
            let rest = &code[start + call.len()..];
            if let Some(comma) = top_level_comma(rest)
                && nonzero_literal(rest[comma + 1..].trim_start())
            {
                found.push((
                    line_of(start),
                    format!("literal radius in `{call}…)`: use the style's radii"),
                ));
            }
        }
    }
    for (start, _) in code.match_indices("RichText::new(") {
        let statement = &code[start..];
        let end = statement.find([';', '{']).unwrap_or(statement.len());
        for (size, _) in statement[..end].match_indices(".size(") {
            if nonzero_literal(statement[size + ".size(".len()..].trim_start()) {
                found.push((
                    line_of(start + size),
                    "literal text size: use `style::get(ctx).font_size(…)`".to_owned(),
                ));
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Whether `text` starts with a number literal other than zero.
fn nonzero_literal(text: &str) -> bool {
    let number: String = text
        .chars()
        .take_while(|character| character.is_ascii_digit() || *character == '.' || *character == '_')
        .collect();
    let rest = &text[number.len()..];
    let ends = rest.starts_with(')') || rest.starts_with(',') || rest.starts_with("f32") || rest.starts_with("u8");
    !number.is_empty()
        && ends
        && !number
            .trim_matches(|character| character == '0' || character == '.' || character == '_')
            .is_empty()
}

/// The byte offset of the first comma outside parentheses.
fn top_level_comma(text: &str) -> Option<usize> {
    let mut depth = 0_i32;
    for (index, character) in text.char_indices() {
        match character {
            '(' | '[' => depth += 1,
            ')' | ']' if depth == 0 => return None,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::check_source;

    fn flagged(source: &str) -> bool {
        !check_source(source).is_empty()
    }

    #[test]
    fn flags_literals_and_accepts_style_values() {
        assert!(flagged("painter.rect_filled(rect, CornerRadius::same(4), fill);"));
        assert!(flagged("painter.rect_filled(rect, 4.0, fill);"));
        assert!(flagged("painter.rect(\n    rect,\n    6.0,\n    fill,\n);"));
        assert!(flagged("    .corner_radius(7)"));
        assert!(flagged("let font = FontId::proportional(13.0);"));
        assert!(flagged("ui.label(RichText::new(text).size(12.0));"));
        assert!(flagged(
            "ui.label(\n    RichText::new(text)\n        .size(12.0)\n        .weak(),\n);"
        ));
        assert!(!flagged("painter.rect_filled(rect, 0.0, fill);"));
        assert!(!flagged("painter.rect_filled(rect, style.inner(), fill);"));
        assert!(!flagged(
            "painter.rect_filled(Rect::from_min_size(a, b), style.item(), fill);"
        ));
        assert!(!flagged("let font = FontId::proportional(size);"));
        assert!(!flagged(
            "painter.rect_filled(bar, CornerRadius::same(1), c); // style: fixed"
        ));
        assert!(!flagged("/// A 4 pt `CornerRadius::same(4)` in docs."));
        assert!(!flagged("egui::Spinner::new().size(16.0);"));
    }

    #[test]
    fn reports_the_line_of_the_call() {
        assert_eq!(check_source("let a = 1;\npainter.rect_filled(r, 3.0, f);")[0].0, 2);
    }
}
