# Live data panel

> Stream samples into a plot, follow the newest window, select an area, fit it, and export. Live page: <https://jigonzalez930209.github.io/egui-workbench/recipes/live-plot.html>

## State

```rust
use egui_workbench::components::plot::{Fit, FitKind, Plot, PlotState, Series};

struct Panel {
    samples: Vec<[f64; 2]>,
    elapsed: f64,
    streaming: bool,
    plot: PlotState,
    fit: Option<Fit>,
    receiver: Option<std::sync::mpsc::Receiver<f64>>,
    icons: Icons,
}

impl Panel {
    fn push(&mut self, value: f64) {
        self.elapsed += 0.1;
        self.samples.push([self.elapsed, value]);
        if self.samples.len() > 10_000 {
            self.samples.drain(..1_000);
        }
    }
}
```

## Draw

```rust
fn show(&mut self, ui: &mut egui::Ui) {
    let plot = Plot::new("telemetry")
        .title("Telemetry")
        .x_label("t (s)")
        .y_label("value")
        .x_window(60.0)
        .include([0.0, 0.0])
        .series(Series::line("sensor", &self.samples))
        .series(Series::hline("limit", 80.0).hide_from_legend())
        .export_button(true)
        .settings_button(true);

    let response = plot.clone().show(ui, &mut self.icons, &mut self.plot);
    if response.selection_changed
        && let Some(bounds) = self.plot.selection
    {
        let indices = egui_workbench::components::plot::points_in(&self.samples, &bounds);
        if indices.len() >= 2 {
            let subset: Vec<[f64; 2]> = indices.iter().map(|index| self.samples[*index]).collect();
            self.fit = Fit::new(FitKind::Linear, &subset);
        }
    }
    if response.settings_requested {
        self.settings_open = true;
    }
    if let Some(format) = response.export_requested {
        self.export(format, &plot);
    }
}
```

The fit is computed when the selection ends, not on every pointer move; the fit is the
expensive part on large data.

## Receive samples

```rust
fn logic(&mut self, ctx: &egui::Context) {
    if let Some(receiver) = &self.receiver {
        let mut received = false;
        while let Ok(value) = receiver.try_recv() {
            self.push(value);
            received = true;
        }
        if received {
            ctx.request_repaint();
        }
    }
    if self.streaming {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
```

Waking on the stream cadence is enough; do not repaint at the display rate when the
producer is slower.

## Start and stop

```rust
fn toggle_stream(&mut self, ctx: &egui::Context) {
    if self.streaming {
        self.streaming = false;
        self.receiver = None;
        return;
    }
    let (sender, receiver) = std::sync::mpsc::channel();
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        let mut value = 20.0_f64;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            value = (value + 0.2).sin() * 10.0 + 50.0;
            if sender.send(value).is_err() {
                break;
            }
            ctx.request_repaint();
        }
    });
    self.receiver = Some(receiver);
    self.streaming = true;
}
```

For production, use a cancellation flag or a channel that closes when the panel drops; the
`send` error is the signal to stop.

## Selection and fit

- `points_in(&data, &bounds)` returns the indices inside the selected area.
- `Fit::new(FitKind::Linear, &subset)` fits the selection.
- Add `fit.series("fit", 200)` to the plot to draw the dashed curve; its name includes the
  equation and R².
- On double-click, `PlotState::reset_view()` re-enables following the data.

## Export

`response.export_requested` is `Some(Svg)` or `Some(Png)` when the toolbar menu is used.
Build the same `Plot` (keep the clone) and call `to_svg` / `to_png`. See
[Export](../charts/export.md).

## Checklist

- [ ] Sorted x and a bounded buffer.
- [ ] `x_window` only while the view is automatic.
- [ ] Repaint on new samples, not on a permanent timer.
- [ ] Fit on selection end, not on every pointer move.
- [ ] Export from the same `Plot` you render.
