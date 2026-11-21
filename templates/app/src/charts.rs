//! The Charts gallery section: interactive plots, fits, live data, selections,
//! per-chart settings, and SVG/PNG export.
//!
//! Demo-only, like `gallery.rs`. Each chart keeps its [`PlotState`] (view, tool,
//! selection) and its [`PlotConfig`] (appearance) in the tab. The toolbar's gear opens
//! `chart_settings`; its download menu exports the chart as it is shown.

use egui_workbench::Icons;
use egui_workbench::components::plot::{
    Bounds, ExportFormat, FitKind, LegendPosition, Marker, Plot, PlotConfig, PlotState, PlotTool, Scale, Series,
    points_in,
};
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, RichText};

use super::chart_settings::{ExportSettings, ExportedFile, SettingsDialog};

/// The four demo charts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chart {
    /// Signal, samples with error bars, and a fit.
    Signal,
    /// The live stream.
    Live,
    /// The histogram.
    Histogram,
    /// The log–log response.
    Bode,
}

impl Chart {
    /// Every chart, in page order.
    pub const ALL: [Self; 4] = [Self::Signal, Self::Live, Self::Histogram, Self::Bode];

    /// A short name for dialogs and file names.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Signal => "Signal",
            Self::Live => "Live stream",
            Self::Histogram => "Histogram",
            Self::Bode => "Low-pass response",
        }
    }

    fn file_stem(self) -> &'static str {
        match self {
            Self::Signal => "signal",
            Self::Live => "live-stream",
            Self::Histogram => "histogram",
            Self::Bode => "low-pass-response",
        }
    }

    /// Position in [`Self::ALL`].
    pub fn index(self) -> usize {
        self as usize
    }

    /// The default appearance of this chart.
    pub fn default_config(self) -> PlotConfig {
        let mut config = PlotConfig::default();
        match self {
            Self::Signal => {
                config.title = Some("Signal, samples, and a least-squares fit".to_owned());
                config.x.label = Some("Time (s)".to_owned());
                config.y.label = Some("Amplitude (V)".to_owned());
            }
            Self::Live => {
                config.title = Some("Live stream (last 10 s)".to_owned());
                config.x.label = Some("t (s)".to_owned());
                config.y.label = Some("Value".to_owned());
                config.legend = LegendPosition::BottomLeft;
            }
            Self::Histogram => {
                config.title = Some("Histogram".to_owned());
                config.x.label = Some("Bin".to_owned());
                config.y.label = Some("Count".to_owned());
            }
            Self::Bode => {
                config.title = Some("Low-pass response (log–log)".to_owned());
                config.x.label = Some("Frequency (Hz)".to_owned());
                config.y.label = Some("Gain".to_owned());
                config.x.scale = Scale::Log10;
                config.y.scale = Scale::Log10;
                config.legend = LegendPosition::BottomLeft;
            }
        }
        config
    }
}

/// The state of the Charts section.
pub struct ChartsDemo {
    /// When set, exports are collected here instead of written to the export folder (a
    /// web build hands them to the browser as downloads).
    pub collected: Option<Vec<ExportedFile>>,
    states: [PlotState; 4],
    /// The appearance of each chart, edited by the settings dialog.
    pub configs: [PlotConfig; 4],
    /// The settings dialog, while open.
    pub settings: Option<SettingsDialog>,
    /// Export size, scale, background, and folder.
    pub export: ExportSettings,
    fit: usize,
    streaming: bool,
    curve: Vec<[f64; 2]>,
    samples: Vec<[f64; 2]>,
    errors: Vec<f64>,
    histogram: Vec<[f64; 2]>,
    response: Vec<[f64; 2]>,
    stream: Vec<[f64; 2]>,
    stream_clock: Option<f64>,
    seed: u64,
    /// The last on-screen size of each chart (the default export size).
    sizes: [egui::Vec2; 4],
    /// The last on-screen data range of each chart.
    views: [Option<Bounds>; 4],
}

