# Legend animation

> Thirteen hover animations with user-defined easing, plus the direction arrow for streaming data. Live page: <https://jigonzalez930209.github.io/egui-workbench/charts/animations.html>

Resting the pointer on a legend row animates its series and brings that series to the
front of the chart. The legend itself never reorders. The effect is off for every other
row, starts at the first hover frame, and respects reduced motion. The same effects
apply to every `Plot3D` series: lines, bars, and surface cells blend, pulse, and glow.

```rust
use egui_workbench::components::plot::{
    AnimationEasing, LegendAnimation, LegendHoverConfig, Plot,
};

Plot::new("signal")
    .legend_hover(LegendHoverConfig {
        enabled: true,
        animation: LegendAnimation::DashMarch,
        easing: AnimationEasing::Sine,
        period: 1.4,
        highlight: None, // white/black blink; theme warning in other effects
    })
    .series(Series::line("signal", &data))
    .show(ui, &mut icons, &mut state);
```

Every field also has a builder shorthand:

```rust
Plot::new("signal")
    .legend_animation(LegendAnimation::Blink)
    .series(Series::line("signal", &data));
```

## Draw order

The hovered series draws last, above every other series: a line over bars, a pie slice over
its neighbors, a radar polygon over the others. The legend rows keep their original order,
so hiding and showing series still targets the same row.

## The settings

| Field | Default | Meaning |
|---|---|---|
| `enabled` | `true` | Play the effect while a legend row is hovered |
| `animation` | `Blink` | One of the animations below |
| `easing` | `Linear` | How the cycle is shaped |
| `period` | `1.2` s | One full cycle; clamped to 0.2–10 s |
| `highlight` | `None` | Extreme/second color for the effects; `Blink` uses white on dark themes and black on light themes when `None` |

`LegendHoverConfig` lives in `PlotConfig::legend_hover`, so the starter's chart settings
dialog edits it in its **Motion** section. `PlotConfig::sanitize` clamps the period.

## The animations

| Animation | What moves |
|---|---|
| `Blink` | On dark themes, white ↔ series color; on light themes, black ↔ series color |
| `Fade` | Pulses the opacity |
| `Throb` | Pulses the stroke width |
| `DashMarch` | Scrolls the dashes along the line (marching ants) |
| `DashToggle` | Grows the dashes in and out of a solid line |
| `Glow` | Draws a soft halo around the line |
| `Sweep` | A bright segment travels along the line |
| `Wiggle` | Moves the line perpendicular to its local direction |
| `Reveal` | Draws the line again and again from its first point |
| `MarkerBounce` | Bounces markers and radar vertices |
| `MarkerPulse` | Grows and shrinks markers |
| `Twinkle` | Puts small sparkles along the line |
| `Wave` | Washes the color from left to right |

The line effects apply to lines, areas, steps, and reference lines; markers apply to
scatter, line vertices, and radar points. Pie slices lift out of the ring, radar polygons
breathe, and gauges pulse their value arc. Heatmaps tint their hovered cells. Waterfall and
candle bars blend toward the highlight color.

## Easing

`AnimationEasing` shapes one cycle:

| Easing | Shape |
|---|---|
| `Linear` | Constant speed (default) |
| `EaseIn` | Slow start |
| `EaseOut` | Slow end |
| `EaseInOut` | Slow at both ends |
| `Step` | Holds, then jumps at the midpoint |
| `Sine` | Smooth back-and-forth with no abrupt turn |

## Reduced motion

`theme::motion_enabled(ctx)` gates every effect. When reduced motion is on, legend rows
still shade on hover but no animation runs; the plot does not request repaints.

## Direction arrow

The direction arrow marks the newest sample of line-like series. It uses the last
`points` finite samples (5 by default) so a noisy stream still shows where the buffer is
growing.

```rust
Plot::new("telemetry")
    .x_window(30.0)
    .direction_arrow(true)
    .direction_points(5)
    .series(Series::line("sensor", &samples))
    .show(ui, &mut icons, &mut state);
```

`DirectionArrowConfig` lives in `PlotConfig::direction`:

| Field | Default | Meaning |
|---|---|---|
| `enabled` | `false` | Draw the arrow beside the newest point |
| `points` | `5` | Trailing samples used to measure direction (2–64) |
| `length` | `26.0` pt | Shaft length |
| `width` | `2.0` pt | Shaft width |
| `head` | `8.0` pt | Head length |
| `color` | `None` | The series color when `None` |

Per-series overrides win over the global switch:

```rust
let mut config = PlotConfig::default();
config.direction.enabled = true;
config.series_style_mut("noise").direction_arrow = Some(false);
```

The arrow applies to `line`, `area`, `step`, and `candles`; reference lines, bars,
scatter, heatmaps, waterfalls, and the non-cartesian families never draw it. It is part of
the scene, so exports include it.
