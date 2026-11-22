//! The non-cartesian chart families, candlesticks, heatmaps, and waterfalls.

use egui_workbench::components::plot::*;
use egui_workbench::components::prelude::*;
use egui_workbench::{combo, egui};

use super::{Host, Scene};

const CHART_KINDS: [(&str, NewChart); 6] = [
    ("Pie / donut", NewChart::Pie),
    ("Candlesticks", NewChart::Candles),
    ("Heatmap", NewChart::Heatmap),
    ("Radar", NewChart::Radar),
    ("Gauge", NewChart::Gauge),
    ("Waterfall", NewChart::Waterfall),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewChart {
    Pie,
    Candles,
    Heatmap,
    Radar,
    Gauge,
    Waterfall,
}

/// The non-cartesian chart families plus candlesticks, a heatmap, and a waterfall, with
/// the legend hover animation and direction arrow on live controls.
pub struct NewCharts {
    state: PlotState,
    kind: usize,
    animation: usize,
    easing: usize,
    direction: bool,
    candles: Vec<[f64; 5]>,
    cells: Vec<[f64; 3]>,
    waterfall: Vec<[f64; 2]>,
}

impl Default for NewCharts {
    fn default() -> Self {
        let candles = (0..56)
            .map(|index| {
                let base = 100.0 + (f64::from(index) * 0.21).sin() * 8.0 + f64::from(index) * 0.12;
                let open = base + (f64::from(index) * 1.7).sin() * 2.0;
                let close = base + (f64::from(index) * 2.3).cos() * 2.0;
                let high = open.max(close) + 1.5 + (f64::from(index) * 0.9).sin().abs() * 2.0;
                let low = open.min(close) - 1.5 - (f64::from(index) * 1.1).cos().abs() * 2.0;
                [f64::from(index), open, high, low, close]
            })
            .collect();
        let cells = (0..24)
            .flat_map(|x| {
                (0..18).map(move |y| {
                    let x = f64::from(x);
                    let y = f64::from(y);
                    [x, y, (x * 0.45).sin() * (y * 0.35).cos() * 3.0 + (x + y) * 0.08]
                })
            })
            .collect();
        let waterfall = vec![
            [0.0, 42.0],
            [1.0, -11.0],
            [2.0, 18.0],
            [3.0, 7.0],
            [4.0, -25.0],
            [5.0, 14.0],
        ];
        Self {
            state: PlotState::default(),
            kind: 0,
            animation: 0,
            easing: 0,
            direction: true,
            candles,
            cells,
            waterfall,
        }
    }
}

impl NewCharts {
    fn config(&self) -> PlotConfig {
        let mut config = PlotConfig {
            legend: LegendPosition::BottomRight,
            ..PlotConfig::default()
        };
        config.legend_hover.animation = LegendAnimation::ALL[self.animation];
        config.legend_hover.easing = AnimationEasing::ALL[self.easing];
        config.direction.enabled = self.direction;
        config.radar = RadarConfig::new(["Speed", "Comfort", "Cost", "Safety", "Range", "Style"]);
        config.pie.inner_radius = 0.52;
        config
    }
}

impl Scene for NewCharts {
    fn scroll(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        ui.horizontal_wrapped(|ui| {
            let kinds: Vec<(usize, String)> = CHART_KINDS
                .iter()
                .enumerate()
                .map(|(index, (label, _))| (index, (*label).to_owned()))
                .collect();
            combo(ui, "Chart type", &mut self.kind, &kinds);
            let animations: Vec<(usize, String)> = LegendAnimation::ALL
                .iter()
                .enumerate()
                .map(|(index, animation)| (index, animation.label().to_owned()))
                .collect();
            combo(ui, "Legend animation", &mut self.animation, &animations);
            let easings: Vec<(usize, String)> = AnimationEasing::ALL
                .iter()
                .enumerate()
                .map(|(index, easing)| (index, easing.label().to_owned()))
                .collect();
            combo(ui, "Easing", &mut self.easing, &easings);
            Switch::new(&mut self.direction).label("Direction arrow").show(ui);
        });
        ui.add_space(6.0);
        let mut plot = Plot::new(("new-chart", self.kind))
            .config(self.config())
            .height(ui.available_height().max(240.0))
            .export_button(true);
        plot = match CHART_KINDS[self.kind].1 {
            NewChart::Pie => plot
                .title("Traffic by platform")
                .series(Series::pie("Desktop", 54.0))
                .series(Series::pie("Mobile", 31.0))
                .series(Series::pie("Tablet", 9.0))
                .series(Series::pie("Other", 6.0)),
            NewChart::Candles => plot
                .title("Daily close")
                .x_label("Session")
                .y_label("Price")
                .series(Series::candles("price", &self.candles).bar_width(0.62)),
            NewChart::Heatmap => plot
                .title("Activity by hour and weekday")
                .x_label("Hour")
                .y_label("Weekday")
                .series(Series::heatmap("activity", &self.cells)),
            NewChart::Radar => plot
                .title("Model comparison")
                .legend(LegendPosition::TopLeft)
                .series(Series::radar("Model A", [8.0, 7.0, 6.0, 9.0, 5.0, 7.0]))
                .series(Series::radar("Model B", [6.0, 9.0, 8.0, 7.0, 8.0, 5.0])),
            NewChart::Gauge => plot
                .title("System load")
                .legend(LegendPosition::Hidden)
                .series(Series::gauge("CPU", 63.0, 0.0..=100.0))
                .series(Series::gauge("Memory", 81.0, 0.0..=100.0))
                .series(Series::gauge("Disk", 47.0, 0.0..=100.0)),
            NewChart::Waterfall => plot
                .title("Cash flow")
                .x_label("Month")
                .y_label("Change")
                .series(Series::waterfall("cash", &self.waterfall)),
        };
        // The legend animation runs while the pointer is over a row; it is on by
        // default, so hovering a slice/polygon/gauge in these charts plays it.
        let mut state = std::mem::take(&mut self.state);
        let response = plot.clone().show(ui, host.icons, &mut state);
        self.state = state;
        if let Some(format) = response.export_requested {
            let options = ExportOptions::default();
            let bytes = match format {
                ExportFormat::Svg => plot.to_svg(ui.ctx(), &self.state, &options).into_bytes(),
                ExportFormat::Png => plot.to_png(ui.ctx(), &self.state, &options).unwrap_or_default(),
            };
            let (name, mime) = match format {
                ExportFormat::Svg => ("chart-types.svg", "image/svg+xml"),
                ExportFormat::Png => ("chart-types.png", "image/png"),
            };
            if !bytes.is_empty() {
                let place = (host.save)(name, mime, bytes);
                host.report(place);
            }
        }
    }
}