impl Default for ChartsDemo {
    fn default() -> Self {
        let mut seed = 0x5eed_u64;
        let mut noise = move || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((seed >> 33) as f64 / f64::from(u32::MAX >> 1)) - 0.5
        };
        let curve = (0..=600)
            .map(|i| {
                let x = f64::from(i) * 0.02;
                [x, (x * 1.3).sin() * 1.6 + 0.35 * x]
            })
            .collect();
        let samples: Vec<[f64; 2]> = (0..48)
            .map(|i| {
                let x = 0.2 + f64::from(i) * 0.25;
                [x, 0.12 * x * x - 0.4 * x + 1.5 + noise() * 1.2]
            })
            .collect();
        let errors = samples.iter().map(|_| 0.35).collect();
        let histogram = (0..16)
            .map(|i| {
                let x = f64::from(i) - 7.5;
                [
                    f64::from(i),
                    (120.0 * (-x * x / 18.0).exp() + noise() * 10.0).max(0.0).round(),
                ]
            })
            .collect();
        let response = (0..=240)
            .map(|i| {
                let frequency = 10f64.powf(f64::from(i) / 40.0);
                let ratio = frequency / 1_000.0;
                [frequency, 1.0 / (1.0 + ratio.powi(4)).sqrt()]
            })
            .collect();
        Self {
            collected: None,
            states: Default::default(),
            configs: Chart::ALL.map(Chart::default_config),
            settings: None,
            export: ExportSettings::default(),
            fit: 3,
            streaming: true,
            curve,
            samples,
            errors,
            histogram,
            response,
            stream: Vec::new(),
            stream_clock: None,
            seed: 7,
            sizes: [egui::vec2(960.0, 540.0); 4],
            views: [None; 4],
        }
    }
}

const FITS: [(usize, &str, Option<FitKind>); 7] = [
    (0, "No fit", None),
    (1, "Linear", Some(FitKind::Linear)),
    (2, "Quadratic", Some(FitKind::Polynomial(2))),
    (3, "Cubic", Some(FitKind::Polynomial(3))),
    (4, "Exponential", Some(FitKind::Exponential)),
    (5, "Logarithmic", Some(FitKind::Logarithmic)),
    (6, "Moving average", Some(FitKind::MovingAverage(5))),
];

