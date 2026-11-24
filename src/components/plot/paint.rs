//! Series as scene primitives, decimation, and hit-testing.

use egui::{Color32, Pos2, Rect, Vec2, pos2, vec2};

use super::animation::{Effect, LegendAnimation};
use super::axis::Transform;
use super::decimate::{LINE_SPACING, MARKER_SPACING, SIMPLIFY_TOLERANCE, one_per_cell, simplify, thin};
use super::scene::{Dash, Measure, Prim};
use super::series::{Kind, LineStyle, Marker, Series, Smoothing};

/// Lines with more points than this many per horizontal pixel are reduced to the
/// first/min/max/last point of every pixel column (M4), which is visually lossless.
const DECIMATE_PER_PIXEL: usize = 4;

/// The indices to draw: for a sorted series the visible range, decimated per pixel
/// column when dense; for an unsorted one (a voltammogram, a Nyquist arc, a loop)
/// every point at least half a pixel from the last one drawn, which looks the same
/// and keeps a 50 000-point loop to the few thousand points the screen can show.
pub(super) fn visible_indices(transform: &Transform, series: &Series<'_>, sorted: bool, radius: f32) -> Vec<usize> {
    let data = series.data();
    if series.is_extra() {
        // Candles, heatmap cells, and waterfall bars are discrete: keep every point in
        // the visible x range instead of reducing them to a line per pixel column.
        if !sorted {
            return (0..data.len()).collect();
        }
        let min_x = transform.x_scale.inverse(transform.view.min_x);
        let max_x = transform.x_scale.inverse(transform.view.max_x);
        let start = data.partition_point(|point| point[0] < min_x).saturating_sub(1);
        let end = (data.partition_point(|point| point[0] <= max_x) + 1).min(data.len());
        return (start..end).collect();
    }
    if !sorted {
        return match series.kind {
            Kind::Line { .. } => thin(transform, data, LINE_SPACING),
            Kind::Scatter { .. } => one_per_cell(transform, data, (radius * MARKER_SPACING).max(1.0)),
            _ => (0..data.len()).collect(),
        };
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
    /// How lines pass through their points.
    pub smoothing: Smoothing,
    /// Skip line/scatter primitives because the GPU layer draws them.
    pub skip_gpu_lines: bool,
}

pub(super) fn dash(style: LineStyle, width: f32) -> Dash {
    match style {
        LineStyle::Solid => Dash::Solid,
        LineStyle::Dashed => Dash::Dashed(6.0 * width.max(1.0) / 1.5, 4.0 * width.max(1.0) / 1.5),
        LineStyle::Dotted => Dash::Dotted((width * 2.6).max(3.0)),
    }
}

/// The primitives of one series (clipped to the frame by the caller).
pub(super) fn series_prims(
    out: &mut Vec<Prim>,
    measure: &Measure,
    transform: &Transform,
    series: &Series<'_>,
    look: &Look,
    indices: &[usize],
    effect: Option<Effect>,
) {
    if look.skip_gpu_lines && matches!(series.kind, Kind::Line { .. } | Kind::Scatter { .. }) {
        return;
    }
    let data = series.data();
    let frame = transform.frame;
    let color = effect.map_or(look.color, |effect| effect.color(look.color));
    let width = effect.map_or(look.width, |effect| effect.width(look.width));
    match series.kind {
        Kind::Line { fill_to, step, .. } => {
            let smoothing = if step { Smoothing::None } else { look.smoothing };
            for run in runs(transform, data, indices, step) {
                let mut run = if smoothing == Smoothing::None {
                    simplify(run, SIMPLIFY_TOLERANCE)
                } else {
                    super::smooth::smooth(run, smoothing, frame.width())
                };
                if let Some(effect) = effect {
                    run = wiggle(run, effect);
                }
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
                if let Some(effect) = effect
                    && let Some(halo) = effect.halo()
                {
                    out.push(Prim::Line {
                        points: run.clone(),
                        width: width * 4.0,
                        color: halo,
                        dash: Dash::Solid,
                    });
                }
                let line = |points: Vec<Pos2>, color: Color32, width: f32, dash: Dash| Prim::Line {
                    points,
                    width,
                    color,
                    dash,
                };
                match effect {
                    Some(effect) if matches!(effect.kind, LegendAnimation::Reveal | LegendAnimation::Sweep) => {
                        let base = color.gamma_multiply(if effect.kind == LegendAnimation::Reveal {
                            0.22
                        } else {
                            0.7
                        });
                        out.push(line(
                            run.clone(),
                            base,
                            width,
                            effect.dash(dash(look.style, look.width)),
                        ));
                        if let Some(points) = effect.highlighted(&run) {
                            out.push(line(points, effect.highlight, width * 1.2, Dash::Solid));
                        }
                    }
                    Some(effect) => {
                        out.push(line(
                            run.clone(),
                            color,
                            width,
                            effect.dash(dash(look.style, look.width)),
                        ));
                        if effect.kind == LegendAnimation::Twinkle {
                            twinkle(out, &run, effect, width);
                        }
                    }
                    None => out.push(line(run.clone(), color, width, dash(look.style, look.width))),
                }
            }
            if let Some(marker) = look.marker {
                for (slot, &index) in indices.iter().enumerate() {
                    let offset = effect.map_or(Vec2::ZERO, |effect| effect.marker_offset(slot));
                    let position = transform.to_screen(data[index]) + offset;
                    if position.is_finite() {
                        let radius = effect.map_or(look.radius, |effect| effect.marker_radius(look.radius, slot));
                        marker_prims(out, position, marker, radius, color);
                    }
                }
            }
        }
        Kind::Scatter { marker, .. } => {
            let marker = look.marker.unwrap_or(marker);
            for (slot, &index) in indices.iter().enumerate() {
                let offset = effect.map_or(Vec2::ZERO, |effect| effect.marker_offset(slot));
                let point = transform.to_screen(data[index]) + offset;
                if point.is_finite() && frame.expand(look.radius * 2.0).contains(point) {
                    let radius = effect.map_or(look.radius, |effect| effect.marker_radius(look.radius, slot));
                    marker_prims(out, point, marker, radius, color);
                }
            }
        }
        Kind::Bars { width: bar_width, base } => {
            let bar_width = bar_width.unwrap_or_else(|| series.min_spacing() * 0.8);
            let fill = color.gamma_multiply(look.fill_opacity);
            for &index in indices {
                if let Some(rect) = bar_rect(transform, data[index], bar_width, base) {
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
                    width,
                    color,
                    dash: effect.map_or_else(
                        || dash(look.style, look.width),
                        |effect| effect.dash(dash(look.style, look.width)),
                    ),
                });
            }
        }
        Kind::VLine(x) => {
            let x = measure.center(transform.to_screen([x, 1.0]).x);
            if x.is_finite() {
                out.push(Prim::Line {
                    points: vec![pos2(x, frame.top()), pos2(x, frame.bottom())],
                    width,
                    color,
                    dash: effect.map_or_else(
                        || dash(look.style, look.width),
                        |effect| effect.dash(dash(look.style, look.width)),
                    ),
                });
            }
        }
        Kind::Pie { .. }
        | Kind::Candles { .. }
        | Kind::Heatmap
        | Kind::Radar
        | Kind::Gauge { .. }
        | Kind::Waterfall { .. } => {}
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

/// Displace each point perpendicular to the local tangent (the wiggle effect).
fn wiggle(points: Vec<Pos2>, effect: Effect) -> Vec<Pos2> {
    let count = points.len();
    (0..count)
        .map(|index| {
            let tangent = if index == 0 {
                points[1] - points[0]
            } else if index + 1 == count {
                points[index] - points[index - 1]
            } else {
                points[index + 1] - points[index - 1]
            };
            points[index] + effect.point_offset(index, tangent)
        })
        .collect()
}

/// Sparkles along a line (the twinkle effect).
fn twinkle(out: &mut Vec<Prim>, points: &[Pos2], effect: Effect, width: f32) {
    let stride = (points.len() / 18).max(1);
    for (index, point) in points.iter().enumerate().step_by(stride) {
        let radius = 1.2 + 1.8 * (index as f32 * 0.9 + effect.phase * std::f32::consts::TAU).sin().abs();
        out.push(Prim::Circle {
            center: *point,
            radius: radius.max(width * 0.5),
            fill: effect.highlight,
            stroke: None,
        });
    }
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
    drawn: Option<&[usize]>,
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
    let mut best: Option<(usize, Pos2, f32)> = None;
    let mut consider = |index: usize| {
        let position = transform.to_screen(data[index]);
        if !position.is_finite() {
            return;
        }
        let distance = position.distance(pointer);
        if distance <= radius && best.is_none_or(|(_, _, current)| distance < current) {
            best = Some((index, position, distance));
        }
    };
    if sorted {
        let low = transform.to_data(pointer - vec2(radius, 0.0))[0];
        let high = transform.to_data(pointer + vec2(radius, 0.0))[0];
        let range = data.partition_point(|point| point[0] < low)..data.partition_point(|point| point[0] <= high);
        range.for_each(&mut consider);
        return best;
    }
    match (drawn, &series.kind) {
        // Scatter: the markers on screen are the ones to hover.
        (Some(drawn), Kind::Scatter { .. }) => drawn.iter().for_each(|&index| consider(index)),
        // Line: a skipped point lies within LINE_SPACING of the drawn point before it,
        // so only the stretches after drawn points near the pointer are searched, and
        // the result is the exact nearest point.
        (Some(drawn), Kind::Line { .. }) => {
            for (position, &index) in drawn.iter().enumerate() {
                let screen = transform.to_screen(data[index]);
                if !screen.is_finite() || screen.distance(pointer) > radius + LINE_SPACING {
                    continue;
                }
                let end = drawn.get(position + 1).copied().unwrap_or(index + 1);
                (index..end).for_each(&mut consider);
            }
        }
        _ => (0..data.len()).for_each(&mut consider),
    }
    best
}
