//! Two panes with a draggable divider (the editor/preview split).
//!
//! At 620 pt or wider the panes sit side by side; narrower, they stack with the second
//! pane below. The divider is a 6 pt drag handle with a 1 pt line in the inactive border
//! color and a resize cursor on hover. The ratio is the second pane's share, kept in
//! 0.2–0.8, and each pane keeps at least 200 pt (columns) or 120 pt (rows) when the
//! space allows; in a smaller space both panes shrink evenly and never overflow it.

use egui::{CursorIcon, Layout, Sense, Stroke, Ui, Vec2};

/// Width below which the panes stack vertically.
pub const STACK_BELOW: f32 = 620.0;
/// Thickness of the drag handle.
const HANDLE: f32 = 6.0;

/// Draw `first` and `second` split by a draggable divider. `ratio` is updated while
/// dragging. Returns true when the divider moved this frame.
pub fn split_panes(ui: &mut Ui, ratio: &mut f32, first: impl FnOnce(&mut Ui), second: impl FnOnce(&mut Ui)) -> bool {
    *ratio = ratio.clamp(0.2, 0.8);
    // Never wider or taller than the space given: a forced minimum pushed the second
    // pane past the window edge on small windows.
    let width = ui.available_width().max(2.0 * HANDLE).floor();
    let height = ui.available_height().max(2.0 * HANDLE).floor();
    let before = *ratio;
    if width < STACK_BELOW {
        let min = 120.0_f32.min((height - HANDLE) * 0.5);
        let second_height = (height * *ratio).clamp(min, height - HANDLE - min).round();
        let first_height = height - second_height - HANDLE;
        // No item spacing between pane, handle, and pane: the three heights already add
        // up to the available height.
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            pane(ui, Vec2::new(width, first_height), first);
            let handle = drag_handle(ui, Vec2::new(width, HANDLE));
            paint_divider(ui, &handle, true);
            if handle.dragged() {
                *ratio = (*ratio - handle.drag_delta().y / height).clamp(0.2, 0.8);
            }
            pane(ui, Vec2::new(width, second_height), second);
        });
    } else {
        let min = 200.0_f32.min((width - HANDLE) * 0.5);
        let second_width = (width * *ratio).clamp(min, width - HANDLE - min).round();
        let first_width = width - second_width - HANDLE;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            pane(ui, Vec2::new(first_width, height), first);
            let handle = drag_handle(ui, Vec2::new(HANDLE, height));
            paint_divider(ui, &handle, false);
            if handle.dragged() {
                *ratio = (*ratio - handle.drag_delta().x / width).clamp(0.2, 0.8);
            }
            pane(ui, Vec2::new(second_width, height), second);
        });
    }
    (*ratio - before).abs() > f32::EPSILON
}

/// A pane that always takes exactly `size`, even when its content is smaller, so the
/// divider and the next pane never move into it.
fn pane(ui: &mut Ui, size: Vec2, add: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(size, Layout::top_down(egui::Align::Min), |ui| {
        ui.set_min_size(size);
        ui.set_max_size(size);
        add(ui);
    });
}

/// The divider gets a stable id of its own: an automatic one collided with widgets
/// inside the panes.
fn drag_handle(ui: &mut Ui, size: Vec2) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.interact(rect, ui.scope_id().with("workbench_split_handle"), Sense::drag())
}

fn paint_divider(ui: &Ui, response: &egui::Response, horizontal: bool) {
    let stroke = Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color);
    let rect = response.rect;
    if horizontal {
        let y = rect.center().y;
        ui.painter()
            .line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], stroke);
    } else {
        let x = rect.center().x;
        ui.painter()
            .line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], stroke);
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(if horizontal {
            CursorIcon::ResizeVertical
        } else {
            CursorIcon::ResizeHorizontal
        });
    }
}
