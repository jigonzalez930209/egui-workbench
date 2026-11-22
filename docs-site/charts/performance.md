---
title: Performance
description: Per-pixel decimation, sorted data, million-point series, and what costs a frame.
---

# Performance

The plot is designed for large time series: dense sorted lines are reduced per pixel
column, so a million points draw in a few milliseconds in release builds. Nothing
repaints while idle.

<LiveDemo
  scene="plot-big"
  height="440"
  alt="A line with two million points"
  caption="Up to two million points in your browser. Zoom and pan: decimation keeps the cost proportional to the width."
/>

## Decimation

For a sorted line, each pixel column keeps four points: the first, the minimum, the
maximum, and the last. The drawn polyline preserves the visual envelope of the data
without sending every point to the painter. The cost is proportional to pixels, not
points.

Consequences:

- **Sort x ascending when you can.** Sorted series use binary search for the visible
  range and the four-point reduction.
- **Unsorted lines** (a voltammogram, a Nyquist arc, any loop) keep each point that is
  at least half a pixel from the last one drawn: a 50 000-point loop becomes the few
  thousand points the screen can show.
- **Every line** is then simplified on screen with Ramer–Douglas–Peucker at 0.3 pt: a
  vertex closer than that to the straight path between its neighbors is dropped. A
  smooth curve of thousands of samples tessellates as a few hundred vertices.
- **Unsorted scatter** draws one marker per screen cell three quarters of a radius wide:
  a curve that retraces itself (the cycles of a voltammogram) draws its markers once,
  and a dense band of markers keeps its outline to within 7 % of the radius.
- **Bars are not decimated.** Aggregate or filter before building the series.
- **The hover lookup is cheap**: sorted data binary-searches the visible range, and
  unsorted data searches the points it drew, refined to the exact nearest point.
- **Extents are cached** in the `PlotState`: the automatic view does not walk every
  point each frame. A series is re-measured when its buffer, length, or sampled values
  change.

## What costs a frame

| Work | Cost |
|---|---|
| Line with sorted x | O(pixels) after the range search |
| Line with unsorted x | O(points) screen transforms, O(pixels) vertices |
| Scatter with unsorted x | O(points) screen transforms, O(occupied cells) markers |
| Bars | O(points) rectangles |
| Fit curve | O(samples) after the fit |
| Fit itself | O(points · degree²) |
| Text measurement | Cached by egui; one measurement per label change |
| Repaint | Only on input, drag, animation, or a requested wake |

The fit is the expensive part of an interactive selection: on a million points it can
take a noticeable fraction of a frame. Fit the selection when the drag ends, not on every
pointer move.

## Building large series

- Borrow `&[[f64; 2]]`; the plot never copies it.
- Build the `Vec` once and mutate it in place; do not rebuild it every frame.
- Keep a bounded live buffer (see [Live data](/charts/live-data)).
- Use `Series::function` for analytic curves instead of sampling them into a `Vec` in
  application code.

## Downsampling in the application

If you need a faster scatter or bar chart, aggregate before creating the series:

```rust
let buckets: Vec<[f64; 2]> = raw
    .chunks(100)
    .map(|chunk| {
        let x = chunk.iter().map(|point| point[0]).sum::<f64>() / chunk.len() as f64;
        let y = chunk.iter().map(|point| point[1]).sum::<f64>() / chunk.len() as f64;
        [x, y]
    })
    .collect();
```

Keep the raw data for selection and export; draw the aggregate.

## Release builds

The crate is fast in `dev`, but a million-point series is noticeably smoother in
release. Measure with `cargo run --release --example showcase`. The plot's own work does
not allocate per point; the painter and text layout do allocate, but egui caches text
galleys between frames.

## Idle behavior

- No periodic `request_repaint_after`.
- No repaint while the pointer is still.
- Streaming wakes at the producer's cadence.
- Animations respect reduced motion; reduced motion sets the animation time to zero.
