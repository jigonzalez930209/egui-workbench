//! Two y axes, the Alt-wheel split between them, and the draggable legend.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::Harness;
use egui_workbench::components::plot::{Bounds, Plot, PlotHover, PlotState, Series, Smoothing, YAxis};
use egui_workbench::egui::{self, Event, Modifiers, MouseWheelUnit, PointerButton, Pos2, TouchPhase, pos2, vec2};
use egui_workbench::{Icons, ThemeId, theme};

struct Board {
    icons: Icons,
    state: PlotState,
    temperature: Vec<[f64; 2]>,
    rain: Vec<[f64; 2]>,
    view: Option<Bounds>,
    y2_view: Option<[f64; 2]>,
    frame: Option<egui::Rect>,
    hovered: Option<PlotHover>,
    rain_screen: Option<Pos2>,
}

fn harness<'a>() -> Harness<'a, Board> {
    let board = Board {
        icons: Icons::new(),
        state: PlotState::default(),
        temperature: (0..12).map(|i| [f64::from(i), 10.0 + f64::from(i % 6)]).collect(),
        rain: (0..12)
            .map(|i| [f64::from(i), 100.0 + 50.0 * f64::from(i % 3)])
            .collect(),
        view: None,
        y2_view: None,
        frame: None,
        hovered: None,
        rain_screen: None,
    };
    Harness::builder()
        .with_size(vec2(640.0, 420.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .with_context_setup(|ctx| theme::apply_id(ctx, ThemeId::GithubLight))
        .build_ui_state(
            |ui, board: &mut Board| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let response = Plot::new("axes_plot")
                        .y_label("°C")
                        .y2_label("mm")
                        .series(Series::line("temperature", &board.temperature).smooth(Smoothing::Monotone))
                        .series(Series::scatter("rain", &board.rain).y_axis(YAxis::Right))
                        .show(ui, &mut board.icons, &mut board.state);
                    board.view = Some(response.view);
                    board.y2_view = response.y2_view;
                    board.frame = Some(response.frame);
                    if response.hovered.is_some() {
                        board.hovered = response.hovered.clone();
                    }
                    board.rain_screen = Some(response.screen_pos_on(YAxis::Right, [1.0, 150.0]));
                });
            },
            board,
        )
}

fn wheel(harness: &mut Harness<'_, Board>, at: Pos2, modifiers: Modifiers) {
    harness.event(Event::ModifiersChanged(modifiers));
    harness.event(Event::PointerMoved(at));
    harness.run_steps(2);
    harness.event(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: vec2(0.0, 120.0),
        phase: TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers,
    });
    harness.run_steps(30);
    harness.event(Event::ModifiersChanged(Modifiers::NONE));
    harness.run_steps(1);
}

fn spans(harness: &Harness<'_, Board>) -> (f64, f64, f64) {
    let view = harness.state().view.unwrap();
    let [low, high] = harness.state().y2_view.unwrap();
    (view.width(), view.height(), high - low)
}

#[test]
fn each_axis_fits_its_own_series() {
    let mut harness = harness();
    harness.run_steps(3);
    let view = harness.state().view.unwrap();
    let [low, high] = harness.state().y2_view.expect("the right axis is shown");
    assert!(view.min_y > 9.0 && view.max_y < 16.0, "left fits temperature: {view:?}");
    assert!(
        low > 90.0 && low < 100.0 && high > 200.0 && high < 210.0,
        "right fits rain: {low}..{high}"
    );
}

#[test]
fn alt_wheel_zooms_the_axis_on_its_half() {
    let mut harness = harness();
    harness.run_steps(3);
    let frame = harness.state().frame.unwrap();
    let (width, left, right) = spans(&harness);

    wheel(
        &mut harness,
        frame.center() - vec2(frame.width() * 0.25, 0.0),
        Modifiers::ALT,
    );
    let (width_1, left_1, right_1) = spans(&harness);
    assert!((width_1 - width).abs() < 1e-9, "x untouched");
    assert!(left_1 < left * 0.95, "left half zooms the left axis: {left} → {left_1}");
    assert!((right_1 - right).abs() < 1e-9, "right axis untouched");

    wheel(
        &mut harness,
        frame.center() + vec2(frame.width() * 0.25, 0.0),
        Modifiers::ALT,
    );
    let (width_2, left_2, right_2) = spans(&harness);
    assert!((width_2 - width_1).abs() < 1e-9, "x untouched");
    assert!((left_2 - left_1).abs() < 1e-9, "left axis untouched");
    assert!(
        right_2 < right_1 * 0.95,
        "right half zooms the right axis: {right_1} → {right_2}"
    );
}

