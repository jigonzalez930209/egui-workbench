//! 3D series primitives: projected lines, markers, bar cuboids, and surface cells.

use egui::{Color32, Pos2, Vec2};

use super::super::animation::{Effect, LegendAnimation};
use super::super::direction;
use super::super::extra::heat_color;
use super::super::scene::{Dash, Prim, Scene};
use super::super::series::{LineStyle, Marker};
use super::Plot3D;
use super::camera::Projector;
use super::config::Plot3DConfig;
use super::series::{Kind3D, Series3D};
use super::types::Bounds3D;

/// The look of one 3D series after config overrides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Look3D {
    pub color: Color32,
    pub width: f32,
    pub style: LineStyle,
    pub marker: Option<Marker>,
    pub radius: f32,
    pub fill_opacity: f32,
}

pub(super) fn look(plot: &Plot3D<'_>, index: usize, theme: &crate::theme::Theme) -> Look3D {
    let series = &plot.series[index];
    let style = plot.config.series_style(&series.name);
    let color = style
        .and_then(|style| style.color)
        .or(series.color)
        .unwrap_or_else(|| super::super::palette(theme, index));
    Look3D {
        color,
        width: style.and_then(|style| style.width).unwrap_or(series.width) * plot.config.line_scale,
        style: style.and_then(|style| style.line_style).unwrap_or(series.line_style),
        marker: style
            .and_then(|style| style.marker)
            .or(series.marker)
            .or((series.kind == Kind3D::Scatter).then_some(Marker::Circle)),
        radius: style.and_then(|style| style.marker_size).unwrap_or(series.radius) * plot.config.marker_scale,
        fill_opacity: style.and_then(|style| style.fill_opacity).unwrap_or(0.86),
    }
}

/// One projected point kept for hover search.
#[derive(Clone, Copy, Debug)]
pub(super) struct HoverPoint {
    pub index: usize,
    pub pos: Pos2,
    pub depth: f64,
}

/// The primitives of one series and the points hover should search.
#[allow(clippy::too_many_arguments)]
pub(super) fn series(
    scene: &mut Scene,
    projector: &Projector,
    series: &Series3D<'_>,
    look: &Look3D,
    effect: Option<Effect>,
    config: &Plot3DConfig,
    theme: &crate::theme::Theme,
    bounds: Bounds3D,
    direction: bool,
) -> Vec<HoverPoint> {
    match series.kind {
        Kind3D::Line => line(scene, projector, series, look, effect, config, direction),
        Kind3D::Scatter => scatter(scene, projector, series, look, effect),
        Kind3D::Bars => bars(scene, projector, series, look, effect, theme),
        Kind3D::Surface => surface(scene, projector, series, look, effect, config, theme, bounds),
    }
}

fn line(
    scene: &mut Scene,
    projector: &Projector,
    series: &Series3D<'_>,
    look: &Look3D,
    effect: Option<Effect>,
    config: &Plot3DConfig,
    direction: bool,
) -> Vec<HoverPoint> {
    let color = effect.map_or(look.color, |effect| effect.color(look.color));
    let width = effect.map_or(look.width, |effect| effect.width(look.width));
    let dash = effect.map_or_else(
        || super::super::paint::dash(look.style, look.width),
        |effect| effect.dash(super::super::paint::dash(look.style, look.width)),
    );
    let mut hover = Vec::with_capacity(series.data.len());
    let mut run = Vec::new();
    for (index, point) in series.data.iter().enumerate() {
        let Some((pos, depth)) = projector.project(*point) else {
            flush(scene, &mut run, color, width, dash, effect);
            continue;
        };
        hover.push(HoverPoint { index, pos, depth });
        run.push(pos);
    }
    flush(scene, &mut run, color, width, dash, effect);
    if let Some(marker) = look.marker {
        for (slot, point) in hover.iter().enumerate() {
            let offset = effect.map_or(Vec2::ZERO, |effect| effect.marker_offset(slot));
            let radius = effect.map_or(look.radius, |effect| effect.marker_radius(look.radius, slot));
            super::super::paint::marker_prims(&mut scene.data, point.pos + offset, marker, radius, color);
        }
    }
    if direction
        && series.has_direction()
        && series.data.len() >= 2
        && config.direction.enabled
        && let Some(last) = series.data.last()
        && last.iter().all(|value| value.is_finite())
    {
        let mut samples: Vec<Pos2> = hover
            .iter()
            .rev()
            .take(config.direction.points.clamp(2, 64))
            .map(|point| point.pos)
            .collect();
        samples.reverse();
        direction::arrow(&mut scene.data, projector.frame(), &samples, &config.direction, color);
    }
    hover
}

