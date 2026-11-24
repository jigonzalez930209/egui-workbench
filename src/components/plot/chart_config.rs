//! Layout settings for the chart families that are not plain lines.

use egui::Color32;

/// Settings for the direction arrow drawn beside the newest point of a line.
///
/// The arrow uses the last `points` finite samples (5 by default), so a noisy stream
/// still shows the direction the buffer is growing. Only cartesian lines, areas, steps,
/// and candlesticks draw it; pie, radar, gauge, heatmap, and reference series do not.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectionArrowConfig {
    /// Draw the arrow.
    pub enabled: bool,
    /// Samples used to measure the direction; 5 by default.
    pub points: usize,
    /// Shaft length in points.
    pub length: f32,
    /// Shaft width in points.
    pub width: f32,
    /// Head length in points.
    pub head: f32,
    /// Arrow color; the series color when `None`.
    pub color: Option<Color32>,
}

impl Default for DirectionArrowConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            points: 5,
            length: 26.0,
            width: 2.0,
            head: 8.0,
            color: None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Layout of pie and donut charts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PieConfig {
    /// Inner radius as a fraction of the outer radius, 0 for a pie.
    pub inner_radius: f32,
    /// Where the first slice starts, in radians; -π/2 is 12 o'clock.
    pub start_angle: f32,
    /// Draw the value or percentage on slices with room for it.
    pub labels: bool,
    /// Show percentages instead of raw values.
    pub percentages: bool,
    /// Write the total in the middle of a donut.
    pub total: bool,
}

impl Default for PieConfig {
    fn default() -> Self {
        Self {
            inner_radius: 0.0,
            start_angle: -std::f32::consts::FRAC_PI_2,
            labels: true,
            percentages: true,
            total: true,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Layout of radar charts. One series draws one closed polygon; several series overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct RadarConfig {
    /// Names around the ring; indexed by each point's x. Numbers are used when empty.
    pub labels: Vec<String>,
    /// Fixed maximum; automatic when `None`.
    pub max: Option<f64>,
    /// Grid rings.
    pub rings: u8,
    /// Fill opacity of each series, 0–1.
    pub fill_opacity: f32,
}

impl Default for RadarConfig {
    fn default() -> Self {
        Self {
            labels: Vec::new(),
            max: None,
            rings: 4,
            fill_opacity: 0.16,
        }
    }
}

impl RadarConfig {
    /// A radar with `labels` around the ring.
    pub fn new(labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Layout of gauge charts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaugeConfig {
    /// Start of the arc in radians.
    pub start_angle: f32,
    /// End of the arc in radians.
    pub end_angle: f32,
    /// Arc thickness in points.
    pub thickness: f32,
    /// Draw the value under the center.
    pub show_value: bool,
    /// Draw the range ends.
    pub show_range: bool,
}

impl Default for GaugeConfig {
    fn default() -> Self {
        Self {
            start_angle: 0.75 * std::f32::consts::PI,
            end_angle: 2.25 * std::f32::consts::PI,
            thickness: 10.0,
            show_value: true,
            show_range: true,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Layout of heatmaps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatmapConfig {
    /// Fill opacity, 0–1.
    pub opacity: f32,
    /// Draw a value scale at the right edge.
    pub colorbar: bool,
    /// Color scale.
    pub colormap: super::Colormap,
}

impl Default for HeatmapConfig {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            colorbar: true,
            colormap: super::Colormap::Theme,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Layout of waterfall charts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterfallConfig {
    /// Bar width in data units; 80 % of the smallest x spacing when `None`.
    pub bar_width: Option<f64>,
    /// Dashed connectors between consecutive bars.
    pub connectors: bool,
}

impl Default for WaterfallConfig {
    fn default() -> Self {
        Self {
            bar_width: None,
            connectors: true,
        }
    }
}
