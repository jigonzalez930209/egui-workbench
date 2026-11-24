//! Axis, tick, and transform tests.

use super::*;
use egui::{Rect, pos2};

#[test]
fn nice_steps_are_one_two_five() {
    assert_eq!(nice_step(10.0, 10.0), 1.0);
    assert_eq!(nice_step(10.0, 4.0), 5.0);
    assert!((nice_step(0.9, 5.0) - 0.2).abs() < 1e-12);
    assert_eq!(nice_step(1000.0, 5.0), 200.0);
}

#[test]
fn linear_ticks_cover_the_range_with_clean_labels() {
    let ticks = ticks(Scale::Linear, -0.05, 1.05, 500.0, 50.0, TickRules::default());
    let labels: Vec<_> = ticks
        .iter()
        .filter(|tick| tick.major)
        .map(|tick| tick.label.as_str())
        .collect();
    assert_eq!(labels, ["0.0", "0.2", "0.4", "0.6", "0.8", "1.0"]);
}

#[test]
fn log_ticks_are_decades() {
    let ticks = ticks(Scale::Log10, 0.0, 3.0, 300.0, 40.0, TickRules::default());
    let majors: Vec<_> = ticks
        .iter()
        .filter(|tick| tick.major)
        .map(|tick| tick.label.clone())
        .collect();
    assert_eq!(majors, ["1", "10", "100", "1000"]);
    let wide = super::ticks(Scale::Log10, 0.0, 6.0, 120.0, 40.0, TickRules::default());
    let labels: Vec<_> = wide
        .iter()
        .filter(|tick| tick.major)
        .map(|tick| tick.label.as_str())
        .collect();
    assert_eq!(labels, ["1e0", "1e2", "1e4", "1e6"], "one notation per axis");
    assert_eq!(
        wide.iter().filter(|tick| tick.value.fract() == 0.0).count(),
        7,
        "every decade keeps a grid line"
    );
}

#[test]
fn fixed_steps_minor_divisions_and_decimals() {
    let rules = TickRules {
        step: Some(0.25),
        minor: super::super::config::Minor::Divisions(5),
        decimals: Some(3),
        ..TickRules::default()
    };
    let ticks = ticks(Scale::Linear, 0.0, 1.0, 400.0, 50.0, rules);
    let majors: Vec<_> = ticks
        .iter()
        .filter(|tick| tick.major)
        .map(|tick| tick.label.as_str())
        .collect();
    assert_eq!(majors, ["0.000", "0.250", "0.500", "0.750", "1.000"]);
    assert_eq!(
        ticks.iter().filter(|tick| !tick.major).count(),
        16,
        "4 minors in each of 4 intervals"
    );
    let off = super::ticks(
        Scale::Linear,
        0.0,
        1.0,
        400.0,
        50.0,
        TickRules {
            minor: super::super::config::Minor::Off,
            ..TickRules::default()
        },
    );
    assert!(off.iter().all(|tick| tick.major));
}

#[test]
fn transform_round_trips() {
    let transform = Transform {
        frame: Rect::from_min_max(pos2(10.0, 10.0), pos2(410.0, 210.0)),
        view: Bounds::from_corners([0.0, -1.0], [4.0, 1.0]),
        x_scale: Scale::Linear,
        y_scale: Scale::Linear,
        y2: Some(SecondAxis {
            min: 0.0,
            max: 100.0,
            scale: Scale::Linear,
        }),
    };
    let right = transform.for_axis(super::super::series::YAxis::Right);
    assert_eq!(right.to_screen([1.0, 75.0]), pos2(110.0, 60.0));
    let screen = transform.to_screen([1.0, 0.5]);
    assert_eq!(screen, pos2(110.0, 60.0));
    let back = transform.to_data(screen);
    assert!((back[0] - 1.0).abs() < 1e-6 && (back[1] - 0.5).abs() < 1e-6);
}

#[test]
fn symmetric_log_is_odd_and_inverts() {
    let scale = Scale::SymmetricLog;
    for value in [-10.0, -1.0, 0.0, 1.0, 10.0] {
        assert!((scale.inverse(scale.forward(value)) - value).abs() < 1e-9);
    }
    assert!((scale.forward(-10.0) + scale.forward(10.0)).abs() < 1e-12);
}

#[test]
fn category_ticks_use_the_configured_labels() {
    let categories = ["idle".to_owned(), "run".to_owned(), "stop".to_owned()];
    let rules = TickRules {
        categories: Some(&categories),
        ..TickRules::default()
    };
    let ticks = ticks(Scale::Category, 0.0, 2.0, 300.0, 50.0, rules);
    let labels: Vec<_> = ticks.iter().map(|tick| tick.label.as_str()).collect();
    assert_eq!(labels, ["idle", "run", "stop"]);
}

#[test]
fn time_ticks_are_calendar_labels() {
    let ticks = ticks(Scale::Time, 0.0, 3_600.0, 400.0, 50.0, TickRules::default());
    assert!(!ticks.is_empty());
    assert!(ticks.iter().any(|tick| tick.label.contains(':')));
}

#[test]
fn readouts_are_short() {
    assert_eq!(format_readout(5.432_109), "5.432");
    assert_eq!(format_readout(1234.5678), "1235");
    assert_eq!(format_readout(0.0), "0");
    assert_eq!(format_readout(2.5e-7), "2.500e-7");
}
