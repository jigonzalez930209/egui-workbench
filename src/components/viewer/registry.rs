//! A viewer registry selected by extension and content sniffing.

use std::path::Path;

use egui::{RichText, ScrollArea, TextureHandle, TextureOptions, Ui};

use crate::components::DataGridState;
#[cfg(feature = "csv")]
use crate::components::{DataGrid, DataGridEvent};

/// The built-in viewer families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ViewerKind {
    /// Plain text.
    Text,
    /// A delimited table.
    Table,
    /// JSON as a tree.
    Json,
    /// YAML as a tree.
    Yaml,
    /// TOML as a tree.
    Toml,
    /// PNG or JPEG.
    Image,
    /// SVG.
    Svg,
    /// A hex dump.
    Hex,
    /// Rendered Markdown.
    Markdown,
    /// Structured logs with a level filter.
    Logs,
    /// Source code in a monospace pane.
    Code,
}

impl ViewerKind {
    /// A short label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Table => "Table",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
            Self::Image => "Image",
            Self::Svg => "SVG",
            Self::Hex => "Hex",
            Self::Markdown => "Markdown",
            Self::Logs => "Logs",
            Self::Code => "Code",
        }
    }
}

/// One file handed to the registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewerFile {
    /// File name, used for detection and the title.
    pub name: String,
    /// The bytes.
    pub bytes: Vec<u8>,
}

impl ViewerFile {
    /// A file from bytes.
    pub fn new(name: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            bytes: bytes.into(),
        }
    }

    /// Read a file from disk.
    ///
    /// # Errors
    /// When the file cannot be read.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let bytes = crate::data::read_limited(path).map_err(|error| error.to_string())?;
        Ok(Self::new(
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("file")
                .to_owned(),
            bytes,
        ))
    }

    /// The UTF-8 text, when the bytes are text.
    pub fn text(&self) -> Option<&str> {
        std::str::from_utf8(&self.bytes).ok()
    }
}

/// Decode a PNG or JPEG with bounds. A few bytes can declare a 60000 x 60000 image, which
/// would otherwise allocate gigabytes.
fn decode_image(bytes: &[u8]) -> Option<image::DynamicImage> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16_384);
    limits.max_image_height = Some(16_384);
    limits.max_alloc = Some(512 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().ok()
}

/// Log level filter for the log viewer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum LogLevel {
    /// Every line.
    #[default]
    All,
    /// Warnings and errors.
    Warning,
    /// Errors only.
    Error,
}

/// What the viewer did this frame.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ViewerEvent {
    /// A table copy button produced text.
    Copy {
        /// The text.
        text: String,
        /// The format label.
        format: &'static str,
    },
}

/// A registry with the built-in viewers and their small UI states.
pub struct ViewerRegistry {
    kind: ViewerKind,
    grid: DataGridState,
    log_level: LogLevel,
    texture: Option<TextureHandle>,
    texture_key: String,
}

impl Default for ViewerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewerRegistry {
    /// A registry that detects on every `show`.
    pub fn new() -> Self {
        Self {
            kind: ViewerKind::Text,
            grid: DataGridState::default(),
            log_level: LogLevel::All,
            texture: None,
            texture_key: String::new(),
        }
    }

    /// The last detected kind.
    pub fn kind(&self) -> ViewerKind {
        self.kind
    }

    /// The table state.
    pub fn grid(&self) -> &DataGridState {
        &self.grid
    }

    /// The table state, mutably.
    pub fn grid_mut(&mut self) -> &mut DataGridState {
        &mut self.grid
    }

