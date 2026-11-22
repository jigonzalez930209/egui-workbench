//! The data a plot draws: lines, scatter, bars, areas, steps, and reference lines.

use std::borrow::Cow;

use egui::Color32;

use super::axis::{Bounds, Scale};

/// Point markers for scatter series and line vertices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Marker {
    /// Filled circle.
    #[default]
    Circle,
    /// Filled square.
    Square,
    /// Filled diamond.
    Diamond,
    /// Filled upward triangle.
    Triangle,
    /// Diagonal cross (stroke only).
    Cross,
    /// Plus sign (stroke only).
    Plus,
    /// Circle outline.
    Ring,
}

/// How a line is stroked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineStyle {
    /// Continuous.
    #[default]
    Solid,
    /// 6 pt dashes, 4 pt gaps.
    Dashed,
    /// 1.5 pt dots, 3 pt gaps.
    Dotted,
}

/// Which vertical axis a series is measured against.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum YAxis {
    /// The primary axis on the left.
    #[default]
    Left,
    /// The secondary axis on the right; it appears as soon as one series uses it.
    Right,
}

/// How a line passes through its points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Smoothing {
    /// Straight segments between points.
    #[default]
    None,
    /// A centripetal Catmull–Rom spline through every point: smooth, no loops or
    /// cusps, works in any point order.
    Spline,
    /// A monotone cubic (Fritsch–Carlson): never overshoots between points, so peaks
    /// stay at the data. Needs increasing x; falls back to [`Smoothing::Spline`].
    Monotone,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Kind {
    Line {
        width: f32,
        style: LineStyle,
        fill_to: Option<f64>,
        step: bool,
        markers: Option<Marker>,
        smoothing: Smoothing,
    },
    Scatter {
        marker: Marker,
        radius: f32,
    },
    Bars {
        width: Option<f64>,
        base: f64,
    },
    HLine(f64),
    VLine(f64),
    /// One slice of a pie or donut. The series name is the slice label.
    Pie {
        value: f64,
    },
    /// Open–high–low–close candles; the projected close is the series' data.
    Candles {
        width: Option<f64>,
    },
    /// A regular x/y/value grid.
    Heatmap,
    /// A closed polygon over the configured radar axes.
    Radar,
    /// One gauge, with its own range.
    Gauge {
        value: f64,
        min: f64,
        max: f64,
    },
    /// A cumulative bar chart of signed changes.
    Waterfall {
        width: Option<f64>,
    },
}

/// One named data series. Build it with [`Series::line`], [`Series::scatter`],
/// [`Series::bars`], [`Series::area`], [`Series::step`], [`Series::function`],
/// [`Series::hline`], or [`Series::vline`], then add it to a [`super::Plot`].
///
/// Data is borrowed when possible (`&[[f64; 2]]`), so a million-point buffer is not
/// copied every frame. Non-finite points break a line into segments.
#[derive(Clone, Debug)]
pub struct Series<'a> {
    pub(super) name: String,
    pub(super) data: Cow<'a, [[f64; 2]]>,
    pub(super) kind: Kind,
    pub(super) color: Option<Color32>,
    pub(super) y_errors: Option<Cow<'a, [f64]>>,
    pub(super) legend: bool,
    pub(super) axis: YAxis,
    /// OHLC rows for [`Kind::Candles`].
    pub(super) ohlc: Option<Cow<'a, [[f64; 5]]>>,
    /// x, y, value rows for [`Kind::Heatmap`].
    pub(super) cells: Option<Cow<'a, [[f64; 3]]>>,
}

impl<'a> Series<'a> {
    pub(super) fn new(name: impl Into<String>, data: Cow<'a, [[f64; 2]]>, kind: Kind) -> Self {
        Self {
            name: name.into(),
            data,
            kind,
            color: None,
            y_errors: None,
            legend: true,
            axis: YAxis::Left,
            ohlc: None,
            cells: None,
        }
    }