fn flush(scene: &mut Scene, run: &mut Vec<Pos2>, color: Color32, width: f32, dash: Dash, effect: Option<Effect>) {
    if run.len() >= 2 {
        let mut points = std::mem::take(run);
        if let Some(effect) = effect
            && effect.kind == LegendAnimation::Wiggle
        {
            points = wiggle(points, effect);
        }
        if let Some(halo) = effect.and_then(Effect::halo) {
            scene.data.push(Prim::Line {
                points: points.clone(),
                width: width * 4.0,
                color: halo,
                dash: Dash::Solid,
            });
        }
        let highlighted = effect.and_then(|effect| effect.highlighted(&points));
        let reveal =
            effect.is_some_and(|effect| matches!(effect.kind, LegendAnimation::Reveal | LegendAnimation::Sweep));
        if reveal {
            scene.data.push(Prim::Line {
                points: points.clone(),
                width,
                color: color.gamma_multiply(0.22),
                dash,
            });
            if let Some(highlighted) = highlighted {
                scene.data.push(Prim::Line {
                    points: highlighted,
                    width: width * 1.2,
                    color: effect.map_or(color, |effect| effect.highlight),
                    dash: Dash::Solid,
                });
            }
        } else {
            scene.data.push(Prim::Line {
                points: points.clone(),
                width,
                color,
                dash,
            });
        }
        if let Some(effect) = effect
            && effect.kind == LegendAnimation::Twinkle
        {
            let stride = (points.len() / 18).max(1);
            for (index, point) in points.iter().enumerate().step_by(stride) {
                let radius = 1.2 + 1.8 * (index as f32 * 0.9 + effect.phase * std::f32::consts::TAU).sin().abs();
                scene.data.push(Prim::Circle {
                    center: *point,
                    radius: radius.max(width * 0.5),
                    fill: effect.highlight,
                    stroke: None,
                });
            }
        }
    } else {
        run.clear();
    }
}

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

fn scatter(
    scene: &mut Scene,
    projector: &Projector,
    series: &Series3D<'_>,
    look: &Look3D,
    effect: Option<Effect>,
) -> Vec<HoverPoint> {
    let color = effect.map_or(look.color, |effect| effect.color(look.color));
    let marker = look.marker.unwrap_or(Marker::Circle);
    let mut points: Vec<HoverPoint> = series
        .data
        .iter()
        .enumerate()
        .filter_map(|(index, point)| {
            projector
                .project(*point)
                .map(|(pos, depth)| HoverPoint { index, pos, depth })
        })
        .collect();
    points.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    for (slot, point) in points.iter().enumerate() {
        let offset = effect.map_or(Vec2::ZERO, |effect| effect.marker_offset(slot));
        let radius = effect.map_or(look.radius, |effect| effect.marker_radius(look.radius, slot));
        super::super::paint::marker_prims(&mut scene.data, point.pos + offset, marker, radius, color);
    }
    points
}

#[allow(clippy::too_many_arguments)]
fn bars(
    scene: &mut Scene,
    projector: &Projector,
    series: &Series3D<'_>,
    look: &Look3D,
    effect: Option<Effect>,
    theme: &crate::theme::Theme,
) -> Vec<HoverPoint> {
    let width = series.bar_width_value() * 0.5;
    let base = series.base;
    let eye = projector.eye();
    let mut hover = Vec::with_capacity(series.data.len());
    let mut faces: Vec<(f64, Prim, Option<Prim>)> = Vec::new();
    for (index, point) in series.data.iter().enumerate() {
        let [x, y, value] = *point;
        if !(x.is_finite() && y.is_finite() && value.is_finite()) {
            continue;
        }
        let z0 = base.min(value);
        let z1 = base.max(value);
        let corners = [
            [x - width, y - width, z0],
            [x + width, y - width, z0],
            [x - width, y + width, z0],
            [x + width, y + width, z0],
            [x - width, y - width, z1],
            [x + width, y - width, z1],
            [x - width, y + width, z1],
            [x + width, y + width, z1],
        ];
        let mut projected = [Pos2::ZERO; 8];
        let mut ok = true;
        for (slot, corner) in corners.iter().enumerate() {
            match projector.project(*corner) {
                Some((pos, _)) => projected[slot] = pos,
                None => ok = false,
            }
        }
        if !ok {
            continue;
        }
        let top = [x, y, value];
        if let Some((pos, depth)) = projector.project(top) {
            hover.push(HoverPoint { index, pos, depth });
        }
        let base_color = if value >= base { look.color } else { theme.danger };
        let color = effect.map_or(base_color, |effect| effect.color(base_color));
        let fill = color.gamma_multiply(look.fill_opacity);
        let outline = color.gamma_multiply(0.55);
        for (indices, normal, shade) in [
            ([4, 5, 7, 6], [0.0, 0.0, 1.0], 1.0),
            ([0, 1, 5, 4], [0.0, -1.0, 0.0], 0.72),
            ([1, 3, 7, 5], [1.0, 0.0, 0.0], 0.84),
            ([3, 2, 6, 7], [0.0, 1.0, 0.0], 0.78),
            ([2, 0, 4, 6], [-1.0, 0.0, 0.0], 0.66),
        ] {
            let face_center = [
                indices.iter().map(|slot| corners[*slot][0]).sum::<f64>() / 4.0,
                indices.iter().map(|slot| corners[*slot][1]).sum::<f64>() / 4.0,
                indices.iter().map(|slot| corners[*slot][2]).sum::<f64>() / 4.0,
            ];
            let towards = [
                eye[0] - face_center[0],
                eye[1] - face_center[1],
                eye[2] - face_center[2],
            ];
            if normal[0] * towards[0] + normal[1] * towards[1] + normal[2] * towards[2] <= 0.0 {
                continue;
            }
            let points: Vec<Pos2> = indices.iter().map(|slot| projected[*slot]).collect();
            // An edge-on face projects to a line or a sliver. Filling that with the
            // anti-aliased polygon tessellator produces long spikes; drop it instead.
            if polygon_area(&points) < 8.0 {
                continue;
            }
            let depth = indices.iter().map(|slot| projector.depth(corners[*slot])).sum::<f64>() / 4.0;
            let face_color = if shade < 1.0 {
                color.gamma_multiply(shade).gamma_multiply(look.fill_opacity)
            } else {
                fill
            };
            let outline = Prim::Line {
                points: points.iter().copied().chain(std::iter::once(points[0])).collect(),
                width: 1.0,
                color: outline,
                dash: Dash::Solid,
            };
            faces.push((
                depth,
                Prim::Quad {
                    points: [points[0], points[1], points[2], points[3]],
                    color: face_color,
                },
                Some(outline),
            ));
        }
    }
    faces.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (_, fill, outline) in faces {
        scene.data.push(fill);
        if let Some(outline) = outline {
            scene.data.push(outline);
        }
    }
    hover
}

