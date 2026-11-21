---
title: Charts overview
description: "An interactive 2D plot component: series, axes, interaction, fits, live data, and export."
---

# Charts overview

`components::plot::Plot` is a 2D plotting component in the spirit of matplotlib. It is
part of the same crate, uses the same themes and icon cache, and keeps its interaction
state in a `PlotState` the application owns.

<LiveDemo
  scene="plot"
  height="440"
  alt="A line chart with a legend, grid, axes, and a toolbar"
  caption="Scroll to zoom, drag to pan, double-click to reset, click a legend entry to hide a series."
/>

```rust
use egui_workbench::components::plot::{Plot, PlotState, Series};

let data: Vec<[f64; 2]> = (0..200)
    .map(|index| {
        let x = index as f64 * 0.05;
        [x, x.sin()]
    })
    .collect();

let mut state = PlotState::default();
let response = Plot::new("signal")
    .title("Signal")
    .x_label("t (s)")
    .y_label("V")
    .height(320.0)
    .series(Series::line("sin", &data))
    .show(ui, &mut icons, &mut state);

if let Some(hover) = response.hovered {
    self.status = format!("{}: {:.2}", hover.series, hover.point[1]);
}
```

## Capabilities

| Area | What you get |
|---|---|
| Series | `line`, `area`, `step`, `scatter` (7 markers), `bars`, `function`, `hline`, `vline`; color, width, dashed/dotted, marker, `y_errors`, `fill_to` |
| Axes | Linear or log10 per axis; 1-2-5 ticks measured so labels never overlap; minor log ticks; fixed limits; grid per axis |
| Interaction | Drag pans; wheel zooms around the cursor; right-drag or the zoom tool draws a box; Shift-drag or the select tool selects an area; double-click fits; toolbar |
| Feedback | Hover ring, dashed guides to the axes, a popup with series/x/y; legend toggles series |
| Analysis | Least-squares fits: linear, polynomial 1–8, exponential, power, logarithmic; moving average; equation and R² |
| Live data | `.x_window(10.0)` follows the newest 10 x units while the view is automatic |
| Export | `to_svg`, `to_png` from the same scene as the screen; toolbar export and settings buttons |
| Performance | Per-pixel decimation for sorted lines; nothing repaints while idle |

## Sections

- [Series](/charts/series): every constructor and modifier.
- [Axes and ticks](/charts/axes-and-ticks): scales, limits, ticks, labels.
- [Interaction](/charts/interaction): pan, zoom, selection, hover, response.
- [Configuration](/charts/configuration): `PlotConfig`, colors, per-series styles.
- [Fits and smoothing](/charts/fits): models, equations, R², selections.
- [Export](/charts/export): SVG/PNG, sizes, transparency, saving.
- [Live data](/charts/live-data): streaming windows and repaint.
- [Performance](/charts/performance): million-point series and decimation.

## Ownership

`Plot` is `Clone` and borrows its data. `PlotState` holds the view, selection, active
tool, hidden series, and drag state. Keep both in your model:

```rust
struct TabData {
    points: Vec<[f64; 2]>,
    plot: PlotState,
}
```

Build the plot in one function when you need it for both `show` and export; the export
uses the same configuration, view, colors, legend, and selection.

## The starter gallery

The template's **Charts** section demonstrates every feature, including a settings
dialog and export to a folder (a download here). Read `templates/app/src/charts.rs` and
`templates/app/src/chart_settings/` for a complete implementation.

<LiveDemo
  scene="charts"
  height="780"
  alt="The starter Charts gallery"
  caption="The whole gallery, live: fit selector, live data, tools, selection summary, the settings dialog, and export."
/>
