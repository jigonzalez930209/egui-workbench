# Components

`egui_workbench::components` is the shadcn-style layer over the workbench primitives.
The root API still works; this layer adds builders, shared variants, and one vocabulary
for sizes and tones.

```rust
use egui_workbench::components::prelude::*;

if Button::new("Save").variant(ButtonVariant::Primary).show(ui).clicked() {
    app.save();
}
TextField::new(&mut app.name).label("Name").width(240.0).show(ui);
if Switch::new(&mut app.wrap).label("Word wrap").show(ui) {
    app.mark_dirty();
}
```

Run the live catalog:

```bash
cargo run --example catalog
```

## Reactivity contract

egui is immediate mode, so there is no virtual DOM and no hidden component state.

- Interactive components take `&mut` state and mutate it in the same frame.
- Components return the result: `bool` for changed/toggled, `Option<T>` for a choice,
  or `egui::Response` for position, hover, focus, and clicks.
- The application remains the source of truth. A click does not schedule a hidden
  update; the caller applies the event in `App::handle`.
- Paint code never performs I/O. Commands and file operations stay in the application.
- Animations use egui's animation clock and respect reduced motion through the theme.

This is the main difference from shadcn: variants and composition are shared, but state
is passed in every frame instead of living in a component instance.

## Primitives

| Component | Variants / options | State | Returns |
|---|---|---|---|
| `Button` | `Primary`, `Secondary`, `Outline`, `Ghost`, `Destructive`, `Link`; `Size::{Sm,Md,Lg}` | `.enabled`, `.selected`, `.width`, `.tooltip` | `Response` |
| `IconButton` | `IconButtonSize::{Action,Tool,Small}` | `.active`, `.open`, `.enabled` | `Response` |
| `CloseButton` | `.size` | — | `bool` |
| `IconPreview` | `.size`, `.color` | — | `Response` |
| `IconGrid` | `IconSize::{Auto,Fixed}`, `IconLayout::{Grid,List}`, `.prefix`, `.labels`, `.gap` | `.selected` name | `Option<usize>` (clicked index) |
| `NavTab` | `.active`, `.enabled` | — | `bool` |
| `TextField` | `.label`, `.hint`, `.width`, `.enabled` | `&mut String` | `Response` |
| `TextArea` | `.label`, `.hint`, `.rows`, `.width` | `&mut String` | `Response` |
| `Switch` | `.label`, `.enabled` | `&mut bool` | `bool` (changed) |
| `Checkbox` | `.label`, `.enabled` | `&mut bool` | `bool` (changed) |
| `Select<T>` | `.label`, `.width`, typed options | `&mut T` | `bool` (changed) |
| `Stepper` | `.range`, `.step`, `.decimals`, `.suffix`, `.width` | `&mut f32` | `bool` (changed) |
| `SegmentedControl` | `.selected`, `.enabled` | selected index | `Option<usize>` |
| `Badge` | `Tone`, `.dot` | — | `Response` |
| `Banner` | `Tone` or `.color(token)`; spans the full width | actions closure | `()` |
| `Label` | `Size`, `Tone`, `.strong` | — | `Response` |
| `Card` | `.title`, `.description`, `.width` (else fills the width), `.padding`, `.tone` | body closure | body value |
| `Separator` | horizontal / vertical, `.spacing` | — | `()` |
| `StatusBar` | left/right `StatusItem`s | — | `()` |
| `MenuItem` | `.shortcut`, `.danger`, `.enabled` | — | `bool` |
| `MenuToggle` | `.enabled` | on state | `bool` |
| `MenuButton` | — | — | `bool` |
| `Menu` | `.width` | body closure | `()` |
| `MenuIconButton` | `.width` | menu closure | `()` |
| `Popover` | `.width` | body closure | body value |
| `TabStrip` | `.close_tooltip`, `.controls_width` | active index | `Option<TabEvent>` (fixed-height panel) |
| `CommandPalette` | labels + items | `&mut PaletteState` | `Option<T>` |
| `Tree` | `.active`, `.light`, id salt | `&mut TreeState<K>` | `Vec<TreeEvent<K>>` |
| `Dialog` | `.width`, `.close`, `.title_size` | `&mut bool` open | body value |
| `ConfirmDialog` | `.confirm`, `.cancel`, `.danger` | — | `Option<PromptAction>` |
| `NameDialog` | `.description`, `.error` | `&mut String` | `Option<PromptAction>` |
| `ProgressDialog` | `.cancel` | progress value | `bool` (cancelled) |

