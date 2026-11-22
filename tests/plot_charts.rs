//! The new chart families and the streaming direction arrow.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_workbench::components::plot::*;
use egui_workbench::{Icons, ThemeId, egui, theme};

fn frame(ctx: &egui::Context, icons: &mut Icons, state: &mut PlotState, build: impl FnOnce() -> Plot<'static>) {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(760.0, 500.0))),
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

#[test]
fn pie_and_donut_render() {
    let ctx = context();
    let mut icons = Icons::new();
    let mut state = PlotState::default();
    let config = PlotConfig {
        legend: LegendPosition::BottomRight,
        pie: PieConfig {
            inner_radius: 0.55,
            ..PieConfig::default()
        },
        ..PlotConfig::default()
    };
    frame(&ctx, &mut icons, &mut state, move || {
        Plot::new("pie")
            .title("Share")
            .config(config)
            .series(Series::pie("Desktop", 62.0))
            .series(Series::pie("Mobile", 28.0))
            .series(Series::pie("Tablet", 10.0))
    });
}

#[test]
fn radar_gauge_candles_heatmap_and_waterfall_render() {
    let ctx = context();
    let mut icons = Icons::new();
    let candles = vec![
        [0.0, 10.0, 12.0, 9.0, 11.0],
        [1.0, 11.0, 13.0, 10.0, 10.5],
        [2.0, 10.5, 14.0, 10.0, 13.5],
        [3.0, 13.5, 15.0, 12.0, 12.5],
    ];
    let cells: Vec<[f64; 3]> = (0..6)
        .flat_map(|x| (0..6).map(move |y| [f64::from(x), f64::from(y), f64::from((x * y) % 11)]))
        .collect();
    let changes = vec![[0.0, 12.0], [1.0, -5.0], [2.0, 9.0], [3.0, -3.0]];

    let mut state = PlotState::default();
    frame(&ctx, &mut icons, &mut state, {
        let candles = candles.clone();
        move || {
            Plot::new("candles")
                .direction_arrow(true)
                .series(Series::candles("price", candles))
        }
    });
    let mut state = PlotState::default();
    frame(&ctx, &mut icons, &mut state, {
        let cells = cells.clone();
        move || Plot::new("heatmap").series(Series::heatmap("heat", cells))
    });
    let mut state = PlotState::default();
    frame(&ctx, &mut icons, &mut state, move || {
        Plot::new("waterfall").series(Series::waterfall("change", changes))
    });
    let mut state = PlotState::default();
    frame(&ctx, &mut icons, &mut state, || {
        Plot::new("radar")
            .radar(RadarConfig::new(["Speed", "Weight", "Cost", "Safety", "Range"]))
            .series(Series::radar("Model A", [8.0, 6.0, 7.0, 9.0, 5.0]))
            .series(Series::radar("Model B", [6.0, 9.0, 5.0, 7.0, 8.0]))
    });
    let mut state = PlotState::default();
    frame(&ctx, &mut icons, &mut state, || {
        Plot::new("gauge")
            .series(Series::gauge("CPU", 63.0, 0.0..=100.0))
            .series(Series::gauge("Memory", 81.0, 0.0..=100.0))
    });
}

#[test]
fn the_direction_arrow_reaches_the_export() {
    let ctx = context();
    let data: Vec<[f64; 2]> = (0..40).map(|i| [f64::from(i), f64::from(i).sin()]).collect();
    let plain = Plot::new("arrow")
        .series(Series::line("signal", &data))
        .direction_arrow(false);
    let arrow = Plot::new("arrow")
        .series(Series::line("signal", &data))
        .direction_arrow(true)
        .direction_points(5);
    let state = PlotState::default();
    let options = ExportOptions::default();
    // Export needs egui's fonts, which exist after the first frame.
    let mut warmup = ctx.run_ui(egui::RawInput::default(), |_| {});
    warmup.textures_delta.clear();
    let plain_svg = plain.to_svg(&ctx, &state, &options);
    let arrow_svg = arrow.to_svg(&ctx, &state, &options);
    assert_eq!(plain_svg.matches("<polygon").count(), 0);
    assert!(arrow_svg.matches("<polygon").count() > plain_svg.matches("<polygon").count());
}

struct HoverBoard {
    icons: Icons,
    state: PlotState,
    animation: LegendAnimation,
    frame: Option<egui::Rect>,
    data: Vec<[f64; 2]>,
}

fn hover_harness() -> egui_kittest::Harness<'static, HoverBoard> {
    egui_kittest::Harness::builder()
        .with_size(egui::vec2(720.0, 460.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(120)
        .with_context_setup(|ctx| theme::apply_id(ctx, ThemeId::GithubLight))
        .build_ui_state(
            |ui, board: &mut HoverBoard| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let config = PlotConfig {
                        legend_hover: LegendHoverConfig {
                            animation: board.animation,
                            ..LegendHoverConfig::default()
                        },
                        ..PlotConfig::default()
                    };
                    let response = Plot::new("hover-animation")
                        .config(config)
                        .series(Series::line("signal", &board.data))
                        .series(Series::line("other", &board.data))
                        .show(ui, &mut board.icons, &mut board.state);
                    board.frame = Some(response.frame);
                });
            },
            HoverBoard {
                icons: Icons::new(),
                state: PlotState::default(),
                animation: LegendAnimation::Blink,
                frame: None,
                data: (0..40).map(|i| [f64::from(i), f64::from(i).sin()]).collect(),
            },
        )
}

#[test]
fn hovering_a_legend_row_selects_and_animates_it() {
    for animation in LegendAnimation::ALL {
        let mut harness = hover_harness();
        harness.state_mut().animation = animation;
        harness.run_steps(3);
        let frame = harness.state().frame.expect("frame");
        harness.hover_at(frame.left_top() + egui::vec2(30.0, 16.0));
        harness.run_steps(4);
        assert_eq!(
            harness.state().state.hovered_series(),
            Some(0),
            "{animation:?} did not hover the first legend row"
        );
    }
}
