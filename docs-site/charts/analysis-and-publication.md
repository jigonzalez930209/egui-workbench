---
title: Analysis and publication
description: New chart families, colormaps, annotations, analysis tools, math text, vector PDF, and the 3D GPU layer.
---

# Analysis and publication

0.10 turns a chart into an analysis surface: new families, a color system, annotations,
signal tools, math text, and publication-ready export.

## Chart families

```rust
use egui_workbench::components::plot::{
    Plot, box_plot, contour, hexbin, histogram, histogram_density, polar, smith, violin,
};
```

| Family | Call |
|---|---|
| Histogram | `histogram(name, values, bins)` |
| Density histogram | `histogram_density(name, values, bins)` returns bars + a smoothed curve |
| Box plot | `box_plot(name, values)` returns box, median, whiskers, caps |
| Violin | `violin(name, values, samples)` |
| Hexbin / 2D density | `hexbin(name, points, bins)` returns a heatmap series |
| Contour | `contour(name, grid, level, origin, step)` |
| Polar | `polar(name, theta, radius)` |
| Smith | `smith(name, resistance, reactance)` |

Multi-series families use `Plot::series_all(...)`:

```rust
# use egui_workbench::components::plot::{Plot, box_plot};
# let values = [1.0, 2.0, 3.0];
let plot = Plot::new("stats").series_all(box_plot("run", &values));
```

## Colormaps and colorbars

`Colormap` has `Theme`, `Viridis`, `Magma`, `Cividis`, `Turbo`, `CoolWarm`, and a
color-blind-safe categorical palette. Set `HeatmapConfig::colormap`; the heatmap and its
colorbar use the same scale. `show_colorbar(ui, colormap, min, max, height)` draws one
standalone.

```rust
# use egui_workbench::components::plot::{Colormap, HeatmapConfig};
let heatmap = HeatmapConfig {
    colormap: Colormap::Viridis,
    ..HeatmapConfig::default()
};
```

## Analysis tools

All are pure functions over slices:

| Tool | Function |
|---|---|
| Peak detection | `find_peaks(values, min_height, min_distance)` |
| Baseline | `rolling_baseline`, `remove_baseline` |
| Savitzky–Golay | `savgol`, `savgol_derivative` |
| Derivative / integral | `derivative`, `integral` |
| FFT / spectrum | `fft`, `spectrum` |
| Resampling | `resample` |
| Nonlinear fit | `fit_model(x, y, initial, &model)` returns params, standard errors, RMSE |
| Selection statistics | `Statistics::of`, `Statistics::range` |

## Annotations and regions

`PlotState::annotations` holds serializable text, arrows, x/y spans, rectangles,
ellipses, and polygons in data coordinates. They render on screen and in SVG/PNG/PDF
exports. Regions carry a `draggable` flag for a host editor.

```rust
use egui_workbench::components::plot::{PlotAnnotation, PlotState};

let mut state = PlotState::default();
state.annotations.push(PlotAnnotation::text([1.0, 2.0], r"peak $\alpha$"));
state.annotations.push(PlotAnnotation::arrow([1.0, 2.0], [3.0, 4.0]));
state.annotations.push(PlotAnnotation::rect([1.0, 1.0], [2.0, 3.0]));
```

## Math text

Titles, axis labels, legend names, and annotations accept a LaTeX subset: Greek letters,
sub/superscripts, `\frac`, `\sqrt`, `\pm`, `\times`, `\cdot`, `\infty`, `\degree`, and
units. `to_unicode_math` exposes the conversion for a host that measures text itself.

```rust
# use egui_workbench::components::plot::to_unicode_math;
assert_eq!(to_unicode_math(r"$\alpha_{i}^{2}$"), "αᵢ²");
```

## Vector PDF and presets

`Plot::to_pdf(ctx, state, options)` writes a one-page vector PDF from the same scene as
the screen. `ExportFormat::Pdf` is in the toolbar menu. `ExportPreset` provides
`Screen`, `HalfPage`, `Letter`, and `Journal` sizes; `ExportOptions::for_preset` applies
one. `scale` controls PNG DPI; `transparent` drops the background.

```rust
# use egui_workbench::components::plot::{ExportOptions, ExportPreset};
let options = ExportOptions::for_preset(ExportPreset::Journal);
```

## Plot3D on the GPU

`Plot3DConfig::renderer = Renderer::Gpu` (and `Plot3D::renderer`) sends filled surfaces,
bars, quads, rectangles, and circles through the same `egui_wgpu` triangle pipeline;
lines, text, axes, legend, and hover stay on the CPU reference painter. The `gpu` feature
is required, and the host calls `set_output_format` once.

## Budgets

The release probes are `tests/gpu_plot.rs` (2D and 3D through a real headless wgpu
device), `tests/stream_budget.rs` (1M points/s), and `tests/data_budget.rs` (1M × 20 CSV).
The recorded numbers are in
[`docs/BENCHMARKS.md`](https://github.com/jigonzalez930209/egui-workbench/blob/main/docs/BENCHMARKS.md).
