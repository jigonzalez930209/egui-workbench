//! Dividers between groups.

use egui::{Stroke, Ui};

use crate::components::Component;

/// Horizontal or vertical separator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Orientation {
    /// A full-width 1 pt line.
    #[default]
    Horizontal,
    /// A 1 pt line filling the available height.
    Vertical,
}

/// A themed 1 pt separator with 6 pt spacing.
///
/// ```
/// # use egui_workbench::components::{Separator, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// ui.label("Above");
/// Separator::new().show(ui);
/// ui.label("Below");
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Separator {
    orientation: Orientation,
    spacing: f32,
}

impl Separator {
    /// A horizontal separator.
    pub fn new() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            spacing: 6.0,
        }
    }

    /// A vertical separator.
    pub fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
            spacing: 6.0,
        }
    }

    /// Space before and after the line.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Separator {
    type Output = ();

    fn show(self, ui: &mut Ui) {
        let color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        match self.orientation {
            Orientation::Horizontal => {
                ui.add_space(self.spacing);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                ui.painter()
                    .hline(rect.x_range(), rect.center().y, Stroke::new(1.0, color));
                ui.add_space(self.spacing);
            }
            Orientation::Vertical => {
                ui.add_space(self.spacing);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, ui.available_height()), egui::Sense::hover());
                ui.painter()
                    .vline(rect.center().x, rect.y_range(), Stroke::new(1.0, color));
                ui.add_space(self.spacing);
            }
        }
    }
}
