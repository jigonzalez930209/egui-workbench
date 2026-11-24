//! Native viewers for tabular data and plain text.
//!
//! One [`FileViewer`] opens an Excel workbook (`.xlsx`, `.xlsm`, `.xls`, `.xlsb`,
//! `.ods`) with a tab per sheet, a CSV or TSV file, or a UTF-8 text file. It loads once,
//! keeps the data in memory, and paints a virtualized table or text buffer: scrolling a
//! million-row sheet never lays out a million widgets.
//!
//! ```no_run
//! # use egui_workbench::components::viewer::FileViewer;
//! # let ctx = egui::Context::default();
//! # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
//! // Native: open by path. The bytes may also come from a browser download,
//! // an embedded document, or a channel with [`FileViewer::from_bytes`].
//! if let Ok(mut viewer) = FileViewer::open("sales.xlsx") {
//!     viewer.show(ui);
//! }
//! # });
//! # output.textures_delta.clear();
//! ```
//!
//! Painting never performs I/O. `open` is a convenience for desktop applications;
//! applications that own a document model can build a [`Table`] or [`TextFile`] directly.

mod delimited;
mod excel;
mod registry;
mod table;
mod text;

use std::fmt;
use std::path::{Path, PathBuf};

use egui::Ui;

pub use registry::{LogLevel, ViewerEvent, ViewerFile, ViewerKind, ViewerRegistry};
pub use table::{Cell, CellKind, Sheet, Table, Workbook};
pub use text::TextFile;

use crate::components::prelude::*;

/// The file format of a viewer.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileFormat {
    /// Plain UTF-8 text.
    Text,
    /// Comma-separated values.
    Csv,
    /// Tab-separated values.
    Tsv,
    /// An Excel or OpenDocument workbook with one or more sheets.
    Spreadsheet,
}

impl FileFormat {
    /// The format implied by a file name's extension; unknown extensions are text.
    pub fn from_name(name: &str) -> Self {
        let extension = Path::new(name)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match extension.as_str() {
            "xlsx" | "xlsm" | "xlam" | "xls" | "xla" | "xlsb" | "ods" => Self::Spreadsheet,
            "csv" => Self::Csv,
            "tsv" | "tab" => Self::Tsv,
            _ => Self::Text,
        }
    }

    /// A short name for a badge.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Csv => "CSV",
            Self::Tsv => "TSV",
            Self::Spreadsheet => "Workbook",
        }
    }
}

/// What went wrong while opening a file.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ViewerError {
    /// Reading or writing the file failed.
    Io(String),
    /// The workbook could not be parsed.
    Spreadsheet(String),
    /// The bytes are not UTF-8 text, and the extension did not name a workbook.
    Binary,
    /// The file has no sheets.
    Empty,
}

impl fmt::Display for ViewerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Spreadsheet(error) => write!(formatter, "could not read the workbook: {error}"),
            Self::Binary => formatter.write_str("the file is not UTF-8 text"),
            Self::Empty => formatter.write_str("the workbook has no sheets"),
        }
    }
}

impl std::error::Error for ViewerError {}

/// The loaded contents of one file.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Document {
    /// A text buffer with line numbers.
    Text(TextFile),
    /// One or more sheets of cells.
    Workbook(Workbook),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// One file open in the viewer, with its UI state (active sheet, header, line numbers).
#[derive(Clone, Debug, PartialEq)]
pub struct FileViewer {
    path: Option<PathBuf>,
    name: String,
    format: FileFormat,
    document: Document,
    sheet: usize,
    header: bool,
    line_numbers: bool,
}