    /// The log filter.
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Set the log filter.
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }

    /// Detect a kind from the name and, when the extension is not enough, the bytes.
    pub fn detect(name: &str, bytes: &[u8]) -> ViewerKind {
        let extension = Path::new(name)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match extension.as_str() {
            "csv" | "tsv" | "tab" => ViewerKind::Table,
            "json" => ViewerKind::Json,
            "yaml" | "yml" => ViewerKind::Yaml,
            "toml" => ViewerKind::Toml,
            "png" | "jpg" | "jpeg" => ViewerKind::Image,
            "svg" => ViewerKind::Svg,
            "md" | "markdown" => ViewerKind::Markdown,
            "log" => ViewerKind::Logs,
            "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "c" | "h" | "cpp" | "hpp" | "go" | "java" | "kt" | "swift"
            | "sh" | "bash" | "zsh" | "css" | "html" => ViewerKind::Code,
            "txt" | "text" => ViewerKind::Text,
            _ => sniff(bytes),
        }
    }

    /// Show `file` with the detected viewer and return any event.
    pub fn show(&mut self, ui: &mut Ui, file: &ViewerFile) -> Option<ViewerEvent> {
        self.kind = Self::detect(&file.name, &file.bytes);
        ui.horizontal(|ui| {
            ui.label(RichText::new(&file.name).strong());
            ui.label(RichText::new(self.kind.label()).weak());
            ui.label(RichText::new(format!("{} bytes", file.bytes.len())).weak());
        });
        ui.separator();
        match self.kind {
            ViewerKind::Table => self.table(ui, file),
            ViewerKind::Json => self.tree(ui, file, TreeFormat::Json),
            ViewerKind::Yaml => self.tree(ui, file, TreeFormat::Yaml),
            ViewerKind::Toml => self.tree(ui, file, TreeFormat::Toml),
            ViewerKind::Image => self.image(ui, file, false),
            ViewerKind::Svg => self.image(ui, file, true),
            ViewerKind::Hex => self.hex(ui, file),
            ViewerKind::Markdown => self.markdown(ui, file),
            ViewerKind::Logs => self.logs(ui, file),
            ViewerKind::Text | ViewerKind::Code => self.mono(ui, file),
        }
    }

    fn table(&mut self, ui: &mut Ui, file: &ViewerFile) -> Option<ViewerEvent> {
        #[cfg(feature = "csv")]
        {
            let options = if file.name.ends_with(".tsv") || file.name.ends_with(".tab") {
                crate::data::CsvOptions::tsv()
            } else {
                crate::data::CsvOptions::csv()
            };
            match crate::data::CsvSource::parse(&file.name, file.text().unwrap_or_default(), options) {
                Ok(source) => {
                    let event = DataGrid::new(source.data(), &mut self.grid).height(420.0).show(ui);
                    match event {
                        Some(DataGridEvent::Copy { text, format }) => Some(ViewerEvent::Copy { text, format }),
                        _ => None,
                    }
                }
                Err(error) => {
                    ui.label(RichText::new(error.to_string()).color(ui.visuals().error_fg_color));
                    None
                }
            }
        }
        #[cfg(not(feature = "csv"))]
        {
            let _ = file;
            self.mono(ui, file)
        }
    }

    fn tree(&mut self, ui: &mut Ui, file: &ViewerFile, format: TreeFormat) -> Option<ViewerEvent> {
        let text = file.text().unwrap_or_default();
        let value: Result<serde_json::Value, String> = match format {
            TreeFormat::Json => serde_json::from_str::<serde_json::Value>(text).map_err(|error| error.to_string()),
            TreeFormat::Yaml => serde_norway::from_str::<serde_json::Value>(text).map_err(|error| error.to_string()),
            TreeFormat::Toml => toml::from_str::<serde_json::Value>(text).map_err(|error| error.to_string()),
        };
        match value {
            Ok(value) => {
                ScrollArea::both().show(ui, |ui| show_value(ui, "", &value));
            }
            Err(error) => {
                ui.label(RichText::new(error.to_string()).color(ui.visuals().error_fg_color));
            }
        }
        None
    }

    fn image(&mut self, ui: &mut Ui, file: &ViewerFile, svg: bool) -> Option<ViewerEvent> {
        let key = format!("{}:{}", file.name, file.bytes.len());
        if self.texture.is_none() || self.texture_key != key {
            let image = if svg {
                crate::foundation::icons::rasterize(&file.bytes, 512)
            } else {
                decode_image(&file.bytes).map(|image| {
                    let rgba = image.to_rgba8();
                    egui::ColorImage::from_rgba_unmultiplied(
                        [rgba.width() as usize, rgba.height() as usize],
                        rgba.as_raw(),
                    )
                })
            };
            self.texture = image.map(|image| {
                ui.ctx()
                    .load_texture(format!("workbench-viewer-{key}"), image, TextureOptions::LINEAR)
            });
            self.texture_key = key;
        }
        match &self.texture {
            Some(texture) => {
                let size = texture.size_vec2();
                ScrollArea::both().show(ui, |ui| {
                    ui.add(egui::Image::new(texture).max_size(size));
                });
            }
            None => {
                ui.label(RichText::new("Could not decode this image").color(ui.visuals().error_fg_color));
            }
        }
        None
    }

    fn hex(&mut self, ui: &mut Ui, file: &ViewerFile) -> Option<ViewerEvent> {
        let mut text = String::new();
        for (index, chunk) in file.bytes.chunks(16).enumerate() {
            text.push_str(&format!("{:08x}  ", index * 16));
            for byte in chunk {
                text.push_str(&format!("{byte:02x} "));
            }
            for _ in chunk.len()..16 {
                text.push_str("   ");
            }
            text.push_str(" |");
            for byte in chunk {
                text.push(if byte.is_ascii_graphic() || *byte == b' ' {
                    char::from(*byte)
                } else {
                    '.'
                });
            }
            text.push_str("|\n");
        }
        ScrollArea::both().show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .monospace()
                    .size(crate::style::get(ui.ctx()).font_size(11.0)),
            );
        });
        None
    }

    fn markdown(&mut self, ui: &mut Ui, file: &ViewerFile) -> Option<ViewerEvent> {
        let text = file.text().unwrap_or_default();
        let mut heading = None::<u8>;
        let mut paragraph = String::new();
        let mut code = false;
        let flush = |ui: &mut Ui, heading: Option<u8>, paragraph: &mut String, code: bool| {
            if paragraph.trim().is_empty() {
                paragraph.clear();
                return;
            }
            let text = std::mem::take(paragraph);
            if code {
                ui.label(
                    RichText::new(text)
                        .monospace()
                        .size(crate::style::get(ui.ctx()).font_size(11.5)),
                );
            } else {
                let mut rich = RichText::new(text);
                if let Some(level) = heading {
                    rich = rich.strong().size(crate::style::get(ui.ctx()).font_size(match level {
                        1 => 20.0,
                        2 => 17.0,
                        _ => 15.0,
                    }));
                }
                ui.label(rich);
            }
        };
        for event in pulldown_cmark::Parser::new(text) {
            match event {
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading { level, .. }) => {
                    flush(ui, heading, &mut paragraph, code);
                    heading = Some(level as u8);
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                    flush(ui, heading, &mut paragraph, code);
                    heading = None;
                }
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(_)) => {
                    flush(ui, heading, &mut paragraph, code);
                    code = true;
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                    flush(ui, heading, &mut paragraph, code);
                    code = false;
                }
                pulldown_cmark::Event::Text(text) => paragraph.push_str(&text),
                pulldown_cmark::Event::Code(text) => {
                    paragraph.push('`');
                    paragraph.push_str(&text);
                    paragraph.push('`');
                }
                pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                    flush(ui, heading, &mut paragraph, code);
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Paragraph) => {
                    flush(ui, heading, &mut paragraph, code);
                }
                _ => {}
            }
        }
        flush(ui, heading, &mut paragraph, code);
        None
    }

    fn logs(&mut self, ui: &mut Ui, file: &ViewerFile) -> Option<ViewerEvent> {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.log_level, LogLevel::All, "All");
            ui.selectable_value(&mut self.log_level, LogLevel::Warning, "Warnings");
            ui.selectable_value(&mut self.log_level, LogLevel::Error, "Errors");
        });
        let text = file.text().unwrap_or_default();
        ScrollArea::both().show(ui, |ui| {
            for line in text.lines() {
                let level = line_level(line);
                let visible = match self.log_level {
                    LogLevel::All => true,
                    LogLevel::Warning => matches!(level, Some(LogLevel::Warning | LogLevel::Error)),
                    LogLevel::Error => level == Some(LogLevel::Error),
                };
                if !visible {
                    continue;
                }
                let color = match level {
                    Some(LogLevel::Error) => ui.visuals().error_fg_color,
                    Some(LogLevel::Warning) => ui.visuals().warn_fg_color,
                    _ => ui.visuals().text_color(),
                };
                ui.label(
                    RichText::new(line)
                        .monospace()
                        .size(crate::style::get(ui.ctx()).font_size(11.0))
                        .color(color),
                );
            }
        });
        None
    }

    fn mono(&mut self, ui: &mut Ui, file: &ViewerFile) -> Option<ViewerEvent> {
        let text = file.text().unwrap_or_default();
        ScrollArea::both().show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .monospace()
                    .size(crate::style::get(ui.ctx()).font_size(11.5)),
            );
        });
        None
    }
}

mod helpers;

use helpers::{TreeFormat, line_level, show_value, sniff};

#[cfg(test)]
mod tests;
