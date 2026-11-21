//! A raised content card.

use egui::{Color32, Frame, Margin, RichText, Stroke, Ui};

use crate::components::style::Tone;

/// A card with an optional title, description, and body.
///
/// ```
/// # use egui_workbench::components::{Card, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// Card::new().title("Appearance").description("Theme and motion.").show(ui, |ui| {
///     ui.label("Body content");
/// });
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct Card<'a> {
    title: Option<&'a str>,
    description: Option<&'a str>,
    tone: Tone,
    width: Option<f32>,
    padding: f32,
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    radius: Option<u8>,
}

impl<'a> Card<'a> {
    /// An empty card.
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            tone: Tone::Neutral,
            width: None,
            padding: 16.0,
            fill: None,
            stroke: None,
            radius: None,
        }
    }

    /// Add a 15 pt title.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Add a small weak description under the title.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Select the title tone.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Fixed width in points; the card fills the available width otherwise.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Override the background (the raised surface otherwise).
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Override the border (`Stroke::NONE` removes it).
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Override the corner radius (the style's `container_radius` otherwise).
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Inner padding in points.
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl Default for Card<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Card<'_> {
    /// Paint the card and its body. Returns the body value.
    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
        let style = crate::style::get(ui.ctx());
        let border = ui.visuals().window_stroke.color;
        let fill = self.fill.unwrap_or(ui.visuals().window_fill);
        let stroke = self.stroke.unwrap_or(Stroke::new(style.border_width, border));
        let radius = self.radius.unwrap_or(style.container_radius);
        let frame = Frame::new()
            .fill(fill)
            .stroke(stroke)
            .corner_radius(radius)
            .inner_margin(Margin::same(self.padding.round() as i8));
        frame
            .show(ui, |ui| {
                match self.width {
                    Some(width) => ui.set_width(width.min(ui.available_width()).max(0.0)),
                    None => ui.set_min_width(ui.available_width()),
                }
                if let Some(title) = self.title {
                    ui.add(egui::Label::new(self.tone.rich(ui, title, 15.0, true)).wrap());
                    if let Some(description) = self.description {
                        ui.add_space(2.0);
                        ui.add(egui::Label::new(RichText::new(description).small().weak()).wrap());
                    }
                    ui.add_space(10.0);
                }
                body(ui)
            })
            .inner
    }
}
