//! The [`Plot`] builder methods.

use egui::Id;

use super::axis::{Bounds, Scale};
use super::config::PlotConfig;
use super::fit::{Fit, FitKind};
use super::series::Series;
use super::{
    DirectionArrowConfig, GaugeConfig, HeatmapConfig, LegendAnimation, LegendHoverConfig, LegendPosition, PieConfig,
    Plot, RadarConfig, WaterfallConfig,
};

impl<'a> Plot<'a> {
    /// A plot with a stable id.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            id: Id::unique(("workbench_plot", id)),
            series: Vec::new(),
            config: PlotConfig::default(),
            width: None,
            height: None,
            x_window: None,
            include: Bounds::NOTHING,
            settings_button: false,
            export_button: false,
        }
    }

    /// Set the area tool mode: rectangle or curve-vs-chord.
    pub fn measurement_mode(mut self, mode: super::MeasurementMode) -> Self {
        self.config.measurement.mode = mode;
        self
    }

    /// Set which area the curve-segment label emphasizes.
    pub fn measurement_area_style(mut self, style: super::MeasurementAreaStyle) -> Self {
        self.config.measurement.area_style = style;
        self
    }

    /// Set the measurement snap radius in points.
    pub fn measurement_snap_radius(mut self, radius: f32) -> Self {
        self.config.measurement.snap_radius = radius.max(2.0);
        self
    }

    /// Show or hide measurement labels.
    pub fn measurement_labels(mut self, show: bool) -> Self {
        self.config.measurement.show_labels = show;
        self
    }

    /// Replace the whole appearance (text, sizes, colors, axes, grid, legend, and
    /// per-series styles). Call it first; the other builder methods edit it.
    pub fn config(mut self, config: PlotConfig) -> Self {
        self.config = config;
        self
    }

    /// The current appearance.
    pub fn current_config(&self) -> &PlotConfig {
        &self.config
    }

    /// The names of the series, in drawing order (for a settings dialog).
    pub fn series_names(&self) -> Vec<String> {
        self.series.iter().map(|series| series.name.clone()).collect()
    }

    /// Fixed x limits in data units (`None` keeps that side automatic).
    pub fn x_limits(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        (self.config.x.min, self.config.x.max) = (min, max);
        self
    }

    /// Fixed y limits in data units (`None` keeps that side automatic).
    pub fn y_limits(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        (self.config.y.min, self.config.y.max) = (min, max);
        self
    }

    /// Show a settings button in the toolbar; see [`PlotResponse::settings_requested`](super::PlotResponse::settings_requested).
    pub fn settings_button(mut self, show: bool) -> Self {
        self.settings_button = show;
        self
    }

    /// Show an SVG/PNG export menu in the toolbar; see
    /// [`PlotResponse::export_requested`](super::PlotResponse::export_requested).
    pub fn export_button(mut self, show: bool) -> Self {
        self.export_button = show;
        self
    }

    /// Add a series. Series draw in order; the palette color follows the order too.
    pub fn series(mut self, series: Series<'a>) -> Self {
        self.series.push(series);
        self
    }

    /// Fit `kind` to `data` and add the dashed curve, named with its equation and R².
    /// Nothing is added when the fit fails (too few points, singular system).
    pub fn fit(mut self, kind: FitKind, data: &[[f64; 2]]) -> Self {
        if let Some(fit) = Fit::new(kind, data) {
            self.series.push(fit.series(fit.equation(), 200));
        }
        self
    }

    /// A title above the frame.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    /// The x axis label (also used in the hover popup).
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.config.x.label = Some(label.into());
        self
    }

    /// The y axis label, drawn vertically.
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.config.y.label = Some(label.into());
        self
    }

    /// Linear or log10 x axis.
    pub fn x_scale(mut self, scale: Scale) -> Self {
        self.config.x.scale = scale;
        self
    }

    /// Linear or log10 y axis.
    pub fn y_scale(mut self, scale: Scale) -> Self {
        self.config.y.scale = scale;
        self
    }

    /// The right y axis title, drawn vertically (the axis shows when a series uses
    /// [`YAxis::Right`](super::YAxis::Right)).
    pub fn y2_label(mut self, label: impl Into<String>) -> Self {
        self.config.y2.label = Some(label.into());
        self
    }

    /// Linear or log10 right y axis.
    pub fn y2_scale(mut self, scale: Scale) -> Self {
        self.config.y2.scale = scale;
        self
    }

    /// Fixed right y limits in data units (`None` keeps that side automatic).
    pub fn y2_limits(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        (self.config.y2.min, self.config.y2.max) = (min, max);
        self
    }

    /// Fixed width; fills the available width otherwise.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Fixed height; fills the available height otherwise (320 pt in scroll areas).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Legend corner, or [`LegendPosition::Hidden`].
    pub fn legend(mut self, legend: LegendPosition) -> Self {
        self.config.legend = legend;
        self
    }

    /// Replace the legend hover animation settings.
    pub fn legend_hover(mut self, config: LegendHoverConfig) -> Self {
        self.config.legend_hover = config;
        self
    }

    /// Select the CPU or GPU data layer.
    pub fn renderer(mut self, renderer: super::Renderer) -> Self {
        self.config.renderer = renderer;
        self
    }

    /// The animation played while the pointer rests on a legend row (enabled by default).
    pub fn legend_animation(mut self, animation: LegendAnimation) -> Self {
        self.config.legend_hover.animation = animation;
        self
    }

    /// Add every series from an iterator, for multi-series families such as box plots.
    pub fn series_all(mut self, series: impl IntoIterator<Item = super::Series<'a>>) -> Self {
        self.series.extend(series);
        self
    }

    /// Replace the direction-arrow settings.
    pub fn direction(mut self, config: DirectionArrowConfig) -> Self {
        self.config.direction = config;
        self
    }

    /// Show or hide the direction arrow beside the newest point of line-like series.
    pub fn direction_arrow(mut self, show: bool) -> Self {
        self.config.direction.enabled = show;
        self
    }

    /// How many trailing samples measure the direction arrow (5 by default).
    pub fn direction_points(mut self, points: usize) -> Self {
        self.config.direction.points = points.max(2);
        self
    }

    /// Replace the pie/donut layout.
    pub fn pie(mut self, config: PieConfig) -> Self {
        self.config.pie = config;
        self
    }

    /// Replace the radar layout (labels, maximum, rings, fill).
    pub fn radar(mut self, config: RadarConfig) -> Self {
        self.config.radar = config;
        self
    }

    /// Replace the gauge layout.
    pub fn gauge(mut self, config: GaugeConfig) -> Self {
        self.config.gauge = config;
        self
    }

    /// Replace the heatmap layout.
    pub fn heatmap(mut self, config: HeatmapConfig) -> Self {
        self.config.heatmap = config;
        self
    }

    /// Replace the waterfall layout.
    pub fn waterfall(mut self, config: WaterfallConfig) -> Self {
        self.config.waterfall = config;
        self
    }

    /// Draw the grid (on by default).
    pub fn grid(mut self, grid: bool) -> Self {
        for axis in [&mut self.config.x, &mut self.config.y] {
            axis.grid_major = grid;
            axis.grid_minor = grid;
        }
        self
    }

    /// Show the tool buttons in the top-right corner (on by default).
    pub fn toolbar(mut self, toolbar: bool) -> Self {
        self.config.toolbar = toolbar;
        self
    }

    /// For streaming data: while the view is automatic, show only the last `width`
    /// x units.
    pub fn x_window(mut self, width: f64) -> Self {
        self.x_window = Some(width);
        self
    }

    /// Keep `point` inside the automatic view (for example `[0.0, 0.0]`).
    pub fn include(mut self, point: [f64; 2]) -> Self {
        self.include.extend(point);
        self
    }
}
