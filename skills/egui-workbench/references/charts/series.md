# Series

> Lines, areas, steps, scatter, bars, functions, reference lines, error bars, and markers. Live page: <https://jigonzalez930209.github.io/egui-workbench/charts/series.html>

A `Series<'a>` is one named data set. Data is borrowed when possible (`&[[f64; 2]]`), so
a million-point buffer is not copied every frame. Non-finite points break a line into
segments.

```rust
use egui_workbench::components::plot::{LineStyle, Marker, Series};

Plot::new("series")
    .series(Series::line("signal", &data))
    .series(Series::scatter("samples", &samples).marker(Marker::Diamond).radius(4.0))
    .series(Series::bars("histogram", &bars).bar_width(0.8).base(0.0))
    .series(Series::hline("limit", 4.0))
    .series(Series::vline("now", 12.5))
    .show(ui, &mut icons, &mut state);
```

## Constructors

| Constructor | Default drawing |
|---|---|
| `Series::line(name, data)` | 1.5 pt polyline through the points |
| `Series::area(name, data)` | Line filled down to `y = 0` at 18 % opacity |
| `Series::step(name, data)` | Staircase: each value holds until the next x |
| `Series::scatter(name, data)` | Unconnected markers, 3 pt radius circles |
| `Series::bars(name, data)` | Vertical bars from `y = 0`; width 80 % of the smallest x spacing |
| `Series::function(name, range, samples, f)` | `f(x)` sampled `samples` times over the range |
| `Series::hline(name, y)` | Horizontal reference line across the plot |
| `Series::vline(name, x)` | Vertical reference line across the plot |

`data` accepts `&[[f64; 2]]`, `Vec<[f64; 2]>`, or any `Into<Cow<[[f64; 2]]>>`. Functions
return `Series<'static>` because they own the sampled points.

## Modifiers

| Method | Applies to | Effect |
|---|---|---|
| `.color(Color32)` | all | Explicit color; the theme palette is used in order otherwise |
| `.width(f32)` | lines, steps, areas, reference lines | Stroke width in points, minimum 0.5 |
| `.style(LineStyle)` | lines | `Solid`, `Dashed` (6/4), `Dotted` (1.5/3) |
| `.fill_to(f64)` | lines | Fill between the curve and a base |
| `.marker(Marker)` | scatter and line vertices | Marker shape |
| `.radius(f32)` | scatter | Marker radius, minimum 1 |
| `.bar_width(f64)` | bars | Bar width in data units |
| `.base(f64)` | bars | Where bars start (default 0) |
| `.y_errors(&[f64])` | line, scatter, bars | Symmetric ±error whiskers |
| `.hide_from_legend()` | all | Draw the series without a legend row |

`LineStyle` and `Marker` are re-exported from `components::plot`.

## Markers

`Circle` (default), `Square`, `Diamond`, `Triangle`, `Cross`, `Plus`, `Ring`. Cross and
Plus are stroked; the others are filled. A `Ring` is a circle outline.

## Error bars

```rust
let errors = vec![0.1, 0.2, 0.15, 0.3];
Plot::new("measurements")
    .series(Series::scatter("samples", &data).y_errors(&errors))
    .series(Series::line("fit", &fit_points))
    .show(ui, &mut icons, &mut state);
```

Errors affect the automatic y bounds, so whiskers are never clipped.

## Colors and themes

Without `.color`, series take the theme palette in drawing order. The palette is
resolved from the active `Theme`; dark and light themes stay distinguishable. Use
`PlotConfig::series_style_mut(name)` to override color, width, style, marker, marker size,
and fill opacity without rebuilding the series.

## Reference lines

`hline` and `vline` have no data points; they span the visible frame. Use them for
thresholds, targets, zero lines, or the current time. They appear in the legend unless
you call `.hide_from_legend()`.

## Legend behavior

The legend is drawn inside the frame (top-left by default). Clicking a row hides or shows
the series; hidden names are stored in `PlotState`, so the state survives frames. Long
names elide instead of colliding with the toolbar.

## Data rules

- Sort x ascending for lines when possible: sorted series enable binary search,
  per-pixel decimation, and hover lookup.
- Use `f64::NAN` or infinity to break a line into segments.
- Bars with log y axes and non-positive bases are not meaningful; clamp the data or use
  a linear axis.
- A series with fewer than two points still appears in the legend and hover, but draws no
  line.
