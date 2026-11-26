//! Distance and area measurements over a plot.
//!
//! A measurement is stored in data coordinates in [`PlotState`], so it survives zoom,
//! pan, and a saved workspace. The plot draws the last frame's geometry and the public
//! summaries expose the same numbers to the application.

use egui::{Align2, Color32, Pos2, Rect, pos2, vec2};

use super::axis::Bounds;
use super::scene::{Dash, Measure, Prim, Scene};
use super::transform::Transform;
use super::{Plot, PlotState};
use crate::theme::Theme;

mod curve;

pub use curve::{CurveAreaSummary, curve_area_summary};

/// How the area tool measures.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum MeasurementMode {
    /// Drag a rectangle and measure its area and the series inside it.
    #[default]
    Rectangle,
    /// Drag between two points on the same series and measure the area between the
    /// curve and the chord.
    CurveSegment,
}

/// Which area a curve-segment label shows first.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum MeasurementAreaStyle {
    /// Show the signed area (positive above the chord, negative below).
    Signed,
    /// Show the absolute area between the curve and the chord.
    #[default]
    Absolute,
}

/// Measurement settings owned by the chart configuration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasurementConfig {
    /// Rectangle or curve-segment area mode.
    pub mode: MeasurementMode,
    /// Which area the label emphasizes.
    pub area_style: MeasurementAreaStyle,
    /// Snap radius in points.
    pub snap_radius: f32,
    /// Draw measurement labels over the data.
    pub show_labels: bool,
}

impl Default for MeasurementConfig {
    fn default() -> Self {
        Self {
            mode: MeasurementMode::Rectangle,
            area_style: MeasurementAreaStyle::Absolute,
            snap_radius: 14.0,
            show_labels: true,
        }
    }
}

/// A persistent measurement drawn over the data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum PlotMeasurement {
    /// A line between two data points.
    Distance {
        /// First point in data coordinates.
        start: [f64; 2],
        /// Second point in data coordinates.
        end: [f64; 2],
    },
    /// A rectangular region in data coordinates.
    Area {
        /// Region corners, in any order.
        bounds: Bounds,
    },
    /// The area between one curve segment and its chord, from `start` to `end`.
    CurveArea {
        /// Series the segment belongs to.
        series: String,
        /// First point in data coordinates.
        start: [f64; 2],
        /// Second point in data coordinates.
        end: [f64; 2],
        /// Index of `start` in the series data.
        start_index: usize,
        /// Index of `end` in the series data.
        end_index: usize,
    },
}

impl PlotMeasurement {
    /// A distance between two points.
    pub fn distance(start: [f64; 2], end: [f64; 2]) -> Self {
        Self::Distance { start, end }
    }

    /// A rectangular region.
    pub fn area(bounds: Bounds) -> Self {
        Self::Area { bounds }
    }

    /// The area between a curve segment and its chord.
    pub fn curve_area(
        series: impl Into<String>,
        start: [f64; 2],
        end: [f64; 2],
        start_index: usize,
        end_index: usize,
    ) -> Self {
        Self::CurveArea {
            series: series.into(),
            start,
            end,
            start_index,
            end_index,
        }
    }

    /// The region this measurement covers.
    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Distance { start, end } => Bounds::from_corners(*start, *end),
            Self::Area { bounds } => *bounds,
            Self::CurveArea { start, end, .. } => Bounds::from_corners(*start, *end),
        }
    }

    /// The distance, slope, and angle summary for [`Self::Distance`].
    pub fn distance_summary(&self) -> Option<DistanceSummary> {
        let Self::Distance { start, end } = self else {
            return None;
        };
        let dx = end[0] - start[0];
        let dy = end[1] - start[1];
        Some(DistanceSummary {
            start: *start,
            end: *end,
            dx,
            dy,
            distance: dx.hypot(dy),
            slope: (dx != 0.0).then_some(dy / dx),
            angle_deg: dy.atan2(dx).to_degrees(),
        })
    }
}

/// The numbers behind a distance measurement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistanceSummary {
    /// First point.
    pub start: [f64; 2],
    /// Second point.
    pub end: [f64; 2],
    /// Horizontal difference.
    pub dx: f64,
    /// Vertical difference.
    pub dy: f64,
    /// Euclidean distance.
    pub distance: f64,
    /// `dy / dx`, or `None` for a vertical line.
    pub slope: Option<f64>,
    /// Direction in degrees, from the positive x axis.
    pub angle_deg: f64,
}

/// One local extremum of a series inside a region.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extremum {
    /// Index in the series' original data.
    pub index: usize,
    /// Point in data coordinates.
    pub point: [f64; 2],
    /// Slope through the neighboring points, when both sides exist.
    pub slope: Option<f64>,
}

