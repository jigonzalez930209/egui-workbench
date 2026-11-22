//! Chart scenes beyond the single plots in `plots.rs`: every series kind with live
//! modifiers, fits over a selection, millions of points, and the starter's full Charts
//! gallery (settings dialog and export included).

use egui_workbench::components::plot::*;
use egui_workbench::components::prelude::*;
use egui_workbench::{combo, egui};

use super::{Host, Scene, charts};

/// The starter's Charts gallery, unchanged: four charts, a fit selector, live data, the
/// `PlotConfig` settings dialog, and SVG/PNG export.
pub struct Gallery {
    demo: charts::ChartsDemo,
}

impl Default for Gallery {
    fn default() -> Self {
        let mut demo = charts::ChartsDemo::default();
        // Exports go to the host (a browser download on the web) instead of a folder.
        demo.collected = Some(Vec::new());
        Self { demo }
    }
}

impl Scene for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        charts::show(ui, host.icons, &mut self.demo, host.toasts);
        let files: Vec<_> = self.demo.collected.as_mut().map(std::mem::take).unwrap_or_default();
        for file in files {
            let size = file.bytes.len();
            let place = (host.save)(&file.name, file.mime, file.bytes);
            host.toasts.add(
                Toast::new(
                    ToastKind::Success,
                    format!("{} ({} KB)", file.name, size.div_ceil(1024)),
                )
                .description(place),
            );
        }
    }
}

const MARKERS: [(Marker, &str); 7] = [
    (Marker::Circle, "Circle"),
    (Marker::Square, "Square"),
    (Marker::Diamond, "Diamond"),
    (Marker::Triangle, "Triangle"),
    (Marker::Cross, "Cross"),
    (Marker::Plus, "Plus"),
    (Marker::Ring, "Ring"),
];

const STYLES: [(LineStyle, &str); 3] = [
    (LineStyle::Solid, "Solid"),
    (LineStyle::Dashed, "Dashed"),
    (LineStyle::Dotted, "Dotted"),
];

/// Every series constructor, with the modifiers on live controls.
pub struct SeriesScene {
    state: PlotState,
    marker: usize,
    style: usize,
    width: f32,
    wave: Vec<[f64; 2]>,
    points: Vec<[f64; 2]>,
    steps: Vec<[f64; 2]>,
    bars: Vec<[f64; 2]>,
}

impl Default for SeriesScene {
    fn default() -> Self {
        Self {
            state: PlotState::default(),
            marker: 0,
            style: 0,
            width: 1.5,
            wave: (0..=200)
                .map(|i| {
                    let x = f64::from(i) * 0.05;
                    [x, (x * 1.1).sin() + 3.0]
                })
                .collect(),
            points: (0..=20)
                .map(|i| {
                    let x = f64::from(i) * 0.5;
                    [x, (x * 0.7).cos() * 0.8 + 5.5]
                })
                .collect(),
            steps: (0..=10).map(|i| [f64::from(i), f64::from(i % 4) * 0.4 + 0.6]).collect(),
            bars: (0..=10)
                .map(|i| [f64::from(i), f64::from((i * 7) % 5) * 0.25 + 0.3])
                .collect(),
        }
    }
}

impl Scene for SeriesScene {
    fn scroll(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        ui.horizontal_wrapped(|ui| {
            let markers: Vec<(usize, String)> = MARKERS
                .iter()
                .enumerate()
                .map(|(i, (_, label))| (i, format!("Marker::{label}")))
                .collect();
            combo(ui, "Marker", &mut self.marker, &markers);
            let styles: Vec<(usize, String)> = STYLES
                .iter()
                .enumerate()
                .map(|(i, (_, label))| (i, format!("LineStyle::{label}")))
                .collect();
            combo(ui, "Line style", &mut self.style, &styles);
            Stepper::new(&mut self.width, 0.5..=6.0, 0.5)
                .decimals(1)
                .suffix(" pt")
                .show(ui, host.icons);
        });
        ui.add_space(6.0);
        let (marker, style) = (MARKERS[self.marker].0, STYLES[self.style].0);
        let mut state = std::mem::take(&mut self.state);
        let response = Plot::new("scene-series")
            .title("Series kinds")
            .settings_button(false)
            .export_button(false)
            .series(Series::bars("bars", &self.bars).bar_width(0.6))
            .series(Series::step("step", &self.steps).width(self.width).style(style))
            .series(
                Series::area("area", &self.wave)
                    .fill_to(2.0)
                    .width(self.width)
                    .style(style),
            )
            .series(
                Series::scatter("scatter", &self.points)
                    .marker(marker)
                    .radius(self.width + 2.0),
            )
            .series(
                Series::function("function", 0.0..=10.0, 200, |x| (x * 0.6).sin() * 0.7 + 7.5)
                    .width(self.width)
                    .style(style),
            )
            .series(Series::hline("hline", 6.6))
            .series(Series::vline("vline", 8.5))
            .height(ui.available_height().max(200.0))
            .show(ui, host.icons, &mut state);
        self.state = state;
        if let Some(clicked) = response.clicked {
            host.report(format!(
                "{} [{}] = ({:.2}, {:.2})",
                clicked.series, clicked.index, clicked.point[0], clicked.point[1]
            ));
        }
    }
}

