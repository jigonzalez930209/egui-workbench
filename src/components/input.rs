//! Single-line and multi-line text inputs.

use egui::{Align2, Frame, Margin, Response, RichText, TextEdit, Ui};

use crate::components::Component;
use crate::text_field;

/// A labelled single-line text field.
///
/// ```
/// # use egui_workbench::components::{TextField, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let mut name = String::new();
/// TextField::new(&mut name).label("Name").hint("Untitled").show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct TextField<'a> {
    value: &'a mut String,
    hint: &'a str,
    label: Option<&'a str>,
    width: f32,
    enabled: bool,
}

impl<'a> TextField<'a> {
    /// A field bound to `value`.
    pub fn new(value: &'a mut String) -> Self {
        Self {
            value,
            hint: "",
            label: None,
            width: 200.0,
            enabled: true,
        }
    }

    /// A small visible label above the field, also used as its accessible name.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Placeholder text.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    /// Total width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Disable editing.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Component for TextField<'_> {
    type Output = Response;

    fn show(self, ui: &mut Ui) -> Response {
        ui.add_enabled_ui(self.enabled, |ui| {
            if let Some(label) = self.label {
                let label = ui.label(RichText::new(label).small().weak());
                text_field(ui, self.value, self.hint, self.width).labelled_by(label.id)
            } else {
                text_field(ui, self.value, self.hint, self.width)
            }
        })
        .inner
    }
}

/// A labelled multi-line text area with the sunken field frame.
pub struct TextArea<'a> {
    value: &'a mut String,
    label: Option<&'a str>,
    hint: &'a str,
    rows: usize,
    width: f32,
    enabled: bool,
}

impl<'a> TextArea<'a> {
    /// A text area bound to `value`.
    pub fn new(value: &'a mut String) -> Self {
        Self {
            value,
            label: None,
            hint: "",
            rows: 4,
            width: 320.0,
            enabled: true,
        }
    }

    /// A small visible label above the field.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Placeholder text.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    /// Visible text rows.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    /// Total width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Disable editing.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Component for TextArea<'_> {
    type Output = Response;

    fn show(self, ui: &mut Ui) -> Response {
        ui.add_enabled_ui(self.enabled, |ui| {
            let label = self.label.map(|label| ui.label(RichText::new(label).small().weak()));
            let border = ui.visuals().widgets.inactive.bg_stroke.color;
            let frame = Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(crate::style::get(ui.ctx()).border(border))
                .corner_radius(crate::style::get(ui.ctx()).control())
                .inner_margin(Margin::symmetric(8, 6));
            let response = frame
                .show(ui, |ui| {
                    ui.add(
                        TextEdit::multiline(self.value)
                            .hint_text(self.hint)
                            .frame(Frame::NONE)
                            .margin(Margin::ZERO)
                            .desired_rows(self.rows)
                            .desired_width(self.width - 16.0)
                            .align(Align2::LEFT_TOP),
                    )
                })
                .inner;
            match label {
                Some(label) => response.labelled_by(label.id),
                None => response,
            }
        })
        .inner
    }
}
