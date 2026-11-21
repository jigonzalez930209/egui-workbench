//! Series as scene primitives, decimation, and hit-testing.

use egui::{Color32, Pos2, Rect, pos2, vec2};

use super::axis::Transform;
use super::scene::{Dash, Measure, Prim};
use super::series::{Kind, LineStyle, Marker, Series};

/// Lines with more points than this many per horizontal pixel are reduced to the
/// first/min/max/last point of every pixel column (M4), which is visually lossless.
const DECIMATE_PER_PIXEL: usize = 4;

/// The indices to draw for a sorted series: the visible range, decimated per pixel
/// column when dense. Unsorted series draw every point.
pub(super) fn visible_indices(transform: &Transform, series: &Series<'_>, sorted: bool) -> Vec<usize> {
    let data = series.data();
    if !sorted {
        return (0..data.len()).collect();
    }
    let min_x = transform.x_scale.inverse(transform.view.min_x);
    let max_x = transform.x_scale.inverse(transform.view.max_x);
    // One point beyond each edge keeps the line running out of the frame.
    let start = data.partition_point(|point| point[0] < min_x).saturating_sub(1);
    let end = (data.partition_point(|point| point[0] <= max_x) + 1).min(data.len());
    let columns = transform.frame.width().max(1.0) as usize;
    if end - start <= columns * DECIMATE_PER_PIXEL {
        return (start..end).collect();
    }
    let mut indices = Vec::with_capacity(columns * 4);
    let mut column = None;
    let (mut first, mut low, mut high, mut last) = (0, 0, 0, 0);
    let flush = |indices: &mut Vec<usize>, first: usize, low: usize, high: usize, last: usize| {
        let mut four = [first, low, high, last];
        four.sort_unstable();
        for index in four {
            if indices.last() != Some(&index) {
                indices.push(index);
            }
        }
    };
    for index in start..end {
        let point = data[index];
        if !point[1].is_finite() {
            continue;
        }
        let x = transform.to_screen(point).x.floor() as i64;
        if column != Some(x) {
            if column.is_some() {
                flush(&mut indices, first, low, high, last);
            }
            column = Some(x);
            (first, low, high) = (index, index, index);
        }
        if point[1] < data[low][1] {
            low = index;
        }
        if point[1] > data[high][1] {
            high = index;
        }
        last = index;
    }
    if column.is_some() {
        flush(&mut indices, first, low, high, last);
    }
    indices
}

/// The look of one series after the config overrides and the palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Look {
    pub color: Color32,
    /// Line width (lines, steps, areas, reference lines, whiskers).
    pub width: f32,
    pub style: LineStyle,
    /// Marker for scatter, or for line vertices when set.
    pub marker: Option<Marker>,
    pub radius: f32,
    pub fill_opacity: f32,
}

fn dash(style: LineStyle, width: f32) -> Dash {
    match style {
        LineStyle::Solid => Dash::Solid,
        LineStyle::Dashed => Dash::Dashed(6.0 * width.max(1.0) / 1.5, 4.0 * width.max(1.0) / 1.5),
        LineStyle::Dotted => Dash::Dotted((width * 2.6).max(3.0)),
    }
}