const MODELS: [(FitKind, &str); 7] = [
    (FitKind::Linear, "Linear"),
    (FitKind::Polynomial(2), "Polynomial(2)"),
    (FitKind::Polynomial(3), "Polynomial(3)"),
    (FitKind::Exponential, "Exponential"),
    (FitKind::Power, "Power"),
    (FitKind::Logarithmic, "Logarithmic"),
    (FitKind::MovingAverage(7), "MovingAverage(7)"),
];

/// Pick a model, then select points to fit only those.
pub struct FitsScene {
    state: PlotState,
    model: usize,
    data: Vec<[f64; 2]>,
}

impl Default for FitsScene {
    fn default() -> Self {
        let data = (0..60)
            .map(|i| {
                let x = 0.5 + f64::from(i) * 0.16;
                let noise = ((f64::from(i) * 12.9898).sin() * 43758.545).fract() - 0.5;
                [x, 1.6 * (0.22 * x).exp() + noise * 0.9]
            })
            .collect();
        let mut state = PlotState::default();
        state.tool = PlotTool::Select;
        Self { state, model: 3, data }
    }
}

impl Scene for FitsScene {
    fn scroll(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let kind = MODELS[self.model].0;
        let fit = Fit::new(kind, &self.data);
        let subset: Vec<[f64; 2]> = self
            .state
            .selection
            .map(|bounds| {
                points_in(&self.data, &bounds)
                    .into_iter()
                    .map(|index| self.data[index])
                    .collect()
            })
            .unwrap_or_default();
        let selection_fit = (subset.len() >= 3).then(|| Fit::new(kind, &subset)).flatten();
        ui.horizontal_wrapped(|ui| {
            let models: Vec<(usize, String)> = MODELS
                .iter()
                .enumerate()
                .map(|(i, (_, label))| (i, format!("FitKind::{label}")))
                .collect();
            combo(ui, "Model", &mut self.model, &models);
            if let Some(fit) = &fit {
                Badge::new(&fit.equation()).tone(Tone::Accent).show(ui);
            }
        });
        ui.horizontal_wrapped(|ui| {
            let text = match (&selection_fit, subset.len()) {
                (Some(fit), count) => format!("Selection ({count} points): {}", fit.equation()),
                (None, 0) => "Drag over the points to fit a selection.".to_owned(),
                (None, count) => format!("{count} points selected: too few for this model."),
            };
            Label::new(&text).tone(Tone::Muted).size(Size::Sm).show(ui);
        });
        ui.add_space(4.0);
        let mut plot = Plot::new("scene-fits")
            .settings_button(false)
            .x_label("x")
            .y_label("y")
            .series(Series::scatter("samples", &self.data));
        if let Some(fit) = &fit {
            plot = plot.series(fit.series("fit (all points)", 200));
        }
        if let Some(fit) = &selection_fit {
            plot = plot.series(fit.series("fit (selection)", 200).style(LineStyle::Solid).width(2.0));
        }
        let mut state = std::mem::take(&mut self.state);
        let response = plot
            .height(ui.available_height().max(200.0))
            .show(ui, host.icons, &mut state);
        self.state = state;
        if response.selection_changed && self.state.selection.is_none() {
            host.report("Selection cleared");
        }
    }
}

const COUNTS: [usize; 4] = [10_000, 100_000, 1_000_000, 2_000_000];

/// Millions of points, drawn with per-pixel decimation.
pub struct BigScene {
    state: PlotState,
    count: usize,
    data: Vec<[f64; 2]>,
}

impl Default for BigScene {
    fn default() -> Self {
        Self {
            state: PlotState::default(),
            count: 2,
            data: Vec::new(),
        }
    }
}

fn signal(count: usize) -> Vec<[f64; 2]> {
    let mut seed = 0x9e37_79b9_u64;
    (0..count)
        .map(|i| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let noise = ((seed >> 40) as f64 / f64::from(1u32 << 24)) - 0.5;
            let x = i as f64 / count as f64 * 100.0;
            [x, (x * 0.35).sin() * 2.0 + (x * 3.1).sin() * 0.4 + noise * 0.6]
        })
        .collect()
}

impl Scene for BigScene {
    fn scroll(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let before = self.count;
        ui.horizontal_wrapped(|ui| {
            let counts: Vec<(usize, String)> = COUNTS
                .iter()
                .enumerate()
                .map(|(i, count)| (i, format!("{} points", group(*count))))
                .collect();
            combo(ui, "Points", &mut self.count, &counts);
            Label::new(
                "Zoom and pan: each pixel column draws its min and max, so the cost follows the width, not the data.",
            )
            .tone(Tone::Muted)
            .size(Size::Sm)
            .show(ui);
        });
        if self.data.len() != COUNTS[self.count] {
            self.data = signal(COUNTS[self.count]);
            self.state.reset_view();
            if before != self.count {
                host.report(format!("{} points generated", group(self.data.len())));
            }
        }
        ui.add_space(4.0);
        let mut state = std::mem::take(&mut self.state);
        Plot::new("scene-big")
            .settings_button(false)
            .export_button(false)
            .x_label("Time (s)")
            .y_label("Value")
            .series(Series::line("signal", &self.data).width(1.0))
            .height(ui.available_height().max(200.0))
            .show(ui, host.icons, &mut state);
        self.state = state;
    }
}

fn group(count: usize) -> String {
    let digits = count.to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

pub use super::chart_types::NewCharts;