impl ChartsDemo {
    /// The chart as a builder, shared by the screen and the export.
    pub fn plot(&self, chart: Chart) -> Plot<'_> {
        let base = Plot::new(("charts", chart.index()))
            .config(self.configs[chart.index()].clone())
            .settings_button(true)
            .export_button(true);
        match chart {
            Chart::Signal => {
                let mut plot = base
                    .series(Series::line("signal", &self.curve))
                    .series(Series::scatter("samples", &self.samples).y_errors(&self.errors))
                    .series(Series::hline("limit", 4.0));
                if let Some(kind) = FITS.get(self.fit).and_then(|(_, _, kind)| *kind) {
                    plot = plot.fit(kind, &self.samples);
                }
                plot
            }
            Chart::Live => base
                .x_window(10.0)
                .series(Series::line("raw", &self.stream).width(1.0))
                .series(Series::line("mean (25)", moving_mean(&self.stream, 25)).width(2.0)),
            Chart::Histogram => base
                .include([0.0, 0.0])
                .series(Series::bars("count", &self.histogram))
                .series(Series::line("trend", &self.histogram).marker(Marker::Circle).width(1.5)),
            Chart::Bode => base
                .series(Series::area("|H(f)|", &self.response))
                .series(Series::vline("cutoff", 1_000.0)),
        }
    }

    /// The view and selection of a chart.
    pub fn state(&self, chart: Chart) -> &PlotState {
        &self.states[chart.index()]
    }

    /// The data range currently on screen for `chart`.
    pub fn current_view(&self, chart: Chart) -> Option<Bounds> {
        self.views[chart.index()]
    }

    /// Reset a chart's view (after its limits changed).
    pub fn reset_view(&mut self, chart: Chart) {
        self.states[chart.index()].reset_view();
    }

    /// Export a chart to the export folder (or [`Self::collected`]) and report the result
    /// as a toast.
    pub fn export(&mut self, ctx: &egui::Context, chart: Chart, format: ExportFormat, toasts: &mut Toasts) {
        let mut options = self.export.options;
        if self.export.screen_size {
            let size = self.sizes[chart.index()];
            (options.width, options.height) = (size.x.round(), size.y.round());
        }
        let (extension, mime) = match format {
            ExportFormat::Svg => ("svg", "image/svg+xml"),
            ExportFormat::Png => ("png", "image/png"),
        };
        let bytes = {
            let (plot, state) = (self.plot(chart), self.state(chart));
            match format {
                ExportFormat::Svg => Ok(plot.to_svg(ctx, state, &options).into_bytes()),
                ExportFormat::Png => plot.to_png(ctx, state, &options),
            }
        };
        let result = if let Some(collected) = &mut self.collected {
            let name = format!("{}.{extension}", chart.file_stem());
            bytes.map(|bytes| collected.push(ExportedFile { name, mime, bytes }))
        } else {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_secs());
            let name = format!("{}-{stamp}.{extension}", chart.file_stem());
            let path = std::path::Path::new(&self.export.folder).join(name);
            let written = bytes.and_then(|bytes| std::fs::write(&path, bytes).map_err(|error| error.to_string()));
            written.map(|()| {
                toasts.add(
                    Toast::new(ToastKind::Success, format!("{} exported", extension.to_uppercase()))
                        .description(path.display().to_string()),
                );
            })
        };
        if let Err(error) = result {
            toasts.add(Toast::new(ToastKind::Error, "Export failed").description(error));
        }
    }
}

/// Draw the section.
pub fn show(ui: &mut egui::Ui, icons: &mut Icons, demo: &mut ChartsDemo, toasts: &mut Toasts) {
    controls(ui, demo);
    ui.add_space(8.0);
    tick_stream(ui, demo);

    let mut requests = Vec::new();
    let mut draw = |ui: &mut egui::Ui, demo: &mut ChartsDemo, chart: Chart, height: f32| {
        // The plot borrows the demo's data, so the state is moved out for this call.
        let mut state = std::mem::take(&mut demo.states[chart.index()]);
        let response = demo.plot(chart).height(height).show(ui, icons, &mut state);
        demo.states[chart.index()] = state;
        demo.sizes[chart.index()] = response.response.rect.size();
        demo.views[chart.index()] = Some(response.view);
        if response.settings_requested {
            requests.push((chart, None));
        }
        if let Some(format) = response.export_requested {
            requests.push((chart, Some(format)));
        }
    };

    draw(ui, demo, Chart::Signal, 340.0);
    selection_summary(ui, demo);
    ui.add_space(12.0);
    let height = 260.0;
    if ui.available_width() >= 900.0 {
        ui.columns(2, |columns| {
            draw(&mut columns[0], demo, Chart::Live, height);
            draw(&mut columns[1], demo, Chart::Histogram, height);
        });
    } else {
        draw(ui, demo, Chart::Live, height);
        ui.add_space(12.0);
        draw(ui, demo, Chart::Histogram, height);
    }
    ui.add_space(12.0);
    draw(ui, demo, Chart::Bode, height);

    for (chart, format) in requests {
        match format {
            Some(format) => demo.export(ui.ctx(), chart, format, toasts),
            None => demo.settings = Some(SettingsDialog::new(chart, &demo.configs[chart.index()])),
        }
    }
    super::chart_settings::show(ui.ctx(), demo, toasts);
}

