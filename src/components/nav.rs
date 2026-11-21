//! Full-width navigator tabs without a close affordance.

use egui::{Align2, Role, Sense, Ui, Vec2, WidgetInfo};

use crate::components::Component;

/// A full-width left-navigation tab.
///
/// It has the tab semantics and selection fill but no close button; use it for a fixed
/// set of example sections or application views. Returns `true` when clicked.
///
/// ```
/// # use egui_workbench::components::{NavTab, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// if NavTab::new("Buttons").active(true).show(ui) {
///     // select the Buttons view
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct NavTab<'a> {
    label: &'a str,
    active: bool,
    enabled: bool,
}

impl<'a> NavTab<'a> {
    /// A navigator row.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            active: false,
            enabled: true,
        }
    }

    /// Paint the selected state.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Disable the row.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Component for NavTab<'_> {
    type Output = bool;

    fn show(self, ui: &mut Ui) -> bool {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 28.0),
            if self.enabled { Sense::click() } else { Sense::hover() },
        );
        response.widget_info(|| WidgetInfo::selected(Role::Tab, self.enabled, self.active, self.label));
        let visuals = ui.visuals();
        let fill = if self.active {
            visuals.selection.bg_fill
        } else if response.hovered() && self.enabled {
            visuals.widgets.hovered.weak_bg_fill
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, crate::style::get(ui.ctx()).item(), fill);
        let color = if !self.enabled {
            visuals.weak_text_color().gamma_multiply(0.45)
        } else if self.active {
            visuals.selection.stroke.color
        } else {
            visuals.text_color()
        };
        ui.painter().text(
            egui::pos2(rect.left() + 8.0, rect.center().y),
            Align2::LEFT_CENTER,
            self.label,
            crate::style::get(ui.ctx()).font(13.0),
            color,
        );
        self.enabled && response.clicked()
    }
}
