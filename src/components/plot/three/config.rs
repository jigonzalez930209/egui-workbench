//! Everything about a 3D plot that is not data: axes, camera, text, colors, and
//! per-series overrides. Edit a [`Plot3DConfig`] and pass it to [`super::Plot3D`].

use egui::Color32;

use super::super::animation::LegendHoverConfig;
use super::super::chart_config::DirectionArrowConfig;
use super::super::config::{AxisConfig, PlotColors, TitleAlign};
use super::super::series::{LineStyle, Marker};
use super::super::types::LegendPosition;
use super::camera::Projection;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Overrides for one 3D series, matched by name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Series3DStyle {
    /// The series name this applies to.
    pub name: String,
    /// Stroke and fill color.
    pub color: Option<Color32>,
    /// Line width in points.
    pub width: Option<f32>,
    /// Solid, dashed, or dotted lines.
    pub line_style: Option<LineStyle>,
    /// Marker shape for scatter series or line vertices.
    pub marker: Option<Marker>,
    /// Marker radius in points.
    pub marker_size: Option<f32>,
    /// Opacity of bars and surface cells, 0–1.
    pub fill_opacity: Option<f32>,
    /// Show the direction arrow for this line; [`Plot3DConfig::direction`] when `None`.
    pub direction_arrow: Option<bool>,
}

impl Series3DStyle {
    /// Whether nothing is overridden.
    pub fn is_empty(&self) -> bool {
        self.color.is_none()
            && self.width.is_none()
            && self.line_style.is_none()
            && self.marker.is_none()
            && self.marker_size.is_none()
            && self.fill_opacity.is_none()
            && self.direction_arrow.is_none()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Fill, lines, and color scale of a surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Surface3DConfig {
    /// Draw the grid lines over the cells.
    pub wireframe: bool,
    /// Fill opacity, 0–1.
    pub opacity: f32,
    /// Color cells from the height scale instead of the series color.
    pub color_scale: bool,
}

impl Default for Surface3DConfig {
    fn default() -> Self {
        Self {
            wireframe: true,
            opacity: 0.92,
            color_scale: true,
        }
    }
}

/// The complete appearance of a 3D plot.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Plot3DConfig {
    /// Title above the frame.
    pub title: Option<String>,
    /// The horizontal x axis.
    pub x: AxisConfig,
    /// The horizontal y axis (depth).
    pub y: AxisConfig,
    /// The vertical z axis.
    pub z: AxisConfig,
    /// Title size in points.
    pub title_size: f32,
    /// Title color; [`PlotColors::text`] (or the theme) when `None`.
    pub title_color: Option<Color32>,
    /// Title alignment over the frame.
    pub title_align: TitleAlign,
    /// Axis title size in points.
    pub label_size: f32,
    /// Tick label size in points.
    pub tick_size: f32,
    /// Legend text size in points.
    pub legend_size: f32,
    /// Frame line width in points.
    pub frame_width: f32,
    /// Grid line width in points.
    pub grid_width: f32,
    /// Opacity of major grid lines, 0–1.
    pub grid_major_opacity: f32,
    /// Opacity of minor grid lines, 0–1.
    pub grid_minor_opacity: f32,
    /// Length of tick marks.
    pub tick_length: f32,
    /// Multiplies every line width.
    pub line_scale: f32,
    /// Multiplies every marker size.
    pub marker_scale: f32,
    /// Legend corner or hidden.
    pub legend: LegendPosition,
    /// Draw the legend's background box.
    pub legend_frame: bool,
    /// Show the tool buttons.
    pub toolbar: bool,
    /// Dashed drop line from the hovered point to the floor.
    pub hover_guides: bool,
    /// The hover popup.
    pub hover_popup: bool,
    /// The animation played while a legend row is hovered.
    pub legend_hover: LegendHoverConfig,
    /// The arrow beside the newest point of a line.
    pub direction: DirectionArrowConfig,
    /// Surface fill and wireframe.
    pub surface: Surface3DConfig,
    /// Perspective or orthographic.
    pub projection: Projection,
    /// Initial rotation around the vertical axis, in radians.
    pub yaw: f32,
    /// Initial elevation, in radians.
    pub pitch: f32,
    /// Vertical field of view in degrees.
    pub fov: f32,
    /// Theme overrides.
    pub colors: PlotColors,
    /// Per-series overrides.
    pub series: Vec<Series3DStyle>,
    /// CPU or GPU filled surfaces.
    pub renderer: super::super::Renderer,
}

impl Default for Plot3DConfig {
    fn default() -> Self {
        Self {
            title: None,
            x: AxisConfig::default(),
            y: AxisConfig::default(),
            z: AxisConfig::default(),
            title_size: 14.0,
            title_color: None,
            title_align: TitleAlign::Center,
            label_size: 12.0,
            tick_size: 11.0,
            legend_size: 11.5,
            frame_width: 1.0,
            grid_width: 1.0,
            grid_major_opacity: 0.5,
            grid_minor_opacity: 0.18,
            tick_length: 4.0,
            line_scale: 1.0,
            marker_scale: 1.0,
            legend: LegendPosition::TopLeft,
            legend_frame: true,
            toolbar: true,
            hover_guides: true,
            hover_popup: true,
            legend_hover: LegendHoverConfig::default(),
            direction: DirectionArrowConfig::default(),
            surface: Surface3DConfig::default(),
            projection: Projection::Perspective,
            yaw: -0.72,
            pitch: 0.42,
            fov: 45.0,
            colors: PlotColors::default(),
            series: Vec::new(),
            renderer: super::super::Renderer::Cpu,
        }
    }
}

impl Plot3DConfig {
    /// The overrides for `name`, if any.
    pub fn series_style(&self, name: &str) -> Option<&Series3DStyle> {
        self.series.iter().find(|style| style.name == name)
    }