/// The primitives of one series (clipped to the frame by the caller).
pub(super) fn series_prims(
    out: &mut Vec<Prim>,
    measure: &Measure<'_>,
    transform: &Transform,
    series: &Series<'_>,
    look: &Look,
    indices: &[usize],
) {
    let data = series.data();
    let frame = transform.frame;
    let color = look.color;
    match series.kind {
        Kind::Line { fill_to, step, .. } => {
            for run in runs(transform, data, indices, step) {
                if let Some(base) = fill_to {
                    // A base a log axis cannot show (≤ 0) fills to the bottom edge.
                    let base_y = transform.to_screen([data[indices[0]][0], base]).y;
                    let base_y = if base_y.is_finite() {
                        base_y.clamp(frame.top(), frame.bottom())
                    } else {
                        frame.bottom()
                    };
                    out.push(Prim::Area {
                        points: run.clone(),
                        base_y,
                        color: color.gamma_multiply(look.fill_opacity),
                    });
                }
                out.push(Prim::Line {
                    points: run,
                    width: look.width,
                    color,
                    dash: dash(look.style, look.width),
                });
            }
            if let Some(marker) = look.marker {
                for &index in indices {
                    let position = transform.to_screen(data[index]);
                    if position.is_finite() {
                        marker_prims(out, position, marker, look.radius, color);
                    }
                }
            }
        }
        Kind::Scatter { marker, .. } => {
            let marker = look.marker.unwrap_or(marker);
            for &index in indices {
                let position = transform.to_screen(data[index]);
                if position.is_finite() && frame.expand(look.radius * 2.0).contains(position) {
                    marker_prims(out, position, marker, look.radius, color);
                }
            }
        }
        Kind::Bars { width, base } => {
            let width = width.unwrap_or_else(|| series.min_spacing() * 0.8);
            let fill = color.gamma_multiply(look.fill_opacity);
            for &index in indices {
                if let Some(rect) = bar_rect(transform, data[index], width, base) {
                    let rect = Rect::from_min_max(
                        pos2(measure.snap(rect.left()), measure.snap(rect.top())),
                        pos2(measure.snap(rect.right()), measure.snap(rect.bottom())),
                    );
                    out.push(Prim::Rect {
                        rect,
                        radius: 1.0,
                        fill,
                        stroke: None,
                    });
                }
            }
        }
        Kind::HLine(y) => {
            let y = measure.center(transform.to_screen([1.0, y]).y);
            if y.is_finite() {
                out.push(Prim::Line {
                    points: vec![pos2(frame.left(), y), pos2(frame.right(), y)],
                    width: look.width,
                    color,
                    dash: dash(look.style, look.width),
                });
            }
        }
        Kind::VLine(x) => {
            let x = measure.center(transform.to_screen([x, 1.0]).x);
            if x.is_finite() {
                out.push(Prim::Line {
                    points: vec![pos2(x, frame.top()), pos2(x, frame.bottom())],
                    width: look.width,
                    color,
                    dash: dash(look.style, look.width),
                });
            }
        }
    }
    if let Some(errors) = &series.y_errors {
        let width = (look.width * 0.7).max(1.0);
        for &index in indices {
            let (Some(point), Some(error)) = (data.get(index), errors.get(index)) else {
                continue;
            };
            let top = transform.to_screen([point[0], point[1] + error]);
            let bottom = transform.to_screen([point[0], point[1] - error]);
            if !(top.is_finite() && bottom.is_finite()) {
                continue;
            }
            let x = measure.center(top.x);
            let cap = (look.radius).max(3.0);
            for points in [
                vec![pos2(x, top.y), pos2(x, bottom.y)],
                vec![
                    pos2(x - cap, measure.center(top.y)),
                    pos2(x + cap, measure.center(top.y)),
                ],
                vec![
                    pos2(x - cap, measure.center(bottom.y)),
                    pos2(x + cap, measure.center(bottom.y)),
                ],
            ] {
                out.push(Prim::Line {
                    points,
                    width,
                    color,
                    dash: Dash::Solid,
                });
            }
        }
    }
}

/// Continuous screen-space runs (non-finite points split a line), expanded to steps.
fn runs(transform: &Transform, data: &[[f64; 2]], indices: &[usize], step: bool) -> Vec<Vec<Pos2>> {
    let mut runs = Vec::new();
    let mut current: Vec<Pos2> = Vec::with_capacity(indices.len() * if step { 2 } else { 1 });
    for &index in indices {
        let position = transform.to_screen(data[index]);
        if !position.is_finite() {
            if current.len() > 1 {
                runs.push(std::mem::take(&mut current));
            }
            current.clear();
            continue;
        }
        if step && let Some(previous) = current.last().copied() {
            current.push(pos2(position.x, previous.y));
        }
        current.push(position);
    }
    if current.len() > 1 {
        runs.push(current);
    }
    runs
}

