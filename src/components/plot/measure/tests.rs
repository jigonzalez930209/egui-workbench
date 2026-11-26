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

#[test]
fn a_straight_segment_has_no_curve_area() {
    let data = [[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]];
    let plot = Plot::new("curve").series(Series::line("line", &data));
    let state = PlotState::default();
    let measurement = PlotMeasurement::curve_area("line", [0.0, 0.0], [2.0, 2.0], 0, 2);
    let summary = curve_area_summary(&plot, &state, &measurement).expect("summary");
    assert_eq!(summary.area, 0.0);
    assert_eq!(summary.absolute_area, 0.0);
    assert_eq!(summary.max_deviation, 0.0);
    assert!((summary.arc_length - (8.0_f64).sqrt()).abs() < 1.0e-9);
}

#[test]
fn a_parabola_reports_the_area_between_curve_and_chord() {
    let data = [[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]];
    let plot = Plot::new("curve").series(Series::line("line", &data));
    let state = PlotState::default();
    let measurement = PlotMeasurement::curve_area("line", [0.0, 0.0], [2.0, 0.0], 0, 2);
    let summary = curve_area_summary(&plot, &state, &measurement).expect("summary");
    assert_eq!(summary.area, 1.0);
    assert_eq!(summary.absolute_area, 1.0);
    assert_eq!(summary.max_deviation, 1.0);
    assert_eq!(summary.max_deviation_at, [1.0, 1.0]);
    assert_eq!(summary.chord.distance, 2.0);
    assert_eq!(summary.chord.slope, Some(0.0));
}

#[test]
fn a_curve_measurement_needs_its_series() {
    let data = [[0.0, 0.0], [1.0, 1.0]];
    let plot = Plot::new("curve").series(Series::line("line", &data));
    let state = PlotState::default();
    let measurement = PlotMeasurement::curve_area("missing", [0.0, 0.0], [1.0, 1.0], 0, 1);
    assert!(curve_area_summary(&plot, &state, &measurement).is_none());
}
