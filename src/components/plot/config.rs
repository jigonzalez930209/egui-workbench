//! Everything about a plot that is not data: text, sizes, colors, axes, grid, legend,
//! and per-series overrides. Edit a [`PlotConfig`] (for example in a settings dialog)
//! and pass it to [`super::Plot::config`]; the screen and the SVG/PNG export use it.

use egui::Color32;

use super::LegendPosition;
use super::animation::LegendHoverConfig;
use super::axis::Scale;
use super::chart_config::{DirectionArrowConfig, GaugeConfig, HeatmapConfig, PieConfig, RadarConfig, WaterfallConfig};
use super::measure::MeasurementConfig;
use super::series::{LineStyle, Marker, Smoothing, YAxis};

/// Which renderer draws the data layer.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Renderer {
    /// The CPU painter: the reference, snapshots, and the fallback.
    #[default]
    Cpu,
    /// GPU lines and scatter through the `gpu` feature; charts, axes, and legend stay CPU.
    Gpu,
}

/// Where the chart title sits along the top.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TitleAlign {
    /// Over the left edge of the data area.
    Left,
    /// Centered over the data area.
    #[default]
    Center,
    /// Over the right edge of the data area.
    Right,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// How many minor ticks sit between two major ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Minor {
    /// Chosen from the space between majors (5, 2, or none); 2…9 per decade on log axes.
    #[default]
    Auto,
    /// No minor ticks.
    Off,
    /// This many subdivisions per major interval (2 = one minor tick in the middle).
    Divisions(u8),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// One axis: range, scale, ticks, grid, and labels.
