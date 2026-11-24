---
title: Linked views
description: A selection bus shared by plots and tables, dataset-bound plots, plot grids, and zoom history.
---

# Linked views

0.8 connects the data line: one `Dataset`, one `LinkHub`, and any number of views. A
selection made in a plot, a grid, or the table publishes rows; every other view reads the
same revision and follows.

## Selection bus

```rust
use egui_workbench::data::{Dataset, LinkHub, LinkMode};

let mut hub = LinkHub::new("trace.csv");
hub.set_rows([1, 2, 5], "scatter");
assert!(hub.selection().contains(2));
hub.set_mode(LinkMode::Filter); // Highlight, Filter, or Isolate
let rows = hub.rows_for(&data, "voltage > 0.3"); // predicate brush
```

`LinkSelection` carries selected rows, the publisher name, and an optional predicate (the
query text behind a brush). `revision()` changes with every selection or mode change, so
a view only recomputes when something actually moved. `clear()` empties it.

| Mode | Table | Plot |
|---|---|---|
| `Highlight` | selected rows emphasized | brush stays visible |
| `Filter` | unselected rows hidden | points outside selection hidden |
| `Isolate` | only selected rows | only selected points |

## Plots bound to columns

```rust
use egui_workbench::components::plot::{Plot, PlotState};
use egui_workbench::data::LinkHub;

let mut state = PlotState::default();
let mut hub = LinkHub::new("trace");
let response = Plot::from_dataset(&data)
    .x("t")
    .y("v")
    .color_by("state")
    .height(260.0)
    .show_linked(ui, &mut icons, &mut state, &mut hub);
```

`x`, `y`, `color_by`, `scatter`, `title`, and `height` build the series. `color_by`
splits a text/category column into one colored series per value. `show_linked` maps the
plot's rectangular selection back to dataset rows and publishes them to the hub. A plot
in Filter or Isolate mode draws only the hub's rows. `Series::from_dataset` remains the
lower-level one-series form.

## Plot grid

`PlotGrid` lays plots out in rows and columns and shares one `PlotState`, so pan, zoom,
hidden series, and selection stay linked:

```rust
use egui_workbench::components::plot::{Plot, PlotGrid, PlotState, Series};

let grid = PlotGrid::new(2)
    .cell_height(220.0)
    .plot(Plot::new("line").series(Series::line("v", &points)))
    .plot(Plot::new("scatter").series(Series::scatter("v", &points)))
    .plot(Plot::new("second").series(Series::line("v", &points)));
let responses = grid.show(ui, &mut icons, &mut state);
```

Because the state is shared, a wheel zoom in one cell moves every cell. A per-cell legend
override still lives in `PlotConfig::series_style_mut`.

## View history

`ViewHistory` is a bounded back/forward stack of zoom levels:

```rust
use egui_workbench::components::plot::{Bounds, ViewHistory};

let mut history = ViewHistory::new();
history.push(Bounds::from_corners([0.0, 0.0], [1.0, 1.0]));
history.push(Bounds::from_corners([0.2, 0.2], [0.8, 0.8]));
assert!(history.back());
assert!(history.forward());
```

`PlotState::view()` exposes the explicit view; push it whenever the response's view
changes, and call `PlotState::set_view` when walking the history.

## The table follows

`DataGrid::link(&hub)` reads the selection and the mode. In Filter/Isolate it keeps only
the selected rows; in Highlight it paints them selected even when the grid's own
selection is elsewhere. Publish a table click by reading `DataGridState::selected_rows`
and calling `hub.set_rows`.

## Example

`cargo run --example linked` opens one dataset with a colored scatter, a line/scatter
grid, and a linked `DataGrid`. The toolbar switches Highlight, Filter, and Isolate, and
the status line reports the published row count.
