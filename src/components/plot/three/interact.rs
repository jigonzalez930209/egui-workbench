//! 3D camera interaction and hover search.

use egui::{CursorIcon, Pos2, Response, Ui};

use super::camera::Projector;
use super::paint::{HoverPoint, Look3D};
use super::types::Drag;
use super::widgets;
use super::{Plot3D, Plot3DHover, Plot3DState, Plot3DTool};

/// Orbit, pan, wheel zoom, and double-click reset.
pub(super) fn interact(
    ui: &Ui,
    response: &Response,
    frame: egui::Rect,
    state: &mut Plot3DState,
    camera: super::Camera3D,
) {
    if response.double_clicked() {
        state.reset_view();
        return;
    }
    let (shift, alt) = ui.input(|input| (input.modifiers.shift, input.modifiers.alt));
    if response.drag_started() && response.interact_pointer_pos().is_some_and(|pos| frame.contains(pos)) {
        let secondary = response.dragged_by(egui::PointerButton::Secondary);
        state.drag = Some(if shift || secondary || state.tool == Plot3DTool::Pan {
            Drag::Pan
        } else {
            Drag::Orbit
        });
    }
    match state.drag {
        Some(Drag::Orbit) if response.dragged() => {
            state.set_camera(camera.orbit(response.drag_delta()));
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            ui.ctx().request_repaint();
        }
        Some(Drag::Pan) if response.dragged() => {
            state.set_camera(camera.pan(response.drag_delta(), frame));
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            ui.ctx().request_repaint();
        }
        Some(_) if !response.dragged() && !response.drag_stopped() => state.drag = None,
        _ => {}
    }
    if state.drag.is_none() && response.hover_pos().is_some_and(|pos| frame.contains(pos)) {
        ui.ctx().set_cursor_icon(if shift || alt {
            CursorIcon::Grabbing
        } else {
            CursorIcon::Grab
        });
    }
    if response.hover_pos().is_some_and(|pos| frame.contains(pos)) {
        let (scroll, zoom) = ui.input(|input| (input.smooth_scroll_delta, input.zoom_delta()));
        let wheel = scroll.y + scroll.x;
        let factor = if (zoom - 1.0).abs() > f32::EPSILON {
            f64::from(zoom)
        } else if wheel != 0.0 {
            (f64::from(wheel) * 0.002).exp()
        } else {
            1.0
        };
        if (factor - 1.0).abs() > f64::EPSILON {
            ui.ctx().input_mut(|input| input.smooth_scroll_delta = egui::Vec2::ZERO);
            state.set_camera(camera.zoom(factor));
            ui.ctx().request_repaint();
        }
    }
}

/// The nearest projected point under the pointer, with its ring, drop line, and popup.
pub(super) fn hover(
    ui: &Ui,
    plot: &Plot3D<'_>,
    projector: &Projector,
    pointer: Pos2,
    theme: &crate::theme::Theme,
    bounds: super::Bounds3D,
    drawn: &[(usize, Vec<HoverPoint>)],
) -> Option<Plot3DHover> {
    let mut best: Option<(usize, HoverPoint, f32)> = None;
    for (series_index, points) in drawn {
        for point in points {
            let distance = point.pos.distance(pointer);
            if distance <= 12.0
                && best.is_none_or(|(_, current, best)| {
                    distance < best || (distance == best && point.depth < current.depth)
                })
            {
                best = Some((*series_index, *point, distance));
            }
        }
    }
    let (series_index, point, _) = best?;
    let series = &plot.series[series_index];
    let value = series.point(point.index)?;
    let look: Look3D = super::paint::look(plot, series_index, theme);
    let visuals = ui.visuals();
    let guide = plot.config.hover_guides.then(|| visuals.weak_text_color());
    let painter = ui.painter().with_clip_rect(projector.frame());
    widgets::highlight(
        &painter,
        projector,
        value,
        bounds.min_z,
        look.color,
        visuals.extreme_bg_color,
        guide,
    );
    if plot.config.hover_popup {
        widgets::tooltip(ui, plot, series.name(), look.color, value, point.pos);
    }
    Some(Plot3DHover {
        series: series.name.clone(),
        index: point.index,
        point: value,
    })
}