#[allow(clippy::too_many_arguments)]
fn surface(
    scene: &mut Scene,
    projector: &Projector,
    series: &Series3D<'_>,
    look: &Look3D,
    effect: Option<Effect>,
    config: &Plot3DConfig,
    theme: &crate::theme::Theme,
    bounds: Bounds3D,
) -> Vec<HoverPoint> {
    let Some(grid) = &series.surface else {
        return Vec::new();
    };
    let (columns, rows) = (grid.columns(), grid.rows());
    if columns < 2 || rows < 2 {
        return Vec::new();
    }
    let mut hover = Vec::with_capacity(columns * rows);
    let mut projected = Vec::with_capacity(columns * rows);
    for index in 0..columns * rows {
        match grid.point(index) {
            Some(point) => {
                if let Some((pos, depth)) = projector.project(point) {
                    hover.push(HoverPoint { index, pos, depth });
                    projected.push(Some(pos));
                } else {
                    projected.push(None);
                }
            }
            None => projected.push(None),
        }
    }
    let span = (bounds.max_z - bounds.min_z).max(1e-12);
    let mut faces: Vec<(f64, Option<Prim>, Option<Prim>)> = Vec::new();
    for row in 0..rows - 1 {
        for column in 0..columns - 1 {
            let slots = [
                row * columns + column,
                row * columns + column + 1,
                (row + 1) * columns + column + 1,
                (row + 1) * columns + column,
            ];
            let (Some(a), Some(b), Some(c), Some(d)) = (
                projected[slots[0]],
                projected[slots[1]],
                projected[slots[2]],
                projected[slots[3]],
            ) else {
                continue;
            };
            let values: Vec<f64> = slots
                .iter()
                .filter_map(|slot| grid.point(*slot).map(|point| point[2]))
                .collect();
            if values.len() != 4 {
                continue;
            }
            let average = values.iter().sum::<f64>() / 4.0;
            let base = if config.surface.color_scale {
                heat_color(((average - bounds.min_z) / span) as f32, theme)
            } else {
                look.color
            };
            let color = effect.map_or(base, |effect| effect.color(base));
            let depth = slots
                .iter()
                .filter_map(|slot| grid.point(*slot))
                .map(|point| projector.depth(point))
                .sum::<f64>()
                / 4.0;
            // A projected cell can be concave (the four z values need not be planar),
            // so a quad mesh fills it without the convex-polygon tessellator.
            let fill = Prim::Quad {
                points: [a, b, c, d],
                color: color.gamma_multiply(config.surface.opacity),
            };
            let outline = config.surface.wireframe.then(|| Prim::Line {
                points: vec![a, b, c, d, a],
                width: look.width * 0.7,
                color: color.gamma_multiply(0.55),
                dash: Dash::Solid,
            });
            faces.push((depth, Some(fill), outline));
        }
    }
    faces.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (_, fill, outline) in faces {
        if let Some(fill) = fill {
            scene.data.push(fill);
        }
        if let Some(outline) = outline {
            scene.data.push(outline);
        }
    }
    hover
}

/// Absolute shoelace area of a projected polygon.
fn polygon_area(points: &[Pos2]) -> f32 {
    let mut area = 0.0;
    for pair in points.windows(2) {
        area += pair[0].x * pair[1].y - pair[1].x * pair[0].y;
    }
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        area += last.x * first.y - first.x * last.y;
    }
    (area * 0.5).abs()
}
