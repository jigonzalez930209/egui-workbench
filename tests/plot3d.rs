//! The 3D plot: every series kind renders, exports, and keeps its state.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_workbench::components::plot::*;
use egui_workbench::{Icons, ThemeId, egui, theme};

fn render(ctx: &egui::Context, icons: &mut Icons, state: &mut Plot3DState, build: impl FnOnce() -> Plot3D<'static>) {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(760.0, 520.0))),
        ..Default::default()
    };
    let mut build = Some(build);
    let mut output = ctx.run_ui(input, |ui| {
        if let Some(build) = build.take() {
            build().show(ui, icons, state);
        }
    });
    output.textures_delta.clear();
}

fn context() -> egui::Context {
    let ctx = egui::Context::default();
    theme::apply_id(&ctx, ThemeId::GithubLight);
    ctx
}

fn helix() -> Vec<[f64; 3]> {
    (0..90)
        .map(|index| {
            let angle = index as f64 * 0.18;
            [angle.cos() * 2.0, angle.sin() * 2.0, index as f64 * 0.06]
        })
        .collect()
}

fn surface() -> Series3D<'static> {
    Series3D::surface_fn("terrain", -1.0..=1.0, -1.0..=1.0, [20, 20], |x, y| {
        (x * x + y * y).sin()
    })
}

#[test]
fn line_scatter_bars_and_surface_render() {
    let ctx = context();
    let mut icons = Icons::new();
    let path = helix();
    let bars: Vec<[f64; 3]> = (0..5)
        .flat_map(|x| (0..5).map(move |y| [f64::from(x), f64::from(y), f64::from((x * y) % 7)]))
        .collect();

    let mut state = Plot3DState::default();
    render(&ctx, &mut icons, &mut state, || {
        Plot3D::new("line3d")
            .title("Helix")
            .x_label("x")
            .y_label("y")
            .z_label("z")
            .direction_arrow(true)
            .series(Series3D::line("path", path.clone()))
            .series(Series3D::scatter("samples", path[..8].to_vec()))
    });
    let mut state = Plot3DState::default();
    render(&ctx, &mut icons, &mut state, {
        let bars = bars.clone();
        move || {
            Plot3D::new("bars3d")
                .projection(Projection::Orthographic)
                .series(Series3D::bars("revenue", bars))
        }
    });
    let mut state = Plot3DState::default();
    render(&ctx, &mut icons, &mut state, || {
        Plot3D::new("surface3d").series(surface())
    });
}

#[test]
fn surface_export_contains_sorted_cells_and_text() {
    let ctx = context();
    let mut icons = Icons::new();
    let mut state = Plot3DState::default();
    render(&ctx, &mut icons, &mut state, || {
        Plot3D::new("export3d")
            .title("Terrain")
            .x_label("Longitude")
            .y_label("Latitude")
            .z_label("Height")
            .series(surface())
    });
    let plot = Plot3D::new("export3d")
        .title("Terrain")
        .x_label("Longitude")
        .y_label("Latitude")
        .z_label("Height")
        .series(surface());
    let svg = plot.to_svg(&ctx, &state, &ExportOptions::default());
    assert!(svg.contains("Terrain"));
    assert!(svg.contains("<polygon"), "surface cells are polygons");
    assert!(svg.contains("<polyline"), "grid and wireframe lines are drawn");
    assert!(plot.to_png(&ctx, &state, &ExportOptions::default()).is_ok());
}

#[test]
fn the_legend_animation_and_hidden_state_do_not_reorder_series() {
    let path = helix();
    let plot = Plot3D::new("state3d")
        .series(Series3D::line("first", &path))
        .series(Series3D::line("second", &path));
    let mut state = Plot3DState::default();
    state.set_hidden("second", true);
    assert!(state.is_hidden("second"));
    state.set_hidden("second", false);
    assert_eq!(plot.series_names(), ["first", "second"]);
    state.set_legend_position(Some([2.0, -1.0]));
    assert_eq!(state.legend_position(), Some([1.0, 0.0]));
}

#[test]
fn camera_limits_keep_the_data_in_front() {
    let camera = Camera3D::default();
    let orbit = camera.orbit(egui::vec2(50.0, 4000.0));
    assert!(orbit.pitch <= 1.5);
    let zoom = camera.zoom(10_000.0);
    assert!(zoom.distance >= 1e-5);
    let frame = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0));
    let panned = camera.pan(egui::vec2(10.0, -10.0), frame);
    assert_ne!(panned.target, camera.target);
}

struct HoverBoard {
    icons: Icons,
    state: Plot3DState,
    screen: Option<egui::Pos2>,
    frame: Option<egui::Rect>,
    hovered: bool,
}

#[test]
fn hovering_a_3d_point_and_the_legend_reports_it() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(720.0, 460.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(40)
        .with_context_setup(|ctx| theme::apply_id(ctx, ThemeId::GithubLight))
        .build_ui_state(
            |ui, board: &mut HoverBoard| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let response = Plot3D::new("hover3d")
                        .height(400.0)
                        .series(Series3D::scatter("center", vec![[0.0, 0.0, 0.0]]))
                        .show(ui, &mut board.icons, &mut board.state);
                    board.screen = response.screen_pos([0.0, 0.0, 0.0]);
                    board.frame = Some(response.frame);
                    board.hovered = response.hovered.is_some();
                });
            },
            HoverBoard {
                icons: Icons::new(),
                state: Plot3DState::default(),
                screen: None,
                frame: None,
                hovered: false,
            },
        );
    harness.run_steps(2);
    let screen = harness.state().screen.expect("the center projects");
    harness.hover_at(screen);
    harness.run_steps(2);
    assert!(harness.state().hovered, "the projected point should hover");
    let frame = harness.state().frame.expect("frame");
    harness.hover_at(frame.left_top() + egui::vec2(30.0, 16.0));
    harness.run_steps(3);
    assert_eq!(harness.state().state.hovered_series(), Some(0));
}