/// What one series contributes to an area measurement.
#[derive(Clone, Debug, PartialEq)]
pub struct SeriesMeasurement {
    /// Series name.
    pub series: String,
    /// Lowest point inside the region.
    pub min: Option<Extremum>,
    /// Highest point inside the region.
    pub max: Option<Extremum>,
    /// Trapezoidal integral over the region, in data units.
    pub integral: f64,
    /// Mean y of the points inside the region.
    pub mean: f64,
    /// Slope between the first and last in-range points, by increasing x.
    pub endpoint_slope: Option<f64>,
}

/// The statistics of every visible series inside `bounds`.
pub fn region_measurements(plot: &Plot<'_>, state: &PlotState, bounds: Bounds) -> Vec<SeriesMeasurement> {
    plot.visible(state)
        .filter(|series| !series.is_reference())
        .filter_map(|series| {
            let mut points: Vec<(usize, [f64; 2])> = series
                .data()
                .iter()
                .enumerate()
                .filter(|(_, point)| {
                    point[0].is_finite() && point[1].is_finite() && point[0] >= bounds.min_x && point[0] <= bounds.max_x
                })
                .map(|(index, point)| (index, *point))
                .collect();
            if points.is_empty() {
                return None;
            }
            points.sort_by(|a, b| a.1[0].total_cmp(&b.1[0]));
            let mut min: Option<(usize, [f64; 2])> = None;
            let mut max: Option<(usize, [f64; 2])> = None;
            let mut sum = 0.0;
            for (position, (_, point)) in points.iter().enumerate() {
                sum += point[1];
                if min.is_none_or(|(_, current)| point[1] < current[1]) {
                    min = Some((position, *point));
                }
                if max.is_none_or(|(_, current)| point[1] > current[1]) {
                    max = Some((position, *point));
                }
            }
            let integral = points
                .windows(2)
                .map(|pair| {
                    let (a, b) = (pair[0].1, pair[1].1);
                    (b[0] - a[0]) * (a[1] + b[1]) * 0.5
                })
                .sum::<f64>();
            let slope_at = |position: usize| {
                let point = points[position].1;
                let left = position.checked_sub(1).and_then(|index| points.get(index));
                let right = points.get(position + 1);
                match (left, right) {
                    (Some(left), Some(right)) => {
                        let dx = right.1[0] - left.1[0];
                        (dx != 0.0).then_some((right.1[1] - left.1[1]) / dx)
                    }
                    (Some(left), None) => {
                        let dx = point[0] - left.1[0];
                        (dx != 0.0).then_some((point[1] - left.1[1]) / dx)
                    }
                    (None, Some(right)) => {
                        let dx = right.1[0] - point[0];
                        (dx != 0.0).then_some((right.1[1] - point[1]) / dx)
                    }
                    (None, None) => None,
                }
            };
            let extremum = |entry: Option<(usize, [f64; 2])>| {
                entry.map(|(position, point)| Extremum {
                    index: points[position].0,
                    point,
                    slope: slope_at(position),
                })
            };
            let endpoint_slope = match (points.first(), points.last()) {
                (Some(first), Some(last)) if first.1[0] != last.1[0] => {
                    Some((last.1[1] - first.1[1]) / (last.1[0] - first.1[0]))
                }
                _ => None,
            };
            Some(SeriesMeasurement {
                series: series.name().to_owned(),
                min: extremum(min),
                max: extremum(max),
                integral: integral.abs(),
                mean: sum / points.len() as f64,
                endpoint_slope,
            })
        })
        .collect()
}

/// Draw every measurement and its labels.
pub(super) fn paint(
    scene: &mut Scene,
    measure: &Measure,
    transform: &Transform,
    plot: &Plot<'_>,
    state: &PlotState,
    theme: &Theme,
) {
    for measurement in &state.measurements {
        match measurement {
            PlotMeasurement::Distance { start, end } => {
                let a = transform.to_screen(*start);
                let b = transform.to_screen(*end);
                if !a.is_finite() || !b.is_finite() {
                    continue;
                }
                scene.data.push(Prim::Line {
                    points: vec![a, b],
                    width: 1.5,
                    color: theme.warning,
                    dash: Dash::Dashed(5.0, 4.0),
                });
                for point in [a, b] {
                    scene.data.push(Prim::Circle {
                        center: point,
                        radius: 3.0,
                        fill: theme.warning,
                        stroke: None,
                    });
                }
                if let Some(summary) = measurement.distance_summary() {
                    let label = format!(
                        "d={}  Δx={}  Δy={}  slope={}  {}°",
                        number(summary.distance),
                        number(summary.dx),
                        number(summary.dy),
                        summary.slope.map_or_else(|| "∞".to_owned(), number),
                        number(summary.angle_deg)
                    );
                    if state.measurement.show_labels {
                        label_block(
                            scene,
                            measure,
                            pos2((a.x + b.x) * 0.5, (a.y + b.y) * 0.5) + vec2(8.0, -8.0),
                            &[label],
                            theme.warning,
                            theme,
                        );
                    }
                }
            }
            PlotMeasurement::Area { bounds } => {
                let a = transform.to_screen([bounds.min_x, bounds.min_y]);
                let b = transform.to_screen([bounds.max_x, bounds.max_y]);
                if !a.is_finite() || !b.is_finite() {
                    continue;
                }
                let rect = Rect::from_two_pos(a, b).intersect(transform.frame);
                if rect.width() <= 0.0 || rect.height() <= 0.0 {
                    continue;
                }
                scene.data.push(Prim::Rect {
                    rect,
                    radius: 0.0,
                    fill: theme.link.gamma_multiply(0.08),
                    stroke: Some((1.0, theme.link)),
                });
                let width = bounds.max_x - bounds.min_x;
                let height = bounds.max_y - bounds.min_y;
                let mut lines = vec![format!(
                    "area={}  Δx={}  Δy={}",
                    number(width * height),
                    number(width),
                    number(height)
                )];
                let stats = region_measurements(plot, state, *bounds);
                for (index, series) in stats.iter().enumerate() {
                    if index == 4 {
                        lines.push("…".to_owned());
                        break;
                    }
                    lines.push(format!(
                        "{}: min={} max={} ∫={}",
                        series.series,
                        series
                            .min
                            .map_or_else(|| "—".to_owned(), |value| number(value.point[1])),
                        series
                            .max
                            .map_or_else(|| "—".to_owned(), |value| number(value.point[1])),
                        number(series.integral)
                    ));
                    draw_extrema(scene, measure, transform, series, theme, state.measurement.show_labels);
                }
                if state.measurement.show_labels {
                    label_block(
                        scene,
                        measure,
                        rect.left_top() + vec2(6.0, 6.0),
                        &lines,
                        theme.link,
                        theme,
                    );
                }
            }
            PlotMeasurement::CurveArea { .. } => {
                curve::paint(scene, measure, transform, plot, state, theme, measurement);
            }
        }
    }
}

