//! Text labels with the shared tone scale.

use egui::{Response, RichText, Ui};

use crate::components::Component;
use crate::components::style::{Size, Tone};

/// A text label.
///
/// ```
/// # use egui_workbench::components::{Label, Tone, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// Label::new("Workspace").strong().show(ui);
/// Label::new("Unsaved changes").tone(Tone::Warning).show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct Label<'a> {
    text: &'a str,
    size: Size,
    tone: Tone,
    strong: bool,
}

impl<'a> Label<'a> {
    /// A default body label.
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            size: Size::Md,
            tone: Tone::Neutral,
            strong: false,
        }
    }

    /// Select the type scale.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Select the semantic color.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Use the bold weight.
    pub fn strong(mut self) -> Self {
        self.strong = true;
        self
    }
}

impl Component for Label<'_> {
    type Output = Response;

    fn show(self, ui: &mut Ui) -> Response {
        let mut rich = RichText::new(self.text)
            .size(crate::style::get(ui.ctx()).text(self.size))
            .color(self.tone.text(ui.visuals()));
        if self.strong {
            rich = rich.strong();
        }
        ui.label(rich)
    }
}
