//! Layout and scene construction: axes, grid, labels, series, selection, and legend.

use egui::{Align2, Color32, Pos2, Rect, Visuals, pos2};

use super::axis::{Bounds, SecondAxis, Tick, TickRules, Transform, ticks};
use super::config::{AxisConfig, PlotConfig, TitleAlign};
use super::paint::{self, Look};
use super::scene::{Dash, Measure, Prim, Scene};
use super::series::{Kind, LineStyle, Smoothing, YAxis};
use super::{Plot, PlotState, palette};

/// Points between a tick mark and its label.
const TICK_GAP: f32 = 3.0;

pub(super) struct Layout {
    pub rect: Rect,
    pub frame: Rect,
    /// Points per x tick that keeps the labels 12 pt apart.
    pub x_spacing: f32,
}

/// The colors of one frame: config overrides over the theme.
pub(super) struct Colors {
    pub background: Color32,
    pub canvas: Option<Color32>,
    pub frame: Color32,
    pub grid: Color32,
    pub text: Color32,
    pub tick_text: Color32,
    pub selection: Color32,
    pub legend_fill: Color32,
    pub legend_stroke: Color32,
}

impl Colors {
    pub fn resolve(config: &PlotConfig, visuals: &Visuals) -> Self {
        let colors = &config.colors;
        let frame = colors.frame.unwrap_or(visuals.window_stroke.color);
        Self {
            background: colors.background.unwrap_or(visuals.extreme_bg_color),
            canvas: colors.canvas,
            frame,
            grid: colors.grid.unwrap_or(frame),
            text: colors.text.unwrap_or(visuals.text_color()),
            tick_text: colors.tick_text.unwrap_or(visuals.weak_text_color()),
            selection: colors.selection.unwrap_or(visuals.widgets.active.bg_fill),
            legend_fill: visuals.window_fill,
            legend_stroke: visuals.window_stroke.color,
        }
    }
}

/// The size of an axis title.
pub(super) fn label_size(config: &PlotConfig, axis: &AxisConfig) -> f32 {
    axis.label_size.unwrap_or(config.label_size)
}

/// Room beside the frame for a vertical axis: tick marks, tick labels, and title.
fn vertical_margin(measure: &Measure<'_>, config: &PlotConfig, axis: &AxisConfig, ticks: &[Tick]) -> f32 {
    let labels = if axis.tick_labels {
        widest(measure, ticks, config.tick_size) + TICK_GAP
    } else {
        0.0
    };
    let title = if axis.label.is_some() {
        measure.text("Ag", label_size(config, axis)).1 + 6.0
    } else {
        0.0
    };
    4.0 + labels + config.tick_length + title
}

/// Margins from measured text: the y tick labels set the left margin (and the right
/// axis the right one), the x labels the bottom one, and x ticks are thinned until no
/// two labels are closer than 12 pt.
pub(super) fn layout(
    measure: &Measure<'_>,
    config: &PlotConfig,
    rect: Rect,
    view: Bounds,
    y2: Option<SecondAxis>,
) -> Layout {
    let tick_height = measure.text("0", config.tick_size).1;
    let mut top = 8.0;
    if let Some(title) = &config.title {
        top += measure.text(title, config.title_size).1 + 6.0;
    }
    let mut bottom = config.tick_length + 4.0;
    if config.x.tick_labels {
        bottom += TICK_GAP + tick_height;
    }
    if config.x.label.is_some() {
        bottom += measure.text("Ag", label_size(config, &config.x)).1 + 4.0;
    }
    let frame_height = (rect.height() - top - bottom).max(40.0);
    let y_ticks = ticks(
        config.y.scale,
        view.min_y,
        view.max_y,
        frame_height,
        36.0,
        TickRules::of(&config.y),
    );
    let left = vertical_margin(measure, config, &config.y, &y_ticks);
    let y2_margin = y2.map(|axis| {
        let y2_ticks = ticks(
            axis.scale,
            axis.min,
            axis.max,
            frame_height,
            36.0,
            TickRules::of(&config.y2),
        );
        vertical_margin(measure, config, &config.y2, &y2_ticks)
    });

    let frame_width = (rect.width() - left - y2_margin.unwrap_or(12.0)).max(40.0);
    let mut spacing = 64.0;
    let mut x_ticks = Vec::new();
    for _ in 0..8 {
        x_ticks = ticks(
            config.x.scale,
            view.min_x,
            view.max_x,
            frame_width,
            spacing,
            TickRules::of(&config.x),
        );
        let majors: Vec<f64> = x_ticks
            .iter()
            .filter(|tick| tick.major)
            .map(|tick| tick.value)
            .collect();
        let gap = majors
            .windows(2)
            .map(|pair| ((pair[1] - pair[0]) / view.width() * f64::from(frame_width)) as f32)
            .fold(f32::INFINITY, f32::min);
        // A fixed step is the user's choice; only automatic steps are thinned.
        if config.x.major_step.is_some() || gap >= widest(measure, &x_ticks, config.tick_size) + 12.0 {
            break;
        }
        spacing *= 1.4;
    }
    let last_width = x_ticks
        .iter()
        .rev()
        .find(|tick| tick.major)
        .map_or(0.0, |tick| measure.text(&tick.label, config.tick_size).0);
    let right = (last_width * 0.5 + 2.0).max(y2_margin.unwrap_or(10.0));

    let frame = Rect::from_min_max(
        pos2(measure.snap(rect.left() + left), measure.snap(rect.top() + top)),
        pos2(measure.snap(rect.right() - right), measure.snap(rect.bottom() - bottom)),
    );
    Layout {
        rect,
        frame,
        x_spacing: spacing,
    }
}