impl FileViewer {
    /// Open a workbook, CSV/TSV file, or text file from disk.
    ///
    /// # Errors
    /// When the file cannot be read or parsed, or is binary with no workbook format.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ViewerError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|error| ViewerError::Io(error.to_string()))?;
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled")
            .to_owned();
        let mut viewer = Self::from_bytes(name, bytes)?;
        viewer.path = Some(path.to_path_buf());
        Ok(viewer)
    }

    /// Parse bytes already in memory (a browser download, an attached document, or a
    /// file read by the application's own worker).
    ///
    /// # Errors
    /// When the workbook cannot be parsed, or the bytes are binary and not a workbook.
    pub fn from_bytes(name: impl Into<String>, bytes: impl AsRef<[u8]>) -> Result<Self, ViewerError> {
        let name = name.into();
        let bytes = bytes.as_ref();
        let format = FileFormat::from_name(&name);
        let document = match format {
            FileFormat::Spreadsheet => Document::Workbook(excel::workbook(bytes)?),
            FileFormat::Csv => Document::Workbook(Workbook::from_delimited(name.clone(), bytes, b',')?),
            FileFormat::Tsv => Document::Workbook(Workbook::from_delimited(name.clone(), bytes, b'\t')?),
            FileFormat::Text => Document::Text(TextFile::from_bytes(bytes)?),
        };
        let header = match &document {
            Document::Workbook(workbook) => workbook.looks_like_header(),
            Document::Text(_) => false,
        };
        Ok(Self {
            path: None,
            name,
            format,
            document,
            sheet: 0,
            header,
            line_numbers: true,
        })
    }

    /// A viewer over an in-memory plain-text buffer (no parsing, no I/O).
    pub fn text(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: None,
            name: name.into(),
            format: FileFormat::Text,
            document: Document::Text(TextFile::new(text)),
            sheet: 0,
            header: false,
            line_numbers: true,
        }
    }

    /// The file name (without its folder).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The file path, when it was opened from disk.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// The detected format.
    pub fn format(&self) -> FileFormat {
        self.format
    }

    /// The loaded document.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Whether the first row is painted as a header.
    pub fn header(&self) -> bool {
        self.header
    }

    /// The active sheet index.
    pub fn active_sheet(&self) -> usize {
        self.sheet
    }

    /// The sheet tabs of a workbook (empty for text and delimited files).
    pub fn sheet_names(&self) -> Vec<&str> {
        match &self.document {
            Document::Workbook(workbook) => workbook.sheets.iter().map(|sheet| sheet.name.as_str()).collect(),
            Document::Text(_) => Vec::new(),
        }
    }

    /// Paint the toolbar, the sheet tabs, and the document.
    pub fn show(&mut self, ui: &mut Ui) {
        self.toolbar(ui);
        match &self.document {
            Document::Workbook(workbook) => {
                if workbook.sheets.len() > 1
                    && let Some(index) = sheet_tabs(ui, &workbook.sheets, self.sheet)
                {
                    self.sheet = index;
                }
                if let Some(sheet) = workbook.sheets.get(self.sheet) {
                    sheet.table.show(ui, self.header);
                }
            }
            Document::Text(text) => text.show(ui, self.line_numbers),
        }
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.set_row_height(Size::Md.height());
            Badge::new(self.format.label()).tone(Tone::Accent).show(ui);
            Label::new(&self.name).strong().show(ui);
            match &self.document {
                Document::Workbook(workbook) => {
                    if let Some(sheet) = workbook.sheets.get(self.sheet) {
                        let size = format!("{} rows · {} columns", sheet.table.height(), sheet.table.width());
                        Label::new(&size).tone(Tone::Muted).size(Size::Sm).show(ui);
                    }
                    Switch::new(&mut self.header).label("First row is a header").show(ui);
                }
                Document::Text(text) => {
                    let size = format!("{} lines", text.len());
                    Label::new(&size).tone(Tone::Muted).size(Size::Sm).show(ui);
                    Switch::new(&mut self.line_numbers).label("Line numbers").show(ui);
                }
            }
        });
        ui.add_space(6.0);
    }
}

/// One tab per sheet, scrolled horizontally. Returns the clicked sheet.
fn sheet_tabs(ui: &mut Ui, sheets: &[Sheet], active: usize) -> Option<usize> {
    let mut chosen = None;
    egui::ScrollArea::horizontal()
        .id_salt("viewer_sheets")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (index, sheet) in sheets.iter().enumerate() {
                    if Button::new(&sheet.name)
                        .size(Size::Sm)
                        .variant(ButtonVariant::Ghost)
                        .selected(index == active)
                        .show(ui)
                        .clicked()
                    {
                        chosen = Some(index);
                    }
                }
            });
        });
    ui.add_space(4.0);
    chosen
}

impl fmt::Display for FileViewer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_follow_extensions() {
        assert_eq!(FileFormat::from_name("report.xlsx"), FileFormat::Spreadsheet);
        assert_eq!(FileFormat::from_name("report.ODS"), FileFormat::Spreadsheet);
        assert_eq!(FileFormat::from_name("data.csv"), FileFormat::Csv);
        assert_eq!(FileFormat::from_name("data.tsv"), FileFormat::Tsv);
        assert_eq!(FileFormat::from_name("notes.md"), FileFormat::Text);
        assert_eq!(FileFormat::from_name("no-extension"), FileFormat::Text);
    }

    #[test]
    fn csv_open_detects_a_header_and_numbers() {
        let viewer = FileViewer::from_bytes("scores.csv", "name,score\nAda,10\nGrace,12\n").unwrap();
        assert_eq!(viewer.format(), FileFormat::Csv);
        assert!(viewer.header());
        let Document::Workbook(workbook) = viewer.document() else {
            panic!("expected a workbook");
        };
        assert_eq!(workbook.sheets[0].table.height(), 3);
        assert_eq!(workbook.sheets[0].table.width(), 2);
    }
}
