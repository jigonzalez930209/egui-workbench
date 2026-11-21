//! Form controls that share one geometry so dialogs never mix baselines.
//!
//! Every text field uses the same sunken surface, 30 pt height, 8 pt side padding, and
//! vertically centered text. Primary, secondary, and danger buttons share the same
//! height and a 6 pt radius.

use std::ops::RangeInclusive;

use egui::{Align2, Button, Frame, Margin, Response, RichText, Role, Sense, Stroke, TextEdit, Ui, WidgetInfo};

use crate::Icons;
use crate::metrics::ICON_SMALL;

/// A single-line text field with centered text and the sunken frame.
///
/// While focused, a 1.5 pt ring in the selection color is drawn inside the frame.
pub fn text_field(ui: &mut Ui, value: &mut String, hint: &str, width: f32) -> Response {
    let style = crate::style::get(ui.ctx());
    let height = style.control_height();
    let border = ui.visuals().widgets.inactive.bg_stroke.color;
    let frame = Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(Stroke::new(style.border_width, border))
        .corner_radius(style.control())
        .inner_margin(Margin::symmetric(8, 0));
    let output = frame.show(ui, |ui| {
        ui.add_sized(
            egui::vec2((width - 16.0).max(40.0), height),
            TextEdit::singleline(value)
                .hint_text(hint)
                .frame(Frame::NONE)
                .margin(Margin::symmetric(0, ((height - 16.0) / 2.0).max(0.0) as i8))
                .align(Align2::LEFT_CENTER)
                .desired_width(f32::INFINITY),
        )
    });
    if output.inner.has_focus() {
        ui.painter().rect_stroke(
            output.response.rect,
            style.control(),
            Stroke::new(1.5, ui.visuals().selection.bg_fill),
            egui::StrokeKind::Inside,
        );
    }
    output.inner
}

/// The primary action of a dialog: selection fill, no border, at least 92 pt wide.
/// `danger` paints it with the error color and the more readable of white or the
/// background color for the text.
pub fn primary_button(ui: &mut Ui, label: &str, danger: bool) -> Response {
    let (fill, text) = if danger {
        let fill = ui.visuals().error_fg_color;
        (fill, crate::theme::readable_on(fill, ui.visuals().panel_fill))
    } else {
        (ui.visuals().selection.bg_fill, ui.visuals().selection.stroke.color)
    };
    let style = crate::style::get(ui.ctx());
    ui.add(
        Button::new(
            RichText::new(label)
                .size(crate::style::get(ui.ctx()).font_size(13.0))
                .color(text),
        )
        .fill(fill)
        .stroke(Stroke::NONE)
        .corner_radius(style.control())
        .min_size(egui::vec2(92.0, style.control_height())),
    )
}

/// A neutral action: surface fill, 1 pt border, at least 88 pt wide.
pub fn secondary_button(ui: &mut Ui, label: &str) -> Response {
    let style = crate::style::get(ui.ctx());
    ui.add(
        Button::new(
            RichText::new(label)
                .size(crate::style::get(ui.ctx()).font_size(13.0))
                .color(ui.visuals().text_color()),
        )
        .fill(ui.visuals().widgets.inactive.weak_bg_fill)
        .stroke(Stroke::new(
            style.border_width,
            ui.visuals().widgets.inactive.bg_stroke.color,
        ))
        .corner_radius(style.control())
        .min_size(egui::vec2(88.0, style.control_height())),
    )
}

