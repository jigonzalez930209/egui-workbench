//! The plot scenes: zoom, pan, box zoom, selection, fits, log axes, bars, live data,
//! and SVG/PNG export, all on the real `Plot` component.

use egui_workbench::components::plot::*;
use egui_workbench::components::prelude::*;
use egui_workbench::egui;

use super::{Host, Scene};

/// Which plot a scene shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A signal, noisy samples with error bars, a linear fit, and a threshold.
    Signal,
    /// A signal with a cubic fit and the selection tool active.
    Select,
    /// A Bode magnitude on log-log axes.
    Log,
    /// Bars and a cumulative step line.
    Bars,
    /// A streaming signal in a moving window.
    Live,
}

/// One plot with its data and interactive state.
pub struct PlotScene {
    kind: Kind,
    state: PlotState,
    data: Vec<[f64; 2]>,
    samples: Vec<[f64; 2]>,
    errors: Vec<f64>,
    paused: bool,
    next: f64,
}

impl PlotScene {
    /// The scene for `kind`.
    pub fn new(kind: Kind) -> Self {
        let mut state = PlotState::default();
        let data: Vec<[f64; 2]> = match kind {
            Kind::Log => (1..200)
                .map(|i| {
                    let f = 10f64.powf(f64::from(i) / 40.0);
                    [f, 1.0 / (1.0 + (f / 300.0).powi(2)).sqrt()]
                })
                .collect(),
            Kind::Bars => (1..=8).map(|i| [f64::from(i), f64::from(i * i % 7 + 2)]).collect(),
            Kind::Live => Vec::new(),
            Kind::Signal | Kind::Select => (0..400)
                .map(|i| {
                    let x = f64::from(i) * 0.025;
                    [x, (x * 1.7).sin() * 2.0 + x * 0.3]
                })
                .collect(),
        };
        let samples: Vec<[f64; 2]> = (0..40)
            .map(|i| {
                let x = f64::from(i) * 0.25;
                [x, 0.8 * x + 1.0 + (f64::from(i * 7919 % 13) - 6.0) * 0.15]
            })
            .collect();
        if kind == Kind::Select {
            state.tool = PlotTool::Select;
            state.selection = Some(Bounds::from_corners([3.0, 0.0], [5.5, 3.0]));
        }
        let errors = samples.iter().map(|_| 0.3).collect();
        Self {
            kind,
            state,
            data,
            samples,
            errors,
            paused: false,
            next: 0.0,
        }
    }

    fn plot(&self) -> Plot<'_> {
        let plot = Plot::new(("scene-plot", self.kind as u8)).settings_button(false);
        match self.kind {
            Kind::Signal => plot
                .title("Signal and samples")
                .x_label("Time (s)")
                .y_label("Voltage (V)")
                .series(Series::line("signal", &self.data))
                .series(Series::scatter("samples", &self.samples).y_errors(&self.errors))
                .fit(FitKind::Linear, &self.samples)
                .series(Series::hline("threshold", 2.5)),
            Kind::Select => plot
                .title("Drag to select a range")
                .x_label("x")
                .y_label("y")
                .series(Series::line("signal", &self.data).style(LineStyle::Solid))
                .fit(FitKind::Polynomial(3), &self.data),
            Kind::Log => plot
                .title("Bode magnitude")
                .x_label("Frequency (Hz)")
                .y_label("|H|")
                .x_scale(Scale::Log10)
                .y_scale(Scale::Log10)
                .series(Series::area("response", &self.data).marker(Marker::Circle)),
            Kind::Bars => plot
                .title("Bars")
                .x_label("Bin")
                .y_label("Count")
                .legend(LegendPosition::BottomRight)
                .series(Series::bars("counts", &self.data))
                .series(Series::step("cumulative", &self.data).width(2.0)),
            Kind::Live => plot
                .title("Live signal (10 s window)")
                .x_label("Time (s)")
                .y_label("Value")
                .x_window(10.0)
                .y_limits(Some(-3.0), Some(3.0))
                .direction_arrow(true)
                .direction_points(5)
                .series(Series::line("sensor", &self.data)),
        }
    }

    fn stream(&mut self, ctx: &egui::Context) {
        if self.kind != Kind::Live || self.paused {
            return;
        }
        let now = ctx.input(|input| input.time);
        while self.next <= now {
            let t = self.next;
            let noise = ((t * 1000.0).sin() * 43758.545).fract() * 0.3;
            self.data.push([t, (t * 2.1).sin() * 1.6 + (t * 0.37).sin() + noise]);
            self.next += 0.02;
        }
        let cutoff = now - 30.0;
        if self.data.first().is_some_and(|point| point[0] < cutoff) {
            self.data.retain(|point| point[0] >= cutoff);
        }
        ctx.request_repaint();
    }
}

impl Scene for PlotScene {
    fn scroll(&self) -> bool {
        false
    }

    fn tick(&mut self, ctx: &egui::Context) {
        self.stream(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        if self.kind == Kind::Live {
            ui.horizontal(|ui| {
                let label = if self.paused { "Resume" } else { "Pause" };
                if Button::new(label)
                    .variant(ButtonVariant::Outline)
                    .size(Size::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.paused = !self.paused;
                    if !self.paused {
                        self.next = ui.input(|input| input.time);
                    }
                }
                Label::new(&format!(
                    "{} points · pause, then zoom or pan the history",
                    self.data.len()
                ))
                .tone(Tone::Muted)
                .size(Size::Sm)
                .show(ui);
            });
            ui.add_space(4.0);
        }
        let mut state = std::mem::take(&mut self.state);
        let response = self
            .plot()
            .height(ui.available_height().max(200.0))
            .show(ui, host.icons, &mut state);
        self.state = state;
        if let Some(clicked) = &response.clicked {
            host.report(format!(
                "{} [{}]: ({:.3}, {:.3})",
                clicked.series, clicked.index, clicked.point[0], clicked.point[1]
            ));
        }
        if response.selection_changed {
            host.report(match self.state.selection {
                Some(selection) => format!("Selection: x from {:.2} to {:.2}", selection.min_x, selection.max_x),
                None => "Selection cleared".to_owned(),
            });
        }
        if let Some(format) = response.export_requested {
            let options = ExportOptions::default();
            let ctx = ui.ctx().clone();
            let plot = self.plot();
            let saved = match format {
                ExportFormat::Svg => {
                    let svg = plot.to_svg(&ctx, &self.state, &options);
                    let size = svg.len();
                    let place = (host.save)("chart.svg", "image/svg+xml", svg.into_bytes());
                    Ok((size, place))
                }
                ExportFormat::Png => plot.to_png(&ctx, &self.state, &options).map(|bytes| {
                    let size = bytes.len();
                    (size, (host.save)("chart.png", "image/png", bytes))
                }),
                _ => Ok((0, String::new())),
            };
            match saved {
                Ok((size, place)) => host.toasts.add(
                    Toast::new(
                        ToastKind::Success,
                        format!("Exported {format:?} ({} KB)", size.div_ceil(1024)),
                    )
                    .description(place),
                ),
                Err(error) => host
                    .toasts
                    .add(Toast::new(ToastKind::Error, "Export failed").description(error)),
            };
        }
    }
}
