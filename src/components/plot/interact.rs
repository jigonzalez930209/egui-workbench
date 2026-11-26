//! Pointer interaction: pan, wheel zoom, box zoom, selection, and reset.

use egui::{CursorIcon, Pos2, Rect, Response, Ui, Vec2};

use super::axis::{Bounds, Scale, Transform};
use super::series::{self, YAxis};
use super::{
    Drag, MeasurementMode, Plot, PlotHover, PlotMeasurement, PlotState, PlotTool, overlay, paint, snap, widgets,
};

impl Plot<'_> {
    /// The extents of the visible series in scaled space, split by y axis: (left, right).
    fn extents(&self, state: &PlotState) -> (Bounds, Bounds) {
        let x_scale = self.config.x.scale;
        let scale_of = |axis: YAxis| match axis {
            YAxis::Left => self.config.y.scale,
            YAxis::Right => self.config.y2.scale,
        };
        let (mut left, mut right) = (Bounds::NOTHING, Bounds::NOTHING);
        for series in self.visible(state) {
            let axis = self.axis_of(series);
            let bounds = state.cached_bounds(series, x_scale, scale_of(axis));
            match axis {
                YAxis::Left => left.union(&bounds),
                YAxis::Right => right.union(&bounds),
            }
        }
        if self.include.is_valid() {
            left.extend([
                x_scale.forward(self.include.min_x),
                self.config.y.scale.forward(self.include.min_y),
            ]);
        }
        for series in self.visible(state) {
            let axis = self.axis_of(series);
            let target = match axis {
                YAxis::Left => &mut left,
                YAxis::Right => &mut right,
            };
            match series.kind {
                series::Kind::HLine(y) if target.is_valid() => {
                    target.extend([target.min_x, scale_of(axis).forward(y)]);
                }
                series::Kind::VLine(x) if target.is_valid() => {
                    target.extend([x_scale.forward(x), target.min_y]);
                }
                _ => {}
            }
        }
        (left, right)
    }

    /// The automatic view: every visible series plus 5 % margins, in scaled space,
    /// with the fixed axis limits of the config applied on top. The x range spans the
    /// series of both y axes; the y range only those on the left.
    pub(super) fn auto_bounds(&self, state: &PlotState) -> Bounds {
        let (x_scale, y_scale) = (self.config.x.scale, self.config.y.scale);
        let (mut bounds, right) = self.extents(state);
        if right.is_valid() {
            if bounds.is_valid() {
                bounds.min_x = bounds.min_x.min(right.min_x);
                bounds.max_x = bounds.max_x.max(right.max_x);
            } else {
                bounds = Bounds {
                    min_y: 0.0,
                    max_y: 1.0,
                    ..right
                };
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
        pad(&mut bounds.min_x, &mut bounds.max_x);
        pad(&mut bounds.min_y, &mut bounds.max_y);
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

    /// The automatic range of the right axis in scaled space: its series plus 5 %
    /// margins, with its fixed limits on top.
    pub(super) fn auto_y2(&self, state: &PlotState) -> [f64; 2] {
        let (_, right) = self.extents(state);
        let (mut min, mut max) = if right.is_valid() {
            (right.min_y, right.max_y)
        } else {
            (0.0, 1.0)
        };
        pad(&mut min, &mut max);
        let scale = self.config.y2.scale;
        if let Some(fixed) = limit(self.config.y2.min, scale) {
            min = fixed;
        }
        if let Some(fixed) = limit(self.config.y2.max, scale) {
            max = fixed;
        }
        if max <= min {
            max = min + 1.0;
        }
        [min, max]
    }

    /// The nearest point over every visible series, highlighted with its popup.
    pub(super) fn hover(
        &self,
        ui: &Ui,
        transform: &Transform,
        state: &PlotState,
        pointer: Pos2,
        theme: &crate::theme::Theme,
        drawn: &[Option<Vec<usize>>],
    ) -> Option<PlotHover> {
        let mut best: Option<(usize, usize, Pos2, f32)> = None;
        for (series_index, series) in self.series.iter().enumerate() {
            if state.is_hidden(&series.name) {
                continue;
            }
            let transform = transform.for_axis(self.axis_of(series));
            if let Some((index, position, distance)) = paint::nearest(
                &transform,
                series,
                pointer,
                12.0,
                series.sorted_x(),
                drawn.get(series_index).and_then(Option::as_deref),
            ) && best.is_none_or(|(_, _, _, current)| distance < current)
            {
                best = Some((series_index, index, position, distance));
            }
        }
        let (series_index, index, position, _) = best?;
        let series = &self.series[series_index];
        let axis = self.axis_of(series);
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
            axis == YAxis::Right,
        );
        if self.config.hover_popup {
            widgets::tooltip(ui, self, series.name(), look.color, point, position, axis);
        }
        Some(PlotHover {
            series: series.name.clone(),
            index,
            point,
            axis,
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
            state.reset_view();
            state.drag = None;
            return;
        }
        let (shift, alt) = ui.input(|input| (input.modifiers.shift, input.modifiers.alt));
        if response.drag_started() && response.interact_pointer_pos().is_some_and(|pos| frame.contains(pos)) {
            let start = ui
                .input(|input| input.pointer.press_origin())
                .or_else(|| response.interact_pointer_pos())
                .unwrap_or(frame.center());
            let secondary = response.dragged_by(egui::PointerButton::Secondary);
            state.drag = Some(if state.tool == PlotTool::MeasureDistance {
                Drag::Measure { start, distance: true }
            } else if state.tool == PlotTool::MeasureArea {
                Drag::Measure { start, distance: false }
            } else if shift || (state.tool == PlotTool::Select && !secondary) {
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
                if let Some(y2) = transform.y2 {
                    let per_point = (y2.max - y2.min) / f64::from(frame.height().max(1.0));
                    let shift = f64::from(delta.y) * per_point;
                    state.y2_view = Some([y2.min + shift, y2.max + shift]);
                }
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
            Some(Drag::Measure { start, distance }) if response.drag_stopped() => {
                let end = response.interact_pointer_pos().unwrap_or(start);
                let radius = self.config.measurement.snap_radius;
                if distance {
                    let from = snap::snap_point(self, state, transform, start, radius);
                    let to = snap::snap_point(self, state, transform, end, radius);
                    if from != to && from.iter().chain(to.iter()).all(|value| value.is_finite()) {
                        state.add_measurement(PlotMeasurement::distance(from, to));
                    }
                } else if self.config.measurement.mode == MeasurementMode::CurveSegment {
                    if let Some((series, start_index, from)) = snap::snap_hit(self, state, transform, start, radius)
                        && let Some((end_index, to)) =
                            snap::nearest_on_series(self, state, transform, &series, end, radius)
                        && start_index != end_index
                    {
                        state.add_measurement(PlotMeasurement::curve_area(series, from, to, start_index, end_index));
                    }
                } else {
                    let rect = Rect::from_two_pos(start, end).intersect(frame);
                    if rect.width() >= 3.0 && rect.height() >= 3.0 {
                        let a = transform.to_data(rect.left_bottom());
                        let b = transform.to_data(rect.right_top());
                        state.add_measurement(PlotMeasurement::area(Bounds::from_corners(a, b)));
                    }
                }
                state.drag = None;
            }
            Some(_) if !response.dragged() && !response.drag_stopped() => state.drag = None,
            _ => {}
        }
        // Inside the frame the cursor is a crosshair; pressing to pan turns it into a
        // closed hand until the button is released.
        let pressed = response.is_pointer_button_down_on() || response.dragged();
        if pointer.is_some() || state.drag.is_some() {
            let panning = match state.drag {
                Some(Drag::Pan) => true,
                Some(Drag::Box { .. } | Drag::Measure { .. }) => false,
                None => {
                    pressed
                        && state.tool == PlotTool::Pan
                        && !shift
                        && !ui.input(|input| input.pointer.secondary_down())
                }
            };
            ui.ctx().set_cursor_icon(if panning {
                CursorIcon::Grabbing
            } else {
                CursorIcon::Crosshair
            });
        }
        // Over the frame the wheel follows the modifiers; over an axis's ticks (outside
        // the frame, inside the plot) it zooms that axis alone, like Shift/Alt do.
        let target = match pointer {
            Some(pointer) => Some((pointer, wheel_axes(transform, pointer, shift, alt))),
            None if state.drag.is_none() => response
                .hover_pos()
                .and_then(|pos| axis_under(transform, pos).map(|axes| (pos, axes))),
            None => None,
        };
        if let Some((pointer, axes)) = target {
            if !frame.contains(pointer) {
                ui.ctx().set_cursor_icon(if axes.x {
                    CursorIcon::ResizeHorizontal
                } else {
                    CursorIcon::ResizeVertical
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
                zoom_at(state, transform, pointer, f64::from(factor), axes);
            }
        }
    }
}

/// The axis whose tick band is under `pos`: below the frame is x, left of it the left
/// y axis, right of it the right y axis (when shown). Corners belong to no axis.
pub(super) fn axis_under(transform: &Transform, pos: Pos2) -> Option<ZoomAxes> {
    let frame = transform.frame;
    let within_x = (frame.left()..=frame.right()).contains(&pos.x);
    let within_y = (frame.top()..=frame.bottom()).contains(&pos.y);
    let only = |x, y, y2| ZoomAxes { x, y, y2 };
    if pos.y > frame.bottom() && within_x {
        Some(only(true, false, false))
    } else if pos.x < frame.left() && within_y {
        Some(only(false, true, false))
    } else if pos.x > frame.right() && within_y && transform.y2.is_some() {
        Some(only(false, false, true))
    } else {
        None
    }
}

/// The span every automatic range is padded by on both sides.
fn pad(min: &mut f64, max: &mut f64) {
    let span = *max - *min;
    let span = if span > 0.0 { span } else { min.abs().max(1.0) * 0.2 };
    let margin = span * 0.05;
    *min -= margin;
    *max += margin;
}

/// A fixed limit in scaled units, when it can be shown on `scale`.
fn limit(value: Option<f64>, scale: Scale) -> Option<f64> {
    value.map(|value| scale.forward(value)).filter(|v| v.is_finite())
}

/// Which axes a zoom acts on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ZoomAxes {
    pub x: bool,
    pub y: bool,
    pub y2: bool,
}

impl ZoomAxes {
    pub const ALL: Self = Self {
        x: true,
        y: true,
        y2: true,
    };
}

/// The wheel zooms every axis; Shift only x; Alt only y: the left axis over the left
/// half of the frame and the right axis over the right half when there are two.
pub(super) fn wheel_axes(transform: &Transform, pointer: Pos2, shift: bool, alt: bool) -> ZoomAxes {
    if shift {
        return ZoomAxes {
            x: true,
            y: false,
            y2: false,
        };
    }
    if alt {
        let right = transform.y2.is_some() && pointer.x > transform.frame.center().x;
        return ZoomAxes {
            x: false,
            y: !right,
            y2: right,
        };
    }
    ZoomAxes::ALL
}

pub(super) fn zoom_at(state: &mut PlotState, transform: &Transform, pointer: Pos2, factor: f64, axes: ZoomAxes) {
    let [cx, cy] = transform.to_scaled(pointer);
    let mut view = transform.view;
    if axes.x {
        view.min_x = cx - (cx - view.min_x) / factor;
        view.max_x = cx + (view.max_x - cx) / factor;
    }
    if axes.y {
        view.min_y = cy - (cy - view.min_y) / factor;
        view.max_y = cy + (view.max_y - cy) / factor;
    }
    let y2 = transform.y2.map(|axis| {
        let (mut min, mut max) = (axis.min, axis.max);
        if axes.y2 {
            let center = transform.for_axis(YAxis::Right).to_scaled(pointer)[1];
            min = center - (center - min) / factor;
            max = center + (max - center) / factor;
        }
        [min, max]
    });
    let sane = |span: f64| span > 1e-12 && span < 1e15;
    if sane(view.width()) && sane(view.height()) && y2.is_none_or(|[min, max]| sane(max - min)) {
        state.view = Some(view);
        if y2.is_some() {
            state.y2_view = y2;
        }
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
        if let Some(axis) = transform.y2 {
            let right = transform.for_axis(YAxis::Right);
            let (low, high) = (
                right.to_scaled(rect.left_bottom())[1],
                right.to_scaled(rect.right_top())[1],
            );
            state.y2_view = Some(if rect.height() >= 4.0 && high - low > 1e-12 {
                [low, high]
            } else {
                [axis.min, axis.max]
            });
        }
    }
}
