//! Plot tests.

use super::*;

#[test]
fn special_series_choose_the_chart_mode() {
    let pie = Plot::new("pie").series(Series::pie("a", 1.0));
    assert_eq!(pie.chart_mode(), ChartMode::Pie);
    let radar = Plot::new("radar").series(Series::radar("a", [1.0, 2.0, 3.0]));
    assert_eq!(radar.chart_mode(), ChartMode::Radar);
    let gauge = Plot::new("gauge").series(Series::gauge("a", 4.0, 0.0..=10.0));
    assert_eq!(gauge.chart_mode(), ChartMode::Gauge);
    let candles = Plot::new("candles").series(Series::candles("a", vec![[0.0, 1.0, 2.0, 0.5, 1.5]]));
    assert_eq!(candles.chart_mode(), ChartMode::Cartesian);
}

#[test]
fn direction_and_legend_hover_builders_write_the_config() {
    let plot = Plot::new("p")
        .direction_arrow(true)
        .direction_points(9)
        .legend_animation(LegendAnimation::DashMarch);
    assert!(plot.config.direction.enabled);
    assert_eq!(plot.config.direction.points, 9);
    assert_eq!(plot.config.legend_hover.animation, LegendAnimation::DashMarch);
}
