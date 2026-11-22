//! The [`Plot`] builder methods.

use egui::Id;

use super::axis::{Bounds, Scale};
use super::config::PlotConfig;
use super::fit::{Fit, FitKind};
use super::series::Series;
use super::{LegendPosition, Plot};

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
