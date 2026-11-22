# Performance

> Per-pixel decimation, sorted data, million-point series, and what costs a frame. Live page: <https://jigonzalez930209.github.io/egui-workbench/charts/performance.html>

The plot is designed for large time series: dense sorted lines are reduced per pixel
column, so a million points draw in a few milliseconds in release builds. Nothing
repaints while idle.

## Decimation

For a sorted line, each pixel column keeps four points: the first, the minimum, the
maximum, and the last. The drawn polyline preserves the visual envelope of the data
without sending every point to the painter. The cost is proportional to pixels, not
points.

Consequences:

- **Sort x ascending.** Sorted series use binary search for the visible range and the
  four-point reduction. Unsorted series are drawn point by point.
- **Scatter is not decimated.** Every marker is a shape; for a million points, use a line
  or aggregate first.
- **Bars are not decimated.** Aggregate or filter before building the series.
- **The hover lookup is cheap for sorted data**; it binary-searches the visible range and
  checks a small neighborhood.

## What costs a frame

| Work | Cost |
|---|---|
| Line with sorted x | O(pixels) after the range search |
| Line with unsorted x | O(points) |
| Scatter | O(points) markers |
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
- Keep a bounded live buffer (see [Live data](../charts/live-data.md)).
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
