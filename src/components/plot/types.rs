//! Public result types and small enums shared across the plot modules.

use egui::{Pos2, Rect, Response};

use super::axis::{Bounds, Transform};
use super::series::YAxis;

/// A file format the toolbar's export menu asked for.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExportFormat {
    /// Scalable vector graphics.
    Svg,
    /// A raster image.
    Png,
    /// Vector PDF.
    Pdf,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
/// What a primary-button drag does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlotTool {
    /// Drag moves the view.
    #[default]
    Pan,
    /// Drag draws a box and zooms to it.
    Zoom,
    /// Drag draws a box and reports it as the selection.
    Select,
    /// Drag measures the distance and slope between two points.
    MeasureDistance,
    /// Drag measures a rectangular region: area, extrema, slopes, and integrals.
    MeasureArea,
}

/// The family of a plot, inferred from its series.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChartMode {
    /// x/y axes: lines, bars, candles, heatmaps, waterfalls.
    #[default]
    Cartesian,
    /// Slices from the center of the frame.
    Pie,
    /// Closed polygons over labeled axes.
    Radar,
    /// One arc per series.
    Gauge,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Drag {
    Pan,
    Box { start: Pos2, select: bool },
    Measure { start: Pos2, distance: bool },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The nearest data point under the pointer.
#[derive(Clone, Debug, PartialEq)]
pub struct PlotHover {
    /// Series name.
    pub series: String,
    /// Index into the series data.
    pub index: usize,
    /// The point, in data coordinates.
    pub point: [f64; 2],
    /// The y axis the series is measured on.
    pub axis: YAxis,
}

/// What happened in the plot this frame.
pub struct PlotResponse {
    /// The response of the whole plot area.
    pub response: Response,
    /// The pointer position in data coordinates, while over the frame.
    pub pointer: Option<[f64; 2]>,
    /// The nearest point within 12 pt of the pointer.
    pub hovered: Option<PlotHover>,
    /// A point that was clicked.
    pub clicked: Option<PlotHover>,
    /// The selection changed (drawn, or cleared with a click).
    pub selection_changed: bool,
    /// The visible range in data coordinates (x and the left y axis).
    pub view: Bounds,
    /// The visible range of the right y axis in data coordinates, when it is shown.
    pub y2_view: Option<[f64; 2]>,
    /// The data area on screen (inside the axes).
    pub frame: Rect,
    /// The toolbar's settings button was clicked: open your settings dialog.
    pub settings_requested: bool,
    /// The toolbar's export menu chose a format: call [`super::Plot::to_svg`] or
    /// [`super::Plot::to_png`] and save the result.
    pub export_requested: Option<ExportFormat>,
    pub(super) transform: Transform,
}

impl PlotResponse {
    /// Where a data point lands on screen this frame, for annotations drawn on top.
    pub fn screen_pos(&self, point: [f64; 2]) -> Pos2 {
        self.transform.to_screen(point)
    }

    /// Where a point measured on `axis` lands on screen this frame.
    pub fn screen_pos_on(&self, axis: YAxis, point: [f64; 2]) -> Pos2 {
        self.transform.for_axis(axis).to_screen(point)
    }

    /// The data coordinates of a screen position.
    pub fn data_pos(&self, pos: Pos2) -> [f64; 2] {
        self.transform.to_data(pos)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
/// Where the legend sits inside the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum LegendPosition {
    /// Top-left corner.
    #[default]
    TopLeft,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    BottomRight,
    /// No legend.
    Hidden,
}
