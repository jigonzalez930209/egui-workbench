//! Layout and scene construction: axes, grid, labels, series, selection, and legend.

use egui::{Align2, Color32, Pos2, Rect, pos2};

use super::animation::Effect;
use super::axis::{TickRules, Transform, ticks};
use super::config::{AxisConfig, PlotConfig, TitleAlign};
use super::direction;
use super::paint::{self, Look};
use super::scene::{Dash, Measure, Prim, Scene};
use super::series::{Kind, LineStyle, Marker, Smoothing, YAxis};
use super::{ChartMode, Plot, PlotState, palette};

/// Points between a tick mark and its label.
const TICK_GAP: f32 = 3.0;

pub(super) use super::layout::{Colors, Layout, label_size, layout};

/// The look of series `index` after the config overrides.
pub(super) fn look(plot: &Plot<'_>, index: usize, theme: &crate::theme::Theme) -> Look {
    let series = &plot.series[index];
    let config = &plot.config;
    let style = config.series_style(&series.name);
    let color = style
        .and_then(|style| style.color)
        .or(series.color)
        .unwrap_or_else(|| palette(theme, index));
    let (width, line_style, marker, radius, fill, smoothing) = match series.kind {
        Kind::Line {
            width,
            style,
            markers,
            fill_to,
            smoothing,
            ..
        } => (
            width,
            style,
            markers,
            width + 1.5,
            if fill_to.is_some() { 0.18 } else { 1.0 },
            smoothing,
        ),
        Kind::Scatter { marker, radius } => (1.5, LineStyle::Solid, Some(marker), radius, 1.0, Smoothing::None),
        Kind::Bars { .. } => (1.0, LineStyle::Solid, None, 3.0, 0.85, Smoothing::None),
        Kind::HLine(_) | Kind::VLine(_) => (1.0, LineStyle::Solid, None, 3.0, 1.0, Smoothing::None),
        Kind::Pie { .. } => (1.0, LineStyle::Solid, None, 3.0, 0.9, Smoothing::None),
        Kind::Candles { .. } => (1.0, LineStyle::Solid, None, 2.0, 1.0, Smoothing::None),
        Kind::Heatmap => (1.0, LineStyle::Solid, None, 3.0, 1.0, Smoothing::None),
        Kind::Radar => (1.5, LineStyle::Solid, Some(Marker::Circle), 3.0, 1.0, Smoothing::None),
        Kind::Gauge { .. } => (1.5, LineStyle::Solid, None, 3.0, 1.0, Smoothing::None),
        Kind::Waterfall { .. } => (1.0, LineStyle::Solid, None, 3.0, 0.8, Smoothing::None),
    };
    Look {
        color,
        width: style.and_then(|style| style.width).unwrap_or(width) * config.line_scale,
        style: style.and_then(|style| style.line_style).unwrap_or(line_style),
        marker: style.and_then(|style| style.marker).or(marker),
        radius: style.and_then(|style| style.marker_size).unwrap_or(radius) * config.marker_scale,
        fill_opacity: style.and_then(|style| style.fill_opacity).unwrap_or(fill),
        smoothing: style.and_then(|style| style.smoothing).unwrap_or(smoothing),
        skip_gpu_lines: false,
    }
}

/// The order series draw in: normal order, except that the series whose legend row is
/// hovered draws last (on top of every other series). The legend itself keeps its order.
pub(super) fn draw_order(plot: &Plot<'_>, state: &PlotState) -> Vec<usize> {
    let mut order: Vec<usize> = (0..plot.series.len()).collect();
    if let Some(hovered) = state.legend_hover()
        && plot
            .series
            .get(hovered)
            .is_some_and(|series| !state.is_hidden(&series.name))
        && let Some(position) = order.iter().position(|index| *index == hovered)
    {
        let index = order.remove(position);
        order.push(index);
    }
    order
}

