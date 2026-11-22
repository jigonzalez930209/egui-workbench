//! Cartesian chart families beyond lines and bars: candlesticks, heatmaps, and
//! waterfalls. They share the axes, hover, legend, and export with every other series.

use egui::{Color32, Rect, pos2, vec2};

use super::animation::{Effect, mix};
use super::axis::Transform;
use super::config::PlotConfig;
use super::paint::Look;
use super::scene::{Dash, Measure, Prim};
use super::series::{Kind, Series};

/// One series of a cartesian chart family that draws its own primitives.
#[allow(clippy::too_many_arguments)]
pub(super) fn series_prims(
    out: &mut Vec<Prim>,
    measure: &Measure,
    transform: &Transform,
    series: &Series<'_>,
    look: &Look,
    indices: &[usize],
    config: &PlotConfig,
    theme: &crate::theme::Theme,
    effect: Option<Effect>,
) {
    match series.kind {
        Kind::Candles { width } => candles(out, transform, series, look, indices, width, theme, effect),
        Kind::Heatmap => heatmap(out, measure, transform, series, config, theme, effect),
        Kind::Waterfall { width } => {
            waterfall(out, transform, series, look, width, config, theme, effect);
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn candles(
    out: &mut Vec<Prim>,
    transform: &Transform,
    series: &Series<'_>,
    look: &Look,
    indices: &[usize],
    width: Option<f64>,
    theme: &crate::theme::Theme,
    effect: Option<Effect>,
) {
    let Some(rows) = series.ohlc() else {
        return;
    };
    let width = width.unwrap_or_else(|| series.min_spacing() * 0.65);
    let frame = transform.frame.expand(2.0);
    for &index in indices {
        let Some(row) = rows.get(index) else {
            continue;
        };
        let [x, open, high, low, close] = *row;
        if !(x.is_finite() && open.is_finite() && high.is_finite() && low.is_finite() && close.is_finite()) {
            continue;
        }
        let left = transform.to_screen([x - width * 0.5, open]);
        let right = transform.to_screen([x + width * 0.5, close]);
        if !(left.is_finite() && right.is_finite()) {
            continue;
        }
        let body = Rect::from_two_pos(left, right);
        if !frame.intersects(body) {
            continue;
        }
        let base = if close >= open { look.color } else { theme.danger };
        let color = effect.map_or(base, |effect| effect.color(base));
        let top = transform.to_screen([x, high]);
        let bottom = transform.to_screen([x, low]);
        if top.is_finite() && bottom.is_finite() {
            out.push(Prim::Line {
                points: vec![pos2(top.x, top.y), pos2(bottom.x, bottom.y)],
                width: 1.0,
                color,
                dash: Dash::Solid,
            });
        }
        // A doji (open == close) keeps a one-pixel body so the candle stays visible.
        let body = if body.height() < 1.0 {
            Rect::from_min_size(pos2(body.left(), body.center().y - 0.5), vec2(body.width(), 1.0))
        } else {
            body
        };
        out.push(Prim::Rect {
            rect: body,
            radius: 0.0,
            fill: color,
            stroke: Some((1.0, color)),
        });
    }
}

fn heatmap(
    out: &mut Vec<Prim>,
    measure: &Measure,
    transform: &Transform,
    series: &Series<'_>,
    config: &PlotConfig,
    theme: &crate::theme::Theme,
    effect: Option<Effect>,
) {
    let Some(cells) = series.cells() else {
        return;
    };
    if cells.is_empty() {
        return;
    }
    let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);
    for cell in cells {
        if cell[2].is_finite() {
            min = min.min(cell[2]);
            max = max.max(cell[2]);
        }
    }
    if !(min.is_finite() && max.is_finite()) {
        return;
    }
    let span = (max - min).max(1e-12);
    let x_step = axis_step(cells.iter().map(|cell| cell[0]));
    let y_step = axis_step(cells.iter().map(|cell| cell[1]));
    let opacity = config.heatmap.opacity;
    for cell in cells {
        if !(cell[0].is_finite() && cell[1].is_finite() && cell[2].is_finite()) {
            continue;
        }
        let low = transform.to_screen([cell[0] - x_step * 0.5, cell[1] - y_step * 0.5]);
        let high = transform.to_screen([cell[0] + x_step * 0.5, cell[1] + y_step * 0.5]);
        let rect = Rect::from_two_pos(low, high);
        if !(rect.is_finite() && transform.frame.intersects(rect)) {
            continue;
        }
        let t = ((cell[2] - min) / span) as f32;
        let color = heat_color(t, theme);
        let color = effect.map_or(color, |effect| mix(color, effect.highlight, 0.45 * effect.wave()));
        out.push(Prim::Rect {
            rect,
            radius: 0.0,
            fill: color.gamma_multiply(opacity),
            stroke: None,
        });
    }
    if config.heatmap.colorbar {
        colorbar(
            out,
            measure,
            transform.frame,
            min,
            max,
            theme,
            opacity,
            config.tick_size,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn colorbar(
    out: &mut Vec<Prim>,
    measure: &Measure,
    frame: Rect,
    min: f64,
    max: f64,
    theme: &crate::theme::Theme,
    opacity: f32,
    label_size: f32,
) {
    const BANDS: usize = 24;
    let height = (frame.height() * 0.55).max(40.0);
    let width = 10.0;
    let rect = Rect::from_min_size(
        pos2(
            frame.right() - width - 12.0,
            frame.top() + (frame.height() - height) * 0.5,
        ),
        vec2(width, height),
    );
    let band = height / BANDS as f32;
    for index in 0..BANDS {
        let t = 1.0 - index as f32 / (BANDS - 1) as f32;
        out.push(Prim::Rect {
            rect: Rect::from_min_size(
                pos2(rect.left(), rect.top() + index as f32 * band),
                vec2(width, band + 0.5),
            ),
            radius: 0.0,
            fill: heat_color(t, theme).gamma_multiply(opacity),
            stroke: None,
        });
    }
    let label = |value: f64| format!("{value:.3}");
    out.push(measure.label(
        pos2(rect.left() - 4.0, rect.top()),
        egui::Align2::RIGHT_TOP,
        &label(max),
        label_size,
        theme.text,
    ));
    out.push(measure.label(
        pos2(rect.left() - 4.0, rect.bottom()),
        egui::Align2::RIGHT_BOTTOM,
        &label(min),
        label_size,
        theme.text,
    ));
}

#[allow(clippy::too_many_arguments)]
fn waterfall(
    out: &mut Vec<Prim>,
    transform: &Transform,
    series: &Series<'_>,
    look: &Look,
    width: Option<f64>,
    config: &PlotConfig,
    theme: &crate::theme::Theme,
    effect: Option<Effect>,
) {
    let width = config
        .waterfall
        .bar_width
        .or(width)
        .unwrap_or_else(|| series.min_spacing() * 0.65);
    let mut base = 0.0;
    let mut previous: Option<(f32, f32)> = None;
    for point in series.data() {
        if !(point[0].is_finite() && point[1].is_finite()) {
            continue;
        }
        let next = base + point[1];
        let left = transform.to_screen([point[0] - width * 0.5, base]);
        let right = transform.to_screen([point[0] + width * 0.5, next]);
        let rect = Rect::from_two_pos(left, right);
        if rect.is_finite() && transform.frame.intersects(rect) {
            if config.waterfall.connectors
                && let Some((x, y)) = previous
            {
                let center = transform.to_screen([point[0], base]);
                out.push(Prim::Line {
                    points: vec![pos2(x, y), pos2(center.x, center.y)],
                    width: 1.0,
                    color: look.color.gamma_multiply(0.35),
                    dash: Dash::Dashed(4.0, 3.0),
                });
            }
            let base_color = if point[1] >= 0.0 { look.color } else { theme.danger };
            let color = effect.map_or(base_color, |effect| effect.color(base_color));
            out.push(Prim::Rect {
                rect,
                radius: 1.0,
                fill: color.gamma_multiply(look.fill_opacity.max(0.55)),
                stroke: None,
            });
            previous = Some((rect.center().x, rect.top()));
        }
        base = next;
    }
}

/// A blue → purple → amber scale anchored on theme tokens.
fn heat_color(t: f32, theme: &crate::theme::Theme) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        mix(theme.link, theme.done, t * 2.0)
    } else {
        mix(theme.done, theme.warning, (t - 0.5) * 2.0)
    }
}

/// The smallest positive step in a series of coordinates (1 when there is none).
fn axis_step(values: impl Iterator<Item = f64>) -> f64 {
    let mut values: Vec<f64> = values.filter(|value| value.is_finite()).collect();
    values.sort_by(f64::total_cmp);
    let mut step = f64::INFINITY;
    for pair in values.windows(2) {
        let delta = pair[1] - pair[0];
        if delta > 0.0 {
            step = step.min(delta);
        }
    }
    if step.is_finite() { step } else { 1.0 }
}
