//! Icon buttons and the segmented control.
//!
//! Three square sizes exist: 28 pt in the title bar, 26 pt in the tab strip, and 21 pt in
//! the explorer header. All are transparent at rest, take the hover fill on hover and
//! the active fill while pressed, and use a 5 pt (4 pt for the small one) radius.

use egui::{Color32, Rect, Role, Sense, Ui, Vec2, WidgetInfo};

use crate::Icons;
use crate::metrics::{ACTION_BUTTON, ICON_MEDIUM, SMALL_BUTTON, TOOL_BUTTON};

/// The state of an icon button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IconState {
    /// Shows the hover fill and a 2 pt accent underline (a toggled panel, an open popover).
    pub active: bool,
    /// Disabled buttons draw a faded glyph and never report clicks.
    pub disabled: bool,
}

impl IconState {
    /// An enabled, inactive button.
    pub const NORMAL: Self = Self {
        active: false,
        disabled: false,
    };

    /// A toggled button.
    pub fn active(active: bool) -> Self {
        Self {
            active,
            disabled: false,
        }
    }

    /// An enabled or disabled button.
    pub fn enabled(enabled: bool) -> Self {
        Self {
            active: false,
            disabled: !enabled,
        }
    }
}

/// The 28 pt title-bar action button with a 16 pt glyph.
pub fn action_button(ui: &mut Ui, icons: &mut Icons, icon: &str, tooltip: &str, state: IconState) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(ACTION_BUTTON), Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::Button, !state.disabled, state.active, tooltip));
    if state.disabled {
        let faded = ui.visuals().weak_text_color().gamma_multiply(0.45);
        icons.put(ui, icon, rect.center(), ICON_MEDIUM, faded);
        return false;
    }
    let visuals = ui.visuals();
    let fill = if response.is_pointer_button_down_on() {
        visuals.widgets.active.weak_bg_fill
    } else if state.active || response.hovered() {
        visuals.widgets.hovered.weak_bg_fill
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, crate::style::get(ui.ctx()).item(), fill);
    if state.active {
        let underline = Rect::from_min_max(
            egui::pos2(rect.left() + 7.0, rect.bottom() - 2.0),
            egui::pos2(rect.right() - 7.0, rect.bottom()),
        );
        ui.painter()
            .rect_filled(underline, crate::style::get(ui.ctx()).mark(), visuals.selection.bg_fill);
    }
    let color = visuals.text_color();
    icons.put(ui, icon, rect.center(), ICON_MEDIUM, color);
    response.on_hover_text(tooltip).clicked()
}

/// The 26 pt tab-strip tool button with an 18 pt glyph.
pub fn tool_button(ui: &mut Ui, icons: &mut Icons, icon: &str, tooltip: &str, active: bool) -> bool {
    let response = tool_button_response(ui, icons, icon, active, false);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, tooltip));
    response.on_hover_text(tooltip).clicked()
}

/// The tool button's response, for callers that attach their own popup.
pub fn tool_button_response(ui: &mut Ui, icons: &mut Icons, icon: &str, active: bool, open: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(TOOL_BUTTON), Sense::click());
    paint_tool_button(ui, icons, icon, rect, &response, active || open);
    response
}

fn paint_tool_button(ui: &mut Ui, icons: &mut Icons, icon: &str, rect: Rect, response: &egui::Response, lit: bool) {
    let fill = if response.is_pointer_button_down_on() {
        ui.visuals().widgets.active.weak_bg_fill
    } else if lit || response.hovered() {
        ui.visuals().widgets.hovered.weak_bg_fill
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, crate::style::get(ui.ctx()).item(), fill);
    let color = ui.visuals().text_color();
    icons.put(ui, icon, rect.center(), 18.0, color);
}

/// A tool button that opens a popup menu under itself (the "…" overflow menu). The
/// button stays lit while its menu is open.
pub fn menu_tool_button(
    ui: &mut Ui,
    icons: &mut Icons,
    icon: &str,
    tooltip: &str,
    min_width: f32,
    add: impl FnOnce(&mut Ui, &mut Icons),
) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(TOOL_BUTTON), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, tooltip));
    let open = egui::Popup::menu(&response).is_open();
    paint_tool_button(ui, icons, icon, rect, &response, open);
    egui::Popup::menu(&response).show(|ui| {
        crate::compact_menu(ui);
        ui.set_min_width(min_width);
        add(ui, icons);
    });
    response.on_hover_text(tooltip);
}

/// The 21 pt explorer-header button with a 16 pt glyph; no fill at rest.
pub fn small_button(ui: &mut Ui, icons: &mut Icons, icon: &str, tooltip: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(SMALL_BUTTON), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, tooltip));
    if response.is_pointer_button_down_on() || response.hovered() {
        let fill = if response.is_pointer_button_down_on() {
            ui.visuals().widgets.active.weak_bg_fill
        } else {
            ui.visuals().widgets.hovered.weak_bg_fill
        };
        ui.painter()
            .rect_filled(rect, crate::style::get(ui.ctx()).inner(), fill);
    }
    icons.paint(ui, icon, rect.center(), SMALL_BUTTON - 5.0, ui.visuals().text_color());
    response.on_hover_text(tooltip).clicked()
}

/// One option of a [`segmented`] control.
#[derive(Clone, Debug)]
pub struct Segment<'a> {
    /// Glyph name.
    pub icon: &'a str,
    /// Tooltip.
    pub tooltip: &'a str,
    /// Disabled segments draw a faded glyph and ignore clicks.
    pub enabled: bool,
}

/// An icon-only segmented control on a sunken 24 pt track (the View/Edit/Split switch).
///
/// Each segment is 36 pt wide; the selected one takes the selection fill and the
/// selection text color. Returns the index of a newly clicked segment.
pub fn segmented(
    ui: &mut Ui,
    icons: &mut Icons,
    id_salt: &str,
    segments: &[Segment<'_>],
    selected: usize,
) -> Option<usize> {
    let segment_width = 36.0;
    let width = segment_width * segments.len() as f32;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 24.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, crate::style::get(ui.ctx()).item(), ui.visuals().extreme_bg_color);
    let mut clicked = None;
    for (index, segment) in segments.iter().enumerate() {
        let cell = Rect::from_min_size(
            egui::pos2(rect.left() + index as f32 * segment_width + 1.0, rect.top() + 1.0),
            egui::vec2(segment_width - 2.0, rect.height() - 2.0),
        );
        let enabled = ui.is_enabled() && segment.enabled;
        let response = ui
            .interact(
                cell,
                ui.make_persistent_id((id_salt, index)),
                if enabled { Sense::click() } else { Sense::hover() },
            )
            .on_hover_text(segment.tooltip);
        let active = index == selected;
        response.widget_info(|| WidgetInfo::selected(Role::RadioButton, enabled, active, segment.tooltip));
        if active {
            ui.painter().rect_filled(
                cell,
                crate::style::get(ui.ctx()).inner(),
                ui.visuals().selection.bg_fill,
            );
        } else if response.hovered() && enabled {
            ui.painter().rect_filled(
                cell,
                crate::style::get(ui.ctx()).inner(),
                ui.visuals().widgets.hovered.weak_bg_fill,
            );
        }
        let color = if !enabled {
            ui.visuals().weak_text_color().gamma_multiply(0.45)
        } else if active {
            ui.visuals().selection.stroke.color
        } else {
            ui.visuals().text_color()
        };
        icons.paint(ui, segment.icon, cell.center(), 15.0, color);
        if enabled && response.clicked() && !active {
            clicked = Some(index);
        }
        if response.hovered() && enabled {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
    clicked
}
