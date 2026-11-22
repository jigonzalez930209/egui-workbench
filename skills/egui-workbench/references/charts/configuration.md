# Configuration

> PlotConfig field by field, per-series styles, theme colors, sanitizing, and a settings dialog. Live page: <https://jigonzalez930209.github.io/egui-workbench/charts/configuration.html>

Everything that is not data lives in a `PlotConfig` you can keep in your model and edit in
a dialog. The screen and the SVG/PNG export use the same config.

```rust
use egui_workbench::components::plot::{Plot, PlotConfig};

let mut config = PlotConfig::default();
config.title = Some("Signal".to_owned());
config.x.label = Some("t (s)".to_owned());
config.y.label = Some("V".to_owned());
config.legend_size = 12.0;

let plot = Plot::new("signal").config(config.clone()).series(Series::line("sin", &data));
```

`Plot::config(cfg)` replaces the whole appearance. Builder methods such as `.title(...)`
and `.x_scale(...)` edit the config afterwards, so call `.config` first.

## Text and sizes

| Field | Default | Controls |
|---|---|---|
| `title` | `None` | Title above the data area |
| `x.label`, `y.label` | `None` | Axis titles |
| `title_size` | 14.0 | Title points |
| `label_size` | 12.0 | Axis title points |
| `tick_size` | 11.0 | Tick label points |
| `legend_size` | 11.5 | Legend points |

## Axes

Each axis is an `AxisConfig`:

| Field | Default | Controls |
|---|---|---|
| `label` | `None` | Axis title |
| `scale` | `Linear` | `Linear` or `Log10` |
| `min`, `max` | `None` | Fixed limits; `None` is automatic |
| `major_step` | `None` | Major spacing; 1-2-5 when `None` |
| `minor` | `Auto` | `Auto`, `Off`, or `Divisions(n)` |
| `grid_major` | `true` | Grid at major ticks |
| `grid_minor` | `true` | Grid at minor ticks |
| `decimals` | `None` | Tick label decimals |
| `tick_labels` | `true` | Draw labels |

`Plot::x_limits(min, max)` and `y_limits` are shorthands for the limits.

## Lines and chrome

| Field | Default | Controls |
|---|---|---|
| `frame_width` | 1.0 | Frame stroke |
| `grid_width` | 1.0 | Grid stroke |
| `grid_major_opacity` | 0.55 | Major grid alpha |
| `grid_minor_opacity` | 0.22 | Minor grid alpha |
| `tick_length` | 4.0 | Major tick length outside the frame |
| `line_scale` | 1.0 | Multiplies every line width |
| `marker_scale` | 1.0 | Multiplies every marker size |
| `legend` | `TopLeft` | `TopLeft`, `BottomLeft`, `BottomRight`, `Hidden` |
| `legend_frame` | `true` | Legend background |
| `toolbar` | `true` | Tool buttons |
| `hover_guides` | `true` | Dashed guides to the axes |
| `hover_popup` | `true` | Hover popup |

## Colors

`PlotColors` holds optional overrides; `None` follows the theme:

| Field | Colors |
|---|---|
| `background` | Data area fill |
| `canvas` | Area around the data frame (transparent on screen by default) |
| `frame` | Frame and tick marks |
| `grid` | Grid lines (before opacity) |
| `text` | Title and axis titles |
| `tick_text` | Tick labels |
| `selection` | Selection fill and outline |

```rust
config.colors.background = Some(theme.sunken);
config.colors.grid = Some(theme.border);
config.colors.selection = Some(theme.accent);
```

## Per-series styles

```rust
let style = config.series_style_mut("signal");
style.color = Some(theme.success);
style.width = Some(2.0);
style.line_style = Some(LineStyle::Dashed);
style.marker = Some(Marker::Ring);
style.marker_size = Some(4.0);
style.fill_opacity = Some(0.2);
```

`series_style(name)` reads without creating; `series_style_mut(name)` creates an empty
override when missing. Styles are matched by series name.

## Sanitize

Call `config.sanitize()` after editing in a dialog:

- font sizes, widths, opacities, and lengths are clamped;
- an inverted limit pair drops `max`;
- log axes drop non-positive limits;
- `major_step` must be finite and positive;
- `Minor::Divisions` is clamped to 2–20;
- decimals are capped at 10;
- series widths, marker sizes, and fill opacities are clamped.

If limits or scales changed, call `PlotState::reset_view()`.

## Settings dialog pattern

The template's Charts section keeps a `PlotConfig` per tab, edits a copy in a dialog, and
applies it on close:

```rust
if self.chart_settings_open {
    let mut open = self.chart_settings_open;
    let mut config = self.chart_config.clone();
    Dialog::new("chart-settings", "Chart settings", &mut open).width(520.0).show(ctx, |ui| {
        settings_section(ui, "Text", "Titles and sizes.");
        settings_row(ui, "Title", "Shown above the data area.", |ui| {
            TextField::new(config.title.get_or_insert_default()).width(ui.available_width()).show(ui);
        });
        settings_row(ui, "X label", "Axis title.", |ui| {
            TextField::new(config.x.label.get_or_insert_default()).width(ui.available_width()).show(ui);
        });
        // ... axis, grid, colors, series
    });
    if !open {
        config.sanitize();
        self.chart_config = config;
        self.plot_state.reset_view();
    }
    self.chart_settings_open = open;
}
```

The starter has the complete dialog with tabs for text, axes, grid, colors, and each
series. See `templates/app/src/chart_settings/`.

## Export uses the same config

`to_svg` and `to_png` take the `PlotConfig` inside the `Plot` and the `PlotState` view.
Build the plot once:

```rust
fn build_plot<'a>(data: &'a [[f64; 2]], config: &PlotConfig) -> Plot<'a> {
    Plot::new("report")
        .config(config.clone())
        .series(Series::line("signal", data))
}

let plot = build_plot(&self.data, &self.config);
let response = plot.clone().show(ui, &mut icons, &mut state);
```

Then export `plot.to_svg(ctx, &state, &options)`. See [Export](../charts/export.md).