/// Everything static about one frame of the plot, as primitives.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_scene(
    measure: &Measure,
    plot: &Plot<'_>,
    layout: &Layout,
    transform: &Transform,
    state: &PlotState,
    colors: &Colors,
    theme: &crate::theme::Theme,
    exporting: bool,
    mode: ChartMode,
    now: f64,
    motion: bool,
    gpu: bool,
) -> Scene {
    let config = &plot.config;
    let frame = layout.frame;
    let mut scene = Scene {
        size: layout.rect.size(),
        frame,
        under: Vec::new(),
        data: Vec::new(),
        over: Vec::new(),
        drawn: Vec::new(),
    };
    if let Some(canvas) = colors.canvas {
        scene.under.push(Prim::Rect {
            rect: layout.rect,
            radius: 0.0,
            fill: canvas,
            stroke: None,
        });
    }
    scene.under.push(Prim::Rect {
        rect: frame,
        radius: 0.0,
        fill: colors.background,
        stroke: None,
    });
    if mode == ChartMode::Cartesian {
        axis_prims(&mut scene, measure, config, layout, transform, colors);
    }

    if mode == ChartMode::Cartesian {
        scene.drawn = vec![None; plot.series.len()];
        for index in draw_order(plot, state) {
            let series = &plot.series[index];
            if state.is_hidden(&series.name) {
                continue;
            }
            let mut look = look(plot, index, theme);
            look.skip_gpu_lines = gpu;
            let effect = Effect::for_hover(config, state, index, now, theme, motion && !exporting);
            let transform = transform.for_axis(plot.axis_of(series));
            let sorted = series.sorted_x();
            let indices = paint::visible_indices(&transform, series, sorted, look.radius);
            if series.is_extra() {
                super::extra::series_prims(
                    &mut scene.data,
                    measure,
                    &transform,
                    series,
                    &look,
                    &indices,
                    config,
                    theme,
                    effect,
                );
            } else {
                paint::series_prims(&mut scene.data, measure, &transform, series, &look, &indices, effect);
                // Sorted series find their nearest point by binary search already.
                scene.drawn[index] = (!sorted).then_some(indices);
            }
            if plot.draws_direction(index) && series.has_direction() {
                let color = effect.map_or(look.color, |effect| effect.color(look.color));
                direction::direction_arrow(&mut scene.data, &transform, series, &config.direction, color);
            }
        }
    } else {
        super::special::build(
            &mut scene,
            measure,
            plot,
            layout,
            state,
            colors,
            theme,
            mode,
            now,
            motion && !exporting,
        );
    }
    if mode == ChartMode::Cartesian
        && let Some(selection) = state.selection
    {
        let a = transform.to_screen([selection.min_x, selection.min_y]);
        let b = transform.to_screen([selection.max_x, selection.max_y]);
        let rect = Rect::from_two_pos(a, b);
        if rect.is_finite() {
            scene.data.push(Prim::Rect {
                rect: rect.intersect(frame),
                radius: 0.0,
                fill: colors.selection.gamma_multiply(0.12),
                stroke: Some((1.0, colors.selection)),
            });
        }
    }
    if mode == ChartMode::Cartesian && config.frame_width > 0.0 {
        scene.over.push(Prim::Rect {
            rect: frame,
            radius: 0.0,
            fill: Color32::TRANSPARENT,
            stroke: Some((config.frame_width, colors.frame)),
        });
    }
    titles(
        &mut scene,
        measure,
        config,
        layout,
        colors,
        transform.y2.is_some(),
        mode,
    );
    let hovered_effect = state
        .legend_hover()
        .and_then(|index| Effect::for_hover(config, state, index, now, theme, motion && !exporting));
    if let Some(legend) = super::legend::layout(measure, plot, frame, state, !exporting) {
        super::legend::prims(
            &mut scene,
            measure,
            plot,
            &legend,
            colors,
            theme,
            state.legend_hover(),
            hovered_effect.filter(|_| !exporting),
        );
    }
    scene
}

