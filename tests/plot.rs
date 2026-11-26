//! The 2D plot: zoom, pan, box zoom, selection, reset, legend, hover, and speed.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use egui_workbench::components::plot::{Bounds, Plot, PlotHover, PlotState, PlotTool, Series};
use egui_workbench::egui::{self, Event, Modifiers, MouseWheelUnit, PointerButton, Pos2, TouchPhase, pos2, vec2};
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
    exported: Vec<egui_workbench::components::plot::ExportFormat>,
    settings_requested: bool,
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
        exported: Vec::new(),
        settings_requested: false,
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
                        .settings_button(true)
                        .show(ui, &mut board.icons, &mut board.state);
                    board.view = Some(response.view);
                    if response.hovered.is_some() {
                        board.hovered = response.hovered.clone();
                    }
                    board.selection_changed |= response.selection_changed;
                    board.frame = Some(response.frame);
                    board.dot_screen = Some(response.screen_pos([5.0, 2.0]));
                });
            },
            board(),
        )
}

fn view(harness: &Harness<'_, Board>) -> Bounds {
    harness.state().view.unwrap()
}

/// Drag with `button` from `from` to `to` in small steps, holding `modifiers`.
fn drag(harness: &mut Harness<'_, Board>, button: PointerButton, from: Pos2, to: Pos2, modifiers: Modifiers) {
    // Held modifiers live in the input state, not only on the button events.
    harness.event(Event::ModifiersChanged(modifiers));
    harness.event(Event::PointerMoved(from));
    harness.run_steps(2);
    harness.event(Event::PointerButton {
        pos: from,
        button,
        pressed: true,
        modifiers,
    });
    harness.run_steps(2);
    for step in 1..=8 {
        let pos = from + (to - from) * (step as f32 / 8.0);
        harness.event(Event::PointerMoved(pos));
        harness.run_steps(1);
    }
    harness.event(Event::PointerButton {
        pos: to,
        button,
        pressed: false,
        modifiers,
    });
    harness.run_steps(3);
    harness.event(Event::ModifiersChanged(Modifiers::NONE));
    harness.run_steps(1);
}

#[test]
fn the_wheel_zooms_around_the_pointer() {
    let mut harness = harness();
    harness.run_steps(3);
    let before = view(&harness);
    let pointer = pos2(320.0, 200.0);
    harness.event(Event::PointerMoved(pointer));
    harness.run_steps(2);
    harness.event(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: vec2(0.0, 120.0),
        phase: TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers: Modifiers::NONE,
    });
    harness.run_steps(30);
    let after = view(&harness);
    assert!(
        after.width() < before.width() * 0.95,
        "zoomed in: {before:?} → {after:?}"
    );
    assert!(
        after.min_x > before.min_x && after.max_x < before.max_x,
        "around the pointer"
    );
    assert!(!harness.state().state.is_auto());
}

