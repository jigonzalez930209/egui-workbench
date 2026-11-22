# Live data

> Follow the newest x units, keep a bounded buffer, and repaint only while streaming. Live page: <https://jigonzalez930209.github.io/egui-workbench/charts/live-data.html>

A streaming plot follows the newest samples while the view is automatic.

```rust
struct Live {
    data: Vec<[f64; 2]>,
    elapsed: f64,
}

impl Live {
    fn push(&mut self, value: f64) {
        self.elapsed += 0.1;
        self.data.push([self.elapsed, value]);
        // Keep ten minutes at 10 Hz.
        if self.data.len() > 6000 {
            self.data.drain(..1000);
        }
    }
}
```

## x_window

```rust
Plot::new("live")
    .x_window(10.0)      // show the newest 10 x units
    .include([0.0, 0.0]) // keep the origin in view
    .series(Series::line("sensor", &self.data))
    .show(ui, &mut icons, &mut state);
```

`x_window(width)` only applies while the view is automatic. As soon as the user pans or
zooms, the view is theirs; `PlotState::reset_view()` (double-click or the fit button)
returns to following the data.

## Keep the buffer bounded

- Drain in chunks instead of one point per frame, so the line does not jump every frame.
- Keep x ascending; sorted series enable binary search, decimation, and fast hover.
- Use `f64::NAN` to mark gaps instead of splitting the series.
- If the data is a ring buffer, copy it into a contiguous `Vec` when the order wraps, or
  expose two slices and draw two series.

## Repaint

A worker or a timer must wake the UI explicitly:

```rust
if self.streaming {
    ctx.request_repaint_after(std::time::Duration::from_millis(100));
}
```

Do not call `request_repaint_after` "just in case" when the stream is paused. The plot
does not repaint while idle; the application owns the cadence.

A typical worker sends samples over a channel and wakes the context:

```rust
while let Ok(sample) = receiver.try_recv() {
    self.push(sample);
}
if self.streaming {
    ctx.request_repaint_after(std::time::Duration::from_millis(100));
}
```

## Example: 60 seconds of history

```rust
fn show(&mut self, ui: &mut egui::Ui) {
    let plot = Plot::new("telemetry")
        .title("Telemetry")
        .x_label("t (s)")
        .y_label("value")
        .x_window(60.0)
        .include([0.0, 0.0])
        .series(Series::line("temperature", &self.temperature))
        .series(Series::hline("limit", 80.0));

    let response = plot.show(ui, &mut self.icons, &mut self.plot_state);
    if response.settings_requested {
        self.settings_open = true;
    }
}
```

## Reset and pause

- `PlotState::is_auto()` tells you whether the view still follows the data.
- `reset_view()` re-enables following.
- A "Pause" command should stop both the producer and the repaint request; the plot
  itself has no clock.
- If the user selects a range, keeps the view fixed, or hides a series, respect that
  state; do not reset the view on every frame.

## Performance checklist

- Keep sorted x and a bounded buffer.
- Draw one line per logical signal, not one series per sample.
- Use `x_window` instead of manually trimming the view every frame.
- Request repaint at the stream rate, not at the display refresh rate, unless the data
  actually changes that fast.