#[derive(Clone, Debug, PartialEq)]
pub struct AxisConfig {
    /// The axis title.
    pub label: Option<String>,
    /// Linear or log10.
    pub scale: Scale,
    /// Fixed lower limit in data units; automatic when `None`.
    pub min: Option<f64>,
    /// Fixed upper limit in data units; automatic when `None`.
    pub max: Option<f64>,
    /// Distance between major ticks in data units (decades on log axes); 1-2-5 when
    /// `None`.
    pub major_step: Option<f64>,
    /// Minor ticks between majors.
    pub minor: Minor,
    /// Grid lines at major ticks.
    pub grid_major: bool,
    /// Grid lines at minor ticks.
    pub grid_minor: bool,
    /// Decimals of the tick labels; from the step when `None`.
    pub decimals: Option<u8>,
    /// Draw the tick labels.
    pub tick_labels: bool,
    /// Axis title size in points; [`PlotConfig::label_size`] when `None`.
    pub label_size: Option<f32>,
    /// Axis title color; [`PlotColors::text`] (or the theme) when `None`.
    pub label_color: Option<Color32>,
    /// Tick labels and tick marks; the theme when `None`. Coloring the right axis like
    /// its series tells the two axes apart.
    pub tick_color: Option<Color32>,
    /// Labels for [`Scale::Category`], in index order.
    pub categories: Option<Vec<String>>,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            label: None,
            scale: Scale::Linear,
            min: None,
            max: None,
            major_step: None,
            minor: Minor::Auto,
            grid_major: true,
            grid_minor: true,
            decimals: None,
            tick_labels: true,
            label_size: None,
            label_color: None,
            tick_color: None,
            categories: None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Optional color overrides; `None` follows the theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlotColors {
    /// Fill of the data area.
    pub background: Option<Color32>,
    /// Fill around the data area (transparent on screen by default).
    pub canvas: Option<Color32>,
    /// The frame and tick marks.
    pub frame: Option<Color32>,
    /// Grid lines (before the major/minor opacity).
    pub grid: Option<Color32>,
    /// Title and axis titles.
    pub text: Option<Color32>,
    /// Tick labels.
    pub tick_text: Option<Color32>,
    /// Selection fill and outline.
    pub selection: Option<Color32>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Overrides for one series, matched by name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SeriesStyle {
    /// The series name this applies to.
    pub name: String,
    /// Stroke and fill color.
    pub color: Option<Color32>,
    /// Line width in points.
    pub width: Option<f32>,
    /// Solid, dashed, or dotted.
    pub line_style: Option<LineStyle>,
    /// Marker shape (scatter, or markers on line vertices).
    pub marker: Option<Marker>,
    /// Marker radius in points.
    pub marker_size: Option<f32>,
    /// Opacity of area and bar fills, 0–1.
    pub fill_opacity: Option<f32>,
    /// Straight, spline, or monotone segments (lines and areas).
    pub smoothing: Option<Smoothing>,
    /// Move the series to the left or the right y axis.
    pub y_axis: Option<YAxis>,
    /// Show the direction arrow for this series; [`PlotConfig::direction`] when `None`.
    pub direction_arrow: Option<bool>,
}

impl SeriesStyle {
    /// Whether nothing is overridden (a settings dialog can drop it).
    pub fn is_empty(&self) -> bool {
        self.color.is_none()
            && self.width.is_none()
            && self.line_style.is_none()
            && self.marker.is_none()
            && self.marker_size.is_none()
            && self.fill_opacity.is_none()
            && self.smoothing.is_none()
            && self.y_axis.is_none()
            && self.direction_arrow.is_none()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The complete appearance of a plot.
#[derive(Clone, Debug, PartialEq)]
pub struct PlotConfig {
    /// Title above the data area.
    pub title: Option<String>,
    /// The horizontal axis.
    pub x: AxisConfig,
    /// The vertical axis on the left.
    pub y: AxisConfig,
    /// The secondary vertical axis on the right, shown when a series uses
    /// [`YAxis::Right`]. Its grid is off by default.
    pub y2: AxisConfig,
    /// Title size in points.
    pub title_size: f32,
    /// Title color; [`PlotColors::text`] (or the theme) when `None`.
    pub title_color: Option<Color32>,
    /// Title alignment over the data area.
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
    /// Length of the major tick marks outside the frame (minor marks are half).
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
    /// Dashed guides from the hovered point to the axes.
    pub hover_guides: bool,
    /// The hover popup.
    pub hover_popup: bool,
    /// The animation played while a legend row is hovered.
    pub legend_hover: LegendHoverConfig,
    /// The arrow beside the newest point of a line (for streaming data).
    pub direction: DirectionArrowConfig,
    /// Pie and donut layout.
    pub pie: PieConfig,
    /// Radar axes and fill.
    pub radar: RadarConfig,
    /// Gauge arc and labels.
    pub gauge: GaugeConfig,
    /// Heatmap opacity and color scale.
    pub heatmap: HeatmapConfig,
    /// Waterfall width and connectors.
    pub waterfall: WaterfallConfig,
    /// Theme overrides.
    pub colors: PlotColors,
    /// Per-series overrides.
    pub series: Vec<SeriesStyle>,
    /// Measurement mode, snapping, and labels.
    pub measurement: MeasurementConfig,
    /// CPU or GPU data layer.
    pub renderer: Renderer,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            title: None,
            x: AxisConfig::default(),
            y: AxisConfig::default(),
            y2: AxisConfig {
                grid_major: false,
                grid_minor: false,
                ..AxisConfig::default()
            },
            title_size: 14.0,
            title_color: None,
            title_align: TitleAlign::Center,
            label_size: 12.0,
            tick_size: 11.0,
            legend_size: 11.5,
            frame_width: 1.0,
            grid_width: 1.0,
            grid_major_opacity: 0.55,
            grid_minor_opacity: 0.22,
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
            pie: PieConfig::default(),
            radar: RadarConfig::default(),
            gauge: GaugeConfig::default(),
            heatmap: HeatmapConfig::default(),
            waterfall: WaterfallConfig::default(),
            colors: PlotColors::default(),
            series: Vec::new(),
            measurement: MeasurementConfig::default(),
            renderer: Renderer::Cpu,
        }
    }
}

impl PlotConfig {
    /// The overrides for `name`, if any.
    pub fn series_style(&self, name: &str) -> Option<&SeriesStyle> {
        self.series.iter().find(|style| style.name == name)
    }

    /// The overrides for `name`, created empty when missing.
    pub fn series_style_mut(&mut self, name: &str) -> &mut SeriesStyle {
        if let Some(index) = self.series.iter().position(|style| style.name == name) {
            return &mut self.series[index];
        }
        self.series.push(SeriesStyle {
            name: name.to_owned(),
            ..SeriesStyle::default()
        });
        let last = self.series.len() - 1;
        &mut self.series[last]
    }

    /// Clamp every size and opacity to a sensible range (after editing in a dialog).
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
        self.pie.inner_radius = self.pie.inner_radius.clamp(0.0, 0.9);
        if !self.pie.start_angle.is_finite() {
            self.pie.start_angle = -std::f32::consts::FRAC_PI_2;
        }
        self.radar.rings = self.radar.rings.clamp(1, 12);
        self.radar.fill_opacity = self.radar.fill_opacity.clamp(0.0, 1.0);
        self.radar.max = self.radar.max.filter(|max| max.is_finite() && *max > 0.0);
        self.gauge.thickness = self.gauge.thickness.clamp(2.0, 40.0);
        if !(self.gauge.start_angle.is_finite() && self.gauge.end_angle.is_finite())
            || (self.gauge.end_angle - self.gauge.start_angle).abs() < 0.1
        {
            self.gauge.start_angle = 0.75 * std::f32::consts::PI;
            self.gauge.end_angle = 2.25 * std::f32::consts::PI;
        }
        self.heatmap.opacity = self.heatmap.opacity.clamp(0.0, 1.0);
        self.waterfall.bar_width = self
            .waterfall
            .bar_width
            .filter(|width| width.is_finite() && *width > 0.0);
        for axis in [&mut self.x, &mut self.y, &mut self.y2] {
            axis.label_size = axis.label_size.map(|size| size.clamp(6.0, 36.0));
            if let (Some(min), Some(max)) = (axis.min, axis.max)
                && min >= max
            {
                axis.max = None;
            }
            if axis.scale == Scale::Log10 {
                axis.min = axis.min.filter(|min| *min > 0.0);
                axis.max = axis.max.filter(|max| *max > 0.0);
            }
            axis.major_step = axis.major_step.filter(|step| step.is_finite() && *step > 0.0);
            if let Minor::Divisions(divisions) = axis.minor {
                axis.minor = Minor::Divisions(divisions.clamp(2, 20));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_drops_impossible_limits() {
        let mut config = PlotConfig::default();
        config.x.min = Some(5.0);
        config.x.max = Some(1.0);
        config.y.scale = Scale::Log10;
        config.y.min = Some(-1.0);
        config.y.major_step = Some(0.0);
        config.title_size = 400.0;
        config.legend_hover.period = 0.0;
        config.direction.points = 0;
        config.pie.inner_radius = 4.0;
        config.radar.rings = 0;
        config.gauge.thickness = 900.0;
        config.heatmap.opacity = 4.0;
        config.waterfall.bar_width = Some(-1.0);
        config.sanitize();
        assert_eq!((config.x.min, config.x.max), (Some(5.0), None));
        assert_eq!(config.y.min, None);
        assert_eq!(config.y.major_step, None);
        assert_eq!(config.title_size, 48.0);
        assert_eq!(config.legend_hover.period, 0.2);
        assert_eq!(config.direction.points, 2);
        assert_eq!(config.pie.inner_radius, 0.9);
        assert_eq!(config.radar.rings, 1);
        assert_eq!(config.gauge.thickness, 40.0);
        assert_eq!(config.heatmap.opacity, 1.0);
        assert_eq!(config.waterfall.bar_width, None);
    }

    #[test]
    fn series_styles_are_found_or_created() {
        let mut config = PlotConfig::default();
        config.series_style_mut("a").width = Some(3.0);
        config.series_style_mut("a").color = Some(Color32::RED);
        assert_eq!(config.series.len(), 1);
        assert_eq!(config.series_style("a").and_then(|style| style.width), Some(3.0));
    }
}
