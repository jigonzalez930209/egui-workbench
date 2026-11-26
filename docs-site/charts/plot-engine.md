---
title: Plot engine
description: GPU data layers, level-of-detail pyramids, streaming series, and time, category, and symmetric-log axes.
---

# Plot engine

0.9 adds the performance path under `Plot`: an optional GPU data layer, a min/max
pyramid for out-of-core data, a streaming series, and the axis scales that real
instrument data needs.

## GPU data layer

Enable the `gpu` feature and pick the renderer per plot:

```toml [Cargo.toml]
egui-workbench = { version = "0.9", features = ["gpu"] }
```

```rust
use egui_workbench::components::plot::{Plot, Renderer, Series};

let plot = Plot::new("scope")
    .renderer(Renderer::Gpu)
    .series(Series::line("signal", &points));
```

The GPU path draws line and scatter series through an `egui_wgpu` paint callback:
screen-space triangles, one draw per series, in the same render pass as egui. Charts
outside the line/scatter families, axes, legend, hover, selection, and export stay on the
CPU painter; that painter is also the fallback and the snapshot reference.

The host must tell the callback its wgpu surface format once at startup:

```rust
use egui_workbench::components::plot::set_output_format;

// `render_state.target_format` from eframe/egui-wgpu.
let _ = set_output_format(render_state.target_format);
```

Supported formats are `Bgra8UnormSrgb` (default), `Rgba8UnormSrgb`, `Bgra8Unorm`, and
`Rgba8Unorm`. The GPU path is validated in `tests/gpu_plot.rs`, which renders through a
real headless wgpu device. The default build stays glow/CPU and keeps the small footprint.

## Level-of-detail

`MinMaxPyramid` builds min/max tiles lazily over a [`DataSource`](/app/data), so a plot can
ask for the range the viewport shows instead of materializing every point:

```rust
use std::sync::Arc;
use egui_workbench::data::{DataSource, MemorySource, MinMaxPyramid};

# let source: Arc<dyn DataSource> = Arc::new(MemorySource::new(Default::default()));
let pyramid = MinMaxPyramid::new(source, 0, 1, 256);
let tiles = pyramid.range(0..1_000_000, 512).expect("range");
let points = pyramid.polyline(0..1_000_000, 512).expect("polyline");
```

Levels are built on first use and blocks are cached; `invalidate()` clears them when the
source revision changes. A memory-mapped or paged source plugs in unchanged.

## Streaming

`StreamingSeries` is the append-only path for live data:

```rust
use egui_workbench::data::{StreamMode, StreamingSeries, Trigger};

let mut series = StreamingSeries::new(100_000)
    .with_window(10.0)              // last 10 x units
    .with_trigger(Trigger::rising(0.8));
series.push([0.0, 0.1]);
series.set_mode(StreamMode::Frozen); // hold the current window
series.set_mode(StreamMode::Live);   // resume
```

`push` drops the oldest point at capacity, trims the rolling x window, and returns whether
the trigger fired. `revision()` changes for every accepted point; `fired()` counts edges.

## Measurements

Two toolbar tools draw persistent measurements in data coordinates, so they survive zoom,
pan, and a saved `PlotState`:

- **Ruler** (`PlotTool::MeasureDistance`): drag between two points. Snaps to the nearest
  visible point and reports `Δx`, `Δy`, Euclidean distance, slope, and angle.
- **Area** (`PlotTool::MeasureArea`) has two modes, selected in `PlotConfig.measurement`:
  - **Rectangle**: drag a rectangle. Reports its area, and for every visible series the
  minimum and maximum (with the slope at each extremum), the mean, the trapezoidal
  integral, and the endpoint slope.
  - **Curve vs chord**: drag between two points on the same curve. Reports the signed and
  absolute area between the curve and its chord, the arc length, the chord
  distance/slope/angle, and the maximum deviation.

