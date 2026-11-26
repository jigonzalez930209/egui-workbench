//! The measurement tools: distance, rectangle area, and curve-vs-chord area.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use egui_workbench::components::plot::{
    Bounds, MeasurementConfig, MeasurementMode, Plot, PlotHover, PlotMeasurement, PlotState, PlotTool, Series,
    curve_area_summary,
};
use egui_workbench::egui::{self, Event, Modifiers, MouseWheelUnit, PointerButton, Pos2, TouchPhase, vec2};
use egui_workbench::{Icons, ThemeId, theme};

struct Board {
    icons: Icons,
    state: PlotState,
    line: Vec<[f64; 2]>,
    dots: Vec<[f64; 2]>,
    view: Option<Bounds>,
    hovered: Option<PlotHover>,
    selection_changed: bool,
    frame: Option<egui::Rect>,
    dot_screen: Option<Pos2>,
    end_screen: Option<Pos2>,
    line_a_screen: Option<Pos2>,
    line_b_screen: Option<Pos2>,
    measurement: MeasurementConfig,
}

fn board() -> Board {
    Board {
        icons: Icons::new(),
        state: PlotState::default(),
        line: (0..=100).map(|i| [f64::from(i) * 0.1, f64::from(i) * 0.1]).collect(),
        dots: vec![[5.0, 2.0]],
        view: None,
        hovered: None,
        selection_changed: false,
        frame: None,
        dot_screen: None,
        end_screen: None,
        line_a_screen: None,
        line_b_screen: None,
        measurement: MeasurementConfig::default(),
    }
}

fn harness<'a>() -> Harness<'a, Board> {
    Harness::builder()
        .with_size(vec2(640.0, 420.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .with_context_setup(|ctx| theme::apply_id(ctx, ThemeId::GithubLight))
        .build_ui_state(
            |ui, board: &mut Board| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let response = Plot::new("test_plot")
                        .x_label("x")
                        .y_label("y")
                        .series(Series::line("line", &board.line))
                        .series(Series::scatter("dots", &board.dots))
                        .measurement_mode(board.measurement.mode)
                        .settings_button(true)
                        .show(ui, &mut board.icons, &mut board.state);
                    board.view = Some(response.view);
                    if response.hovered.is_some() {
                        board.hovered = response.hovered.clone();
                    }
                    board.selection_changed |= response.selection_changed;
                    board.frame = Some(response.frame);
                    board.dot_screen = Some(response.screen_pos([5.0, 2.0]));
                    board.end_screen = Some(response.screen_pos([10.0, 10.0]));
                    board.line_a_screen = Some(response.screen_pos([2.0, 2.0]));
                    board.line_b_screen = Some(response.screen_pos([8.0, 8.0]));
                });
            },
            board(),
        )
}

fn view(harness: &Harness<'_, Board>) -> Bounds {
    harness.state().view.unwrap()
}

/// Drag with the primary button from `from` to `to` in small steps.
fn drag(harness: &mut Harness<'_, Board>, from: Pos2, to: Pos2) {
    harness.event(Event::ModifiersChanged(Modifiers::NONE));
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
        let pos = from + (to - from) * (step as f32 / 8.0);
        harness.event(Event::PointerMoved(pos));
        harness.run_steps(1);
    }
    harness.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.run_steps(3);
    harness.event(Event::ModifiersChanged(Modifiers::NONE));
    harness.run_steps(1);
}

#[test]
fn the_measure_tools_add_distance_and_area_measurements() {
    let mut harness = harness();
    harness.state_mut().state.tool = PlotTool::MeasureDistance;
    harness.run_steps(3);
    let frame = harness.state().frame.expect("frame");
    drag(
        &mut harness,
        frame.left_top() + vec2(90.0, 90.0),
        frame.left_top() + vec2(240.0, 180.0),
    );
    assert_eq!(harness.state().state.measurements().len(), 1);
    let distance = harness.state().state.measurements()[0]
        .distance_summary()
        .expect("distance");
    assert!(distance.distance > 0.0);
    assert!(distance.dx.abs() > 0.0);

    harness.state_mut().state.tool = PlotTool::MeasureArea;
    drag(
        &mut harness,
        frame.left_top() + vec2(120.0, 100.0),
        frame.left_top() + vec2(300.0, 220.0),
    );
    assert_eq!(harness.state().state.measurements().len(), 2);
    assert!(matches!(
        harness.state().state.measurements()[1],
        PlotMeasurement::Area { .. }
    ));
    harness.state_mut().state.clear_measurements();
    assert!(harness.state().state.measurements().is_empty());
}

