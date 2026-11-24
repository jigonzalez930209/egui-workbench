//! Typed table data and its virtualized grid.

use egui::{Align, Label as TextLabel, RichText, ScrollArea, Sense, TextStyle, Ui};

use super::ViewerError;
use super::delimited;

/// The semantic kind of one cell, used for alignment and color.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum CellKind {
    /// No value.
    #[default]
    Empty,
    /// Plain text.
    Text,
    /// A number: right-aligned.
    Number,
    /// A boolean.
    Bool,
    /// A date or time: left-aligned like text, but tinted.
    Date,
    /// A spreadsheet error (`#DIV/0!`, …).
    Error,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// One cell: its display text and semantic kind.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    /// The text painted in the grid.
    pub text: String,
    /// How the text is aligned and colored.
    pub kind: CellKind,
}

impl Cell {
    /// An empty cell.
    pub fn empty() -> Self {
        Self::default()
    }

    /// A text cell.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: CellKind::Text,
        }
    }

    /// A number cell.
    pub fn number(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: CellKind::Number,
        }
    }

    /// A boolean cell.
    pub fn boolean(value: bool) -> Self {
        Self {
            text: if value { "TRUE" } else { "FALSE" }.to_owned(),
            kind: CellKind::Bool,
        }
    }

    /// A date or time cell.
    pub fn date(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: CellKind::Date,
        }
    }

    /// An error cell.
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: CellKind::Error,
        }
    }

    /// Whether the cell has no value.
    pub fn is_empty(&self) -> bool {
        self.kind == CellKind::Empty
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A rectangular sheet of cells.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Table {
    rows: Vec<Vec<Cell>>,
    columns: usize,
}

impl Table {
    /// A table from rows; missing trailing cells are empty.
    pub fn new(rows: impl IntoIterator<Item = impl IntoIterator<Item = Cell>>) -> Self {
        let rows: Vec<Vec<Cell>> = rows.into_iter().map(|row| row.into_iter().collect()).collect();
        let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
        Self { rows, columns }
    }

    /// The rows.
    pub fn rows(&self) -> &[Vec<Cell>] {
        &self.rows
    }

    /// The number of rows.
    pub fn height(&self) -> usize {
        self.rows.len()
    }

    /// The number of columns (the widest row).
    pub fn width(&self) -> usize {
        self.columns
    }

    /// The cell at `row`, `column`; empty outside the table.
    pub fn cell(&self, row: usize, column: usize) -> Cell {
        self.rows
            .get(row)
            .and_then(|row| row.get(column))
            .cloned()
            .unwrap_or_default()
    }

    /// Whether the first row looks like a header: it has text and no error cells.
    pub fn looks_like_header(&self) -> bool {
        self.rows
            .first()
            .is_some_and(|row| row.iter().any(|cell| cell.kind == CellKind::Text))
    }

    /// Paint the grid inside the remaining space.
    pub fn show(&self, ui: &mut Ui, header: bool) {
        if self.rows.is_empty() || self.columns == 0 {
            ui.label(RichText::new("This sheet is empty.").weak());
            return;
        }
        let row_height = ui.text_style_height(&TextStyle::Monospace) + 6.0;
        let char_width = (row_height * 0.54).max(4.0);
        let mut widths = vec![56.0_f32; self.columns];
        for row in self.rows.iter().take(240) {
            for (column, cell) in row.iter().enumerate() {
                let length = cell.text.chars().count().min(48) as f32;
                widths[column] = widths[column].max((length * char_width + 18.0).min(320.0));
            }
        }
        let gutter_digits = self.rows.len().max(1).to_string().len().max(3);
        let gutter = gutter_digits as f32 * char_width + 16.0;
        let total: f32 = gutter + widths.iter().sum::<f32>();
        ScrollArea::both()
            .id_salt("viewer_table")
            .auto_shrink([false, false])
            .show_rows(ui, row_height, self.rows.len(), |ui, range| {
                ui.set_min_width(total);
                ui.spacing_mut().item_spacing.y = 0.0;
                for row_index in range {
                    let row = &self.rows[row_index];
                    let fill = if row_index % 2 == 1 {
                        ui.visuals().faint_bg_color.gamma_multiply(0.45)
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    egui::Frame::new().fill(fill).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal(|ui| {
                            ui.set_height(row_height);
                            let number = RichText::new(format!("{:>gutter_digits$}", row_index + 1))
                                .monospace()
                                .color(ui.visuals().weak_text_color());
                            ui.add_sized([gutter, row_height], TextLabel::new(number).halign(Align::RIGHT));
                            for (column, width) in widths.iter().enumerate() {
                                let cell = row.get(column).cloned().unwrap_or_default();
                                if cell.is_empty() {
                                    ui.allocate_exact_size(egui::vec2(*width, row_height), Sense::hover());
                                    continue;
                                }
                                let mut rich = RichText::new(&cell.text).monospace();
                                if header && row_index == 0 {
                                    rich = rich.strong();
                                }
                                rich = match cell.kind {
                                    CellKind::Error => rich.color(ui.visuals().error_fg_color),
                                    CellKind::Date => rich.color(ui.visuals().hyperlink_color),
                                    CellKind::Number => rich.color(ui.visuals().text_color()),
                                    _ => rich,
                                };
                                let align = if cell.kind == CellKind::Number {
                                    Align::RIGHT
                                } else {
                                    Align::LEFT
                                };
                                let response =
                                    ui.add_sized([*width, row_height], TextLabel::new(rich).halign(align).truncate());
                                response.on_hover_text(&cell.text);
                            }
                        });
                    });
                }
            });
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// One named sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sheet {
    /// The tab label.
    pub name: String,
    /// The cells.
    pub table: Table,
}

impl Sheet {
    /// A sheet with `name` and `table`.
    pub fn new(name: impl Into<String>, table: Table) -> Self {
        Self {
            name: name.into(),
            table,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The sheets of a workbook, or the single sheet of a CSV/TSV file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workbook {
    /// Sheets in workbook order.
    pub sheets: Vec<Sheet>,
}

impl Workbook {
    /// A workbook from sheets.
    pub fn new(sheets: impl IntoIterator<Item = Sheet>) -> Self {
        Self {
            sheets: sheets.into_iter().collect(),
        }
    }

    /// A single-sheet workbook.
    pub fn single(name: impl Into<String>, table: Table) -> Self {
        Self::new([Sheet::new(name, table)])
    }

    /// Parse CSV or TSV bytes (UTF-8, optional BOM) into a one-sheet workbook.
    ///
    /// # Errors
    /// When the result has no cells at all.
    pub fn from_delimited(name: String, bytes: &[u8], delimiter: u8) -> Result<Self, ViewerError> {
        let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
        let text = String::from_utf8_lossy(bytes);
        let rows = delimited::parse(&text, delimiter);
        if rows.is_empty() {
            return Err(ViewerError::Empty);
        }
        let table = Table::new(rows);
        let label = name.rsplit_once('.').map_or(name.clone(), |(stem, _)| stem.to_owned());
        Ok(Self::single(label, table))
    }

    /// Whether the first sheet's first row looks like a header.
    pub fn looks_like_header(&self) -> bool {
        self.sheets.first().is_some_and(|sheet| sheet.table.looks_like_header())
    }
}