/// A borderless "×" close button for dialog headers.
///
/// The cross is drawn with two pixel-snapped strokes in a square hit area of `size + 8`
/// points, so it centers on the row like an icon button instead of sitting on the text
/// baseline the way a "×" character does. Assistive technology still reads "×".
pub fn close_glyph(ui: &mut Ui, size: f32, tooltip: &str) -> bool {
    let side = (size + 8.0).round();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(side, side), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "×"));
    let visuals = ui.visuals();
    let hovered = response.hovered() || response.has_focus();
    if hovered {
        let fill = if response.is_pointer_button_down_on() {
            visuals.widgets.active.weak_bg_fill.gamma_multiply(0.4)
        } else {
            visuals.widgets.hovered.weak_bg_fill
        };
        ui.painter()
            .rect_filled(rect, crate::style::get(ui.ctx()).inner(), fill);
    }
    let color = if hovered {
        visuals.text_color()
    } else {
        visuals.weak_text_color()
    };
    let painter = ui.painter();
    let center = egui::pos2(
        painter.round_to_pixel_center(rect.center().x),
        painter.round_to_pixel_center(rect.center().y),
    );
    let arm = (size * 0.26).round().max(3.0);
    let stroke = Stroke::new((size / 12.0).clamp(1.1, 1.5), color);
    painter.line_segment([center + egui::vec2(-arm, -arm), center + egui::vec2(arm, arm)], stroke);
    painter.line_segment([center + egui::vec2(-arm, arm), center + egui::vec2(arm, -arm)], stroke);
    response.on_hover_text(tooltip).clicked()
}

/// A numeric field with stacked up/down chevrons on its right edge.
///
/// Returns true when a click changed the value (clamped to `range`).
pub fn numeric_stepper(
    ui: &mut Ui,
    icons: &mut Icons,
    value: &mut f32,
    range: RangeInclusive<f32>,
    step: f32,
    width: f32,
    format: impl Fn(f32) -> String,
) -> bool {
    let style = crate::style::get(ui.ctx());
    let control_height = style.control_height();
    let border = ui.visuals().widgets.inactive.bg_stroke.color;
    let frame = Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(Stroke::new(style.border_width, border))
        .corner_radius(style.control());
    let rect = frame
        .show(ui, |ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(width, control_height), Sense::hover());
            ui.painter().text(
                egui::pos2(rect.left() + 9.0, rect.center().y),
                Align2::LEFT_CENTER,
                format(*value),
                crate::style::get(ui.ctx()).font(13.0),
                ui.visuals().text_color(),
            );
            rect
        })
        .inner;
    let button_width = 22.0;
    let half = egui::vec2(button_width, control_height / 2.0 - 1.0);
    let up = egui::Rect::from_min_size(egui::pos2(rect.right() - button_width, rect.top() + 1.0), half);
    let down = egui::Rect::from_min_size(egui::pos2(rect.right() - button_width, rect.center().y), half);
    let id = ui.make_persistent_id(("stepper", rect.left().to_bits(), rect.top().to_bits()));
    let mut changed = false;
    let enabled = ui.is_enabled();
    for (rect, salt, icon, delta) in [(up, "up", "chevron-up", step), (down, "down", "chevron-down", -step)] {
        let response = ui.interact(
            rect,
            id.with(salt),
            if enabled { Sense::click() } else { Sense::hover() },
        );
        let label = if delta > 0.0 { "+" } else { "−" };
        response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, label));
        if response.hovered() && enabled {
            ui.painter().rect_filled(
                rect,
                crate::style::get(ui.ctx()).tight(),
                ui.visuals().widgets.hovered.weak_bg_fill,
            );
        }
        icons.paint(ui, icon, rect.center(), ICON_SMALL, ui.visuals().weak_text_color());
        if enabled && response.clicked() {
            *value = (*value + delta).clamp(*range.start(), *range.end());
            changed = true;
        }
    }
    changed
}

/// A 36×19 pt toggle switch with an animated knob, announced as a check box named
/// `label`. Returns true when clicked.
pub fn switch(ui: &mut Ui, value: &mut bool, label: &str) -> bool {
    let enabled = ui.is_enabled();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(36.0, 19.0),
        if enabled { Sense::click() } else { Sense::hover() },
    );
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, *value, label));
    let clicked = enabled && response.clicked();
    if clicked {
        *value = !*value;
    }
    let on = *value;
    let t = ui.ctx().animate_bool_with_time(response.id, on, 0.12);
    let visuals = ui.visuals();
    let fill = if on {
        visuals.selection.bg_fill
    } else {
        visuals.extreme_bg_color
    };
    ui.painter()
        .rect_filled(rect, crate::style::get(ui.ctx()).pill(rect.height()), fill);
    ui.painter().rect_stroke(
        rect,
        crate::style::get(ui.ctx()).pill(rect.height()),
        crate::style::get(ui.ctx()).border(visuals.widgets.inactive.bg_stroke.color),
        egui::StrokeKind::Inside,
    );
    let knob_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), t);
    let knob = if on {
        visuals.text_color()
    } else {
        visuals.weak_text_color()
    };
    ui.painter()
        .circle_filled(egui::pos2(knob_x, rect.center().y), 6.5, knob);
    clicked
}

