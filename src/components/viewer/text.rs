//! A virtualized plain-text viewer with line numbers.

use egui::{Align, Label as TextLabel, RichText, ScrollArea, Sense, TextStyle, Ui};

use super::ViewerError;

/// A UTF-8 text file, split into line starts for cheap virtualized painting.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextFile {
    text: String,
    starts: Vec<usize>,
    max_columns: usize,
}

impl TextFile {
    /// Take ownership of UTF-8 bytes, rejecting binary input.
    ///
    /// # Errors
    /// When the bytes are not valid UTF-8 or contain a NUL byte.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ViewerError> {
        let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
        if bytes.contains(&0) {
            return Err(ViewerError::Binary);
        }
        let text = String::from_utf8(bytes.to_vec()).map_err(|_| ViewerError::Binary)?;
        Ok(Self::new(text))
    }

    /// A text file from a string.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let mut starts = vec![0];
        let mut columns = 0;
        let mut current = 0;
        for (index, character) in text.char_indices() {
            if character == '\n' {
                starts.push(index + 1);
                columns = columns.max(current);
                current = 0;
            } else if character == '\t' {
                current += 4;
            } else if character != '\r' {
                current += 1;
            }
        }
        Self {
            max_columns: columns.max(current),
            text,
            starts,
        }
    }

    /// The whole text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The number of lines.
    pub fn len(&self) -> usize {
        self.starts.len()
    }

    /// Whether the file has no text.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// One line, without its terminator.
    pub fn line(&self, index: usize) -> &str {
        let Some(&start) = self.starts.get(index) else {
            return "";
        };
        let end = self.starts.get(index + 1).map_or(self.text.len(), |end| *end);
        self.text[start..end].trim_end_matches(['\r', '\n'])
    }

    /// Paint the text inside the remaining space.
    pub fn show(&self, ui: &mut Ui, line_numbers: bool) {
        let row_height = ui.text_style_height(&TextStyle::Monospace) + 2.0;
        let digits = self.len().to_string().len().max(3);
        let char_width = (row_height * 0.58).max(4.0);
        let gutter = if line_numbers {
            digits as f32 * char_width + 18.0
        } else {
            0.0
        };
        let width = (self.max_columns as f32 * char_width + gutter + 28.0).min(30_000.0);
        ScrollArea::both()
            .id_salt("viewer_text")
            .auto_shrink([false, false])
            .show_rows(ui, row_height, self.len(), |ui, range| {
                ui.set_min_width(width);
                ui.spacing_mut().item_spacing.y = 0.0;
                for index in range {
                    let fill = if index % 2 == 1 {
                        ui.visuals().faint_bg_color.gamma_multiply(0.40)
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    egui::Frame::new().fill(fill).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal(|ui| {
                            ui.set_height(row_height);
                            if line_numbers {
                                let number = RichText::new(format!("{:>digits$}", index + 1))
                                    .monospace()
                                    .color(ui.visuals().weak_text_color());
                                ui.add_sized([gutter, row_height], TextLabel::new(number).halign(Align::RIGHT));
                            }
                            let line = self.line(index).replace('\t', "    ");
                            if line.is_empty() {
                                ui.allocate_exact_size(egui::vec2(1.0, row_height), Sense::hover());
                            } else {
                                ui.add(
                                    TextLabel::new(RichText::new(line).monospace())
                                        .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            }
                        });
                    });
                }
            });
    }
}
