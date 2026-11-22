//! The Motion section: legend hover animation and the streaming direction arrow.

use egui_workbench::components::plot::{AnimationEasing, LegendAnimation, PlotConfig};
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, RichText};

use super::sections::{color_row, form, heading, number, row};

/// Legend hover animations and the direction arrow.
pub(super) fn show(ui: &mut egui::Ui, draft: &mut PlotConfig, visuals: &egui::Visuals) {
    heading(ui, "Legend hover");
    form(ui, "chart_motion_legend", |ui| {
        row(ui, "Animate hovered rows", |ui| {
            Switch::new(&mut draft.legend_hover.enabled)
                .label("Play an effect while the pointer rests on a legend row")
                .show(ui);
        });
        row(ui, "Animation", |ui| {
            let options: Vec<(usize, String)> = LegendAnimation::ALL
                .iter()
                .enumerate()
                .map(|(index, animation)| (index, animation.label().to_owned()))
                .collect();
            let mut selected = LegendAnimation::ALL
                .iter()
                .position(|animation| *animation == draft.legend_hover.animation)
                .unwrap_or(0);
            if Select::new("Animation", &mut selected, &options).width(190.0).show(ui) {
                draft.legend_hover.animation = LegendAnimation::ALL[selected];
            }
        });
        row(ui, "Easing", |ui| {
            let options: Vec<(usize, String)> = AnimationEasing::ALL
                .iter()
                .enumerate()
                .map(|(index, easing)| (index, easing.label().to_owned()))
                .collect();
            let mut selected = AnimationEasing::ALL
                .iter()
                .position(|easing| *easing == draft.legend_hover.easing)
                .unwrap_or(0);
            if Select::new("Easing", &mut selected, &options).width(160.0).show(ui) {
                draft.legend_hover.easing = AnimationEasing::ALL[selected];
            }
        });
        row(ui, "Cycle", |ui| {
            number(ui, &mut draft.legend_hover.period, 0.2..=10.0, 0.05, " s");
        });
        color_row(
            ui,
            "Second color",
            &mut draft.legend_hover.highlight,
            visuals.warn_fg_color,
        );
    });
    ui.label(
        RichText::new(
            "Blink pulses between white (dark themes) or black (light themes) and the series color; \
             dash effects change or scroll the stroke; motion effects move the line, its markers, or a pie slice. \
             Reduced motion disables every effect.",
        )
        .small()
        .weak(),
    );

    heading(ui, "Direction arrow");
    form(ui, "chart_motion_direction", |ui| {
        row(ui, "Show direction", |ui| {
            Switch::new(&mut draft.direction.enabled)
                .label("Arrow at the newest point of line series")
                .show(ui);
        });
        row(ui, "Measure last", |ui| {
            let mut points = draft.direction.points as f64;
            ui.add(
                egui::DragValue::new(&mut points)
                    .range(2.0..=64.0)
                    .speed(0.2)
                    .suffix(" points"),
            );
            draft.direction.points = points.round() as usize;
        });
        row(ui, "Length", |ui| {
            number(ui, &mut draft.direction.length, 4.0..=160.0, 0.5, " pt");
        });
        row(ui, "Head", |ui| {
            number(ui, &mut draft.direction.head, 2.0..=40.0, 0.25, " pt");
        });
        row(ui, "Width", |ui| {
            number(ui, &mut draft.direction.width, 0.5..=8.0, 0.05, " pt");
        });
        color_row(ui, "Arrow color", &mut draft.direction.color, visuals.text_color());
    });
}