#[test]
fn a_distance_drag_snaps_to_the_nearest_data_points() {
    let mut harness = harness();
    harness.state_mut().state.tool = PlotTool::MeasureDistance;
    harness.run_steps(3);
    let dot = harness.state().dot_screen.expect("dot");
    let end = harness.state().end_screen.expect("end");
    drag(&mut harness, dot + vec2(3.0, -2.0), end + vec2(2.0, 2.0));
    let summary = harness.state().state.measurements()[0]
        .distance_summary()
        .expect("distance");
    assert_eq!(summary.start, [5.0, 2.0]);
    assert_eq!(summary.end, [10.0, 10.0]);
    assert_eq!(summary.dx, 5.0);
    assert_eq!(summary.dy, 8.0);
    assert_eq!(summary.slope, Some(1.6));
}

#[test]
fn an_area_drag_uses_data_coordinates_inside_the_view() {
    let mut harness = harness();
    harness.state_mut().state.tool = PlotTool::MeasureArea;
    harness.run_steps(3);
    let frame = harness.state().frame.expect("frame");
    let view = view(&harness);
    drag(
        &mut harness,
        frame.left_top() + vec2(90.0, 90.0),
        frame.left_top() + vec2(300.0, 220.0),
    );
    let measurement = &harness.state().state.measurements()[0];
    let bounds = measurement.bounds();
    assert!(bounds.min_x >= view.min_x && bounds.max_x <= view.max_x);
    assert!(bounds.min_y >= view.min_y && bounds.max_y <= view.max_y);
    assert!(bounds.max_x > bounds.min_x && bounds.max_y > bounds.min_y);
}

#[test]
fn measurements_survive_zoom_and_fit() {
    let mut harness = harness();
    let measurement = PlotMeasurement::distance([1.0, 1.0], [9.0, 8.0]);
    harness.state_mut().state.add_measurement(measurement);
    harness.run_steps(3);
    let before = harness.state().state.measurements()[0].distance_summary();
    harness.event(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: vec2(0.0, 120.0),
        phase: TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers: Modifiers::NONE,
    });
    harness.run_steps(10);
    harness.state_mut().state.reset_view();
    harness.run_steps(2);
    let after = harness.state().state.measurements()[0].distance_summary();
    assert_eq!(before, after);
}

#[test]
fn measurement_list_helpers_pop_remove_and_clear() {
    let mut state = PlotState::default();
    let distance = PlotMeasurement::distance([0.0, 0.0], [3.0, 4.0]);
    let area = PlotMeasurement::area(Bounds::from_corners([0.0, 0.0], [2.0, 2.0]));
    state.add_measurement(distance.clone());
    state.add_measurement(area.clone());
    assert_eq!(state.measurements().len(), 2);
    assert_eq!(state.pop_measurement(), Some(area));
    assert_eq!(state.remove_measurement(0), Some(distance.clone()));
    assert!(state.measurements().is_empty());
    state.add_measurement(distance);
    state.clear_measurements();
    assert!(state.measurements().is_empty());
}

#[test]
fn measurement_state_round_trips_through_serde() {
    let mut state = PlotState::default();
    state.add_measurement(PlotMeasurement::distance([1.0, 2.0], [4.0, 6.0]));
    state.add_measurement(PlotMeasurement::area(Bounds::from_corners([0.0, 1.0], [3.0, 5.0])));
    let json = serde_json::to_string(&state).expect("serialize");
    let restored: PlotState = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.measurements(), state.measurements());
}

#[test]
fn the_toolbar_selects_both_measurement_tools() {
    let mut harness = harness();
    harness.run_steps(3);
    harness.get_by_label("Measure distance and slope (drag)").click();
    harness.run_steps(2);
    assert_eq!(harness.state().state.tool, PlotTool::MeasureDistance);
    harness
        .get_by_label("Measure area, extrema, and integrals (drag)")
        .click();
    harness.run_steps(2);
    assert_eq!(harness.state().state.tool, PlotTool::MeasureArea);
}

