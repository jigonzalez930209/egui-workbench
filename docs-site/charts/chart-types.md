---
title: Chart types
description: Pie and donut slices, candlesticks, heatmaps, radar polygons, gauges, and waterfalls.
---

# Chart types

`Plot` infers the chart family from its series:

| Series | Family | Axes |
|---|---|---|
| `line`, `area`, `step`, `scatter`, `bars`, `candles`, `heatmap`, `waterfall` | Cartesian | x/y |
| `pie` | Pie / donut | None |
| `radar` | Radar | Labels around a ring |
| `gauge` | Gauge | One arc per series |

A plot may mix any cartesian series. Mixing pie, radar, and gauge in one plot is not
meaningful: the first special family wins (`Pie`, then `Radar`, then `Gauge`) and the
others are ignored. `Plot::chart_mode()` reports the inferred family.

<LiveDemo
  scene="plot-types"
  height="560"
  alt="A chart with a selector for pie, candlesticks, heatmap, radar, gauge, and waterfall"
  caption="Switch families, pick a legend hover animation, toggle the direction arrow, and hover a legend row."
/>

## Pie and donut

One series per slice; the name is the slice label and the value is normalized over the
visible slices.

```rust
let config = PlotConfig {
    pie: PieConfig { inner_radius: 0.55, ..PieConfig::default() },
    ..PlotConfig::default()
};

Plot::new("share")
    .title("Traffic by platform")
    .config(config)
    .series(Series::pie("Desktop", 54.0))
    .series(Series::pie("Mobile", 31.0))
    .series(Series::pie("Tablet", 9.0))
    .series(Series::pie("Other", 6.0))
    .show(ui, &mut icons, &mut state);
```

`PieConfig`:

| Field | Default | Meaning |
|---|---|---|
| `inner_radius` | `0.0` | Donut hole as a fraction of the outer radius (`0` is a pie) |
| `start_angle` | `-π/2` | Where the first slice starts (12 o'clock) |
| `labels` | `true` | Paint value/percentage labels on slices with room |
| `percentages` | `true` | Percentages instead of raw values |
| `total` | `true` | Write the sum inside a donut |

Clicking a slice's legend row hides that slice and renormalizes the rest. Hovering a row
plays the legend animation (an exploded slice, a color blend, a pulse, …).

## Candlesticks

Rows are `[x, open, high, low, close]`. The close is the series' projected data, so hover
and the direction arrow use it; the automatic y range includes high and low.

```rust
let rows: Vec<[f64; 5]> = vec![
    // x, open, high, low, close
    [0.0, 10.0, 12.0, 9.0, 11.0],
    [1.0, 11.0, 13.0, 10.0, 10.5],
    [2.0, 10.5, 14.0, 10.0, 13.5],
];

Plot::new("price")
    .series(Series::candles("price", rows).bar_width(0.62))
    .show(ui, &mut icons, &mut state);
```

Bodies use the theme's success color when close ≥ open and the danger color otherwise.
`.bar_width` sets the body width in data units.

## Heatmap

Cells are `[x, y, value]`. The cell size follows the smallest gap on each axis and the
color scale spans the visible values, from the theme link color through the done color
to the warning color.

```rust
let cells: Vec<[f64; 3]> = (0..24)
    .flat_map(|x| (0..18).map(move |y| [f64::from(x), f64::from(y), activity(x, y)]))
    .collect();

Plot::new("activity")
    .x_label("Hour")
    .y_label("Weekday")
    .series(Series::heatmap("activity", cells))
    .show(ui, &mut icons, &mut state);
```

`PlotConfig::heatmap` has `opacity` (0–1) and `colorbar` (a value scale at the right edge,
on by default). A heatmap expects linear axes.

## Radar

One closed polygon per series over the axes in `RadarConfig::labels`. Values are indexed
in order; zero is the center.

```rust
Plot::new("models")
    .radar(RadarConfig::new(["Speed", "Comfort", "Cost", "Safety", "Range"]))
    .series(Series::radar("Model A", [8.0, 7.0, 6.0, 9.0, 5.0]))
    .series(Series::radar("Model B", [6.0, 9.0, 8.0, 7.0, 8.0]))
    .show(ui, &mut icons, &mut state);
```

`RadarConfig`: `labels`, `max` (automatic when `None`), `rings` (4), and `fill_opacity`
(0.16). Drag the legend and the toolbar still work; pan/zoom do not apply to non-cartesian
families.

## Gauge

Each gauge has its own value and range. Several gauges split the frame horizontally.

```rust
Plot::new("load")
    .legend(LegendPosition::Hidden)
    .series(Series::gauge("CPU", 63.0, 0.0..=100.0))
    .series(Series::gauge("Memory", 81.0, 0.0..=100.0))
    .series(Series::gauge("Disk", 47.0, 0.0..=100.0))
    .show(ui, &mut icons, &mut state);
```

`GaugeConfig` controls the arc (`start_angle`, `end_angle`, `thickness`), the value text,
and the range labels. The value arc uses the series color; the needle uses the theme text
color.

## Waterfall

Each `[x, change]` bar starts where the previous one ended. Positive changes use the
series color, negative changes the danger color, and dashed connectors show the running
total.

```rust
Plot::new("cash")
    .x_label("Month")
    .y_label("Change")
    .series(Series::waterfall("cash", vec![
        [0.0, 42.0], [1.0, -11.0], [2.0, 18.0], [3.0, -25.0],
    ]))
    .show(ui, &mut icons, &mut state);
```

`WaterfallConfig` sets `bar_width` (80 % of the smallest x spacing by default) and the
`connectors`.

## Export and hover

Every family renders through the same scene, so `to_svg`, `to_png`, the toolbar export
button, the legend, and the hover animation work unchanged. Hover popups report the point
data for cartesian families; pie, radar, and gauge have no x/y hover point.