fn axis_prims(
    scene: &mut Scene,
    measure: &Measure,
    config: &PlotConfig,
    layout: &Layout,
    transform: &Transform,
    colors: &Colors,
) {
    let frame = layout.frame;
    let view = transform.view;
    let x_ticks = ticks(
        config.x.scale,
        view.min_x,
        view.max_x,
        frame.width(),
        layout.x_spacing,
        TickRules::of(&config.x),
    );
    let mark = config.x.tick_color.unwrap_or(colors.frame);
    let text = config.x.tick_color.unwrap_or(colors.tick_text);
    for tick in &x_ticks {
        let x = measure.center(transform.scaled_to_screen(tick.value, view.min_y).x);
        if x < frame.left() - 0.01 || x > frame.right() + 0.01 {
            continue;
        }
        let show_grid = if tick.major {
            config.x.grid_major
        } else {
            config.x.grid_minor
        };
        if show_grid {
            scene.under.push(line(
                vec![pos2(x, frame.top()), pos2(x, frame.bottom())],
                config.grid_width,
                grid_color(config, colors, tick.major),
            ));
        }
        let length = tick_length(config, tick.major);
        if length > 0.0 {
            scene.over.push(line(
                vec![pos2(x, frame.bottom()), pos2(x, frame.bottom() + length)],
                1.0,
                mark,
            ));
        }
        if tick.major && config.x.tick_labels && !tick.label.is_empty() {
            let (width, _, _) = measure.text(&tick.label, config.tick_size);
            let half = width * 0.5;
            // Keep the first and last label inside the plot's rectangle.
            let center = x.clamp(layout.rect.left() + half, layout.rect.right() - half);
            let at = pos2(center, frame.bottom() + config.tick_length + TICK_GAP);
            scene
                .over
                .push(measure.label(at, Align2::CENTER_TOP, &tick.label, config.tick_size, text));
        }
    }
    vertical_axis(scene, measure, config, &config.y, frame, transform, YAxis::Left, colors);
    if transform.y2.is_some() {
        let right = transform.for_axis(YAxis::Right);
        vertical_axis(scene, measure, config, &config.y2, frame, &right, YAxis::Right, colors);
    }
}

fn line(points: Vec<Pos2>, width: f32, color: Color32) -> Prim {
    Prim::Line {
        points,
        width,
        color,
        dash: Dash::Solid,
    }
}

fn grid_color(config: &PlotConfig, colors: &Colors, major: bool) -> Color32 {
    let opacity = if major {
        config.grid_major_opacity
    } else {
        config.grid_minor_opacity
    };
    colors.grid.gamma_multiply(opacity)
}

fn tick_length(config: &PlotConfig, major: bool) -> f32 {
    if major {
        config.tick_length
    } else {
        config.tick_length * 0.5
    }
}

/// Grid, tick marks, and tick labels of a y axis on either side of the frame.
/// `transform` is already the transform of that axis.
#[allow(clippy::too_many_arguments)]
fn vertical_axis(
    scene: &mut Scene,
    measure: &Measure,
    config: &PlotConfig,
    axis: &AxisConfig,
    frame: Rect,
    transform: &Transform,
    side: YAxis,
    colors: &Colors,
) {
    let view = transform.view;
    let y_ticks = ticks(
        transform.y_scale,
        view.min_y,
        view.max_y,
        frame.height(),
        36.0,
        TickRules::of(axis),
    );
    let mark = axis.tick_color.unwrap_or(colors.frame);
    let text = axis.tick_color.unwrap_or(colors.tick_text);
    let (edge, outward) = match side {
        YAxis::Left => (frame.left(), -1.0),
        YAxis::Right => (frame.right(), 1.0),
    };
    for tick in &y_ticks {
        let y = measure.center(transform.scaled_to_screen(view.min_x, tick.value).y);
        if y < frame.top() - 0.01 || y > frame.bottom() + 0.01 {
            continue;
        }
        let show_grid = if tick.major { axis.grid_major } else { axis.grid_minor };
        if show_grid {
            scene.under.push(line(
                vec![pos2(frame.left(), y), pos2(frame.right(), y)],
                config.grid_width,
                grid_color(config, colors, tick.major),
            ));
        }
        let length = tick_length(config, tick.major);
        if length > 0.0 {
            scene
                .over
                .push(line(vec![pos2(edge + outward * length, y), pos2(edge, y)], 1.0, mark));
        }
        if tick.major && axis.tick_labels && !tick.label.is_empty() {
            let at = pos2(edge + outward * (config.tick_length + TICK_GAP), y);
            let anchor = match side {
                YAxis::Left => Align2::RIGHT_CENTER,
                YAxis::Right => Align2::LEFT_CENTER,
            };
            scene
                .over
                .push(measure.label(at, anchor, &tick.label, config.tick_size, text));
        }
    }
}