/// A marker centered exactly on `center` (legend swatches use the same function).
pub(super) fn marker_prims(out: &mut Vec<Prim>, center: Pos2, marker: Marker, radius: f32, color: Color32) {
    let stroke = (radius * 0.45).clamp(1.0, 2.5);
    let line = |points: Vec<Pos2>| Prim::Line {
        points,
        width: stroke,
        color,
        dash: Dash::Solid,
    };
    match marker {
        Marker::Circle => out.push(Prim::Circle {
            center,
            radius,
            fill: color,
            stroke: None,
        }),
        Marker::Ring => out.push(Prim::Circle {
            center,
            radius,
            fill: Color32::TRANSPARENT,
            stroke: Some((stroke, color)),
        }),
        Marker::Square => out.push(Prim::Rect {
            rect: Rect::from_center_size(center, vec2(radius, radius) * 1.8),
            radius: 0.0,
            fill: color,
            stroke: None,
        }),
        Marker::Diamond => {
            let r = radius * 1.3;
            out.push(Prim::Polygon {
                points: vec![
                    center + vec2(0.0, -r),
                    center + vec2(r, 0.0),
                    center + vec2(0.0, r),
                    center + vec2(-r, 0.0),
                ],
                color,
            });
        }
        Marker::Triangle => {
            // Centered on the triangle's centroid, so it sits on the line it marks.
            let r = radius * 1.3;
            out.push(Prim::Polygon {
                points: vec![
                    center + vec2(0.0, -r),
                    center + vec2(r * 0.9, r * 0.5),
                    center + vec2(-r * 0.9, r * 0.5),
                ],
                color,
            });
        }
        Marker::Cross => {
            let r = radius;
            out.push(line(vec![center + vec2(-r, -r), center + vec2(r, r)]));
            out.push(line(vec![center + vec2(-r, r), center + vec2(r, -r)]));
        }
        Marker::Plus => {
            let r = radius * 1.2;
            out.push(line(vec![center + vec2(-r, 0.0), center + vec2(r, 0.0)]));
            out.push(line(vec![center + vec2(0.0, -r), center + vec2(0.0, r)]));
        }
    }
}

pub(super) fn bar_rect(transform: &Transform, point: [f64; 2], width: f64, base: f64) -> Option<Rect> {
    let left = transform.to_screen([point[0] - width * 0.5, point[1]]);
    let right = transform.to_screen([point[0] + width * 0.5, base]);
    let rect = Rect::from_two_pos(left, right);
    (rect.is_finite() && rect.width() > 0.0).then_some(rect)
}

/// The point of `series` nearest `pointer`, within `radius` points.
pub(super) fn nearest(
    transform: &Transform,
    series: &Series<'_>,
    pointer: Pos2,
    radius: f32,
    sorted: bool,
) -> Option<(usize, Pos2, f32)> {
    let data = series.data();
    if let Kind::Bars { width, base } = series.kind {
        let width = width.unwrap_or_else(|| series.min_spacing() * 0.8);
        return data.iter().enumerate().find_map(|(index, point)| {
            let rect = bar_rect(transform, *point, width, base)?;
            rect.contains(pointer)
                .then(|| (index, transform.to_screen(*point), 0.0))
        });
    }
    if series.is_reference() {
        return None;
    }
    let range = if sorted {
        let low = transform.to_data(pointer - vec2(radius, 0.0))[0];
        let high = transform.to_data(pointer + vec2(radius, 0.0))[0];
        data.partition_point(|point| point[0] < low)..data.partition_point(|point| point[0] <= high)
    } else {
        0..data.len()
    };
    let mut best: Option<(usize, Pos2, f32)> = None;
    for index in range {
        let position = transform.to_screen(data[index]);
        if !position.is_finite() {
            continue;
        }
        let distance = position.distance(pointer);
        if distance <= radius && best.is_none_or(|(_, _, current)| distance < current) {
            best = Some((index, position, distance));
        }
    }
    best
}
