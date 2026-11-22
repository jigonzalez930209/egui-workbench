//! Frame layout and the colors of one plot frame.

use egui::{Color32, Rect, Visuals, pos2};

use super::axis::{Bounds, SecondAxis, Tick, TickRules, ticks};
use super::config::{AxisConfig, PlotConfig};
use super::scene::Measure;

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
fn vertical_margin(measure: &Measure, config: &PlotConfig, axis: &AxisConfig, ticks: &[Tick]) -> f32 {
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
    measure: &Measure,
    config: &PlotConfig,
    rect: Rect,
    view: Bounds,
    y2: Option<SecondAxis>,
    mode: super::ChartMode,
) -> Layout {
    if mode != super::ChartMode::Cartesian {
        let mut top = 8.0;
        if let Some(title) = &config.title {
            top += measure.text(title, config.title_size).1 + 6.0;
        }
        let frame = Rect::from_min_max(
            pos2(measure.snap(rect.left() + 10.0), measure.snap(rect.top() + top)),
            pos2(measure.snap(rect.right() - 10.0), measure.snap(rect.bottom() - 10.0)),
        );
        return Layout {
            rect,
            frame,
            x_spacing: 64.0,
        };
    }
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

fn widest(measure: &Measure, ticks: &[Tick], size: f32) -> f32 {
    ticks
        .iter()
        .filter(|tick| tick.major && !tick.label.is_empty())
        .map(|tick| measure.text(&tick.label, size).0)
        .fold(0.0, f32::max)
}
