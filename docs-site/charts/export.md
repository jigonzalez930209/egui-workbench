---
title: Export
description: Export the plot to SVG or PNG from the same scene the screen draws, including sizes, transparency, and the toolbar menu.
---

# Export

`to_svg` and `to_png` render the same scene as the screen: same view, colors, grid,
legend, and selection. The toolbar and hover feedback are not included.

<LiveDemo
  scene="plot-bars"
  height="400"
  alt="A bar chart exported from the plot scene"
  caption="Open the toolbar's export menu: the SVG or PNG downloads, drawn from the same scene as the screen."
/>

## Options

```rust
use egui_workbench::components::plot::ExportOptions;

let options = ExportOptions {
    width: 960.0,       // SVG user units and PNG points
    height: 540.0,
    scale: 2.0,         // PNG pixels per point (2 = retina)
    transparent: false, // leave the canvas transparent
};
```

| Field | Default | Meaning |
|---|---|---|
| `width` | 960.0 | Width in points / SVG units |
| `height` | 540.0 | Height in points |
| `scale` | 2.0 | PNG pixels per point |
| `transparent` | false | Transparent canvas instead of the theme background |

## SVG

```rust
let svg = plot.to_svg(ctx, &state, &options);
std::fs::write("report.svg", svg)?;
```

The SVG is standalone: shapes, text, colors, and selection, with egui's fonts referenced
by family. It uses the plot's current `PlotConfig` and the `PlotState` view (or the
automatic view when none was set).

## PNG

```rust
let png = plot.to_png(ctx, &state, &options)?;
std::fs::write("report.png", png)?;
```

`to_png` rasterizes the SVG with `resvg` and loads egui's own font bytes under the same
font family, so the raster matches the screen. It returns
`Result<Vec<u8>, String>`; the error is human-readable (for example a font or SVG
failure).

## Toolbar buttons

```rust
let plot = Plot::new("report")
    .export_button(true)      // SVG/PNG menu in the toolbar
    .settings_button(true)    // gear button
    .series(Series::line("signal", &data));

let response = plot.clone().show(ui, &mut icons, &mut state);
if let Some(format) = response.export_requested {
    self.pending_export = Some(format);
}
if response.settings_requested {
    self.settings_open = true;
}
```

`ExportFormat` is `Svg` or `Png`. The application decides where to save and whether to
show a dialog first.

## Build once, use twice

`Plot` is `Clone` and borrows its data. Build it in one function so the screen and the
export cannot drift:

```rust
fn build<'a>(data: &'a [[f64; 2]], config: &PlotConfig) -> Plot<'a> {
    Plot::new("report")
        .config(config.clone())
        .title("Throughput")
        .x_label("time (s)")
        .y_label("req/s")
        .series(Series::line("requests", data))
        .series(Series::hline("target", 1000.0).hide_from_legend())
}

let plot = build(&self.data, &self.config);
let response = plot.clone().show(ui, &mut self.icons, &mut self.plot_state);
if self.save_requested {
    let bytes = plot.to_png(ui.ctx(), &self.plot_state, &ExportOptions::default())?;
    std::fs::write(path, bytes).map_err(|error| error.to_string())?;
}
```

## Saving with feedback

Keep I/O in the application and report the result with a toast:

```rust
let id = self.toasts.loading("Exporting chart…");
match plot.to_png(ctx, &self.plot_state, &options) {
    Ok(bytes) => match std::fs::write(&path, bytes) {
        Ok(()) => {
            self.toasts.update(id, Toast::new(ToastKind::Success, "Exported").description(path.display().to_string()));
        }
        Err(error) => {
            self.toasts.update(id, Toast::new(ToastKind::Error, "Could not write the file").description(error.to_string()));
        }
    },
    Err(error) => {
        self.toasts.update(id, Toast::new(ToastKind::Error, "Could not render the chart").description(error));
    }
}
```

The template's Charts section implements this flow: it asks for a folder, exports from
the same `Plot` it renders, and shows a loading toast that resolves in place.

## Pixel rules

- The frame and all grid, tick, and reference lines sit on pixel centers.
- Bars snap to physical pixels.
- Margins come from the measured tick labels.
- Dense sorted lines are reduced per pixel column.
- The export keeps the selection and hidden series from `PlotState`, but not the toolbar,
  hover ring, guides, or popup.

## Sizes

For a report, use 2× the display size in points at `scale: 2.0` for a crisp raster. For
slides, 1600×900 at `scale: 1.0` is usually enough. SVG has no scale; set `width` and
`height` to the target user units and let the consumer scale it.