fn draw_extrema(
    scene: &mut Scene,
    measure: &Measure,
    transform: &Transform,
    series: &SeriesMeasurement,
    theme: &Theme,
    show_labels: bool,
) {
    for (extremum, color, name) in [(series.min, theme.success, "min"), (series.max, theme.danger, "max")] {
        let Some(extremum) = extremum else {
            continue;
        };
        let position = transform.to_screen(extremum.point);
        if !position.is_finite() {
            continue;
        }
        scene.data.push(Prim::Circle {
            center: position,
            radius: 2.5,
            fill: color,
            stroke: None,
        });
        let slope = extremum.slope.map_or_else(|| "—".to_owned(), number);
        let text = format!("{name} {} · slope {slope}", number(extremum.point[1]));
        let (width, height, _) = measure.text(&text, 10.0);
        if show_labels {
            scene
                .over
                .push(measure.label(position + vec2(4.0, -height), Align2::LEFT_BOTTOM, &text, 10.0, color));
        }
        let _ = width;
    }
}

fn label_block(scene: &mut Scene, measure: &Measure, at: Pos2, lines: &[String], color: Color32, theme: &Theme) {
    let (width, height) = lines
        .iter()
        .map(|line| measure.text(line, 10.0))
        .fold((0.0_f32, 0.0_f32), |(width, height), (line_width, line_height, _)| {
            (width.max(line_width), height + line_height + 2.0)
        });
    let frame = scene.frame;
    let size = vec2(width + 10.0, height + 6.0);
    let mut origin = pos2(at.x, at.y);
    if origin.x + size.x > frame.right() - 2.0 {
        origin.x = frame.right() - size.x - 2.0;
    }
    if origin.y + size.y > frame.bottom() - 2.0 {
        origin.y = frame.bottom() - size.y - 2.0;
    }
    origin.x = origin.x.max(frame.left() + 2.0);
    origin.y = origin.y.max(frame.top() + 2.0);
    let rect = Rect::from_min_size(origin, size);
    scene.over.push(Prim::Rect {
        rect,
        radius: 3.0,
        fill: theme.surface.gamma_multiply(0.94),
        stroke: Some((1.0, color.gamma_multiply(0.6))),
    });
    let mut y = rect.top() + 3.0;
    for line in lines {
        scene
            .over
            .push(measure.label(pos2(rect.left() + 5.0, y), Align2::LEFT_TOP, line, 10.0, theme.text));
        let (_, line_height, _) = measure.text(line, 10.0);
        y += line_height + 2.0;
    }
}

/// A compact number for labels: fixed decimals for the common range, scientific
/// notation for very large or very small values, and thousands separators.
fn number(value: f64) -> String {
    if !value.is_finite() {
        return "—".to_owned();
    }
    if value == 0.0 {
        return "0".to_owned();
    }
    let magnitude = value.abs();
    if !(1.0e-3..1.0e6).contains(&magnitude) {
        return format!("{value:.3e}");
    }
    let decimals = if magnitude >= 1000.0 {
        1
    } else if magnitude >= 10.0 {
        2
    } else {
        4
    };
    let mut text = format!("{value:.decimals$}");
    if text.contains('.') {
        text = text.trim_end_matches('0').trim_end_matches('.').to_owned();
    }
    text
}

#[cfg(test)]
mod tests;