#[test]
fn curve_segment_mode_measures_the_area_between_curve_and_chord() {
    let mut harness = harness();
    harness.state_mut().state.set_measurement(MeasurementConfig {
        mode: MeasurementMode::CurveSegment,
        ..MeasurementConfig::default()
    });
    harness.state_mut().state.tool = PlotTool::MeasureArea;
    harness.run_steps(3);
    let a = harness.state().line_a_screen.expect("line a");
    let b = harness.state().line_b_screen.expect("line b");
    drag(&mut harness, a + vec2(3.0, 3.0), b + vec2(-2.0, -2.0));
    let measurements = harness.state().state.measurements();
    assert_eq!(measurements.len(), 1);
    let PlotMeasurement::CurveArea {
        series,
        start,
        end,
        start_index,
        end_index,
    } = &measurements[0]
    else {
        panic!("expected a curve area, got {:?}", measurements[0]);
    };
    assert_eq!(series, "line");
    assert_eq!(*start, [2.0, 2.0]);
    assert_eq!(*end, [8.0, 8.0]);
    let plot = Plot::new("curve-check").series(Series::line("line", &harness.state().line));
    let summary = curve_area_summary(&plot, &harness.state().state, &measurements[0]).expect("summary");
    assert!(summary.area.abs() < 1.0e-9);
    assert!(summary.arc_length > 8.0);
    assert!(*start_index < *end_index);
}

#[test]
fn the_analysis_icon_toggles_filters_and_transforms() {
    let mut harness = harness();
    harness.run_steps(3);
    harness.get_by_label("Filters and transforms (analysis)").click();
    harness.run_steps(2);
    harness.get_by_label("Gaussian").click();
    harness.run_steps(2);
    assert!(
        harness
            .state()
            .state
            .pipeline()
            .is_filter_enabled(egui_workbench::components::plot::FilterKind::Gaussian)
    );

    harness.get_by_label("Filters and transforms (analysis)").click();
    harness.run_steps(2);
    harness.get_by_label("FFT magnitude").click();
    harness.run_steps(2);
    assert_eq!(
        harness.state().state.pipeline().transform,
        Some(egui_workbench::components::plot::TransformKind::Fft)
    );
}

#[test]
fn hovering_the_pin_unrolls_the_tool_ribbon() {
    let mut harness = harness();
    harness.state_mut().state.collapse_toolbar();
    harness.run_steps(2);
    let frame = harness.state().frame.expect("frame");
    let pin = egui::pos2(frame.right() - 19.0, frame.top() + 13.0);
    harness.event(Event::PointerMoved(pin));
    harness.run_steps(20);
    assert!(harness.query_by_label("Pan (drag) · wheel zooms").is_some());
}

#[test]
fn dragging_the_pin_moves_the_tool_ribbon() {
    let mut harness = harness();
    harness.run_steps(3);
    let frame = harness.state().frame.expect("frame");
    let pin = egui::pos2(frame.right() - 19.0, frame.top() + 13.0);
    drag(&mut harness, pin, pin + vec2(-40.0, 50.0));
    let anchor = harness.state().state.toolbar_anchor.expect("moved toolbar");
    assert!(anchor[1] > 10.0, "{anchor:?}");
}

#[test]
fn the_clear_icon_appears_only_with_overlays_and_clears_them() {
    let mut harness = harness();
    harness.run_steps(3);
    assert!(
        harness
            .query_by_label("Clear measurements, analysis, and annotations")
            .is_none()
    );
    harness
        .state_mut()
        .state
        .add_measurement(PlotMeasurement::distance([0.0, 0.0], [1.0, 1.0]));
    harness
        .state_mut()
        .state
        .set_filter(egui_workbench::components::plot::FilterKind::Gaussian, true);
    harness.run_steps(2);
    harness
        .get_by_label("Clear measurements, analysis, and annotations")
        .click();
    harness.run_steps(2);
    assert!(harness.state().state.measurements().is_empty());
    assert!(!harness.state().state.pipeline().is_active());
    assert!(
        harness
            .query_by_label("Clear measurements, analysis, and annotations")
            .is_none()
    );
}

#[test]
fn the_pin_unrolls_the_tool_ribbon_and_keeps_it_open() {
    let mut harness = harness();
    harness.state_mut().state.collapse_toolbar();
    harness.run_steps(20);
    assert!(harness.query_by_label("Pan (drag) · wheel zooms").is_none());
    harness
        .get_by_label("Toolbar (hover to unroll) · click to pin · drag to move")
        .click();
    harness.run_steps(20);
    assert!(harness.state().state.toolbar_pinned);
    assert!(harness.query_by_label("Pan (drag) · wheel zooms").is_some());
}

#[test]
fn the_gear_opens_the_settings_panel_and_changes_the_measurement_mode() {
    let mut harness = harness();
    harness.run_steps(3);
    harness.get_by_label("Chart settings").click();
    harness.run_steps(2);
    harness.get_by_label("Curve vs chord").click();
    harness.run_steps(2);
    assert_eq!(harness.state().state.measurement.mode, MeasurementMode::CurveSegment);
}
