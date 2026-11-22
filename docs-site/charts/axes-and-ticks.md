---
title: Axes and ticks
description: Linear and log scales, automatic and fixed limits, major and minor ticks, grid, and label precision.
---

# Axes and ticks

<LiveDemo
  scene="plot-log"
  height="420"
  alt="A chart with a logarithmic x axis and minor ticks"
  caption="A log10 axis with minor ticks and one notation for the axis labels."
/>

## Scales

```rust
use egui_workbench::components::plot::{Plot, Scale};

Plot::new("growth")
    .x_scale(Scale::Log10)
    .y_scale(Scale::Linear)
    // ...
```

`Scale::Linear` and `Scale::Log10` apply per axis. On a log axis, values `<= 0` are
ignored for bounds and cannot be drawn; a log axis with no positive data falls back to a
safe linear range.

The plot stores the view in scaled space internally, so zooming and panning behave
naturally on log axes. `PlotResponse::screen_pos` and `data_pos` convert between screen
positions and the original data values.

## Automatic limits

With `min` and `max` both `None`, the view fits all visible series, including error bars,
bar widths, and `fill_to` bases. `Plot::include(point)` adds a point such as the origin to
the automatic bounds without drawing it.

```rust
Plot::new("throughput")
    .include([0.0, 0.0])
    .series(Series::line("requests", &data));
```

## Fixed limits

```rust
Plot::new("bounded")
    .x_limits(Some(0.0), Some(10.0))
    .y_limits(Some(0.0), None) // automatic top
```

`x_limits` and `y_limits` write `PlotConfig::{x,y}.{min,max}`. `None` keeps that side
automatic. `PlotState::reset_view()` returns to the automatic view; double-click and the
toolbar's fit button call the same path.

## Ticks

The default major spacing is the 1-2-5 sequence chosen from the available pixels. Ticks
are measured before painting, so labels never overlap; one notation is chosen per axis so
labels share a scale (`1k`, `1M`, or full numbers) and decimals follow the step.

| Setting | Default | Effect |
|---|---|---|
| `major_step` | `None` | Fixed distance between majors, in data units or decades on log axes |
| `minor` | `Minor::Auto` | `Off`, `Auto`, or `Divisions(n)` (2–20) |
| `decimals` | `None` | Fixed decimals; derived from the step otherwise |
| `tick_labels` | `true` | Draw the labels |
| `grid_major` / `grid_minor` | `true` | Grid lines per axis |

```rust
use egui_workbench::components::plot::{AxisConfig, Minor};

let mut config = PlotConfig::default();
config.x.major_step = Some(1.0);
config.x.minor = Minor::Divisions(4);
config.x.decimals = Some(1);
config.y.grid_minor = false;
```

## Log axes

- Major ticks are decades; minor ticks are 2…9 per decade when space allows.
- Labels use one notation per axis; very large or small ranges switch to `10ⁿ`-style
  labels.
- A minor count that would overlap is dropped automatically.
- Fixed limits must be positive; `PlotConfig::sanitize` removes non-positive values from
  log axes.

## Grid and frame

The frame and every grid, tick, and reference line sit on pixel centers. Major grid
opacity defaults to 0.55, minor to 0.22; both come from `PlotConfig`. The frame and tick
lengths are configurable:

```rust
config.frame_width = 1.0;
config.grid_width = 1.0;
config.tick_length = 4.0;
```

## Axis labels

```rust
Plot::new("voltage")
    .x_label("t (s)")
    .y_label("V")
    .title("Battery discharge");
```

Axis labels also appear in the hover popup. The y label is drawn vertically; the right
axis title (`.y2_label(...)`) runs down the right edge. Each title has its own size and
color (`AxisConfig::label_size`, `label_color`), and the chart title its color and
alignment (`PlotConfig::title_color`, `title_align`). Long labels
are measured; if they do not fit, the plot reduces the data area rather than clipping the
text.

## Sanitizing edits

After a settings dialog edits limits, step, or divisions, call `config.sanitize()`:

- clamps font sizes, line widths, and opacities;
- drops an inverted limit pair;
- removes non-positive limits on log axes;
- clamps `Minor::Divisions` to 2–20;
- clamps decimals to at most 10.

Then rebuild the `Plot` with `.config(config)`. If limits or scales changed, call
`PlotState::reset_view()` so the new automatic bounds take effect.