#[test]
fn dragging_pans_and_double_click_fits_again() {
    let mut harness = harness();
    harness.run_steps(3);
    let before = view(&harness);
    drag(
        &mut harness,
        PointerButton::Primary,
        pos2(300.0, 200.0),
        pos2(400.0, 200.0),
        Modifiers::NONE,
    );
    let after = view(&harness);
    assert!(
        after.min_x < before.min_x - 0.5,
        "dragging right shows lower x: {before:?} → {after:?}"
    );
    assert!((after.width() - before.width()).abs() < 1e-6, "panning keeps the zoom");

    let center = pos2(320.0, 200.0);
    harness.event(Event::PointerMoved(center));
    for _ in 0..2 {
        for pressed in [true, false] {
            harness.event(Event::PointerButton {
                pos: center,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        harness.run_steps(2);
    }
    harness.run_steps(2);
    assert!(harness.state().state.is_auto(), "double-click fits the data");
    assert!((view(&harness).min_x - before.min_x).abs() < 1e-6);
}

#[test]
fn right_drag_zooms_to_the_box() {
    let mut harness = harness();
    harness.run_steps(3);
    let before = view(&harness);
    drag(
        &mut harness,
        PointerButton::Secondary,
        pos2(200.0, 120.0),
        pos2(360.0, 260.0),
        Modifiers::NONE,
    );
    let after = view(&harness);
    assert!(after.width() < before.width() * 0.5, "{before:?} → {after:?}");
    assert!(after.height() < before.height() * 0.6);
}

#[test]
fn shift_drag_selects_an_area_in_data_coordinates() {
    let mut harness = harness();
    harness.run_steps(3);
    let before = view(&harness);
    drag(
        &mut harness,
        PointerButton::Primary,
        pos2(200.0, 120.0),
        pos2(360.0, 260.0),
        Modifiers::SHIFT,
    );
    let state = &harness.state();
    assert!(state.selection_changed);
    let selection = state.state.selection.expect("selection");
    assert!(before.min_x < selection.min_x && selection.max_x < before.max_x);
    assert!(selection.min_y < selection.max_y);
    assert_eq!(view(&harness), before, "selecting does not move the view");
}

#[test]
fn the_select_tool_selects_with_a_plain_drag() {
    let mut harness = harness();
    harness.state_mut().state.tool = PlotTool::Select;
    harness.run_steps(3);
    drag(
        &mut harness,
        PointerButton::Primary,
        pos2(200.0, 120.0),
        pos2(360.0, 260.0),
        Modifiers::NONE,
    );
    assert!(harness.state().state.selection.is_some());
}

#[test]
fn hovering_reports_the_nearest_point() {
    let mut harness = harness();
    harness.run_steps(3);
    let dot = harness.state().dot_screen.expect("frame");
    harness.event(Event::PointerMoved(dot + vec2(3.0, -2.0)));
    harness.run_steps(3);
    let hovered = harness.state().hovered.clone().expect("hover");
    assert_eq!(hovered.series, "dots");
    assert_eq!(hovered.point, [5.0, 2.0]);
}

#[test]
fn the_legend_hides_and_shows_series() {
    let mut harness = harness();
    harness.run();
    let frame = harness.state().frame.expect("frame");
    // The first legend row sits 8 pt inside the top-left corner of the frame.
    common::click_at(&harness, frame.left_top() + vec2(30.0, 16.0));
    harness.run_steps(3);
    assert!(harness.state().state.is_hidden("line"));
    common::click_at(&harness, frame.left_top() + vec2(30.0, 16.0));
    harness.run_steps(3);
    assert!(!harness.state().state.is_hidden("line"));
}

#[test]
fn a_million_points_draw_quickly() {
    let data: Vec<[f64; 2]> = (0..1_000_000)
        .map(|i| {
            let x = f64::from(i) * 1e-4;
            [x, (x * 3.0).sin() + (x * 37.0).sin() * 0.2]
        })
        .collect();
    let mut icons = Icons::new();
    let mut state = PlotState::default();
    let ctx = egui::Context::default();
    theme::apply_id(&ctx, ThemeId::GithubDark);
    let input = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0))),
        ..Default::default()
    };
    let frame = |icons: &mut Icons, state: &mut PlotState| {
        let started = std::time::Instant::now();
        let mut output = ctx.run_ui(input(), |ui| {
            Plot::new("million")
                .series(Series::line("signal", &data))
                .show(ui, icons, state);
        });
        output.textures_delta.clear();
        let primitives = ctx.tessellate(output.shapes, 1.0);
        (started.elapsed(), primitives.len())
    };
    frame(&mut icons, &mut state);
    let (elapsed, _) = frame(&mut icons, &mut state);
    // Debug builds are ~10× slower than release; this bound only catches a regression
    // to per-point drawing (which takes seconds).
    assert!(elapsed.as_millis() < 1500, "{elapsed:?}");
}

fn configured<'a>(config: egui_workbench::components::plot::PlotConfig) -> Harness<'a, Board> {
    Harness::builder()
        .with_size(vec2(640.0, 420.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .with_context_setup(|ctx| theme::apply_id(ctx, ThemeId::GithubLight))
        .build_ui_state(
            move |ui, board: &mut Board| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let response = Plot::new("configured")
                        .config(config.clone())
                        .export_button(true)
                        .settings_button(true)
                        .series(Series::line("line", &board.line))
                        .show(ui, &mut board.icons, &mut board.state);
                    board.view = Some(response.view);
                    board.frame = Some(response.frame);
                    if let Some(format) = response.export_requested {
                        board.exported.push(format);
                    }
                    board.settings_requested |= response.settings_requested;
                });
            },
            board(),
        )
}

#[test]
fn fixed_limits_replace_the_automatic_range() {
    let mut config = egui_workbench::components::plot::PlotConfig::default();
    config.x.min = Some(2.0);
    config.x.max = Some(4.0);
    config.y.max = Some(20.0);
    let mut harness = configured(config);
    harness.run_steps(3);
    let view = view(&harness);
    assert!(
        (view.min_x - 2.0).abs() < 1e-9 && (view.max_x - 4.0).abs() < 1e-9,
        "{view:?}"
    );
    assert!((view.max_y - 20.0).abs() < 1e-9);
    assert!(view.min_y < 0.0, "the free side stays automatic: {view:?}");
}

#[test]
fn the_toolbar_reports_settings_and_export_requests() {
    let mut harness = configured(egui_workbench::components::plot::PlotConfig::default());
    harness.run_steps(3);
    harness.get_by_label("Chart settings").click();
    harness.run_steps(2);
    assert!(harness.state().settings_requested);
    harness.get_by_label("Export as SVG or PNG").click();
    harness.run_steps(2);
    harness.get_by_label("Export PNG").click();
    harness.run_steps(2);
    assert_eq!(
        harness.state().exported,
        [egui_workbench::components::plot::ExportFormat::Png]
    );
}
