//! A live preview of every legend hover animation on line, area, step, scatter, and bar
//! series, with the streaming direction arrow.

use egui_workbench::components::plot::*;
use egui_workbench::components::prelude::*;
use egui_workbench::{combo, egui};

use super::{Host, Scene};

/// The animation preview: pick an effect and easing, then hover a legend row.
pub struct AnimationScene {
    state: PlotState,
    animation: usize,
    easing: usize,
    enabled: bool,
    direction: bool,
    period: f32,
    wave: Vec<[f64; 2]>,
    steps: Vec<[f64; 2]>,
    points: Vec<[f64; 2]>,
    bars: Vec<[f64; 2]>,
}

impl Default for AnimationScene {
    fn default() -> Self {
        let wave = (0..=220)
            .map(|index| {
                let x = f64::from(index) * 0.05;
                [x, (x * 1.4).sin() * 2.0 + (x * 0.31).cos()]
            })
            .collect();
        let steps = (0..=14)
            .map(|index| [f64::from(index) * 0.8, f64::from((index * 3) % 5) * 0.45 - 0.2])
            .collect();
        let points = (0..=18)
            .map(|index| {
                let x = f64::from(index) * 0.6;
                [x, (x * 0.9).cos() * 1.7 - 0.4]
            })
            .collect();
        let bars = (0..=10)
            .map(|index| [f64::from(index) * 1.1, f64::from((index * 7) % 5) * 0.3 + 0.4])
            .collect();
        Self {
            state: PlotState::default(),
            animation: 0,
            easing: 0,
            enabled: true,
            direction: true,
            period: 1.2,
            wave,
            steps,
            points,
            bars,
        }
    }
}

impl AnimationScene {
    fn config(&self) -> PlotConfig {
        let mut config = PlotConfig {
            legend: LegendPosition::BottomRight,
            ..PlotConfig::default()
        };
        config.legend_hover.enabled = self.enabled;
        config.legend_hover.animation = LegendAnimation::ALL[self.animation];
        config.legend_hover.easing = AnimationEasing::ALL[self.easing];
        config.legend_hover.period = self.period;
        config.direction.enabled = self.direction;
        config.direction.points = 5;
        config
    }
}

impl Scene for AnimationScene {
    fn scroll(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        ui.horizontal_wrapped(|ui| {
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
            Stepper::new(&mut self.period, 0.2..=5.0, 0.1)
                .decimals(1)
                .suffix(" s")
                .show(ui, host.icons);
        });
        ui.horizontal_wrapped(|ui| {
            Switch::new(&mut self.enabled).label("Animate hover").show(ui);
            Switch::new(&mut self.direction).label("Direction arrow").show(ui);
            Label::new("Hover a legend row: lines move their stroke, scatter bounces markers, bars tint.")
                .tone(Tone::Muted)
                .size(Size::Sm)
                .show(ui);
        });
        ui.add_space(6.0);
        let plot = Plot::new("scene-animations")
            .config(self.config())
            .title(format!(
                "Legend hover · {}",
                LegendAnimation::ALL[self.animation].label()
            ))
            .x_label("x")
            .y_label("y")
            .direction_arrow(self.direction)
            .series(Series::line("line (dash + motion)", &self.wave).width(2.0))
            .series(Series::area("area (glow + sweep)", &self.wave).fill_to(-2.0))
            .series(Series::step("step (color)", &self.steps).width(1.8))
            .series(Series::scatter("scatter (markers)", &self.points).radius(4.0))
            .series(Series::bars("bars (tint)", &self.bars).bar_width(0.7))
            .height(ui.available_height().max(260.0));
        let mut state = std::mem::take(&mut self.state);
        plot.show(ui, host.icons, &mut state);
        self.state = state;
    }
}
