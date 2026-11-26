//! The curve-vs-chord area measurement.

use egui::{Pos2, Rect, pos2, vec2};

use super::{DistanceSummary, MeasurementAreaStyle, PlotMeasurement};
use crate::components::plot::scene::{Dash, Measure, Prim, Scene};
use crate::components::plot::transform::Transform;
use crate::components::plot::{Plot, PlotState};
use crate::theme::Theme;

/// The numbers behind a curve-vs-chord measurement.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveAreaSummary {
    /// Series name.
    pub series: String,
    /// Signed area between the curve and the chord (positive above the chord).
    pub area: f64,
    /// Absolute area between the curve and the chord.
    pub absolute_area: f64,
    /// The chord and its distance/slope/angle.
    pub chord: DistanceSummary,
    /// Length along the curve.
    pub arc_length: f64,
    /// Largest vertical distance between the curve and the chord.
    pub max_deviation: f64,
    /// Point where the curve is farthest from the chord.
    pub max_deviation_at: [f64; 2],
}

/// Compute the curve-vs-chord summary for a [`PlotMeasurement::CurveArea`].
pub fn curve_area_summary(
    plot: &Plot<'_>,
    state: &PlotState,
    measurement: &PlotMeasurement,
) -> Option<CurveAreaSummary> {
    let PlotMeasurement::CurveArea {
        series,
        start,
        end,
        start_index,
        end_index,
    } = measurement
    else {
        return None;
    };
    let series = plot.visible(state).find(|candidate| candidate.name() == series)?;
    let data = series.data();
    if data.len() < 2 {
        return None;
    }
    let max = data.len() - 1;
    let (low, high) = if start_index <= end_index {
        ((*start_index).min(max), (*end_index).min(max))
    } else {
        ((*end_index).min(max), (*start_index).min(max))
    };
    let mut points = data[low..=high]
        .iter()
        .copied()
        .filter(|point| point[0].is_finite() && point[1].is_finite())
        .collect::<Vec<_>>();
    if points.len() < 2 {
        return None;
    }
    points.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let (chord_start, chord_end) = if start[0] <= end[0] { (start, end) } else { (end, start) };
    let chord = PlotMeasurement::distance(*chord_start, *chord_end).distance_summary()?;
    let chord_dx = chord_end[0] - chord_start[0];
    let chord_y = |x: f64| {
        if chord_dx.abs() <= f64::EPSILON {
            chord_start[1]
        } else {
            chord_start[1] + (x - chord_start[0]) * (chord_end[1] - chord_start[1]) / chord_dx
        }
    };
    let mut area = 0.0;
    let mut absolute_area = 0.0;
    let mut arc_length = 0.0;
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let dx = b[0] - a[0];
        let da = a[1] - chord_y(a[0]);
        let db = b[1] - chord_y(b[0]);
        let strip = dx * (da + db) * 0.5;
        area += strip;
        absolute_area += strip.abs();
        arc_length += (dx * dx + (b[1] - a[1]).powi(2)).sqrt();
    }
    let mut max_deviation = 0.0;
    let mut max_deviation_at = points[0];
    for point in &points {
        let deviation = (point[1] - chord_y(point[0])).abs();
        if deviation > max_deviation {
            max_deviation = deviation;
            max_deviation_at = *point;
        }
    }
    Some(CurveAreaSummary {
        series: series.name().to_owned(),
        area,
        absolute_area,
        chord,
        arc_length,
        max_deviation,
        max_deviation_at,
    })
}

pub(super) fn paint(
    scene: &mut Scene,
    measure: &Measure,
    transform: &Transform,
    plot: &Plot<'_>,
    state: &PlotState,
    theme: &Theme,
    measurement: &PlotMeasurement,
) {
    let PlotMeasurement::CurveArea {
        start_index, end_index, ..
    } = measurement
    else {
        return;
    };
    let Some(summary) = curve_area_summary(plot, state, measurement) else {
        return;
    };
    let Some(series) = plot.visible(state).find(|series| series.name() == summary.series) else {
        return;
    };
    let data = series.data();
    let max = data.len().saturating_sub(1);
    let (low, high) = if start_index <= end_index {
        ((*start_index).min(max), (*end_index).min(max))
    } else {
        ((*end_index).min(max), (*start_index).min(max))
    };
    let points = data[low..=high]
        .iter()
        .filter(|point| point[0].is_finite() && point[1].is_finite())
        .map(|point| transform.to_screen(*point))
        .collect::<Vec<_>>();
    if points.len() < 2 {
        return;
    }
    let chord_start = transform.to_screen(summary.chord.start);
    let chord_end = transform.to_screen(summary.chord.end);
    let mut polygon = points.clone();
    polygon.push(chord_end);
    polygon.push(chord_start);
    scene.data.push(Prim::Polygon {
        points: polygon,
        color: theme.link.gamma_multiply(0.16),
    });
    scene.data.push(Prim::Line {
        points: points.clone(),
        width: 2.0,
        color: theme.link,
        dash: Dash::Solid,
    });
    scene.data.push(Prim::Line {
        points: vec![chord_start, chord_end],
        width: 1.2,
        color: theme.warning,
        dash: Dash::Dashed(5.0, 4.0),
    });
    for point in [chord_start, chord_end, transform.to_screen(summary.max_deviation_at)] {
        scene.data.push(Prim::Circle {
            center: point,
            radius: 2.5,
            fill: theme.warning,
            stroke: None,
        });
    }
    if !plot.config.measurement.show_labels {
        return;
    }
    let area = match plot.config.measurement.area_style {
        MeasurementAreaStyle::Signed => summary.area,
        MeasurementAreaStyle::Absolute => summary.absolute_area,
    };
    let label = format!(
        "{}  area={}  |area|={}",
        summary.series,
        number(area),
        number(summary.absolute_area)
    );
    let chord = format!(
        "chord={}  Δx={}  Δy={}  slope={}  {}°",
        number(summary.chord.distance),
        number(summary.chord.dx),
        number(summary.chord.dy),
        summary.chord.slope.map_or_else(|| "∞".to_owned(), number),
        number(summary.chord.angle_deg)
    );
    let detail = format!(
        "arc={}  max dev={} at ({}, {})",
        number(summary.arc_length),
        number(summary.max_deviation),
        number(summary.max_deviation_at[0]),
        number(summary.max_deviation_at[1])
    );
    let at = label_anchor(&points).unwrap_or_else(|| transform.frame.left_top()) + vec2(6.0, 6.0);
    super::label_block(scene, measure, at, &[label, chord, detail], theme.link, theme);
}

fn label_anchor(points: &[Pos2]) -> Option<Pos2> {
    let (mut min, mut max) = (points[0], points[0]);
    for point in points {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    let rect = Rect::from_min_max(min, max);
    Some(pos2(rect.left(), rect.top()))
}

fn number(value: f64) -> String {
    super::number(value)
}