## Icon grid

`IconGrid` computes the column count from the available width (as many columns as fit
the minimum cell) and then stretches every cell so the row fills the width exactly:
it never overflows the right edge and reflows live as the window resizes. With
`IconSize::Auto` the glyph also scales with the cell (20–56 pt). Only the rows inside
the viewport are laid out, so all 2,410 Lucide icons stay cheap. Glyphs above 16 pt
(or on HiDPI screens) are rasterized at 96/192/384 px through
`Icons::texture_sized`, so large previews stay sharp.

## Customization

Three layers, from global to local:

1. **Colors — `Theme`.** Nine built-in themes, or your own tokens:
   ```rust
   use egui_workbench::{Theme, theme};
   let brand = Theme::dark().with_accent(egui::Color32::from_rgb(0xe0, 0x6c, 0x2b));
   assert!(brand.accessibility_issues().is_empty()); // AA kept by construction
   theme::apply_tokens(ctx, &brand);                  // or ThemeState::set_custom
   ```
   For just a brand color, `ThemeState::set_accent(ctx, Some(color))` applies it to
   whichever built-in theme is effective and re-derives it on every theme change and
   desktop light/dark switch.
   `Theme` fields are public, so `Theme { surface: …, ..Theme::light() }` works too.
   `ThemeState::set_custom(ctx, Some(theme))` installs it next to the built-in ones
   (`None` goes back); `theme::current(ctx)` reads the installed tokens.
2. **Shape — `style::WorkbenchStyle`.** Density (`Compact`, `Default`,
   `Comfortable`), corner radii (controls, items, inner shapes, containers, dialogs,
   pills), border width, and text scale, installed once with `style::set(ctx, …)` and
   kept across theme changes. Presets: `WorkbenchStyle::sharp()`, `::rounded()`,
   `.with_density(…)`. Every component and the chrome text read it (enforced by
   `cargo xtask style-lint`). The default reproduces `metrics` exactly; the window
   chrome (title bar, tab strip, status bar) keeps its fixed heights.
3. **Per instance — builder overrides.** `Button::fill/text_color/stroke/radius`,
   `IconButton::color/radius`, `Card::fill/stroke/radius`, `Badge::radius`,
   `Tone::Custom(color)`, `Banner::color`, and the plot's `PlotConfig`.

Layout: `WorkbenchApp` has an optional resizable **right panel**
(`right_panel_visible`, `right_panel`, `right_panel_width`) and **bottom panel**
(`bottom_panel_visible`, `bottom_panel`, `bottom_panel_height`) besides the explorer,
tab strip, and status bar. Both yield to the content on narrow windows.

## Plots

`components::plot::Plot` is a 2D plotting component in the spirit of matplotlib:

```rust
use egui_workbench::components::plot::{FitKind, Plot, PlotState, Scale, Series};

let response = Plot::new("signal")
    .title("Signal").x_label("t (s)").y_label("V").height(320.0)
    .series(Series::line("signal", &data))                 // borrowed, never copied
    .series(Series::scatter("samples", &samples).y_errors(&errors))
    .series(Series::hline("limit", 4.0))
    .fit(FitKind::Polynomial(3), &samples)                 // dashed curve + equation + R²
    .show(ui, &mut icons, &mut state);                     // PlotState lives in your model
if let Some(hover) = response.hovered { /* series, index, point */ }
if response.selection_changed { let area = state.selection; }
```

