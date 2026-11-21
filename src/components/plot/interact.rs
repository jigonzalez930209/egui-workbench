//! Pointer interaction: pan, wheel zoom, box zoom, selection, and reset.

use egui::{CursorIcon, Pos2, Rect, Response, Ui, Vec2};

use super::axis::{Bounds, Scale, Transform};
use super::series;
use super::{Drag, Plot, PlotHover, PlotState, PlotTool, overlay, paint, widgets};

impl Plot<'_> {
    /// The automatic view: every visible series plus 5 % margins, in scaled space,
    /// with the fixed axis limits of the config applied on top.
    pub(super) fn auto_bounds(&self, state: &PlotState) -> Bounds {
        let (x_scale, y_scale) = (self.config.x.scale, self.config.y.scale);
        let mut bounds = Bounds::NOTHING;
        for series in self.visible(state) {
            bounds.union(&series.bounds(x_scale, y_scale));
        }
        if self.include.is_valid() {
            bounds.extend([x_scale.forward(self.include.min_x), y_scale.forward(self.include.min_y)]);
        }
        for series in self.visible(state) {
            match series.kind {
                series::Kind::HLine(y) if bounds.is_valid() => bounds.extend([bounds.min_x, y_scale.forward(y)]),
                series::Kind::VLine(x) if bounds.is_valid() => bounds.extend([x_scale.forward(x), bounds.min_y]),
                _ => {}
            }
        }
        if !bounds.is_valid() {
            bounds = Bounds::from_corners([0.0, 0.0], [1.0, 1.0]);
        }
        if let Some(window) = self.x_window
            && x_scale == Scale::Linear
        {
            bounds.min_x = bounds.min_x.max(bounds.max_x - window);
        }
        let pad = |min: &mut f64, max: &mut f64| {
            let span = *max - *min;
            let span = if span > 0.0 { span } else { min.abs().max(1.0) * 0.2 };
            let margin = span * 0.05;
            *min -= margin;
            *max += margin;
        };
        pad(&mut bounds.min_x, &mut bounds.max_x);
        pad(&mut bounds.min_y, &mut bounds.max_y);
        let limit =
            |value: Option<f64>, scale: Scale| value.map(|value| scale.forward(value)).filter(|v| v.is_finite());
        if let Some(min) = limit(self.config.x.min, x_scale) {
            bounds.min_x = min;
        }
        if let Some(max) = limit(self.config.x.max, x_scale) {
            bounds.max_x = max;
        }
        if let Some(min) = limit(self.config.y.min, y_scale) {
            bounds.min_y = min;
        }
        if let Some(max) = limit(self.config.y.max, y_scale) {
            bounds.max_y = max;
        }
        if bounds.max_x <= bounds.min_x {
            bounds.max_x = bounds.min_x + 1.0;
        }
        if bounds.max_y <= bounds.min_y {
            bounds.max_y = bounds.min_y + 1.0;
        }
        bounds
    }

    /// The nearest point over every visible series, highlighted with its popup.
    pub(super) fn hover(
        &self,
        ui: &Ui,
        transform: &Transform,
        state: &PlotState,
        pointer: Pos2,
        theme: &crate::theme::Theme,
    ) -> Option<PlotHover> {
        let mut best: Option<(usize, usize, Pos2, f32)> = None;
        for (series_index, series) in self.series.iter().enumerate() {
            if state.is_hidden(&series.name) {
                continue;
            }
            if let Some((index, position, distance)) =
                paint::nearest(transform, series, pointer, 12.0, series.sorted_x())
                && best.is_none_or(|(_, _, _, current)| distance < current)
            {
                best = Some((series_index, index, position, distance));
            }
        }
        let (series_index, index, position, _) = best?;
        let series = &self.series[series_index];
        let look = overlay::look(self, series_index, theme);
        let point = series.data()[index];
        let visuals = ui.visuals();
        let guide = self.config.hover_guides.then(|| visuals.weak_text_color());
        let painter = ui.painter().with_clip_rect(transform.frame);
        widgets::highlight(
            &painter,
            transform.frame,
            position,
            look.color,
            visuals.extreme_bg_color,
            guide,
        );
        if self.config.hover_popup {
            widgets::tooltip(ui, self, series.name(), look.color, point, position);
        }
        Some(PlotHover {
            series: series.name.clone(),
            index,
            point,
        })
    }

    pub(super) fn interact(
        &self,
        ui: &Ui,
        response: &Response,
        transform: &Transform,
        state: &mut PlotState,
        pointer: Option<Pos2>,
        selection_changed: &mut bool,
    ) {
        let frame = transform.frame;
        if response.double_clicked() {
            state.view = None;
            state.drag = None;
            return;
        }
        let (shift, alt) = ui.input(|input| (input.modifiers.shift, input.modifiers.alt));
        if response.drag_started() && response.interact_pointer_pos().is_some_and(|pos| frame.contains(pos)) {
            let start = response.interact_pointer_pos().unwrap_or(frame.center());
            let secondary = response.dragged_by(egui::PointerButton::Secondary);
            state.drag = Some(if shift || (state.tool == PlotTool::Select && !secondary) {
                Drag::Box { start, select: true }
            } else if secondary || state.tool == PlotTool::Zoom {
                Drag::Box { start, select: false }
            } else {
                Drag::Pan
            });
        }
        match state.drag {
            Some(Drag::Pan) if response.dragged() => {
                let delta = response.drag_delta();
                let [ux, uy] = transform.units_per_point();
                let mut view = transform.view;
                view.min_x -= f64::from(delta.x) * ux;
                view.max_x -= f64::from(delta.x) * ux;
                view.min_y += f64::from(delta.y) * uy;
                view.max_y += f64::from(delta.y) * uy;
                state.view = Some(view);
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
            Some(Drag::Box { start, select }) if response.drag_stopped() => {
                let end = response.interact_pointer_pos().unwrap_or(start);
                let rect = Rect::from_two_pos(start, end).intersect(frame);
                if select {
                    if rect.width() >= 3.0 || rect.height() >= 3.0 {
                        let a = transform.to_data(rect.left_bottom());
                        let b = transform.to_data(rect.right_top());
                        state.selection = Some(Bounds::from_corners(a, b));
                        *selection_changed = true;
                    }
                } else {
                    zoom_to_box(state, transform, rect);
                }
                state.drag = None;
            }
            Some(_) if !response.dragged() && !response.drag_stopped() => state.drag = None,
            _ => {}
        }
        if let Some(pointer) = pointer {
            if state.drag.is_none() {
                ui.ctx().set_cursor_icon(match state.tool {
                    PlotTool::Pan => CursorIcon::Grab,
                    PlotTool::Zoom | PlotTool::Select => CursorIcon::Crosshair,
                });
            }
            let (scroll, zoom) = ui.input(|input| (input.smooth_scroll_delta, input.zoom_delta()));
            let wheel = scroll.y + scroll.x;
            let factor = if (zoom - 1.0).abs() > f32::EPSILON {
                zoom
            } else if wheel != 0.0 {
                (wheel * 0.002).exp()
            } else {
                1.0
            };
            if (factor - 1.0).abs() > f32::EPSILON {
                // Consume the wheel so an enclosing scroll area does not scroll too.
                ui.ctx().input_mut(|input| input.smooth_scroll_delta = Vec2::ZERO);
                let horizontal = !alt;
                let vertical = !shift;
                zoom_at(state, transform, pointer, f64::from(factor), horizontal, vertical);
            }
        }
    }
}

