//! The [`Plot3D`] builder methods.

use egui::Id;

use super::super::axis::Scale;
use super::super::{DirectionArrowConfig, LegendAnimation, LegendHoverConfig, LegendPosition};
use super::{Bounds3D, Plot3D, Plot3DConfig, Projection, Series3D, Surface3DConfig};

impl<'a> Plot3D<'a> {
    /// A 3D plot with a stable id.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            id: Id::unique(("workbench_plot3d", id)),
            series: Vec::new(),
            config: Plot3DConfig::default(),
            width: None,
            height: None,
            include: Bounds3D::NOTHING,
            settings_button: false,
            export_button: false,
        }
    }

    /// Replace the whole appearance.
    pub fn config(mut self, config: Plot3DConfig) -> Self {
        self.config = config;
        self
    }

    /// The current appearance.
    pub fn current_config(&self) -> &Plot3DConfig {
        &self.config
    }

    /// The names of the series, in drawing order.
    pub fn series_names(&self) -> Vec<String> {
        self.series.iter().map(|series| series.name.clone()).collect()
    }

    /// Add a series. Series draw in order; the palette color follows the order too.
    pub fn series(mut self, series: Series3D<'a>) -> Self {
        self.series.push(series);
        self
    }

    /// A title above the frame.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    /// The x axis title.
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.config.x.label = Some(label.into());
        self
    }

    /// The y axis title.
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.config.y.label = Some(label.into());
        self
    }

    /// The z axis title.
    pub fn z_label(mut self, label: impl Into<String>) -> Self {
        self.config.z.label = Some(label.into());
        self
    }

    /// Fixed x limits in data units.
    pub fn x_limits(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        (self.config.x.min, self.config.x.max) = (min, max);
        self
    }

    /// Fixed y limits in data units.
    pub fn y_limits(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        (self.config.y.min, self.config.y.max) = (min, max);
        self
    }

    /// Fixed z limits in data units.
    pub fn z_limits(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        (self.config.z.min, self.config.z.max) = (min, max);
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

    /// Linear or log10 z axis.
    pub fn z_scale(mut self, scale: Scale) -> Self {
        self.config.z.scale = scale;
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

    /// The animation played while the pointer rests on a legend row.
    pub fn legend_animation(mut self, animation: LegendAnimation) -> Self {
        self.config.legend_hover.animation = animation;
        self
    }

    /// Replace the direction-arrow settings.
    pub fn direction(mut self, config: DirectionArrowConfig) -> Self {
        self.config.direction = config;
        self
    }

    /// Show or hide the direction arrow beside the newest line point.
    pub fn direction_arrow(mut self, show: bool) -> Self {
        self.config.direction.enabled = show;
        self
    }

    /// How many trailing samples measure the direction arrow.
    pub fn direction_points(mut self, points: usize) -> Self {
        self.config.direction.points = points.max(2);
        self
    }

    /// Perspective or orthographic.
    pub fn projection(mut self, projection: Projection) -> Self {
        self.config.projection = projection;
        self
    }

    /// Initial yaw and pitch in radians.
    pub fn camera_angles(mut self, yaw: f32, pitch: f32) -> Self {
        (self.config.yaw, self.config.pitch) = (yaw, pitch);
        self
    }

    /// Vertical field of view in degrees.
    pub fn fov(mut self, fov: f32) -> Self {
        self.config.fov = fov;
        self
    }

    /// Replace the surface fill and wireframe settings.
    pub fn surface(mut self, config: Surface3DConfig) -> Self {
        self.config.surface = config;
        self
    }

    /// Draw the grid (on by default).
    pub fn grid(mut self, grid: bool) -> Self {
        for axis in [&mut self.config.x, &mut self.config.y, &mut self.config.z] {
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

    /// Keep `point` inside the automatic view.
    pub fn include(mut self, point: [f64; 3]) -> Self {
        self.include.extend(point);
        self
    }

    /// Show a settings button in the toolbar.
    pub fn settings_button(mut self, show: bool) -> Self {
        self.settings_button = show;
        self
    }

    /// Show an SVG/PNG export menu in the toolbar.
    pub fn export_button(mut self, show: bool) -> Self {
        self.export_button = show;
        self
    }
}