#[test]
fn the_plain_wheel_zooms_every_axis_and_double_click_resets_both() {
    let mut harness = harness();
    harness.run_steps(3);
    let frame = harness.state().frame.unwrap();
    let (width, left, right) = spans(&harness);
    wheel(&mut harness, frame.center(), Modifiers::NONE);
    let (width_1, left_1, right_1) = spans(&harness);
    assert!(width_1 < width * 0.95 && left_1 < left * 0.95 && right_1 < right * 0.95);

    let shift_before = spans(&harness);
    wheel(&mut harness, frame.center(), Modifiers::SHIFT);
    let shift_after = spans(&harness);
    assert!(shift_after.0 < shift_before.0 * 0.95, "Shift zooms x");
    assert!((shift_after.1 - shift_before.1).abs() < 1e-9 && (shift_after.2 - shift_before.2).abs() < 1e-9);

    harness.state_mut().state.reset_view();
    harness.run_steps(2);
    let (width_2, left_2, right_2) = spans(&harness);
    assert!((width_2 - width).abs() < 1e-9 && (left_2 - left).abs() < 1e-9 && (right_2 - right).abs() < 1e-9);
}

#[test]
fn the_wheel_over_an_axis_zooms_only_that_axis() {
    let mut harness = harness();
    harness.run_steps(3);
    let frame = harness.state().frame.unwrap();
    let cases = [
        (pos2(frame.center().x, frame.bottom() + 8.0), [true, false, false]),
        (pos2(frame.left() - 10.0, frame.center().y), [false, true, false]),
        (pos2(frame.right() + 10.0, frame.center().y), [false, false, true]),
    ];
    for (at, zoomed) in cases {
        let before = spans(&harness);
        wheel(&mut harness, at, Modifiers::NONE);
        let after = spans(&harness);
        let changed = [
            after.0 < before.0 * 0.95,
            after.1 < before.1 * 0.95,
            after.2 < before.2 * 0.95,
        ];
        assert_eq!(changed, zoomed, "wheel at {at:?}: {before:?} → {after:?}");
    }
    // A corner belongs to no axis.
    let before = spans(&harness);
    wheel(
        &mut harness,
        pos2(frame.left() - 10.0, frame.bottom() + 8.0),
        Modifiers::NONE,
    );
    assert_eq!(spans(&harness), before);
}

#[test]
fn hovering_a_right_axis_point_reports_its_axis() {
    let mut harness = harness();
    harness.run_steps(3);
    let at = harness.state().rain_screen.unwrap();
    harness.event(Event::PointerMoved(at));
    harness.run_steps(3);
    let hovered = harness.state().hovered.clone().expect("hovered");
    assert_eq!((hovered.series.as_str(), hovered.axis), ("rain", YAxis::Right));
    assert_eq!(hovered.point, [1.0, 150.0]);
}

#[test]
fn the_legend_can_be_dragged_and_stays_there() {
    let mut harness = harness();
    harness.run_steps(3);
    let frame = harness.state().frame.unwrap();
    let from = frame.left_top() + vec2(30.0, 16.0);
    let to = frame.center();
    harness.event(Event::PointerMoved(from));
    harness.run_steps(2);
    harness.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.run_steps(2);
    for step in 1..=8 {
        harness.event(Event::PointerMoved(from + (to - from) * (step as f32 / 8.0)));
        harness.run_steps(1);
    }
    harness.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.run_steps(3);
    let state = &harness.state().state;
    let [x, y] = state.legend_position().expect("moved");
    assert!(x > 0.2 && y > 0.2, "{x}, {y}");
    assert!(!state.is_hidden("temperature"), "a drag is not a click");
    assert!(state.is_auto(), "dragging the legend does not pan");

    // A click on the moved legend still toggles its row.
    let row = pos2(to.x, to.y);
    common::click_at(&harness, row);
    harness.run_steps(3);
    assert!(harness.state().state.is_hidden("temperature"));
    assert!(harness.state().state.legend_position().is_some());
}