pub(super) fn zoom_at(state: &mut PlotState, transform: &Transform, pointer: Pos2, factor: f64, x: bool, y: bool) {
    let [cx, cy] = transform.to_scaled(pointer);
    let mut view = transform.view;
    if x {
        view.min_x = cx - (cx - view.min_x) / factor;
        view.max_x = cx + (view.max_x - cx) / factor;
    }
    if y {
        view.min_y = cy - (cy - view.min_y) / factor;
        view.max_y = cy + (view.max_y - cy) / factor;
    }
    if view.width() > 1e-12 && view.height() > 1e-12 && view.width() < 1e15 && view.height() < 1e15 {
        state.view = Some(view);
    }
}

/// Zoom to a screen box. A thin box zooms only along its long side.
fn zoom_to_box(state: &mut PlotState, transform: &Transform, rect: Rect) {
    let mut view = transform.view;
    let [x0, y0] = transform.to_scaled(rect.left_bottom());
    let [x1, y1] = transform.to_scaled(rect.right_top());
    if rect.width() >= 4.0 {
        (view.min_x, view.max_x) = (x0, x1);
    }
    if rect.height() >= 4.0 {
        (view.min_y, view.max_y) = (y0, y1);
    }
    if (rect.width() >= 4.0 || rect.height() >= 4.0) && view.width() > 1e-12 && view.height() > 1e-12 {
        state.view = Some(view);
    }
}
