//! A grid of plots sharing one state and one view.

use egui::Ui;

use super::{Plot, PlotResponse, PlotState};
use crate::Icons;

/// A rows × columns grid of plots. Every plot shares the same [`PlotState`], so pan,
/// zoom, hidden series, and selection stay linked.
pub struct PlotGrid<'a> {
    columns: usize,
    plots: Vec<Plot<'a>>,
    cell_height: f32,
    spacing: f32,
}

impl<'a> PlotGrid<'a> {
    /// A grid with `columns` columns.
    pub fn new(columns: usize) -> Self {
        Self {
            columns: columns.max(1),
            plots: Vec::new(),
            cell_height: 240.0,
            spacing: 8.0,
        }
    }

    /// Add a plot, left to right and top to bottom.
    pub fn plot(mut self, plot: Plot<'a>) -> Self {
        self.plots.push(plot);
        self
    }

    /// Set the height of each cell.
    pub fn cell_height(mut self, height: f32) -> Self {
        self.cell_height = height.max(80.0);
        self
    }

    /// Set the spacing between cells.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self
    }

    /// Show every plot and return their responses in order.
    pub fn show(self, ui: &mut Ui, icons: &mut Icons, state: &mut PlotState) -> Vec<PlotResponse> {
        let mut responses = Vec::with_capacity(self.plots.len());
        let width = ((ui.available_width() - self.spacing * (self.columns.saturating_sub(1) as f32))
            / self.columns as f32)
            .max(120.0);
        for row in self.plots.chunks(self.columns) {
            ui.horizontal(|ui| {
                for plot in row {
                    let response = plot
                        .clone()
                        .width(width)
                        .height(self.cell_height)
                        .show(ui, icons, state);
                    responses.push(response);
                }
            });
            ui.add_space(self.spacing);
        }
        responses
    }
}
