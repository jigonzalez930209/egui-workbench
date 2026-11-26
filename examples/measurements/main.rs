//! Manual QA workbench for the plot measurement tools.
//!
//! Run with `cargo run --example measurements`. Four charts exercise distance and
//! area measurements over a damped line, a cubic scatter, two sensor traces, and a
//! dense stream; the right panel shows the same summaries the application receives.
//!
//! Use the ruler button and drag between points, or the area button and drag a
//! rectangle. The toolbar is in the top-right corner of every chart.

use egui_workbench::components::plot::{
    FilterKind, MeasurementAreaStyle, MeasurementConfig, MeasurementMode, Plot, PlotMeasurement, PlotState, Series,
    TransformKind, curve_area_summary, region_measurements,
};
use egui_workbench::eframe;
use egui_workbench::egui::{self, RichText};
use egui_workbench::{Icons, ThemeId, theme};

fn main() -> eframe::Result {
    eframe::run_native(
        "Workbench Measurements",
        egui_workbench::native_options("Workbench Measurements"),
        Box::new(|cc| {
            theme::apply_id(&cc.egui_ctx, ThemeId::GithubDark);
            Ok(Box::new(MeasurementsApp::new()))
        }),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChartKind {
    Damped,
    Cubic,
    Sensors,
    Dense,
}

struct Chart {
    title: String,
    hint: String,
    kind: ChartKind,
    primary: Vec<[f64; 2]>,
    secondary: Option<Vec<[f64; 2]>>,
    state: PlotState,
    measurement: MeasurementConfig,
}

impl Chart {
    fn new(title: &str, hint: &str, kind: ChartKind, primary: Vec<[f64; 2]>, secondary: Option<Vec<[f64; 2]>>) -> Self {
        Self {
            title: title.to_owned(),
            hint: hint.to_owned(),
            kind,
            primary,
            secondary,
            state: PlotState::default(),
            measurement: MeasurementConfig::default(),
        }
    }
}

struct MeasurementsApp {
    charts: Vec<Chart>,
    icons: Icons,
}

impl MeasurementsApp {
    fn new() -> Self {
        let mut charts = Vec::new();
        charts.push(Chart::new(
            "Damped oscillation",
            "Line with positive and negative extrema; try a distance across the peak and an area over one period.",
            ChartKind::Damped,
            damped(-0.18),
            None,
        ));
        charts.push(Chart::new(
            "Cubic scatter",
            "A single extremum at the left and a minimum at the right; the scatter points should snap.",
            ChartKind::Cubic,
            cubic(),
            None,
        ));
        let (a, b) = sensors();
        charts.push(Chart::new(
            "Two sensor traces",
            "Two series in one region; each series reports its own min, max, integral, mean, and endpoint slope.",
            ChartKind::Sensors,
            a,
            Some(b),
        ));
        charts.push(Chart::new(
            "Dense stream",
            "50,000 points for interaction and label performance; drag the ruler across a short span.",
            ChartKind::Dense,
            dense(),
            None,
        ));
        Self {
            charts,
            icons: Icons::new(),
        }
    }

    fn charts_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Plot measurements");
        ui.label(
            "Ruler: drag between two points (snaps within 14 pt). Area: drag a rectangle. \
             The right panel shows the exact values returned to the application.",
        );
        ui.add_space(6.0);
        let icons = &mut self.icons;
        egui::Grid::new("measurement_charts")
            .num_columns(2)
            .spacing([12.0, 12.0])
            .show(ui, |ui| {
                for (index, chart) in self.charts.iter_mut().enumerate() {
                    let Chart {
                        title,
                        hint,
                        kind,
                        primary,
                        secondary,
                        state,
                        measurement,
                    } = chart;
                    ui.vertical(|ui| {
                        ui.label(RichText::new(title.as_str()).strong());
                        ui.label(RichText::new(hint.as_str()).small().weak());
                        let plot = plot_for(*kind, title, primary, secondary.as_deref(), measurement);
                        plot.show(ui, icons, state);
                    });
                    if index % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("Chart settings");
        let mode = self
            .charts
            .first()
            .map(|chart| chart.measurement.mode)
            .unwrap_or_default();
        ui.label("Area tool");
        for (value, label) in [
            (MeasurementMode::Rectangle, "Rectangle (current)"),
            (MeasurementMode::CurveSegment, "Curve vs chord"),
        ] {
            if ui.radio(mode == value, label).clicked() {
                for chart in &mut self.charts {
                    chart.measurement.mode = value;
                }
            }
        }
        let style = self
            .charts
            .first()
            .map(|chart| chart.measurement.area_style)
            .unwrap_or_default();
        ui.label("Area label");
        for (value, label) in [
            (MeasurementAreaStyle::Absolute, "Absolute area"),
            (MeasurementAreaStyle::Signed, "Signed area"),
        ] {
            if ui.radio(style == value, label).clicked() {
                for chart in &mut self.charts {
                    chart.measurement.area_style = value;
                }
            }
        }
        let mut snap = self.charts.first().map_or(14.0, |chart| chart.measurement.snap_radius);
        if ui
            .add(egui::Slider::new(&mut snap, 4.0..=30.0).text("Snap radius (pt)"))
            .changed()
        {
            for chart in &mut self.charts {
                chart.measurement.snap_radius = snap;
            }
        }
        let mut labels = self.charts.first().is_none_or(|chart| chart.measurement.show_labels);
        if ui.checkbox(&mut labels, "Show measurement labels").changed() {
            for chart in &mut self.charts {
                chart.measurement.show_labels = labels;
            }
        }
        ui.separator();
        ui.heading("Analysis (all charts)");
        for filter in FilterKind::ALL {
            let enabled = self
                .charts
                .iter()
                .all(|chart| chart.state.pipeline().is_filter_enabled(filter));
            let mut enabled = enabled;
            if ui.checkbox(&mut enabled, filter.label()).changed() {
                for chart in &mut self.charts {
                    chart.state.set_filter(filter, enabled);
                }
            }
        }
        let transform = self.charts.first().and_then(|chart| chart.state.pipeline().transform);
        for kind in TransformKind::ALL {
            if ui.radio(transform == Some(kind), kind.label()).clicked() {
                for chart in &mut self.charts {
                    chart
                        .state
                        .set_transform(if transform == Some(kind) { None } else { Some(kind) });
                }
            }
        }
        if ui.button("Clear analysis").clicked() {
            for chart in &mut self.charts {
                chart.state.clear_analysis();
            }
        }
        egui::CollapsingHeader::new("Filter parameters")
            .default_open(false)
            .show(ui, |ui| {
                let mut settings = self
                    .charts
                    .first()
                    .map(|chart| chart.state.pipeline().filter_settings)
                    .unwrap_or_default();
                let mut changed = false;
                changed |= ui
                    .add(
                        egui::Slider::new(&mut settings.savgol_window, 5..=31)
                            .step_by(2.0)
                            .text("Savitzky–Golay window"),
                    )
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut settings.savgol_order, 1..=5).text("Savitzky–Golay order"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut settings.moving_window, 3..=31)
                            .step_by(2.0)
                            .text("Moving-average window"),
                    )
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut settings.gaussian_sigma, 0.5..=10.0).text("Gaussian sigma"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut settings.median_window, 3..=31)
                            .step_by(2.0)
                            .text("Median window"),
                    )
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut settings.lowpass_alpha, 0.01..=1.0).text("Low-pass alpha"))
                    .changed();
                if changed {
                    for chart in &mut self.charts {
                        chart.state.pipeline_mut().filter_settings = settings;
                    }
                }
            });
        ui.label(
            RichText::new("The sliders icon on each chart opens the same filters and transforms.")
                .small()
                .weak(),
        );
        ui.separator();
        ui.heading("Measurements");
        ui.label("Values are in data coordinates and survive zoom, pan, and fit.");
        ui.horizontal(|ui| {
            if ui.button("Clear all").clicked() {
                for chart in &mut self.charts {
                    chart.state.clear_measurements();
                }
            }
            if ui.button("Reset all views").clicked() {
                for chart in &mut self.charts {
                    chart.state.reset_view();
                }
            }
        });
        ui.separator();
        for chart in &mut self.charts {
            if chart.state.measurements().is_empty() {
                continue;
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new(&chart.title).strong());
                if ui.small_button("Clear").clicked() {
                    chart.state.clear_measurements();
                }
            });
            let measurements = chart.state.measurements().to_vec();
            let plot = plot_for(
                chart.kind,
                &chart.title,
                &chart.primary,
                chart.secondary.as_deref(),
                &chart.measurement,
            );
            for (index, measurement) in measurements.iter().enumerate() {
                match measurement {
                    PlotMeasurement::Distance { .. } => {
                        if let Some(summary) = measurement.distance_summary() {
                            ui.monospace(format!(
                                "#{} distance  Δx={:.4}  Δy={:.4}  d={:.4}  slope={}  {:.2}°",
                                index + 1,
                                summary.dx,
                                summary.dy,
                                summary.distance,
                                summary
                                    .slope
                                    .map_or_else(|| "∞".to_owned(), |slope| format!("{slope:.4}")),
                                summary.angle_deg
                            ));
                        }
                    }
                    PlotMeasurement::Area { bounds } => {
                        ui.monospace(format!(
                            "#{} area  x=[{:.3}, {:.3}]  y=[{:.3}, {:.3}]  A={:.4}",
                            index + 1,
                            bounds.min_x,
                            bounds.max_x,
                            bounds.min_y,
                            bounds.max_y,
                            (bounds.max_x - bounds.min_x) * (bounds.max_y - bounds.min_y)
                        ));
                        for series in region_measurements(&plot, &chart.state, *bounds) {
                            ui.monospace(format!(
                                "   {}  min={}  max={}  ∫={:.4}  mean={:.4}  slope={}",
                                series.series,
                                series
                                    .min
                                    .map_or_else(|| "—".to_owned(), |value| format!("{:.4}", value.point[1])),
                                series
                                    .max
                                    .map_or_else(|| "—".to_owned(), |value| format!("{:.4}", value.point[1])),
                                series.integral,
                                series.mean,
                                series
                                    .endpoint_slope
                                    .map_or_else(|| "—".to_owned(), |slope| format!("{slope:.4}"))
                            ));
                            if let Some(min) = series.min {
                                ui.label(format!(
                                    "      min slope: {}",
                                    min.slope.map_or_else(|| "—".to_owned(), |slope| format!("{slope:.4}"))
                                ));
                            }
                            if let Some(max) = series.max {
                                ui.label(format!(
                                    "      max slope: {}",
                                    max.slope.map_or_else(|| "—".to_owned(), |slope| format!("{slope:.4}"))
                                ));
                            }
                        }
                    }
                    PlotMeasurement::CurveArea { .. } => {
                        if let Some(summary) = curve_area_summary(&plot, &chart.state, measurement) {
                            ui.monospace(format!(
                                "#{} curve area  signed={:.4}  absolute={:.4}  chord={:.4}",
                                index + 1,
                                summary.area,
                                summary.absolute_area,
                                summary.chord.distance
                            ));
                            ui.monospace(format!(
                                "   arc={:.4}  max dev={:.4} at ({:.3}, {:.3})  slope={}",
                                summary.arc_length,
                                summary.max_deviation,
                                summary.max_deviation_at[0],
                                summary.max_deviation_at[1],
                                summary
                                    .chord
                                    .slope
                                    .map_or_else(|| "∞".to_owned(), |slope| format!("{slope:.4}"))
                            ));
                        }
                    }
                    _ => {}
                }
                ui.add_space(4.0);
            }
            ui.separator();
        }
    }
}

