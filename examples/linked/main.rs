//! One dataset, a scatter plot, a line plot, a grid, and a table linked by a selection.
//!
//! Run with `cargo run --example linked`. Brush a plot, select a row, or change the
//! filter mode; the other views follow.

#![allow(clippy::expect_used)]

use egui_workbench::ThemeId;
use egui_workbench::components::plot::{Bounds, Plot, PlotGrid, PlotState, Series};
use egui_workbench::components::{DataGrid, DataGridState};
use egui_workbench::data::{Column, Dataset, LinkHub, LinkMode};
use egui_workbench::eframe;
use egui_workbench::egui;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    eframe::run_native(
        "Linked views",
        egui_workbench::native_options("Linked views"),
        Box::new(|cc| {
            egui_workbench::theme::apply_id(&cc.egui_ctx, ThemeId::GithubDark);
            Ok(Box::new(Linked::new()))
        }),
    )
}

struct Linked {
    data: Dataset,
    hub: LinkHub,
    plot_state: PlotState,
    grid_state: DataGridState,
    status: String,
}

impl Linked {
    fn new() -> Self {
        let t: Vec<f64> = (0..400).map(|index| index as f64 * 0.02).collect();
        let v: Vec<f64> = t.iter().map(|t| (t * 1.7).sin() + (t * 0.3).cos() * 0.2).collect();
        let state: Vec<String> = t
            .iter()
            .map(|t| {
                if t.fract() < 0.5 {
                    "run".to_owned()
                } else {
                    "idle".to_owned()
                }
            })
            .collect();
        let data = Dataset::from_columns([
            Column::floats("t", t).with_unit("s"),
            Column::floats("v", v).with_unit("V"),
            Column::text("state", state),
        ]);
        Self {
            data,
            hub: LinkHub::new("trace"),
            plot_state: PlotState::default(),
            grid_state: DataGridState::default(),
            status: "Brush a plot or click a table row".to_owned(),
        }
    }
}

impl eframe::App for Linked {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.hub.mode, LinkMode::Highlight, "Highlight");
                ui.selectable_value(&mut self.hub.mode, LinkMode::Filter, "Filter");
                ui.selectable_value(&mut self.hub.mode, LinkMode::Isolate, "Isolate");
                if ui.button("Clear selection").clicked() {
                    self.hub.clear();
                }
                ui.label(&self.status);
            });
        });
        let points: Vec<[f64; 2]> = self.data.xy_pairs("t", "v").expect("points");
        egui::CentralPanel::default().show(ui, |ui| {
            let mut icons = egui_workbench::Icons::new();
            ui.columns(2, |columns| {
                let _ = Plot::from_dataset(&self.data)
                    .x("t")
                    .y("v")
                    .color_by("state")
                    .height(220.0)
                    .show_linked(&mut columns[0], &mut icons, &mut self.plot_state, &mut self.hub);
                let grid = PlotGrid::new(1)
                    .cell_height(220.0)
                    .plot(Plot::new("line").series(Series::line("v", points.as_slice())))
                    .plot(Plot::new("scatter").series(Series::scatter("v", points.as_slice())));
                let mut grid_state = self.plot_state.clone();
                let responses = grid.show(&mut columns[1], &mut icons, &mut grid_state);
                if responses.iter().any(|response| response.selection_changed) {
                    self.hub.set_rows(
                        self.data
                            .rows_in_bounds("t", "v", Bounds::from_corners([0.0, -1.0], [4.0, 1.0])),
                        "grid",
                    );
                }
                self.plot_state = grid_state;
            });
            ui.separator();
            let _ = DataGrid::new(&self.data, &mut self.grid_state)
                .link(&self.hub)
                .height(220.0)
                .show(ui);
            if !self.grid_state.selected_rows.is_empty() {
                self.hub
                    .set_rows(self.grid_state.selected_rows.iter().copied(), "table");
                self.grid_state.selected_rows.clear();
                self.status = format!("{} rows selected", self.hub.selection().rows.len());
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