/// A 170 pt combo box over `options`, showing each option's label. `label` is not drawn
/// (the settings row shows it); it names the box for screen readers and salts its id.
pub fn combo<T: PartialEq + Copy>(ui: &mut Ui, label: &str, value: &mut T, options: &[(T, String)]) -> bool {
    let before = *value;
    let selected = options
        .iter()
        .find(|(option, _)| *option == *value)
        .map(|(_, label)| label.clone())
        .unwrap_or_default();
    let response = egui::ComboBox::from_id_salt(("workbench_combo", label))
        .width(170.0)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for (option, text) in options {
                ui.selectable_value(value, *option, text);
            }
        })
        .response;
    response.widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, label));
    *value != before
}

/// The compact typography of context menus and popups: 12 pt rows, 1 pt gaps.
pub fn compact_menu(ui: &mut Ui) {
    ui.spacing_mut().item_spacing.y = 1.0;
    ui.spacing_mut().button_padding = egui::vec2(8.0, 2.0);
    let font = crate::style::get(ui.ctx()).font(12.0);
    ui.style_mut().text_styles.insert(egui::TextStyle::Button, font);
    ui.style_mut().spacing.menu_margin = Margin::same(4);
}

/// A 22 pt context-menu row: a 14 pt glyph, the label, an optional right-aligned
/// shortcut, and the error color when `danger`.
pub fn menu_item(
    ui: &mut Ui,
    icons: &mut Icons,
    icon: &str,
    label: &str,
    shortcut: Option<&str>,
    danger: bool,
) -> bool {
    let width = ui.available_width().max(180.0);
    let enabled = ui.is_enabled();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, crate::style::get(ui.ctx()).menu_row_height()),
        if enabled { Sense::click() } else { Sense::hover() },
    );
    response.widget_info(|| WidgetInfo::labeled(Role::MenuItem, enabled, label));
    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            crate::style::get(ui.ctx()).inner(),
            ui.visuals().widgets.hovered.weak_bg_fill,
        );
    }
    let color = if !enabled {
        ui.visuals().weak_text_color().gamma_multiply(0.45)
    } else if danger {
        ui.visuals().error_fg_color
    } else {
        ui.visuals().text_color()
    };
    icons.paint(
        ui,
        icon,
        egui::pos2(rect.left() + 13.0, rect.center().y),
        ICON_SMALL,
        color,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 25.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        crate::style::get(ui.ctx()).font(12.0),
        color,
    );
    if let Some(shortcut) = shortcut {
        ui.painter().text(
            egui::pos2(rect.right() - 6.0, rect.center().y),
            Align2::RIGHT_CENTER,
            shortcut,
            crate::style::get(ui.ctx()).font(10.5),
            ui.visuals().weak_text_color(),
        );
    }
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
    } else {
        false
    }
}

/// A plain menu button that closes its menu when clicked.
pub fn menu_button(ui: &mut Ui, label: &str) -> bool {
    let clicked = ui.button(label).clicked();
    if clicked {
        ui.close();
    }
    clicked
}

/// A menu toggle drawn as a selectable label; closes the menu when clicked.
pub fn menu_toggle(ui: &mut Ui, label: &str, on: bool, enabled: bool) -> bool {
    let clicked = ui
        .add_enabled_ui(enabled, |ui| ui.selectable_label(on, label).clicked())
        .inner;
    if clicked {
        ui.close();
    }
    clicked
}