```rust
use egui_workbench::components::plot::{
    Bounds, MeasurementMode, PlotMeasurement, PlotState, curve_area_summary, region_measurements,
};

let mut state = PlotState::default();
state.add_measurement(PlotMeasurement::distance([1.0, 2.0], [4.0, 6.0]));
state.add_measurement(PlotMeasurement::area(Bounds::from_corners([0.0, 0.0], [10.0, 5.0])));

// The same numbers the labels show, available to the application.
if let Some(summary) = state.measurements()[0].distance_summary() {
    let _ = (summary.distance, summary.slope, summary.angle_deg);
}
let stats = region_measurements(&plot, &state, state.measurements()[1].bounds());
for series in stats {
    let _ = (series.min, series.max, series.integral, series.mean);
}

// Curve vs chord, created by dragging in CurveSegment mode.
let measurement = PlotMeasurement::curve_area("line", [0.0, 0.0], [2.0, 0.0], 0, 2);
if let Some(summary) = curve_area_summary(&plot, &state, &measurement) {
    let _ = (summary.area, summary.absolute_area, summary.arc_length);
}
```

`MeasurementConfig` sets the mode, the signed/absolute label, the snap radius, and
whether labels draw. The builder mirrors it:

```rust
let plot = plot
    .measurement_mode(MeasurementMode::CurveSegment)
    .measurement_snap_radius(18.0)
    .measurement_labels(true);
```

`PlotState::pop_measurement`, `remove_measurement`, `clear_measurements`, and
`measurements` manage the list. Measurements are serialized with the rest of the state
when the `serde` feature is enabled.

## Filters and transforms

One sliders icon in the chart toolbar opens a menu with every filter and transform. An
active pipeline colours the icon; the same toggles are available in the chart settings.
Filters apply to every visible series; transforms use the first series, and
cross-correlation uses the first two.

| Filter | Control |
|---|---|
| Savitzky–Golay | window, polynomial order |
| Moving average | window |
| Gaussian | sigma |
| Median | window |
| Low-pass | alpha |

| Transform | Result |
|---|---|
| FFT magnitude | one-sided amplitude spectrum |
| Inverse FFT | inverse DFT of the series as a complex spectrum |
| Power spectral density | one-sided PSD |
| Autocorrelation | normalized correlation by lag |
| Cross-correlation | normalized correlation of two series |
| Hilbert envelope | analytic-signal magnitude |
| Cepstrum | real cepstrum |

The state API is `PlotState::toggle_filter`, `set_filter`, `set_transform`,
`clear_analysis`, and `pipeline`. A new transform clears measurements because the data
domain changed. `FilterSettings` and `TransformSettings` expose the parameters, and a
custom settings dialog can edit them directly.

The toolbar also has a clear (eraser) icon that appears only when the chart has
measurements, annotations, selection, or analysis; it calls `PlotState::clear_overlays`.
The pin at the right edge controls the ribbon: hover it to unroll the existing icons with
an animation, click it to keep the ribbon open, and drag it to move the ribbon anywhere in
the plot. The settings gear opens a draggable panel with the measurement mode, area
style, snap radius, labels, filter toggles and parameters, and transforms.

## Axes

| Scale | Use |
|---|---|
| `Scale::Linear` | default |
| `Scale::Log10` | positive decades |
| `Scale::SymmetricLog` | signed data around zero |
| `Scale::Time` | Unix seconds, calendar tick labels (UTC) |
| `Scale::Category` | evenly spaced categories |

Category labels come from `AxisConfig::categories`:

```rust
use egui_workbench::components::plot::{AxisConfig, Scale};

let x = AxisConfig {
    scale: Scale::Category,
    categories: Some(vec!["idle".into(), "run".into(), "stop".into()]),
    ..AxisConfig::default()
};
```

Time ticks choose 1/2/5 seconds, minutes, hours, days, or years from the visible span and
format with chrono (`%H:%M:%S`, `%H:%M`, `%m-%d %H:%M`, or `%Y-%m-%d`). Time zones beyond
UTC are a 0.16 localization concern.

## Budgets

The recorded baselines live in
[`docs/BENCHMARKS.md`](https://github.com/jigonzalez930209/egui-workbench/blob/main/docs/BENCHMARKS.md).
The GPU path is measured by the `gpu_plot` test and the CPU fallback by the existing
`plot_decimation` benchmark; the data path by the 1M-row probe. The 0.9 targets are
interactive pan/zoom over 100M points on the GPU and sustained streaming of 1M points/s;
the reference machine numbers are recorded with each release.
