//! Pie/donut, radar, and gauge charts. They replace the cartesian axes with their own
//! layout, but keep the legend, hover animation, theme, and export pipeline.

use std::f32::consts::{FRAC_PI_2, TAU};

use egui::{Align2, Pos2, Rect, pos2, vec2};

use super::animation::Effect;
use super::axis::format_readout;
use super::overlay::{self, Colors};
use super::paint::Look;
use super::scene::{Dash, Measure, Prim, Scene};
use super::series::{Kind, Series};
use super::{ChartMode, Plot, PlotState};

#[allow(clippy::too_many_arguments)]
pub(super) fn build(
    scene: &mut Scene,
    measure: &Measure,
    plot: &Plot<'_>,
    layout: &overlay::Layout,
    state: &PlotState,
    colors: &Colors,
    theme: &crate::theme::Theme,
    mode: ChartMode,
    now: f64,
    motion: bool,
) {
    let frame = layout.frame;
    let visible: Vec<(usize, &Series<'_>, Look, Option<Effect>)> = overlay::draw_order(plot, state)
        .into_iter()
        .filter_map(|index| {
            let series = &plot.series[index];
            if state.is_hidden(&series.name) {
                return None;
            }
            let look = overlay::look(plot, index, theme);
            let effect = Effect::for_hover(&plot.config, state, index, now, theme, motion);
            Some((index, series, look, effect))
        })
        .collect();
    scene.drawn = vec![None; plot.series.len()];
    match mode {
        ChartMode::Pie => pie(scene, measure, plot, frame, &visible, colors, theme),
        ChartMode::Radar => radar(scene, measure, plot, frame, &visible, colors, theme),
        ChartMode::Gauge => gauge(scene, measure, plot, frame, &visible, colors, theme),
        ChartMode::Cartesian => {}
    }
}

fn pie(
    scene: &mut Scene,
    measure: &Measure,
    plot: &Plot<'_>,
    frame: Rect,
    visible: &[(usize, &Series<'_>, Look, Option<Effect>)],
    colors: &Colors,
    theme: &crate::theme::Theme,
) {
    let slices: Vec<_> = visible
        .iter()
        .filter_map(|(index, series, look, effect)| match series.kind {
            Kind::Pie { value } if value.is_finite() && value > 0.0 => Some((*index, value, *look, *effect)),
            _ => None,
        })
        .collect();
    let total: f64 = slices.iter().map(|(_, value, _, _)| value).sum();
    if total <= 0.0 {
        let at = frame.center();
        scene.over.push(measure.label(
            at,
            Align2::CENTER_CENTER,
            "No values to draw",
            plot.config.legend_size,
            colors.tick_text,
        ));
        return;
    }
    let config = &plot.config.pie;
    let outer = (frame.width().min(frame.height()) * 0.42).max(20.0);
    let center = frame.center();
    let inner = (outer * config.inner_radius.clamp(0.0, 0.9)).max(0.0);
    let mut start = config.start_angle;
    for (_, value, look, effect) in &slices {
        let sweep = (value / total) as f32 * TAU;
        let resolution = ((sweep.abs() / TAU) * 128.0).ceil().clamp(3.0, 128.0) as usize;
        let mut center = center;
        if let Some(effect) = effect {
            let middle = start + sweep * 0.5;
            center += vec2(middle.cos(), middle.sin()) * effect.explode();
        }
        let mut points = Vec::with_capacity(resolution + 1);
        for step in 0..=resolution {
            let angle = start + sweep * step as f32 / resolution as f32;
            points.push(center + vec2(angle.cos(), angle.sin()) * outer);
        }
        let color = effect.map_or(look.color, |effect| effect.color(look.color));
        scene.data.push(Prim::Fan {
            center,
            points: points.clone(),
            color,
        });
        let mut outline = points;
        outline.push(center);
        scene.data.push(Prim::Line {
            points: outline,
            width: 1.0,
            color: color.gamma_multiply(0.72),
            dash: Dash::Solid,
        });
        if plot.config.pie.labels && sweep / TAU >= 0.055 {
            let middle = start + sweep * 0.5;
            let at = center + vec2(middle.cos(), middle.sin()) * outer * 0.66;
            let text = if plot.config.pie.percentages {
                format!("{:.0}%", (value / total) * 100.0)
            } else {
                format_readout(*value)
            };
            let text_color = if crate::theme::relative_luminance(color) > 0.55 {
                colors.background
            } else {
                colors.text
            };
            scene
                .data
                .push(measure.label(at, Align2::CENTER_CENTER, &text, plot.config.legend_size, text_color));
        }
        start += sweep;
    }
    if inner > 1.0 {
        scene.over.push(Prim::Circle {
            center,
            radius: inner,
            fill: colors.background,
            stroke: Some((1.0, colors.frame)),
        });
        if plot.config.pie.total {
            let value = format_readout(total);
            scene.over.push(measure.label(
                center - vec2(0.0, 7.0),
                Align2::CENTER_BOTTOM,
                &value,
                plot.config.title_size,
                colors.text,
            ));
            scene.over.push(measure.label(
                center + vec2(0.0, 5.0),
                Align2::CENTER_TOP,
                "total",
                plot.config.tick_size,
                colors.tick_text,
            ));
        }
    }
    let _ = theme;
}

fn radar(
    scene: &mut Scene,
    measure: &Measure,
    plot: &Plot<'_>,
    frame: Rect,
    visible: &[(usize, &Series<'_>, Look, Option<Effect>)],
    colors: &Colors,
    theme: &crate::theme::Theme,
) {
    let series: Vec<_> = visible
        .iter()
        .filter(|(_, series, _, _)| matches!(series.kind, Kind::Radar))
        .collect();
    if series.is_empty() {
        return;
    }
    let axes = series
        .iter()
        .map(|(_, series, _, _)| series.data().len())
        .chain(std::iter::once(plot.config.radar.labels.len()))
        .max()
        .unwrap_or(0)
        .max(3);
    let mut max = plot.config.radar.max.unwrap_or(0.0);
    for (_, series, _, _) in &series {
        for point in series.data() {
            if point[1].is_finite() {
                max = max.max(point[1]);
            }
        }
    }
    let max = if max > 0.0 { max } else { 1.0 };
    let center = frame.center();
    let radius = (frame.width().min(frame.height()) * 0.5 - 30.0).max(20.0);
    let angle = |index: usize| -FRAC_PI_2 + index as f32 * TAU / axes as f32;
    for ring in 1..=plot.config.radar.rings {
        let scale = f32::from(ring) / f32::from(plot.config.radar.rings.max(1));
        let mut points: Vec<Pos2> = (0..axes)
            .map(|index| center + vec2(angle(index).cos(), angle(index).sin()) * radius * scale)
            .collect();
        points.push(points[0]);
        scene.data.push(Prim::Line {
            points,
            width: 1.0,
            color: colors.grid.gamma_multiply(0.45),
            dash: Dash::Solid,
        });
    }
    for index in 0..axes {
        let direction = vec2(angle(index).cos(), angle(index).sin());
        scene.data.push(Prim::Line {
            points: vec![center, center + direction * radius],
            width: 1.0,
            color: colors.grid.gamma_multiply(0.35),
            dash: Dash::Solid,
        });
        let label = plot
            .config
            .radar
            .labels
            .get(index)
            .cloned()
            .unwrap_or_else(|| (index + 1).to_string());
        let at = center + direction * (radius + 13.0);
        let anchor = if direction.x > 0.35 {
            Align2::LEFT_CENTER
        } else if direction.x < -0.35 {
            Align2::RIGHT_CENTER
        } else if direction.y > 0.0 {
            Align2::CENTER_TOP
        } else {
            Align2::CENTER_BOTTOM
        };
        scene
            .data
            .push(measure.label(at, anchor, &label, plot.config.tick_size, colors.tick_text));
    }
    for (_, series, look, effect) in series {
        let mut points: Vec<Pos2> = (0..axes)
            .map(|index| {
                let value = series.data().get(index).map_or(0.0, |point| point[1].max(0.0));
                let scale = (value / max) as f32 * effect.map_or(1.0, |effect| effect.radial_scale());
                center + vec2(angle(index).cos(), angle(index).sin()) * radius * scale.min(1.0)
            })
            .collect();
        let color = effect.map_or(look.color, |effect| effect.color(look.color));
        let width = effect.map_or(look.width, |effect| effect.width(look.width));
        scene.data.push(Prim::Fan {
            center,
            points: points.clone(),
            color: color.gamma_multiply(plot.config.radar.fill_opacity),
        });
        points.push(points[0]);
        scene.data.push(Prim::Line {
            points: points.clone(),
            width,
            color,
            dash: Dash::Solid,
        });
        for (index, point) in points.iter().take(axes).enumerate() {
            let position = *point + effect.map_or(vec2(0.0, 0.0), |effect| effect.marker_offset(index));
            let marker_radius = effect.map_or(2.5, |effect| effect.marker_radius(2.5, index));
            super::paint::marker_prims(&mut scene.data, position, super::Marker::Circle, marker_radius, color);
        }
    }
    let _ = theme;
}

fn gauge(
    scene: &mut Scene,
    measure: &Measure,
    plot: &Plot<'_>,
    frame: Rect,
    visible: &[(usize, &Series<'_>, Look, Option<Effect>)],
    colors: &Colors,
    theme: &crate::theme::Theme,
) {
    let gauges: Vec<_> = visible
        .iter()
        .filter_map(|(index, series, look, effect)| match series.kind {
            Kind::Gauge { value, min, max } => Some((*index, value, min, max, *look, *effect)),
            _ => None,
        })
        .collect();
    if gauges.is_empty() {
        return;
    }
    let count = gauges.len() as f32;
    let cell_width = frame.width() / count;
    for (slot, (index, value, min, max, look, effect)) in gauges.into_iter().enumerate() {
        let cell = Rect::from_min_size(
            pos2(frame.left() + cell_width * slot as f32, frame.top()),
            vec2(cell_width, frame.height()),
        );
        let center = pos2(cell.center().x, cell.center().y + cell.height() * 0.08);
        let radius = (cell.width().min(cell.height()) * 0.4).max(20.0);
        let config = &plot.config.gauge;
        let sweep = config.end_angle - config.start_angle;
        let fraction = ((value - min) / (max - min).max(1e-12)).clamp(0.0, 1.0) as f32;
        let color = effect.map_or(look.color, |effect| effect.color(look.color));
        let width = effect.map_or(look.width, |effect| effect.width(look.width));
        let arc = |from: f32, to: f32| -> Vec<Pos2> {
            let steps = ((to - from).abs() / TAU * 96.0).ceil().clamp(8.0, 128.0) as usize;
            (0..=steps)
                .map(|step| {
                    let angle = from + (to - from) * step as f32 / steps as f32;
                    center + vec2(angle.cos(), angle.sin()) * radius
                })
                .collect()
        };
        scene.data.push(Prim::Line {
            points: arc(config.start_angle, config.end_angle),
            width: config.thickness,
            color: colors.grid.gamma_multiply(0.35),
            dash: Dash::Solid,
        });
        let end = config.start_angle + sweep * fraction;
        scene.data.push(Prim::Line {
            points: arc(config.start_angle, end),
            width: config.thickness.max(width * 2.0),
            color,
            dash: effect.map_or(Dash::Solid, |effect| effect.dash(Dash::Solid)),
        });
        if config.show_range {
            for (angle, label) in [
                (config.start_angle, format_readout(min)),
                (config.end_angle, format_readout(max)),
            ] {
                let at = center + vec2(angle.cos(), angle.sin()) * (radius + config.thickness * 0.8);
                scene.data.push(measure.label(
                    at,
                    Align2::CENTER_CENTER,
                    &label,
                    plot.config.tick_size,
                    colors.tick_text,
                ));
            }
        }
        let needle = vec2(end.cos(), end.sin());
        scene.data.push(Prim::Line {
            points: vec![center, center + needle * radius * 0.78],
            width: 2.0,
            color: theme.text,
            dash: Dash::Solid,
        });
        scene.data.push(Prim::Circle {
            center,
            radius: 3.5,
            fill: theme.text,
            stroke: None,
        });
        if config.show_value {
            let text = format_readout(value);
            scene.data.push(measure.label(
                center + vec2(0.0, radius * 0.38),
                Align2::CENTER_CENTER,
                &text,
                plot.config.title_size,
                colors.text,
            ));
        }
        scene.data.push(measure.label(
            pos2(cell.center().x, cell.bottom() - 4.0),
            Align2::CENTER_BOTTOM,
            &plot.series[index].name,
            plot.config.legend_size,
            colors.tick_text,
        ));
    }
}