    /// A 1.5 pt polyline through the points, in order.
    pub fn line(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 2]]>>) -> Self {
        Self::new(
            name,
            data.into(),
            Kind::Line {
                width: 1.5,
                style: LineStyle::Solid,
                fill_to: None,
                step: false,
                markers: None,
                smoothing: Smoothing::None,
            },
        )
    }

    /// A line drawn as a smooth spline through the points (see [`Series::smooth`]).
    pub fn spline(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 2]]>>) -> Self {
        Self::line(name, data).smooth(Smoothing::Spline)
    }

    /// A line filled down to `y = 0` with 18 % of its color.
    pub fn area(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 2]]>>) -> Self {
        Self::line(name, data).fill_to(0.0)
    }

    /// A staircase line: each value holds until the next x.
    pub fn step(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 2]]>>) -> Self {
        let mut series = Self::line(name, data);
        if let Kind::Line { step, .. } = &mut series.kind {
            *step = true;
        }
        series
    }

    /// Unconnected markers (3 pt radius circles by default).
    pub fn scatter(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 2]]>>) -> Self {
        Self::new(
            name,
            data.into(),
            Kind::Scatter {
                marker: Marker::Circle,
                radius: 3.0,
            },
        )
    }

    /// Vertical bars from `y = 0` to each point; the width defaults to 80 % of the
    /// smallest x spacing.
    pub fn bars(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 2]]>>) -> Self {
        Self::new(name, data.into(), Kind::Bars { width: None, base: 0.0 })
    }

    /// `f(x)` sampled `samples` times over `x_range`.
    pub fn function(
        name: impl Into<String>,
        x_range: std::ops::RangeInclusive<f64>,
        samples: usize,
        f: impl Fn(f64) -> f64,
    ) -> Series<'static> {
        let samples = samples.max(2);
        let (start, end) = (*x_range.start(), *x_range.end());
        let data: Vec<[f64; 2]> = (0..samples)
            .map(|index| {
                let x = start + (end - start) * index as f64 / (samples - 1) as f64;
                [x, f(x)]
            })
            .collect();
        Series::line(name, data)
    }

    /// A horizontal reference line across the whole plot.
    pub fn hline(name: impl Into<String>, y: f64) -> Series<'static> {
        Series::new(name, Cow::Owned(Vec::new()), Kind::HLine(y))
    }

    /// A vertical reference line across the whole plot.
    pub fn vline(name: impl Into<String>, x: f64) -> Series<'static> {
        Series::new(name, Cow::Owned(Vec::new()), Kind::VLine(x))
    }

    /// Explicit color; the theme palette is used in order otherwise.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Line width in points (lines, steps, areas, reference lines).
    pub fn width(mut self, new_width: f32) -> Self {
        if let Kind::Line { width, .. } = &mut self.kind {
            *width = new_width.max(0.5);
        }
        self
    }

    /// Dashed or dotted lines.
    pub fn style(mut self, new_style: LineStyle) -> Self {
        if let Kind::Line { style, .. } = &mut self.kind {
            *style = new_style;
        }
        self
    }

    /// Fill between the line and `y = base`.
    pub fn fill_to(mut self, base: f64) -> Self {
        if let Kind::Line { fill_to, .. } = &mut self.kind {
            *fill_to = Some(base);
        }
        self
    }

    /// Smooth the line through its points (lines and areas; steps stay square).
    /// Dense lines (more than one point per 2 pt) are drawn straight: a spline would
    /// not change what you see.
    pub fn smooth(mut self, new_smoothing: Smoothing) -> Self {
        if let Kind::Line { smoothing, .. } = &mut self.kind {
            *smoothing = new_smoothing;
        }
        self
    }

    /// Measure the series against the left (default) or the right y axis.
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Draw a marker on every vertex of a line, or change the scatter marker.
    pub fn marker(mut self, new_marker: Marker) -> Self {
        match &mut self.kind {
            Kind::Line { markers, .. } => *markers = Some(new_marker),
            Kind::Scatter { marker, .. } => *marker = new_marker,
            _ => {}
        }
        self
    }

    /// Marker radius in points (scatter).
    pub fn radius(mut self, new_radius: f32) -> Self {
        if let Kind::Scatter { radius, .. } = &mut self.kind {
            *radius = new_radius.max(1.0);
        }
        self
    }

    /// Bar width in data units (bars, candles, and waterfall bars).
    pub fn bar_width(mut self, new_width: f64) -> Self {
        match &mut self.kind {
            Kind::Bars { width, .. } | Kind::Candles { width } | Kind::Waterfall { width } => {
                *width = Some(new_width);
            }
            _ => {}
        }
        self
    }

    /// Where bars start (default 0).
    pub fn base(mut self, new_base: f64) -> Self {
        if let Kind::Bars { base, .. } = &mut self.kind {
            *base = new_base;
        }
        self
    }

    /// Symmetric ±error per point, drawn as whiskers.
    pub fn y_errors(mut self, errors: impl Into<Cow<'a, [f64]>>) -> Self {
        self.y_errors = Some(errors.into());
        self
    }

    /// Leave the series out of the legend.
    pub fn hide_from_legend(mut self) -> Self {
        self.legend = false;
        self
    }

    /// The series name (also the legend label).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The points.
    pub fn data(&self) -> &[[f64; 2]] {
        &self.data
    }

    pub(super) fn is_reference(&self) -> bool {
        matches!(self.kind, Kind::HLine(_) | Kind::VLine(_))
    }

    /// Candles, heatmaps, and waterfalls have their own cartesian renderer.
    pub(super) fn is_extra(&self) -> bool {
        matches!(self.kind, Kind::Candles { .. } | Kind::Heatmap | Kind::Waterfall { .. })
    }

    /// Whether this is a line-like series that can show a direction arrow.
    pub(super) fn has_direction(&self) -> bool {
        matches!(self.kind, Kind::Line { .. } | Kind::Candles { .. })
    }

    /// A cheap key for [`Self::bounds`]: the buffers' addresses and lengths, the kind, the
    /// scales, and 16 sampled values. A different buffer, length, or sampled value is a
    /// different key, so appended or replaced data refreshes the cached extent.
    pub(super) fn fingerprint(&self, x_scale: Scale, y_scale: Scale) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (self.data.as_ptr() as usize, self.data.len()).hash(&mut hasher);
        if let Some(errors) = &self.y_errors {
            (errors.as_ptr() as usize, errors.len()).hash(&mut hasher);
        }
        if let Some(rows) = &self.ohlc {
            (rows.as_ptr() as usize, rows.len()).hash(&mut hasher);
            let step = (rows.len() / 8).max(1);
            for row in rows.iter().step_by(step) {
                for value in row {
                    value.to_bits().hash(&mut hasher);
                }
            }
        }
        if let Some(cells) = &self.cells {
            (cells.as_ptr() as usize, cells.len()).hash(&mut hasher);
            let step = (cells.len() / 8).max(1);
            for cell in cells.iter().step_by(step) {
                for value in cell {
                    value.to_bits().hash(&mut hasher);
                }
            }
        }
        format!("{:?}{x_scale:?}{y_scale:?}", self.kind).hash(&mut hasher);
        let step = (self.data.len() / 16).max(1);
        for point in self.data.iter().step_by(step).chain(self.data.last()) {
            (point[0].to_bits(), point[1].to_bits()).hash(&mut hasher);
        }
        hasher.finish()
    }
}

/// Indices of the points of `data` inside `bounds` (data space), for selections.
pub fn points_in(data: &[[f64; 2]], bounds: &Bounds) -> Vec<usize> {
    data.iter()
        .enumerate()
        .filter(|(_, point)| bounds.contains(**point))
        .map(|(index, _)| index)
        .collect()
}