fn titles(
    scene: &mut Scene,
    measure: &Measure,
    config: &PlotConfig,
    layout: &Layout,
    colors: &Colors,
    right_axis: bool,
    mode: ChartMode,
) {
    let frame = layout.frame;
    if let Some(title) = &config.title {
        let color = config.title_color.unwrap_or(colors.text);
        let (at, anchor) = match config.title_align {
            TitleAlign::Left => (pos2(frame.left(), layout.rect.top() + 6.0), Align2::LEFT_TOP),
            TitleAlign::Center => (pos2(frame.center().x, layout.rect.top() + 6.0), Align2::CENTER_TOP),
            TitleAlign::Right => (pos2(frame.right(), layout.rect.top() + 6.0), Align2::RIGHT_TOP),
        };
        scene
            .over
            .push(measure.label(at, anchor, title, config.title_size, color));
    }
    if mode != ChartMode::Cartesian {
        return;
    }
    if let Some(label) = &config.x.label {
        let at = pos2(frame.center().x, layout.rect.bottom() - 2.0);
        let color = config.x.label_color.unwrap_or(colors.text);
        scene
            .over
            .push(measure.label(at, Align2::CENTER_BOTTOM, label, label_size(config, &config.x), color));
    }
    if let Some(label) = &config.y.label {
        let size = label_size(config, &config.y);
        let (width, _, baseline) = measure.text(label, size);
        // Rotated −90° around its top-left corner: the text runs upward from `pos`.
        scene.over.push(Prim::Text {
            pos: pos2(
                measure.snap(layout.rect.left() + 2.0),
                measure.snap(frame.center().y + width * 0.5),
            ),
            text: label.clone(),
            size,
            color: config.y.label_color.unwrap_or(colors.text),
            baseline,
            angle: -std::f32::consts::FRAC_PI_2,
        });
    }
    if right_axis && let Some(label) = &config.y2.label {
        let size = label_size(config, &config.y2);
        let (width, _, baseline) = measure.text(label, size);
        // Rotated +90° around its top-left corner: the text runs downward and its
        // glyphs extend to the left of `pos`, toward the frame.
        scene.over.push(Prim::Text {
            pos: pos2(
                measure.snap(layout.rect.right() - 2.0),
                measure.snap(frame.center().y - width * 0.5),
            ),
            text: label.clone(),
            size,
            color: config.y2.label_color.unwrap_or(colors.text),
            baseline,
            angle: std::f32::consts::FRAC_PI_2,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::Series;
    use super::*;

    #[test]
    fn the_hovered_series_draws_last_without_moving_the_legend() {
        let data: [[f64; 2]; 2] = [[0.0, 0.0], [1.0, 1.0]];
        let plot = Plot::new("z-order")
            .series(Series::line("first", &data[..]))
            .series(Series::line("second", &data[..]))
            .series(Series::line("third", &data[..]));
        let mut state = PlotState::default();
        assert_eq!(draw_order(&plot, &state), [0, 1, 2]);
        state.set_legend_hover(Some(0), 0.0);
        assert_eq!(draw_order(&plot, &state), [1, 2, 0]);
        state.set_hidden("first", true);
        assert_eq!(draw_order(&plot, &state), [0, 1, 2]);
    }
}
