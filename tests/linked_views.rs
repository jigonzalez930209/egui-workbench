//! Linked views: a dataset plot, a grid, and a selection bus.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_workbench::components::plot::{Bounds, Plot, PlotGrid, PlotState};
use egui_workbench::data::{Column, Dataset, LinkHub, LinkMode};
use egui_workbench::egui;

fn data() -> Dataset {
    Dataset::from_columns([
        Column::floats("t", vec![0.0, 1.0, 2.0, 3.0]),
        Column::floats("v", vec![0.1, 0.4, 0.2, 0.5]),
        Column::text("state", ["idle", "run", "run", "idle"]),
    ])
}

#[test]
fn a_plot_can_read_columns_and_map_a_brush_to_rows() {
    let data = data();
    let builder = Plot::from_dataset(&data).x("t").y("v").color_by("state");
    let rows = builder.rows_in(Bounds::from_corners([0.5, 0.15], [2.5, 0.45]));
    assert_eq!(rows, [1, 2]);
}

#[test]
fn a_selection_bus_publishes_rows_and_a_predicate() {
    let data = data();
    let mut hub = LinkHub::new("trace.csv");
    hub.set_rows([1, 2], "scatter");
    assert!(hub.selection().contains(2));
    hub.set_mode(LinkMode::Filter);
    assert_eq!(hub.mode(), LinkMode::Filter);
    let rows = hub.rows_for(&data, "v > 0.3").expect("predicate");
    assert_eq!(rows, [1, 3]);
    hub.clear();
    assert!(hub.selection().is_empty());
}

#[test]
fn a_linked_plot_publishes_its_brush() {
    let data = data();
    let ctx = egui::Context::default();
    let mut icons = egui_workbench::Icons::new();
    let mut state = PlotState::default();
    state.selection = Some(Bounds::from_corners([0.5, 0.15], [2.5, 0.45]));
    let mut hub = LinkHub::new("trace.csv");
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let _ = Plot::from_dataset(&data)
            .x("t")
            .y("v")
            .show_linked(ui, &mut icons, &mut state, &mut hub);
    });
    output.textures_delta.clear();
    assert_eq!(hub.selection().rows.iter().copied().collect::<Vec<_>>(), [1, 2]);
    assert_eq!(hub.selection().source.as_deref(), Some("plot"));
}

#[test]
fn a_plot_grid_shares_one_state() {
    let data = data();
    let points: Vec<[f64; 2]> = data.xy_pairs("t", "v").expect("points");
    let ctx = egui::Context::default();
    let mut icons = egui_workbench::Icons::new();
    let mut state = PlotState::default();
    let grid = PlotGrid::new(2)
        .cell_height(180.0)
        .plot(Plot::new("a").series(egui_workbench::components::plot::Series::line("v", points.as_slice())))
        .plot(Plot::new("b").series(egui_workbench::components::plot::Series::scatter(
            "v",
            points.as_slice(),
        )))
        .plot(Plot::new("c").series(egui_workbench::components::plot::Series::line("v", points.as_slice())));
    let mut grid = Some(grid);
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        if let Some(grid) = grid.take() {
            let responses = grid.show(ui, &mut icons, &mut state);
            assert_eq!(responses.len(), 3);
        }
    });
    output.textures_delta.clear();
}
