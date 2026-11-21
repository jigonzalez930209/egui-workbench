//! A labelled checkbox.

use egui::{Rect, Role, Sense, Shape, Stroke, Ui, Vec2, WidgetInfo, pos2};

use crate::components::Component;

/// An 18 pt checkbox whose whole row is clickable.
///
/// ```
/// # use egui_workbench::components::{Checkbox, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let mut wrap = true;
/// if Checkbox::new(&mut wrap).label("Word wrap").show(ui) {
///     // wrap changed
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct Checkbox<'a> {
    value: &'a mut bool,
    label: &'a str,
    enabled: bool,
}

impl<'a> Checkbox<'a> {
    /// A checkbox bound to `value`.
    pub fn new(value: &'a mut bool) -> Self {
        Self {
            value,
            label: "",
            enabled: true,
        }
    }

    /// The accessible name and visible label.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// Disable interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Component for Checkbox<'_> {
    type Output = bool;

    fn show(self, ui: &mut Ui) -> bool {
        let checked = *self.value;
        let color = if self.enabled {
            ui.visuals().text_color()
        } else {
            ui.visuals().weak_text_color().gamma_multiply(0.45)
        };
        let galley = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), crate::style::get(ui.ctx()).font(13.0), color);
        let text_width = if self.label.is_empty() {
            0.0
        } else {
            galley.size().x + 8.0
        };
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(18.0 + text_width, 20.0),
            if self.enabled { Sense::click() } else { Sense::hover() },
        );
        response.widget_info(|| WidgetInfo::selected(Role::CheckBox, self.enabled, checked, self.label));
        let box_rect = Rect::from_center_size(pos2(rect.left() + 9.0, rect.center().y), Vec2::splat(18.0));
        let visuals = ui.visuals();
        let style = crate::style::get(ui.ctx());
        if checked {
            ui.painter()
                .rect_filled(box_rect, style.inner(), visuals.selection.bg_fill);
            let stroke = Stroke::new(1.8, visuals.selection.stroke.color);
            let points = [
                pos2(box_rect.left() + 4.0, box_rect.center().y),
                pos2(box_rect.center().x - 1.0, box_rect.bottom() - 5.0),
                pos2(box_rect.right() - 4.0, box_rect.top() + 5.0),
            ];
            ui.painter().add(Shape::line(points.to_vec(), stroke));
        } else if style.border_width > 0.0 {
            ui.painter().rect_stroke(
                box_rect,
                style.inner(),
                style.border(visuals.widgets.inactive.bg_stroke.color),
                egui::StrokeKind::Inside,
            );
        } else {
            // Without borders the empty box is a sunken fill, so it stays visible.
            ui.painter()
                .rect_filled(box_rect, style.inner(), visuals.widgets.inactive.weak_bg_fill);
        }
        if !self.label.is_empty() {
            ui.painter().galley(
                pos2(box_rect.right() + 8.0, rect.center().y - galley.size().y / 2.0),
                galley,
                color,
            );
        }
        if self.enabled && response.clicked() {
            *self.value = !*self.value;
            true
        } else {
            false
        }
    }
}