impl eframe::App for MeasurementsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::right("measurement_sidebar")
            .resizable(true)
            .default_size(420.0)
            .show(ui, |ui| self.sidebar(ui));
        egui::CentralPanel::default().show(ui, |ui| self.charts_ui(ui));
    }
}

fn plot_for<'a>(
    kind: ChartKind,
    title: &str,
    primary: &'a [[f64; 2]],
    secondary: Option<&'a [[f64; 2]]>,
    measurement: &MeasurementConfig,
) -> Plot<'a> {
    let plot = Plot::new(("measurement", title))
        .title(title.to_owned())
        .height(250.0)
        .x_label("x")
        .y_label("y")
        .measurement_mode(measurement.mode)
        .measurement_area_style(measurement.area_style)
        .measurement_snap_radius(measurement.snap_radius)
        .measurement_labels(measurement.show_labels);
    match kind {
        ChartKind::Damped => plot.series(Series::line("damped", primary)),
        ChartKind::Cubic => plot.series(Series::scatter("cubic", primary)),
        ChartKind::Sensors => {
            let plot = plot.series(Series::line("sensor A", primary));
            match secondary {
                Some(second) => plot.series(Series::line("sensor B", second)),
                None => plot,
            }
        }
        ChartKind::Dense => plot.series(Series::line("dense", primary)),
    }
}

fn damped(decay: f64) -> Vec<[f64; 2]> {
    (0..=400)
        .map(|index| {
            let x = f64::from(index) * 0.05;
            [x, (-decay * x).exp() * (2.0 * x).sin()]
        })
        .collect()
}

fn cubic() -> Vec<[f64; 2]> {
    (0..=200)
        .map(|index| {
            let x = -3.0 + f64::from(index) * 0.03;
            [x, x.powi(3) - 3.0 * x]
        })
        .collect()
}

fn sensors() -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
    let mut a = Vec::new();
    let mut b = Vec::new();
    for index in 0..=600 {
        let x = f64::from(index) * 0.02;
        a.push([x, 2.0 + (3.0 * x).sin() + 0.15 * (17.0 * x).sin()]);
        b.push([x, 1.2 + (2.2 * x + 0.6).cos() + 0.08 * (11.0 * x).cos()]);
    }
    (a, b)
}

fn dense() -> Vec<[f64; 2]> {
    (0..50_000)
        .map(|index| {
            let x = f64::from(index) * 0.0002;
            [x, (40.0 * x).sin() + 0.2 * (900.0 * x).sin()]
        })
        .collect()
}
