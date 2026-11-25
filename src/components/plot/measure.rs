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

/// A persistent measurement drawn over the data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
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

    /// The region this measurement covers.
    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Distance { start, end } => Bounds::from_corners(*start, *end),
            Self::Area { bounds } => *bounds,
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
                    label_block(
                        scene,
                        measure,
                        midpoint(a, b) + vec2(8.0, -8.0),
                        &[label],
                        theme.warning,
                        theme,
                    );
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
                    draw_extrema(scene, measure, transform, series, theme);
                }
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
    }
}

fn draw_extrema(
    scene: &mut Scene,
    measure: &Measure,
    transform: &Transform,
    series: &SeriesMeasurement,
    theme: &Theme,
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
        scene
            .over
            .push(measure.label(position + vec2(4.0, -height), Align2::LEFT_BOTTOM, &text, 10.0, color));
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

fn midpoint(a: Pos2, b: Pos2) -> Pos2 {
    pos2((a.x + b.x) * 0.5, (a.y + b.y) * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::plot::{Plot, Series};

    #[test]
    fn a_distance_reports_dx_dy_length_slope_and_angle() {
        let summary = PlotMeasurement::distance([1.0, 2.0], [4.0, 6.0])
            .distance_summary()
            .expect("distance summary");
        assert_eq!(summary.dx, 3.0);
        assert_eq!(summary.dy, 4.0);
        assert_eq!(summary.distance, 5.0);
        assert_eq!(summary.slope, Some(4.0 / 3.0));
        assert!((summary.angle_deg - 53.130_102).abs() < 1e-4);
    }

    #[test]
    fn a_vertical_distance_has_no_slope() {
        let summary = PlotMeasurement::distance([2.0, 0.0], [2.0, 5.0])
            .distance_summary()
            .expect("distance summary");
        assert_eq!(summary.slope, None);
        assert_eq!(summary.distance, 5.0);
    }

    #[test]
    fn a_region_reports_extrema_integral_mean_and_slopes() {
        let data = [[0.0, 0.0], [1.0, 1.0], [2.0, 0.0], [3.0, -1.0], [4.0, 0.0]];
        let plot = Plot::new("measure").series(Series::line("line", &data));
        let state = PlotState::default();
        let stats = region_measurements(&plot, &state, Bounds::from_corners([0.0, -1.0], [4.0, 1.0]));
        assert_eq!(stats.len(), 1);
        let stats = &stats[0];
        assert_eq!(stats.series, "line");
        assert_eq!(stats.min.expect("min").point, [3.0, -1.0]);
        assert_eq!(stats.max.expect("max").point, [1.0, 1.0]);
        assert_eq!(stats.min.expect("min").slope, Some(0.0));
        assert_eq!(stats.integral, 0.0);
        assert_eq!(stats.mean, 0.0);
        assert_eq!(stats.endpoint_slope, Some(0.0));
    }

    #[test]
    fn a_region_without_points_is_omitted() {
        let data = [[0.0, 0.0], [1.0, 1.0]];
        let plot = Plot::new("measure").series(Series::line("line", &data));
        let state = PlotState::default();
        let stats = region_measurements(&plot, &state, Bounds::from_corners([10.0, 10.0], [20.0, 20.0]));
        assert!(stats.is_empty());
    }

    #[test]
    fn a_horizontal_distance_has_a_zero_angle() {
        let summary = PlotMeasurement::distance([1.0, 2.0], [6.0, 2.0])
            .distance_summary()
            .expect("distance");
        assert_eq!(summary.dx, 5.0);
        assert_eq!(summary.dy, 0.0);
        assert_eq!(summary.slope, Some(0.0));
        assert_eq!(summary.angle_deg, 0.0);
    }

    #[test]
    fn a_negative_distance_reports_the_direction() {
        let summary = PlotMeasurement::distance([4.0, 4.0], [1.0, 0.0])
            .distance_summary()
            .expect("distance");
        assert_eq!(summary.dx, -3.0);
        assert_eq!(summary.dy, -4.0);
        assert_eq!(summary.distance, 5.0);
        assert!((summary.angle_deg + 126.869_898).abs() < 1e-4);
    }

    #[test]
    fn an_area_accepts_corners_in_any_order() {
        let measurement = PlotMeasurement::area(Bounds::from_corners([5.0, 6.0], [1.0, 2.0]));
        assert_eq!(measurement.bounds().min_x, 1.0);
        assert_eq!(measurement.bounds().min_y, 2.0);
        assert_eq!(measurement.bounds().max_x, 5.0);
        assert_eq!(measurement.bounds().max_y, 6.0);
        assert!(measurement.distance_summary().is_none());
    }

    #[test]
    fn a_region_ignores_points_outside_the_x_range() {
        let data = [[-1.0, 5.0], [0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 9.0]];
        let plot = Plot::new("measure").series(Series::line("line", &data));
        let state = PlotState::default();
        let stats = region_measurements(&plot, &state, Bounds::from_corners([0.0, 0.0], [2.0, 2.0]));
        let stats = &stats[0];
        assert_eq!(stats.min.expect("min").point, [0.0, 0.0]);
        assert_eq!(stats.max.expect("max").point, [2.0, 2.0]);
        assert_eq!(stats.integral, 2.0);
        assert_eq!(stats.mean, 1.0);
        assert_eq!(stats.endpoint_slope, Some(1.0));
    }

    #[test]
    fn a_region_sorts_unsorted_x_before_integrating() {
        let data = [[2.0, 0.0], [1.0, 1.0], [0.0, 0.0]];
        let plot = Plot::new("measure").series(Series::line("line", &data));
        let state = PlotState::default();
        let stats = region_measurements(&plot, &state, Bounds::from_corners([0.0, 0.0], [2.0, 1.0]));
        assert_eq!(stats[0].integral, 1.0);
        assert_eq!(stats[0].endpoint_slope, Some(0.0));
    }

    #[test]
    fn a_single_point_region_has_no_endpoint_slope() {
        let data = [[1.0, 2.0]];
        let plot = Plot::new("measure").series(Series::line("line", &data));
        let state = PlotState::default();
        let stats = region_measurements(&plot, &state, Bounds::from_corners([0.0, 0.0], [2.0, 3.0]));
        assert_eq!(stats[0].integral, 0.0);
        assert_eq!(stats[0].mean, 2.0);
        assert_eq!(stats[0].endpoint_slope, None);
        assert_eq!(stats[0].min.expect("min").slope, None);
    }

    #[test]
    fn a_region_skips_reference_series() {
        let data = [[0.0, 0.0], [1.0, 1.0]];
        let plot = Plot::new("measure")
            .series(Series::line("line", &data))
            .series(Series::hline("baseline", 0.5));
        let state = PlotState::default();
        let stats = region_measurements(&plot, &state, Bounds::from_corners([0.0, 0.0], [1.0, 1.0]));
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].series, "line");
    }

    #[test]
    fn numbers_use_a_compact_or_scientific_format() {
        assert_eq!(number(0.0), "0");
        assert_eq!(number(1.2345), "1.2345");
        assert_eq!(number(123_456.0), "123456");
        assert_eq!(number(1.0e-6), "1.000e-6");
        assert_eq!(number(f64::NAN), "—");
    }
}