fn widest(measure: &Measure<'_>, ticks: &[Tick], size: f32) -> f32 {
    ticks
        .iter()
        .filter(|tick| tick.major && !tick.label.is_empty())
        .map(|tick| measure.text(&tick.label, size).0)
        .fold(0.0, f32::max)
}

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
    };
    Look {
        color,
        width: style.and_then(|style| style.width).unwrap_or(width) * config.line_scale,
        style: style.and_then(|style| style.line_style).unwrap_or(line_style),
        marker: style.and_then(|style| style.marker).or(marker),
        radius: style.and_then(|style| style.marker_size).unwrap_or(radius) * config.marker_scale,
        fill_opacity: style.and_then(|style| style.fill_opacity).unwrap_or(fill),
        smoothing: style.and_then(|style| style.smoothing).unwrap_or(smoothing),
    }
}

/// Everything static about one frame of the plot, as primitives.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_scene(
    measure: &Measure<'_>,
    plot: &Plot<'_>,
    layout: &Layout,
    transform: &Transform,
    state: &PlotState,
    colors: &Colors,
    theme: &crate::theme::Theme,
    exporting: bool,
) -> Scene {
    let config = &plot.config;
    let frame = layout.frame;
    let mut scene = Scene {
        size: layout.rect.size(),
        frame,
        under: Vec::new(),
        data: Vec::new(),
        over: Vec::new(),
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
    axis_prims(&mut scene, measure, config, layout, transform, colors);

    for (index, series) in plot.series.iter().enumerate() {
        if state.is_hidden(&series.name) {
            continue;
        }
        let look = look(plot, index, theme);
        let transform = transform.for_axis(plot.axis_of(series));
        let indices = paint::visible_indices(&transform, series, series.sorted_x());
        paint::series_prims(&mut scene.data, measure, &transform, series, &look, &indices);
    }
    if let Some(selection) = state.selection {
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
    if config.frame_width > 0.0 {
        scene.over.push(Prim::Rect {
            rect: frame,
            radius: 0.0,
            fill: Color32::TRANSPARENT,
            stroke: Some((config.frame_width, colors.frame)),
        });
    }
    titles(&mut scene, measure, config, layout, colors, transform.y2.is_some());
    if let Some(legend) = super::legend::layout(measure, plot, frame, state, !exporting) {
        super::legend::prims(&mut scene, measure, plot, &legend, colors, theme);
    }
    scene
}

fn axis_prims(
    scene: &mut Scene,
    measure: &Measure<'_>,
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
    measure: &Measure<'_>,
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
    measure: &Measure<'_>,
    config: &PlotConfig,
    layout: &Layout,
    colors: &Colors,
    right_axis: bool,
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