| | |
|---|---|
| Series | `line`, `area` (fill), `step`, `scatter` (7 markers), `bars`, `function`, `hline`, `vline`; `.color`, `.width`, `.style(Dashed/Dotted)`, `.marker`, `.y_errors`, `.fill_to` |
| Chart types | Pie/donut (`Series::pie`), candlesticks (`Series::candles`), heatmaps (`Series::heatmap`), radar (`Series::radar`), gauges (`Series::gauge`), waterfalls (`Series::waterfall`) |
| 3D | `Plot3D` + `Series3D::{line, scatter, bars, surface, surface_fn}`: orbit camera, perspective/orthographic, three axes, depth-sorted faces, the same legend and export |
| Legend animation | `PlotConfig::legend_hover`: 13 effects (`Blink`, `DashMarch`, `Wiggle`, `Glow`, `Sweep`, …), `AnimationEasing`, period, second color |
| Direction arrow | `PlotConfig::direction` / `.direction_arrow(true)` / `.direction_points(n)`: the arrow beside the newest sample of line-like series |
| Axes | `Scale::Linear` / `Scale::Log10` per axis; 1-2-5 ticks measured so labels never overlap; one notation per axis; minor log ticks |
| Interaction | drag pans; wheel zooms around the cursor (Shift: x only, Alt: y only; the wheel is consumed so a surrounding scroll area stays put); right-drag or the zoom tool zooms to a box (a thin box zooms one axis); Shift-drag or the select tool selects an area; double-click fits the data; the toolbar has the three tools, zoom in/out, and fit |
| Feedback | hover ring with dashed guides to the axes and a popup with the series, x, and y; the legend toggles series, and a hovered legend row draws its series above the others without reordering the legend |
| Analysis | `Fit::new(FitKind, &data)`: linear, polynomial (1–8), exponential, power, logarithmic, moving average; `eval`, `equation`, `r_squared`, `series`; `points_in(&data, &bounds)` for selections |
| Live data | `.x_window(10.0)` follows the newest 10 x units while the view is automatic |
| Response | `hovered`, `clicked`, `pointer` (data coordinates), `selection_changed`, `view`, `frame`, `screen_pos(point)`, `data_pos(pos)` for annotations |

### Configuration