fn tick_stream(ui: &egui::Ui, demo: &mut ChartsDemo) {
    if !demo.streaming {
        demo.stream_clock = None;
        return;
    }
    let now = ui.input(|input| input.time);
    let clock = demo.stream_clock.get_or_insert(now);
    // 50 samples per second of wall time, whatever the frame rate.
    while *clock < now {
        *clock += 0.02;
        demo.seed = demo.seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let jitter = ((demo.seed >> 40) as f64 / f64::from(1u32 << 24)) - 0.5;
        let t = *clock;
        demo.stream
            .push([t, (t * 2.1).sin() + 0.35 * (t * 7.3).sin() + jitter * 0.25]);
    }
    let keep = demo.stream.len().saturating_sub(1_500);
    demo.stream.drain(..keep);
    ui.ctx().request_repaint();
}

fn controls(ui: &mut egui::Ui, demo: &mut ChartsDemo) {
    ui.horizontal_wrapped(|ui| {
        ui.set_row_height(Size::Md.height());
        Label::new("Fit").tone(Tone::Muted).show(ui);
        let options: Vec<(usize, String)> = FITS
            .iter()
            .map(|(index, label, _)| (*index, (*label).to_owned()))
            .collect();
        Select::new("Fit", &mut demo.fit, &options).width(150.0).show(ui);
        ui.add_space(8.0);
        Switch::new(&mut demo.streaming).label("Live data").show(ui);
        ui.add_space(8.0);
        let signal = &mut demo.states[Chart::Signal.index()];
        for (label, tool) in [
            ("Pan", PlotTool::Pan),
            ("Zoom box", PlotTool::Zoom),
            ("Select", PlotTool::Select),
        ] {
            if Button::new(label)
                .size(Size::Sm)
                .variant(ButtonVariant::Outline)
                .selected(signal.tool == tool)
                .show(ui)
                .clicked()
            {
                signal.tool = tool;
            }
        }
        if Button::new("Chart settings…")
            .size(Size::Sm)
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            demo.settings = Some(SettingsDialog::new(Chart::Signal, &demo.configs[0]));
        }
        if Button::new("Reset views")
            .size(Size::Sm)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            for state in &mut demo.states {
                state.reset_view();
            }
        }
    });
    ui.label(
        RichText::new(
            "Drag to pan · wheel zooms (Shift: x, Alt: y) · right-drag zooms to a box · Shift-drag selects · \
             double-click fits · the legend hides series · the gear edits a chart · the arrow exports it",
        )
        .small()
        .weak(),
    );
}

fn selection_summary(ui: &mut egui::Ui, demo: &mut ChartsDemo) {
    let Some(selection) = demo.states[0].selection else {
        return;
    };
    let inside = points_in(&demo.samples, &selection);
    let mean = if inside.is_empty() {
        f64::NAN
    } else {
        inside.iter().map(|&index| demo.samples[index][1]).sum::<f64>() / inside.len() as f64
    };
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        ui.set_row_height(Size::Sm.height());
        Badge::new(&format!("{} samples selected", inside.len()))
            .tone(Tone::Accent)
            .show(ui);
        let range = format!(
            "x {:.2} – {:.2} · y {:.2} – {:.2}",
            selection.min_x, selection.max_x, selection.min_y, selection.max_y
        );
        ui.label(RichText::new(range).small().weak());
        if mean.is_finite() {
            ui.label(RichText::new(format!("mean y = {mean:.3}")).small());
        }
        if Button::new("Zoom to selection")
            .size(Size::Sm)
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            demo.states[0].set_view(selection);
        }
        if Button::new("Clear")
            .size(Size::Sm)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            demo.states[0].selection = None;
        }
    });
}

fn moving_mean(data: &[[f64; 2]], window: usize) -> Vec<[f64; 2]> {
    let mut sum = 0.0;
    let mut out = Vec::with_capacity(data.len());
    for (index, point) in data.iter().enumerate() {
        sum += point[1];
        if index >= window {
            sum -= data[index - window][1];
        }
        let count = (index + 1).min(window) as f64;
        out.push([point[0], sum / count]);
    }
    out
}
