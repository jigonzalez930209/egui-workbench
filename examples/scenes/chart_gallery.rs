//! Every non-cartesian chart family plus candlesticks, heatmaps, and waterfalls, shown
//! together so the documentation can compare them at a glance.

use egui_workbench::components::plot::*;
use egui_workbench::egui;

use super::{Host, Scene};

/// Six chart families in a responsive two-column gallery.
pub struct ChartGallery {
    states: [PlotState; 6],
    candles: Vec<[f64; 5]>,
    cells: Vec<[f64; 3]>,
    waterfall: Vec<[f64; 2]>,
}

impl Default for ChartGallery {
    fn default() -> Self {
        let candles = (0..50)
            .map(|index| {
                let base = 100.0 + (f64::from(index) * 0.21).sin() * 8.0 + f64::from(index) * 0.12;
                let open = base + (f64::from(index) * 1.7).sin() * 2.0;
                let close = base + (f64::from(index) * 2.3).cos() * 2.0;
                [
                    f64::from(index),
                    open,
                    open.max(close) + 1.6,
                    open.min(close) - 1.6,
                    close,
                ]
            })
            .collect();
        let cells = (0..24)
            .flat_map(|x| {
                (0..18).map(move |y| {
                    let (x, y) = (f64::from(x), f64::from(y));
                    [x, y, (x * 0.45).sin() * (y * 0.35).cos() * 3.0 + (x + y) * 0.08]
                })
            })
            .collect();
        Self {
            states: std::array::from_fn(|_| PlotState::default()),
            candles,
            cells,
            waterfall: vec![
                [0.0, 42.0],
                [1.0, -11.0],
                [2.0, 18.0],
                [3.0, 7.0],
                [4.0, -25.0],
                [5.0, 14.0],
            ],
        }
    }
}

impl ChartGallery {
    fn chart(&self, index: usize) -> Plot<'_> {
        let mut config = PlotConfig {
            legend: LegendPosition::BottomRight,
            ..PlotConfig::default()
        };
        match index {
            0 => {
                config.title = Some("Pie / donut".to_owned());
                config.pie.inner_radius = 0.5;
                Plot::new(("chart-gallery", index))
                    .config(config)
                    .series(Series::pie("Desktop", 54.0))
                    .series(Series::pie("Mobile", 31.0))
                    .series(Series::pie("Tablet", 9.0))
                    .series(Series::pie("Other", 6.0))
            }
            1 => {
                config.title = Some("Candlesticks".to_owned());
                config.x.label = Some("Session".to_owned());
                config.y.label = Some("Price".to_owned());
                config.legend = LegendPosition::Hidden;
                Plot::new(("chart-gallery", index))
                    .config(config)
                    .series(Series::candles("price", &self.candles).bar_width(0.62))
            }
            2 => {
                config.title = Some("Heatmap".to_owned());
                config.x.label = Some("Hour".to_owned());
                config.y.label = Some("Weekday".to_owned());
                config.legend = LegendPosition::Hidden;
                Plot::new(("chart-gallery", index))
                    .config(config)
                    .series(Series::heatmap("activity", &self.cells))
            }
            3 => {
                config.title = Some("Radar".to_owned());
                config.radar = RadarConfig::new(["Speed", "Comfort", "Cost", "Safety", "Range", "Style"]);
                Plot::new(("chart-gallery", index))
                    .config(config)
                    .series(Series::radar("Model A", [8.0, 7.0, 6.0, 9.0, 5.0, 7.0]))
                    .series(Series::radar("Model B", [6.0, 9.0, 8.0, 7.0, 8.0, 5.0]))
            }
            4 => {
                config.title = Some("Gauges".to_owned());
                config.legend = LegendPosition::Hidden;
                Plot::new(("chart-gallery", index))
                    .config(config)
                    .series(Series::gauge("CPU", 63.0, 0.0..=100.0))
                    .series(Series::gauge("Memory", 81.0, 0.0..=100.0))
                    .series(Series::gauge("Disk", 47.0, 0.0..=100.0))
            }
            _ => {
                config.title = Some("Waterfall".to_owned());
                config.x.label = Some("Month".to_owned());
                config.y.label = Some("Change".to_owned());
                config.legend = LegendPosition::Hidden;
                Plot::new(("chart-gallery", index))
                    .config(config)
                    .series(Series::waterfall("cash", &self.waterfall))
            }
        }
    }
}

impl Scene for ChartGallery {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let mut states = std::mem::take(&mut self.states);
        let mut draw = |ui: &mut egui::Ui, index: usize| {
            self.chart(index).height(225.0).show(ui, host.icons, &mut states[index]);
        };
        if ui.available_width() >= 640.0 {
            for row in 0..3 {
                ui.columns(2, |columns| {
                    draw(&mut columns[0], row * 2);
                    draw(&mut columns[1], row * 2 + 1);
                });
                if row < 2 {
                    ui.add_space(6.0);
                }
            }
        } else {
            for index in 0..6 {
                draw(ui, index);
                if index < 5 {
                    ui.add_space(6.0);
                }
            }
        }
        self.states = states;
    }
}