Everything that is not data lives in a `PlotConfig` you can keep in your model and edit
in a dialog (the starter's Charts section has a complete one):

| Field | What it controls |
|---|---|
| `title`, `x.label`, `y.label` | Texts (the builder's `title`/`x_label`/`y_label` write here) |
| `title_size`, `label_size`, `tick_size`, `legend_size` | Font sizes in points |
| `x` / `y`: `scale`, `min`, `max` | Linear/log axis and fixed limits (`None` = automatic) |
| `x` / `y`: `major_step`, `minor`, `decimals`, `tick_labels` | Major spacing (`None` = 1-2-5), `Minor::{Auto, Off, Divisions(n)}`, label precision |
| `x` / `y`: `grid_major`, `grid_minor` | Grid lines per axis |
| `grid_width`, `grid_major_opacity`, `grid_minor_opacity`, `frame_width`, `tick_length` | Line weights |
| `line_scale`, `marker_scale` | Multipliers for every series |
| `legend`, `legend_frame`, `toolbar`, `hover_popup`, `hover_guides` | Chrome |
| `legend_hover` | `LegendHoverConfig`: animation, easing, period, highlight color |
| `direction` | `DirectionArrowConfig`: enabled, trailing points, length, width, head, color |
| `pie`, `radar`, `gauge`, `heatmap`, `waterfall` | Per-family layout: donut hole, radar labels/rings, gauge arc, heatmap colorbar, waterfall connectors |
| `colors` (`PlotColors`) | Data area, canvas, frame, grid, titles, tick labels, selection (`None` = theme) |
| `series` (`SeriesStyle` by name) | Color, width, line style, marker, marker size, fill opacity |

`PlotConfig::sanitize` clamps values after editing; changed limits or scales show after
`PlotState::reset_view`.

### Export

`plot.to_svg(ctx, &state, &ExportOptions { width, height, scale, transparent })` returns
a standalone SVG; `plot.to_png(...)` rasterizes that SVG with resvg and egui's own fonts,
so the image matches the screen (same view, colors, legend, and selection; no toolbar or
hover). `Plot::export_button(true)` adds a download menu to the toolbar and
`Plot::settings_button(true)` a gear; they surface as `PlotResponse::export_requested`
and `settings_requested`, and the application decides where to save or which dialog
to open. Because `Plot` is `Clone` and borrows its data, build it in one function and
use it for both `show` and the export.

Pixel rules: the frame and all grid, tick, and reference lines sit on pixel centers;
bars snap to physical pixels; margins come from the measured tick labels. Dense sorted
lines are reduced per pixel column (first/min/max/last), so a million points render in
a few milliseconds in release builds; nothing repaints while idle.

## File viewer

`components::viewer::FileViewer` reads the files a data tool usually needs:

```rust
use egui_workbench::components::viewer::FileViewer;

if let Ok(mut viewer) = FileViewer::open("sales.xlsx") {
    viewer.show(ui);
}
```

| Format | Rendering |
|---|---|
| XLSX, XLSM, XLS, XLSB, ODS | One tab per sheet, typed cells, header row |
| CSV / TSV | One sheet, RFC 4180 quoting, typed cells |
| UTF-8 text | Virtualized line-number reader |

`from_bytes(name, bytes)` parses data already in memory (browser downloads, workers);
`text(name, text)` builds a text viewer without parsing. Painting performs no I/O. The
grid and text view only lay out rows inside the viewport. Workbook support uses `calamine`
(MIT). The section above lists the public model (`Workbook`, `Sheet`, `Table`, `Cell`,
`CellKind`, `TextFile`, `ViewerError`).

## Toasts

`Toasts` is a `sonner`-style notification stack. Keep one in the application state
and call `show` once per frame after the panels:

```rust
toasts.success("Saved");
toasts.add(Toast::new(ToastKind::Error, "Could not save").description("Permission denied").action("Retry"));
let id = toasts.loading("Exporting…");          // persistent spinner
toasts.update(id, Toast::new(ToastKind::Success, "Exported")); // resolve in place
for event in toasts.show(ctx, &mut icons) {     // ToastEvent::{Action, Closed, Expired}
    // ...
}
```

- Kinds: `Info`, `Success`, `Warning`, `Error`, `Loading`, colored from the theme
  tokens (`theme::current(ctx)`).
- `ToastPosition` picks the corner; `.offset(x, y)` clears the title or status bar;
  `.max_visible(n)` queues the rest; `.width(w)` shrinks on narrow windows.
- Default timeout 4 s (`.duration`, `.persistent`); hovering any toast pauses every
  timer. Enter/exit animations follow reduced motion. No repaint while the stack is
  empty; while toasts are visible, the frame wakes only for the next timeout.

`Tone` is `Neutral`, `Muted`, `Accent`, `Warning`, `Danger`, or
`Custom(Color32)`. Use `Custom(theme.success)` for success because egui's `Visuals`
does not carry that token.

## Composition

`Card` + `TextField` + `Button` is a form block:

```rust
Card::new()
    .title("Account")
    .description("Shown in the title bar.")
    .width(420.0)
    .show(ui, |ui| {
        TextField::new(&mut app.name).label("Name").show(ui);
        Separator::new().show(ui);
        if Button::new("Save").show(ui).clicked() {
            app.save();
        }
    });
```

Dialogs are context-level composition:

```rust
let mut open = true;
Dialog::new("settings", "Workspace", &mut open).show(ctx, |ui| {
    TextField::new(&mut app.name).label("Name").show(ui);
});
```

`ConfirmDialog` and `NameDialog` wrap the standard prompts. `Dialog` owns the frame,
header, close glyph, backdrop, and Escape handling; the body owns the footer.

## Composite blocks

`components::blocks` re-exports the larger shell blocks: `TitleBar`, `show_title_bar`,
`title_menu`, `SettingsDialog`, `show_settings_dialog`, `settings_row`,
`settings_section`, `settings_switch`, `KeyboardMenuState`, `show_keyboard_menu`,
`open_from_keyboard`, `split_panes`, `banner`, `show_choice_window`, `show_progress`,
`show_popover`, `dialog_frame`, `dialog_header`, `dialog_footer`, `show_empty_state`,
`section_header`, `compact_menu`, `menu_tool_button`, and `with_editor_scrollbars`.
`CommandPalette`, `Tree`, and `TabStrip` have builders in `components` itself.
`TabStrip` paints its whole `Ui::max_rect`, so use a 34 pt `Panel::top` or a sized
container; inside a page, wrap it in `ui.allocate_ui(vec2(width, TAB_HEIGHT), …)`.

## Theming and accessibility

- Colors come from `ui.visuals()` and therefore from the active `Theme`. No component
  defines a literal color.
- Sizes come from `components::Size` or `metrics`; chrome sizes are not duplicated.
- Every custom-painted widget declares an AccessKit role and name. Labels name fields
  with `labelled_by`; icon buttons are named by their tooltip; tree rows are tree items.
- Hover feedback is a fill, not a border. Disabled controls are faded and ignored by
  pointer and keyboard input.
- Reduced motion sets egui's animation time to zero; components do not carry their own
  timers.

## Adding a component

1. Create `src/components/<name>.rs`.
2. Use `Size`, `Tone`, and existing tokens; never a raw color.
3. Decide whether it needs only a `Ui` (`Component`) or the icon cache
   (`IconComponent`). Context-level dialogs expose an inherent `show(ctx, …)`.
4. Interactive controls take `&mut` state and return the change.
5. Declare AccessKit metadata and a tooltip.
6. Add a doc example and a test in `tests/components.rs`; add a catalog section.
7. Re-export it from `src/components/mod.rs` and the prelude.
8. Add it to this table and `WORKBENCH.md` §13.1.

## Migration from the function API

The original functions remain. New code can use builders without rewriting old code.

| Function | Builder |
|---|---|
| `primary_button(ui, label, danger)` | `Button::new(label).variant(Primary or Destructive)` |
| `secondary_button(ui, label)` | `Button::new(label).variant(Secondary)` |
| `text_field(ui, &mut value, hint, width)` | `TextField::new(&mut value).hint(hint).width(width)` |
| `switch(ui, &mut value, label)` | `Switch::new(&mut value).label(label)` |
| `combo(ui, label, &mut value, options)` | `Select::new(label, &mut value, options)` |
| `numeric_stepper(…)` | `Stepper::new(&mut value, range, step)` |
| `action_button` / `tool_button` / `small_button` | `IconButton::new(icon, tooltip).size(…)` |
| `menu_item(…)` | `MenuItem::new(icon, label)` |
| `menu_button(…)` | `MenuButton::new(label)` |
| `menu_toggle(…)` | `MenuToggle::new(label, on)` |
| `show_tab_strip(…)` | `TabStrip::new(tabs, active)` |
| `show_status_bar(…)` | `StatusBar::new().left(…).right(…)` |
| `show_palette(…)` | `CommandPalette::new(state, labels, items)` |

The old functions remain public because the shell and tests use them, and because an
application may prefer direct calls for a one-off. New shared screens should use the
builders so variants stay consistent.
