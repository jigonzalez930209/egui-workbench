//! Small status pills.

use egui::{Frame, Margin, Response, RichText, Role, Stroke, Ui, WidgetInfo};

use crate::components::Component;
use crate::components::style::Tone;

/// Outer height of a badge, in points.
const HEIGHT: f32 = 20.0;

/// A 20 pt status pill with an optional dot.
///
/// ```
/// # use egui_workbench::components::{Badge, Tone, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// Badge::new("Saved").tone(Tone::Neutral).show(ui);
/// Badge::new("Conflicts").tone(Tone::Danger).dot(true).show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Badge<'a> {
    label: &'a str,
    tone: Tone,
    dot: bool,
    radius: Option<u8>,
}

impl<'a> Badge<'a> {
    /// A neutral badge.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            tone: Tone::Neutral,
            dot: false,
            radius: None,
        }
    }

    /// Select the semantic color.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Override the corner radius (the style's `pill_radius` otherwise).
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Show a 5 pt dot before the label.
    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }
}

impl Component for Badge<'_> {
    type Output = Response;

    fn show(self, ui: &mut Ui) -> Response {
        let tone = self.tone;
        let color = tone.text(ui.visuals());
        let fill = tone.fill(ui.visuals());
        let frame = Frame::new()
            .fill(fill)
            .stroke(Stroke::new(
                crate::style::get(ui.ctx()).border_width,
                if tone == Tone::Neutral {
                    ui.visuals().window_stroke.color
                } else {
                    color.gamma_multiply(0.35)
                },
            ))
            .corner_radius(
                self.radius
                    .map_or_else(|| crate::style::get(ui.ctx()).pill(HEIGHT), egui::CornerRadius::same),
            )
            .inner_margin(Margin::symmetric(8, 2));
        let response = frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    ui.set_min_height(16.0);
                    if self.dot {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(5.0, 5.0), egui::Sense::hover());
                        ui.painter().circle_filled(rect.center(), 2.5, color);
                    }
                    ui.add(
                        egui::Label::new(
                            RichText::new(self.label)
                                .size(crate::style::get(ui.ctx()).font_size(11.0))
                                .color(color),
                        )
                        .selectable(false),
                    );
                });
            })
            .response;
        response.widget_info(|| WidgetInfo::labeled(Role::Label, true, self.label));
        response
    }
}
