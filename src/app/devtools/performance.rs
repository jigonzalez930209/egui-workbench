//! Frame time and repaint diagnostics.

use std::collections::VecDeque;

use egui::{Context, RichText, Ui};

/// Frame timing and repaint diagnostics.
#[derive(Clone, Debug, Default)]
pub struct PerformanceState {
    last_time: Option<f64>,
    /// Recent frame times in milliseconds, newest last.
    pub frame_ms: VecDeque<f32>,
    /// Repaint reasons observed in the last frame.
    pub repaint_causes: Vec<String>,
}

impl PerformanceState {
    /// Update from the context clock. Call once per frame before rendering the tab.
    pub fn update(&mut self, ctx: &Context) {
        let now = ctx.input(|input| input.time);
        if let Some(last) = self.last_time {
            let delta = ((now - last) * 1000.0).max(0.0) as f32;
            self.frame_ms.push_back(delta);
            while self.frame_ms.len() > 120 {
                self.frame_ms.pop_front();
            }
        }
        self.last_time = Some(now);
        self.repaint_causes = ctx.repaint_causes().iter().map(ToString::to_string).collect();
    }

    /// The most recent frame time in milliseconds.
    pub fn frame_time(&self) -> Option<f32> {
        self.frame_ms.back().copied()
    }
}

pub fn show_performance(ui: &mut Ui, state: &PerformanceState) {
    ui.horizontal(|ui| {
        ui.label("Frame");
        match state.frame_time() {
            Some(ms) => ui.label(RichText::new(format!("{ms:.2} ms")).strong()),
            None => ui.label(RichText::new("waiting").weak()),
        };
        ui.label(RichText::new(format!("{} samples", state.frame_ms.len())).weak());
    });
    let desired = egui::vec2(ui.available_width(), 80.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, egui::CornerRadius::ZERO, ui.visuals().extreme_bg_color);
    if let Some(max) = state.frame_ms.iter().copied().reduce(f32::max).filter(|max| *max > 0.0) {
        let count = state.frame_ms.len().max(1) as f32;
        let points = state
            .frame_ms
            .iter()
            .enumerate()
            .map(|(index, ms)| {
                let x = rect.left() + rect.width() * index as f32 / count;
                let y = rect.bottom() - rect.height() * (ms / max).clamp(0.0, 1.0);
                egui::pos2(x, y)
            })
            .collect::<Vec<_>>();
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(1.0, ui.visuals().hyperlink_color),
        ));
    }
    ui.add_space(4.0);
    ui.label(RichText::new("Repaint causes").strong());
    if state.repaint_causes.is_empty() {
        ui.label(RichText::new("none").weak());
    }
    for cause in &state.repaint_causes {
        ui.label(RichText::new(cause).small());
    }
}
