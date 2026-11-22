# Charts overview

> An interactive 2D plot component: series, axes, interaction, fits, live data, and export. Live page: <https://jigonzalez930209.github.io/egui-workbench/charts/overview.html>

`components::plot::Plot` is a 2D plotting component in the spirit of matplotlib. It is
part of the same crate, uses the same themes and icon cache, and keeps its interaction
state in a `PlotState` the application owns.

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
| Chart types | Pie/donut (`Series::pie`), candlesticks, heatmaps, radar, gauges, and waterfalls |
| Hover animation | 13 legend-row effects with user-defined easing, plus the direction arrow for streaming series |
| Axes | Linear or log10 per axis; 1-2-5 ticks measured so labels never overlap; minor log ticks; fixed limits; grid per axis |
| Interaction | Drag pans; wheel zooms around the cursor; right-drag or the zoom tool draws a box; Shift-drag or the select tool selects an area; double-click fits; toolbar |
| Feedback | Hover ring, dashed guides to the axes, a popup with series/x/y; legend toggles series |
| Analysis | Least-squares fits: linear, polynomial 1–8, exponential, power, logarithmic; moving average; equation and R² |
| Live data | `.x_window(10.0)` follows the newest 10 x units while the view is automatic |
| Export | `to_svg`, `to_png` from the same scene as the screen; toolbar export and settings buttons |
| Performance | Per-pixel decimation for sorted lines; nothing repaints while idle |

## Sections

- [Series](../charts/series.md): every constructor and modifier.
- [Chart types](../charts/chart-types.md): pie/donut, candles, heatmap, radar, gauge, waterfall.
- [Legend animation](../charts/animations.md): the 13 hover effects, easing, and the direction arrow.
- [Axes and ticks](../charts/axes-and-ticks.md): scales, limits, ticks, labels.
- [Interaction](../charts/interaction.md): pan, zoom, selection, hover, response.
- [Configuration](../charts/configuration.md): `PlotConfig`, colors, per-series styles.
- [Fits and smoothing](../charts/fits.md): models, equations, R², selections.
- [Export](../charts/export.md): SVG/PNG, sizes, transparency, saving.
- [Live data](../charts/live-data.md): streaming windows and repaint.
- [Performance](../charts/performance.md): million-point series and decimation.

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
