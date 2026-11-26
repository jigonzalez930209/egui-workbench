//! Snapping helpers shared by the measurement tools.

use egui::Pos2;

use super::axis::Transform;
use super::{Plot, PlotState, paint};

/// Snap to the nearest visible point within `radius` points.
pub(super) fn snap_hit(
    plot: &Plot<'_>,
    state: &PlotState,
    transform: &Transform,
    pointer: Pos2,
    radius: f32,
) -> Option<(String, usize, [f64; 2])> {
    let mut best: Option<(f32, String, usize, [f64; 2])> = None;
    for series in plot.visible(state) {
        if series.is_reference() {
            continue;
        }
        let axis = transform.for_axis(plot.axis_of(series));
        if let Some((index, _, distance)) = paint::nearest(&axis, series, pointer, radius, series.sorted_x(), None)
            && best.as_ref().is_none_or(|(current, _, _, _)| distance < *current)
        {
            best = Some((distance, series.name().to_owned(), index, series.data()[index]));
        }
    }
    best.map(|(_, series, index, point)| (series, index, point))
}

pub(super) fn snap_point(
    plot: &Plot<'_>,
    state: &PlotState,
    transform: &Transform,
    pointer: Pos2,
    radius: f32,
) -> [f64; 2] {
    snap_hit(plot, state, transform, pointer, radius).map_or_else(|| transform.to_data(pointer), |(_, _, point)| point)
}

pub(super) fn nearest_on_series(
    plot: &Plot<'_>,
    state: &PlotState,
    transform: &Transform,
    name: &str,
    pointer: Pos2,
    radius: f32,
) -> Option<(usize, [f64; 2])> {
    let series = plot
        .visible(state)
        .find(|series| series.name() == name && !series.is_reference())?;
    let axis = transform.for_axis(plot.axis_of(series));
    if let Some((index, _, _)) = paint::nearest(&axis, series, pointer, radius, series.sorted_x(), None) {
        return Some((index, series.data()[index]));
    }
    let target = transform.to_data(pointer)[0];
    series
        .data()
        .iter()
        .enumerate()
        .filter(|(_, point)| point[0].is_finite() && point[1].is_finite())
        .min_by(|a, b| (a.1[0] - target).abs().total_cmp(&(b.1[0] - target).abs()))
        .map(|(index, point)| (index, *point))
}
