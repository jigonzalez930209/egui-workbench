//! PNG references of the plot: lines, scatter with error bars, fits, reference lines,
//! bars, steps, log axes, the legend, the toolbar, the hover popup, and a selection.
//!
//! Refresh with `UPDATE_SNAPSHOTS=1 cargo test --test snapshots_plot`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use egui_workbench::components::plot::*;
use egui_workbench::{Icons, ThemeId, egui, theme};

struct S {
    icons: Icons,
    state: PlotState,
    data: Vec<[f64; 2]>,
    noisy: Vec<[f64; 2]>,
    bars: Vec<[f64; 2]>,
    err: Vec<f64>,
}

fn st() -> S {
    let data: Vec<[f64; 2]> = (0..400)
        .map(|i| {
            let x = i as f64 * 0.025;
            [x, (x * 1.7).sin() * 2.0 + x * 0.3]
        })
        .collect();
    let noisy: Vec<[f64; 2]> = (0..40)
        .map(|i| {
            let x = i as f64 * 0.25;
            [x, 0.8 * x + 1.0 + ((i * 7919 % 13) as f64 - 6.0) * 0.15]
        })
        .collect();
    let bars: Vec<[f64; 2]> = (1..=8).map(|i| [i as f64, (i * i % 7 + 2) as f64]).collect();
    let err = noisy.iter().map(|_| 0.3).collect();
    S {
        icons: Icons::new(),
        state: PlotState::default(),
        data,
        noisy,
        bars,
        err,
    }
}

fn render(name: &str, id: ThemeId, size: [f32; 2], pointer: Option<egui::Pos2>, f: fn(&mut egui::Ui, &mut S)) {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(size[0], size[1]))
        .with_max_steps(8)
        .with_context_setup(move |ctx| theme::apply_id(ctx, id))
        .build_ui_state(
            move |ui, s: &mut S| {
                egui::CentralPanel::default().show(ui, |ui| f(ui, s));
            },
            st(),
        );
    h.run_steps(3);
    if let Some(p) = pointer {
        h.hover_at(p);
        h.run_steps(3);
    }
    h.snapshot(format!("plot-{name}"));
}

fn main_plot(ui: &mut egui::Ui, s: &mut S) {
    Plot::new("p")
        .title("Signal and samples")
        .x_label("Time (s)")
        .y_label("Voltage (V)")
        .series(Series::line("signal", &s.data))
        .series(Series::scatter("samples", &s.noisy).y_errors(&s.err))
        .fit(FitKind::Linear, &s.noisy)
        .series(Series::hline("threshold", 2.5))
        .show(ui, &mut s.icons, &mut s.state);
}
fn bars_plot(ui: &mut egui::Ui, s: &mut S) {
    Plot::new("b")
        .title("Bars")
        .x_label("Bin")
        .y_label("Count")
        .legend(LegendPosition::BottomRight)
        .series(Series::bars("counts", &s.bars))
        .series(Series::step("cumulative", &s.bars).width(2.0))
        .show(ui, &mut s.icons, &mut s.state);
}
fn log_plot(ui: &mut egui::Ui, s: &mut S) {
    let d: Vec<[f64; 2]> = (1..200)
        .map(|i| {
            let f = 10f64.powf(i as f64 / 40.0);
            [f, 1.0 / (1.0 + (f / 300.0).powi(2)).sqrt()]
        })
        .collect();
    Plot::new("l")
        .title("Bode magnitude")
        .x_label("Frequency (Hz)")
        .y_label("|H|")
        .x_scale(Scale::Log10)
        .y_scale(Scale::Log10)
        .series(Series::area("response", d).marker(Marker::Circle))
        .show(ui, &mut s.icons, &mut s.state);
}
fn sel_plot(ui: &mut egui::Ui, s: &mut S) {
    s.state.selection = Some(Bounds::from_corners([3.0, 0.0], [5.5, 3.0]));
    s.state.tool = PlotTool::Select;
    Plot::new("s")
        .x_label("x")
        .y_label("y")
        .series(Series::line("signal", &s.data).style(LineStyle::Solid))
        .fit(FitKind::Polynomial(3), &s.data)
        .show(ui, &mut s.icons, &mut s.state);
}

#[test]
fn plot_main_light_900() {
    render(
        "main-light-900",
        ThemeId::GithubLight,
        [900.0, 520.0],
        Some(egui::pos2(420.0, 260.0)),
        main_plot,
    );
}

#[test]
fn plot_main_dark_520() {
    render("main-dark-520", ThemeId::GithubDark, [520.0, 360.0], None, main_plot);
}

#[test]
fn plot_bars_dark_700() {
    render(
        "bars-dark-700",
        ThemeId::GithubDark,
        [700.0, 420.0],
        Some(egui::pos2(340.0, 300.0)),
        bars_plot,
    );
}

#[test]
fn plot_log_light_800() {
    render("log-light-800", ThemeId::GithubLight, [800.0, 460.0], None, log_plot);
}

#[test]
fn plot_sel_dracula_900() {
    render("sel-dracula-900", ThemeId::Dracula, [900.0, 460.0], None, sel_plot);
}