    /// The overrides for `name`, created empty when missing.
    pub fn series_style_mut(&mut self, name: &str) -> &mut Series3DStyle {
        if let Some(index) = self.series.iter().position(|style| style.name == name) {
            return &mut self.series[index];
        }
        self.series.push(Series3DStyle {
            name: name.to_owned(),
            ..Series3DStyle::default()
        });
        let last = self.series.len() - 1;
        &mut self.series[last]
    }

    /// Clamp every size, angle, and opacity to a sensible range.
    pub fn sanitize(&mut self) {
        self.title_size = self.title_size.clamp(6.0, 48.0);
        self.label_size = self.label_size.clamp(6.0, 36.0);
        self.tick_size = self.tick_size.clamp(6.0, 30.0);
        self.legend_size = self.legend_size.clamp(6.0, 30.0);
        self.frame_width = self.frame_width.clamp(0.0, 6.0);
        self.grid_width = self.grid_width.clamp(0.5, 6.0);
        self.grid_major_opacity = self.grid_major_opacity.clamp(0.0, 1.0);
        self.grid_minor_opacity = self.grid_minor_opacity.clamp(0.0, 1.0);
        self.tick_length = self.tick_length.clamp(0.0, 16.0);
        self.line_scale = self.line_scale.clamp(0.25, 8.0);
        self.marker_scale = self.marker_scale.clamp(0.25, 8.0);
        self.legend_hover.period = self.legend_hover.period.clamp(0.2, 10.0);
        self.direction.points = self.direction.points.clamp(2, 64);
        self.direction.length = self.direction.length.clamp(4.0, 160.0);
        self.direction.width = self.direction.width.clamp(0.5, 8.0);
        self.direction.head = self.direction.head.clamp(2.0, 40.0);
        self.surface.opacity = self.surface.opacity.clamp(0.0, 1.0);
        if !self.yaw.is_finite() {
            self.yaw = Plot3DConfig::default().yaw;
        }
        if !self.pitch.is_finite() {
            self.pitch = Plot3DConfig::default().pitch;
        }
        self.pitch = self.pitch.clamp(-1.5, 1.5);
        self.fov = if self.fov.is_finite() {
            self.fov.clamp(15.0, 120.0)
        } else {
            45.0
        };
        for axis in [&mut self.x, &mut self.y, &mut self.z] {
            axis.label_size = axis.label_size.map(|size| size.clamp(6.0, 36.0));
            if let (Some(min), Some(max)) = (axis.min, axis.max)
                && min >= max
            {
                axis.max = None;
            }
            if axis.scale == super::super::axis::Scale::Log10 {
                axis.min = axis.min.filter(|min| *min > 0.0);
                axis.max = axis.max.filter(|max| *max > 0.0);
            }
            axis.major_step = axis.major_step.filter(|step| step.is_finite() && *step > 0.0);
            if let super::super::config::Minor::Divisions(divisions) = axis.minor {
                axis.minor = super::super::config::Minor::Divisions(divisions.clamp(2, 20));
            }
            axis.decimals = axis.decimals.map(|decimals| decimals.min(10));
        }
        for style in &mut self.series {
            style.width = style.width.map(|width| width.clamp(0.25, 12.0));
            style.marker_size = style.marker_size.map(|size| size.clamp(0.5, 20.0));
            style.fill_opacity = style.fill_opacity.map(|opacity| opacity.clamp(0.0, 1.0));
        }
    }
}
