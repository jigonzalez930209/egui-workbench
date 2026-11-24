---
name: egui-workbench
description: Build native desktop applications in Rust with the egui-workbench crate (an IDE-style shell for egui/eframe 0.37) - custom title bar and menus, document tabs, explorer tree, command palette, keymap, settings, dialogs, toasts, nine themes, interactive 2D and 3D plots with fits, six 2D chart families, orbit-camera surfaces and 3D bars, legend hover animations that bring the hovered series to the front, a streaming direction arrow, SVG/PNG export, a native Excel/CSV/TSV/text viewer, and shadcn-style components. Use this skill whenever the user wants a Rust desktop GUI, an egui or eframe app, a tool or editor or dashboard with a native window, or mentions egui-workbench, egui_workbench, WorkbenchApp, show_workbench, or the workbench components and plots, even if they only say "make me a desktop app in Rust".
license: MIT OR Apache-2.0
compatibility: Rust 1.95+ (edition 2024) with cargo; Linux needs the usual winit/OpenGL development packages.
metadata:
  crate: egui-workbench
  version: "0.3"
  repository: https://github.com/jigonzalez930209/egui-workbench
  docs: https://jigonzalez930209.github.io/egui-workbench/
---

# egui-workbench

`egui-workbench` is the shell of a desktop application built on egui 0.37: undecorated
window with its own title bar, menus, tab strip, explorer, status bar, command palette,
keymap, close guard, settings dialog, toasts, plots (including pie, candles, heatmaps,
radar, gauges, and waterfalls, with legend hover animations), 3D plots with an orbit
camera and surfaces, a file viewer, and a component set with shared sizes and tones. It
owns presentation only. Your application owns state, files, and commands.

## Set up a project

Add the crate and use its re-exports so the project has exactly one copy of egui:

```toml
[dependencies]
egui-workbench = "0.3"
```

```rust
use egui_workbench::{eframe, egui};   // never add a different egui/eframe minor version
use egui_workbench::components::prelude::*;
```

If `Cargo.toml` already lists `egui` or `eframe`, they must be `0.37`; another minor
compiles but `egui::Ui` from one copy is not the other's type, which produces confusing
"expected `Ui`, found `Ui`" errors.

Features: `lucide-icons` (default, ~2,400 icons by name) and `material-icons` (default,
file and folder icons, ~5 MB). Use `default-features = false, features = ["lucide-icons"]`
when the app never shows files.

For a full application with settings persistence and demos, the repository's generator
is faster than writing the shell by hand:

```bash
git clone https://github.com/jigonzalez930209/egui-workbench
cd egui-workbench && cargo xtask new ../my-app --title "My App"
```

Then replace `src/model.rs` (per-tab state), `src/content.rs` (the product screen), and
`App::save_tab` in `src/app.rs`. See `references/guide/quick-start.md`.

## The mental model

egui is immediate mode, and this crate keeps that honest:

- **Caller-owned state.** Components take `&mut` values and return the change in the
  same frame: `bool` (changed or clicked), `Option<T>` (a choice), `Option<TabEvent>`,
  `Vec<TreeEvent<K>>`, or an `egui::Response`. There is no hidden component state to sync.
- **One `Command` enum, one handler.** Menus, the palette, shortcuts, toolbar buttons,
  and context menus all produce the same `Command` value; `App::handle` is the only place
  that changes the model. Every action is then reachable by pointer, keyboard, and search.
- **`WorkbenchApp` + `show_workbench`.** Implement the trait's panel methods
  (`title_bar`, `sidebar`, `tab_strip`, `central`, `status_bar`, `right_panel`,
  `bottom_panel`, `overlays`); each returns `Vec<Command>`. `show_workbench(ui, self)`
  lays the panels out in the fixed order and returns the commands for you to handle.
- **`App::logic` before `App::ui`.** Sync the theme, ask the `CloseGuard` whether to
  prompt, and read shortcuts from the `Keymap` in `logic`; draw in `ui`.

## Minimal application

This compiles against the published crate and is the smallest useful shell. Grow it with
tabs (`Documents<T>` + `TabStrip`), a palette (`CommandPalette`), and a close prompt
(`CloseGuard` + `show_choice_window`); the complete version is in
`references/guide/first-app.md`.

<!-- compile -->
```rust
use egui_workbench::app::{Keymap, ThemeState, WorkbenchApp, show_workbench};
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, Key, Modifiers};
use egui_workbench::{
    Icons, StatusItem, ThemeId, TitleBar, WindowAction, apply_window_action, eframe, is_maximized,
    show_title_bar, title_menu,
};

#[derive(Clone, Copy, Debug)]
enum Command {
    ToggleSidebar,
    Quit,
}

struct App {
    icons: Icons,
    theme: ThemeState,
    keymap: Keymap<Command>,
    sidebar: bool,
    name: String,
    wrap: bool,
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        Self {
            icons: Icons::new(),
            theme: ThemeState::new(ctx, ThemeId::GithubDark, false),
            keymap: Keymap::new()
                .bind(Modifiers::COMMAND, Key::B, Command::ToggleSidebar)
                .bind(Modifiers::COMMAND, Key::Q, Command::Quit),
            sidebar: true,
            name: String::from("report.md"),
            wrap: true,
        }
    }

    fn handle(&mut self, ctx: &egui::Context, command: Command) {
        match command {
            Command::ToggleSidebar => self.sidebar = !self.sidebar,
            Command::Quit => apply_window_action(ctx, WindowAction::Close),
        }
    }
}

impl WorkbenchApp for App {
    type Command = Command;

    fn sidebar_visible(&self) -> bool {
        self.sidebar
    }

    fn status_bar_visible(&self) -> bool {
        true
    }

    fn title_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut bar = TitleBar::new("My App");
        bar.maximized = is_maximized(ui.ctx());
        let output = show_title_bar(
            ui,
            &mut self.icons,
            &bar,
            |ui, _icons| {
                title_menu(ui, "File", 220.0, |ui| {
                    if MenuButton::new("Quit").show(ui) {
                        commands.push(Command::Quit);
                    }
                });
                title_menu(ui, "View", 220.0, |ui| {
                    if MenuToggle::new("Explorer", self.sidebar).show(ui) {
                        commands.push(Command::ToggleSidebar);
                    }
                });
            },
            |_ui, _icons| {},
        );
        if let Some(action) = output.window {
            apply_window_action(ui.ctx(), action);
        }
        commands
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        ui.add_space(8.0);
        Label::new("EXPLORER").size(Size::Sm).tone(Tone::Muted).show(ui);
        Vec::new()
    }

    fn central(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        Card::new().title("Document").show(ui, |ui| {
            TextField::new(&mut self.name).label("Name").width(240.0).show(ui);
            Switch::new(&mut self.wrap).label("Word wrap").show(ui);
        });
        Vec::new()
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        StatusBar::new().left(StatusItem::new(&self.name)).show(ui);
        Vec::new()
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
        if let Some(command) = self.keymap.command_cloned(ctx) {
            self.handle(ctx, command);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        for command in show_workbench(ui, self) {
            self.handle(&ctx, command);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "My App",
        egui_workbench::native_options("My App"),
        Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx)))),
    )
}
```

## Charts and data

Add series to a `Plot` and keep its `PlotState` in the application model:

```rust
let mut state = PlotState::default();
Plot::new("signal")
    .legend_hover(LegendHoverConfig::default()) // Blink: white/black ↔ series color
    .direction_arrow(true)                      // trailing 5 samples
    .series(Series::line("sensor", &samples))
    .series(Series::pie("share", 38.0))         // or candles, heatmap, radar, gauge, waterfall
    .show(ui, &mut icons, &mut state);
```

Hovering a legend row animates that series and draws it above all other series without
moving the legend rows. Reduced motion disables the animation.

For 3D, keep a `Plot3DState` in the model; the camera starts fitted and then lives there:

```rust
let mut state = Plot3DState::default();
Plot3D::new("terrain")
    .projection(Projection::Orthographic)
    .legend_hover(LegendHoverConfig::default())
    .series(Series3D::surface_fn("terrain", -2.0..=2.0, -2.0..=2.0, [34, 34], |x, y| {
        (x * x + y * y).sin()
    }))
    .series(Series3D::scatter("peaks", &peaks))
    .show(ui, &mut icons, &mut state);
```

Drag orbits, Shift-drag pans, the wheel zooms, and double-click fits. `Series3D` also
has `line` and `bars`; the legend, hover popup (`[x, y, z]`), animations, direction arrow,
and `to_svg`/`to_png` work exactly as in 2D. The next section is the complete 3D contract.

The data viewer loads once and paints a virtualized table or text buffer:

```rust
let mut viewer = FileViewer::open("sales.xlsx")?; // or from_bytes(name, bytes)
viewer.show(ui);                                  // no I/O while painting
```

Runnable examples: `cargo run --example viewer`, `cargo run --example plot3d`,
`cargo run --example scenes -- scene=plot-chart-gallery`,
`cargo run --example scenes -- scene=plot-animations`,
`cargo run --example scenes -- scene=plot-3d-surface`,
`cargo run --example scenes -- scene=plot-3d-bars`, and
`cargo run --example scenes -- scene=plot-3d-helix`.

## 3D charts in depth

`Plot3D` is a separate component, not a mode of `Plot`. It owns a camera, projects
`[x, y, z]` data, depth-sorts faces, and uses the same `Prim` scene pipeline for screen
and SVG/PNG. `Plot3DState` is the caller-owned interaction state; `Plot3D` borrows data
and is `Clone`, like `Plot`.

### API map

| Type | Purpose |
|---|---|
| `Plot3D` | Builder: series, axes, camera, legend, toolbar, export |
| `Plot3DState` | Camera, tool, hidden series, legend position/hover, extent cache |
| `Plot3DConfig` | Text, x/y/z axes, colors, camera, legend, feedback, surface, series overrides |
| `Series3D` | One line, scatter, bar field, or surface data set |
| `Surface` | `x` columns, `y` rows, row-major `z` |
| `Camera3D` | `yaw`, `pitch`, `distance`, `target`, `fov`, `projection` |
| `Projection` | `Perspective` (default) or `Orthographic` |
| `Bounds3D` | Data-space box returned by `Plot3DResponse::view` |
| `Plot3DHover` | `series`, `index`, `point: [f64; 3]` |
| `Plot3DResponse` | Response, hovered/clicked, view, camera, frame, settings/export requests |
| `Plot3DTool` | `Orbit` (default) or `Pan` |

### Constructors and modifiers

| Constructor | Data | Default |
|---|---|---|
| `Series3D::line(name, data)` | `&[[f64; 3]]` | 1.6 pt polyline |
| `Series3D::scatter(name, data)` | `&[[f64; 3]]` | 3 pt circle markers |
| `Series3D::bars(name, data)` | `&[[f64; 3]]` | Cuboids from `z = 0` |
| `Series3D::surface(name, x, y, z)` | three slices/vectors | Filled surface + wireframe |
| `Series3D::surface_fn(name, x_range, y_range, [columns, rows], f)` | closure | Sampled regular grid |

```rust
let surface = Series3D::surface_fn(
    "terrain",
    -2.0..=2.0,
    -2.0..=2.0,
    [34, 34], // columns, rows
    |x, y| (x * x + y * y).sin() * 0.7,
);

let bars: Vec<[f64; 3]> = (0..8)
    .flat_map(|x| (0..8).map(move |y| [f64::from(x), f64::from(y), revenue(x, y)]))
    .collect();

Plot3D::new("3d")
    .x_label("x").y_label("y").z_label("z")
    .series(surface)
    .series(Series3D::bars("revenue", bars).bar_width(0.62).base(0.0))
    .show(ui, &mut icons, &mut state);
```

Modifiers: `.color`, `.width`, `.style(LineStyle)`, `.marker(Marker)`, `.radius`,
`.bar_width`, `.base`, `.hide_from_legend`. Lines break on non-finite points.
`Series3D::len`/`is_empty`/`point(index)`/`data()`/`surface_grid()` inspect the data.

### Surface data contract

- `x` has `nx` column coordinates.
- `y` has `ny` row coordinates.
- `z` has exactly `nx * ny` heights, **row-major**: `z[row * nx + column]`.
- Cell `(column, row)` has corners `(c,r)`, `(c+1,r)`, `(c+1,r+1)`, `(c,r+1)`.
- Cells are filled as `Prim::Quad` meshes (two triangles), because a projected cell can be
  concave. `Surface3DConfig { wireframe, opacity, color_scale }` controls the fill.
- `color_scale = true` uses the blue → purple → amber height ramp; `false` uses the series
  color.

### Camera lifecycle

1. `Plot3DConfig::{yaw, pitch, fov, projection}` is the initial camera.
2. On the first frame, the target is the center of the visible bounds and distance fits
   the bounding sphere, field of view, and aspect ratio.
3. After the user orbits/pans/zooms, `Plot3DState::camera()` holds the camera and wins.
4. `Plot3DConfig::projection` and `fov` are reapplied every frame, so a settings panel can
   switch Perspective/Orthographic without resetting the camera.
5. `state.set_camera(camera)` places it explicitly; `state.reset_view()` returns to auto;
   `state.is_auto()` reports whether it still follows the data.

```rust
let camera = state.camera().unwrap_or_default();
state.set_camera(Camera3D { yaw: 0.9, pitch: 0.3, ..camera });
state.reset_view();
```

`Camera3D::orbit(delta)`, `zoom(factor)`, and `pan(delta, frame)` are public and useful
for custom controls. Pitch is clamped to ±85°; distance is clamped away from zero.

### Interaction

| Gesture | Action |
|---|---|
| Drag | Orbit (default tool) |
| Shift-drag, right-drag, or `Plot3DTool::Pan` | Pan target |
| Wheel or pinch | Zoom toward/away from target |
| Double-click or fit button | `Plot3DState::reset_view` |
| Toolbar | Orbit/pan, zoom in/out, fit, settings, export |
| Legend row click | Toggle series |
| Legend row drag | Move legend inside the frame |
| Legend row hover | Play the configured effect; draw that series last |

The wheel is consumed so an enclosing scroll area does not scroll. Hover is a nearest
projected point search within 12 pt, ties broken by camera depth.

### Legend, hover, animation, and response

`Plot3DConfig::legend_hover` is the same `LegendHoverConfig` as 2D: all 13 effects,
`AnimationEasing`, period, and highlight color. Reduced motion disables them. The legend
geometry is shared with `Plot`; only the swatches are 3D-specific.

`Plot3DState` stores `hidden: Vec<String>` (`set_hidden`, `is_hidden`), the dragged
legend position (`legend_position`, `set_legend_position`), and the hovered row
(`hovered_series`). Draw order puts the hovered series last without moving rows.

```rust
let response = plot.show(ui, &mut icons, &mut state);
if let Some(hover) = response.hovered {
    let Plot3DHover { series, index, point: [x, y, z] } = hover;
}
if let Some(pos) = response.screen_pos([1.0, 2.0, 3.0]) {
    // annotate at the projected point
}
if response.settings_requested { /* open settings */ }
if let Some(format) = response.export_requested { /* save */ }
```

The hover ring gets a dashed drop line to `view.min_z` when `hover_guides` is on; the
popup shows the x/y/z axis labels and values when `hover_popup` is on.

### Direction arrow and export

`.direction_arrow(true)`, `.direction_points(n)`, or `Plot3DConfig::direction` draw the
screen-space arrow beside the newest line point. Per-series overrides use
`Series3DStyle::direction_arrow`.

`to_svg(ctx, &state, &options)` and `to_png(...)` rebuild the same scene with the current
camera, axes, depth order, surface shading, hidden series, and legend. They deliberately
omit the toolbar, hover ring, popup, and guides.

### Configuration

`Plot3DConfig` groups:

| Group | Fields |
|---|---|
| Text | `title`, `title_size`, `title_color`, `title_align`, `label_size`, `tick_size`, `legend_size` |
| Axes | `x`, `y`, `z` (`AxisConfig`), `grid_width`, `grid_major_opacity`, `grid_minor_opacity`, `tick_length` |
| Frame | `frame_width`, `line_scale`, `marker_scale` |
| Camera | `projection`, `yaw`, `pitch`, `fov` |
| Legend | `legend`, `legend_frame`, `legend_hover` |
| Feedback | `hover_guides`, `hover_popup`, `direction` |
| Surface | `surface` (`wireframe`, `opacity`, `color_scale`) |
| Toolbar | `toolbar` |
| Colors | `colors` (`PlotColors`) |
| Series | `series` (`Series3DStyle`) |

`Series3DStyle` fields: `color`, `width`, `line_style`, `marker`, `marker_size`,
`fill_opacity`, `direction_arrow`. Use `config.series_style_mut("name")`; `sanitize()`
clamps sizes, opacity, pitch, FOV, tick divisions, and limits after editing.

### Builder and state surface

`Plot3D` builders, in addition to `new` and `series`:

```text
config, current_config, series_names, series, title,
x_label, y_label, z_label, x_limits, y_limits, z_limits,
x_scale, y_scale, z_scale, width, height, legend,
legend_hover, legend_animation, direction, direction_arrow,
direction_points, projection, camera_angles, fov, surface,
grid, toolbar, include, settings_button, export_button
```

`Plot3DState`:

```text
tool: Plot3DTool, reset_view(), is_auto(), camera(), set_camera(),
legend_position(), set_legend_position(), is_hidden(), set_hidden(),
hovered_series()
```

`Bounds3D` exposes `from_corners`, `width`, `depth`, `height`, `center`, and the six
public min/max fields. `Surface` exposes public `x`, `y`, and `z` (`Cow<[f64]>`).
`Plot3DResponse::screen_pos([x, y, z])` returns `Option<Pos2>` for annotations.
`Surface3DConfig::default()` is `wireframe: true`, `opacity: 0.92`,
`color_scale: true`; `Camera3D::default()` is perspective, yaw `-0.72`, pitch `0.42`,
FOV 45°.

### Source map

| File | Responsibility |
|---|---|
| `src/components/plot/three/camera.rs` | Camera basis, projection, orbit/zoom/pan |
| `src/components/plot/three/series.rs` | `Series3D`, `Surface`, bounds, fingerprints |
| `src/components/plot/three/axes.rs` | Box, wall grid, ticks, labels, titles |
| `src/components/plot/three/paint.rs` | Lines, markers, cuboids, surface cells |
| `src/components/plot/three/scene.rs` | Auto bounds, draw order, one frame |
| `src/components/plot/three/interact.rs` | Drag, wheel, hover search |
| `src/components/plot/three/widgets.rs` | Toolbar, legend, hover ring/popup |
| `src/components/plot/three/export.rs` | SVG/PNG from the camera scene |
| `src/components/plot/three/config.rs` | `Plot3DConfig`, styles, surface config |
| `src/components/plot/three/types.rs` | State, response, hover, bounds, tools |
| `src/components/plot/three/mod.rs` | Builder, public exports, `show` |
| `src/components/plot/three/builder.rs` | Builder methods |
| `src/components/plot/three/legend.rs` | 3D legend geometry and swatches |
| `src/components/plot/three/layout.rs` | Frame/title margins |

### Rendering details and pitfalls

- Extents are cached by a fingerprint of buffers, sampled values, and scales. Call nothing
  each frame to recompute them; `refresh_extents` is internal.
- Bar faces are filled as `Prim::Quad` meshes, not `convex_polygon`. Faces with a
  projected area below 8 px² are culled. This prevents the long anti-aliasing spikes that
  appear at edge-on angles in perspective and orthographic views. Do not change this back.
- Surface cells are also quad meshes; concave projections remain correct.
- 3D does not support two y axes, fits, area selection, step lines, candlesticks,
  heatmaps, radar, gauges, waterfalls, decimation, or smoothing. Do not try to pass a
  `Series` to `Plot3D` or a `Series3D` to `Plot`.
- A still plot does not repaint; orbit/pan/zoom and legend hover request repaint only
  while active.
- Large surface grids cost one quad per cell plus hover points; reduce `surface_fn`
  samples for interaction-heavy screens.

### Examples and tests

```bash
cargo run --example plot3d
cargo run --example scenes -- scene=plot-3d
cargo run --example scenes -- scene=plot-3d-surface
cargo run --example scenes -- scene=plot-3d-bars
cargo run --example scenes -- scene=plot-3d-helix
cargo test --test plot3d
```

The native example has a title bar, theme toggle, chart/projection selectors, all 13
animations, easing, wireframe, direction arrow, reset, and export. The live scenes are
the practical documentation examples; the PNGs under `tests/snapshots/doc-scenes/` are
only regression references, not user-facing examples.

## Rules that keep an app consistent

These come from the crate's design rules (`references/reference/design-rules.md`).
Following them is what makes the result look and behave like one product in all nine
themes:

1. **Components over raw widgets.** Use `Button`, `IconButton`, `TextField`, `Select`,
   `Switch`, `Stepper`, `SegmentedControl`, `Card`, `Badge`, `Dialog`, `Toasts`… from
   `components::prelude` instead of `ui.button` / `egui::Window`. They carry the sizes,
   tones, accessibility names, and Escape handling.
2. **No literal colors or sizes.** Colors come from the theme (`theme::current(ctx)`
   tokens or `ui.visuals()`); sizes from `metrics` constants, `Size::{Sm, Md, Lg}`, and
   `style::get(ctx)` (`control()`, `border(color)`, `font(13.0)`). A literal `Color32` or
   radius breaks when the user switches themes or density.
3. **Every action is a `Command`.** Add the variant, a shortcut in the keymap (longer
   chords first: `Ctrl+Shift+P` before `Ctrl+P`, because the first match consumes the
   key), a palette row, a menu item, and one arm in `handle`. Skip shortcuts while a modal
   or the palette is open.
4. **Closures cannot re-borrow `self`.** Inside `show_title_bar`, `title_menu`, and
   `TabStrip::show` closures, push into a local `Vec<Command>` and return it, as above.
5. **No idle repaint.** Call `ctx.request_repaint()` only from input, animation, or a
   worker that finished. Never a periodic `request_repaint_after` "just in case".
6. **Accessible names.** Icon-only buttons need a tooltip (it is the name);
   `TextField::label` names the field. `egui_kittest` tests fail on unnamed widgets.
7. **Plain glyphs.** Default fonts lack many symbols (`→`, emoji, `∈`): use an icon or
   plain text in labels.
8. **Close through the guard.** Quit with `WindowAction::Close`, never
   `std::process::exit`, so `CloseGuard::should_prompt` can ask to save dirty documents.

## Where things are

| Need | API | Read |
|---|---|---|
| Window, title bar, menus | `native_options`, `show_title_bar`, `title_menu`, `apply_window_action` | `references/app/window.md` |
| Panel layout | `WorkbenchApp`, `show_workbench`, right and bottom panels | `references/app/workbench.md` |
| Tabs and dirty state | `Documents<T>`, `Document`, `TabStrip`, `CloseGuard` | `references/app/documents.md` |
| Commands and shortcuts | `Keymap<C>`, `CommandPalette`, keyboard menu | `references/app/commands-and-keymap.md` |
| Settings saved to disk | `SettingsDialog`, persistence pattern | `references/app/settings-persistence.md` |
| Forms and controls | `TextField`, `Select`, `Switch`, `Stepper`, `SegmentedControl` | `references/components/inputs.md` |
| Dialogs and popovers | `Dialog`, `ConfirmDialog`, `NameDialog`, `ProgressDialog` | `references/components/overlays.md` |
| Toasts | `Toasts`, `Toast`, `ToastKind` | `references/components/feedback.md` |
| File tree | `Tree`, `TreeNode`, `TreeState`, `TreeEvent` | `references/components/navigation.md` |
| Charts | `components::plot::{Plot, Series, PlotState, Fit, LegendAnimation, PieConfig, RadarConfig}` | `references/charts/overview.md` |
| 3D charts | `components::plot::{Plot3D, Plot3DState, Plot3DConfig, Series3D, Surface, Camera3D, Projection, Bounds3D, Plot3DResponse}` | `references/charts/3d.md` |
| Data viewer | `components::viewer::{FileViewer, Workbook, Table, TextFile}` | `references/components/data-viewer.md` |
| Themes, accent, density | `ThemeState`, `ThemeId`, `WorkbenchStyle`, `Density` | `references/theming/themes.md` |
| Long tasks, live data | worker thread + `request_repaint` | `references/recipes/long-tasks.md` |
| Every public item | module-by-module list | `references/reference/api-map.md` |
| Errors and fixes | two egui copies, missing icons, repaint, Esc | `references/reference/faq.md` |

API docs for exact signatures: <https://docs.rs/egui-workbench>. When a signature in
these notes and the compiler disagree, trust the compiler and docs.rs.

## Verify the result

Build and lint before reporting success; egui mistakes are usually compile errors, and
layout mistakes show up only when the window runs:

```bash
cargo clippy --all-targets -- -D warnings
cargo run
```

For tests without a window, `egui_kittest` drives the UI headlessly and can save
snapshots; see `references/reference/testing.md`.

## Reference pages

Generated from the documentation site by `docs-site/scripts/build-skill.py`; each page
links to its live version with interactive demos. Read only the pages the task needs.

<!-- references:start -->

**Guide**

- [`references/guide/introduction.md`](references/guide/introduction.md): What egui-workbench is, what it deliberately leaves out, and how its layers fit together.
- [`references/guide/architecture.md`](references/guide/architecture.md): The five layers of egui-workbench, the fixed panel order, and the flow of one frame.
- [`references/guide/concepts.md`](references/guide/concepts.md): The state contract, commands, theming tokens, builders versus functions, accessibility, and repaint behavior.
- [`references/guide/first-app.md`](references/guide/first-app.md): Build a small workbench from scratch: window, title bar, tabs, content, status bar, palette, shortcuts, and close guard.
- [`references/guide/installation.md`](references/guide/installation.md): Add egui-workbench to a new or existing Rust application from crates.io or git, choose features, and keep one copy of egui.
- [`references/guide/platforms.md`](references/guide/platforms.md): The Linux, macOS, and Windows CI matrix, the repeatable IME and HiDPI pass, and the egui upgrade policy.
- [`references/guide/python.md`](references/guide/python.md): Install egui-workbench from PyPI, build a native window from a Python script, and feed NumPy, pandas, Polars, and Arrow data into grids and plots.
- [`references/guide/quick-start.md`](references/guide/quick-start.md): Generate a complete application from templates/app and learn what each generated file is for.
- [`references/guide/web.md`](references/guide/web.md): Run the same host spec in a browser with WebAssembly, render a server over WebSocket, and drive it from C, Go, and other languages.

**Application runtime**

- [`references/app/commands-and-keymap.md`](references/app/commands-and-keymap.md): One registry for menus, the palette, shortcuts, contexts, rebinding, and JSON state.
- [`references/app/data.md`](references/app/data.md): Typed datasets, lazy sources, connectors, the query language, DataGrid, and the viewer registry.
- [`references/app/documents.md`](references/app/documents.md): Documents<T>, the Document trait, dirty tabs, CloseGuard, and the Save / Discard / Cancel prompt.
- [`references/app/history-and-session.md`](references/app/history-and-session.md): An undo tree, an append-only command journal, macros, workspace persistence, crash recovery, and the first DevTools slice.
- [`references/app/host.md`](references/app/host.md): Run an application described as a versioned spec, apply patches, and speak the host protocol from any language.
- [`references/app/i18n.md`](references/app/i18n.md): Translate at the call site, keep ellipses, and lay out for longer text.
- [`references/app/linked-views.md`](references/app/linked-views.md): A selection bus shared by plots and tables, dataset-bound plots, plot grids, and zoom history.
- [`references/app/settings-persistence.md`](references/app/settings-persistence.md): Persist theme, motion, density, corners, text size, and brand accent atomically in the platform config directory.
- [`references/app/window.md`](references/app/window.md): Undecorated native options, resize edges, window actions, and title synchronization.
- [`references/app/workbench.md`](references/app/workbench.md): WorkbenchApp, the fixed panel order, optional right and bottom panels, responsive widths, and the command flow.
- [`references/app/workspace.md`](references/app/workspace.md): Serialized editor groups, splits, tabs, named layouts, focus tools, and the activity bar.

**Components**

- [`references/components/index.md`](references/components/index.md): The builder layer, shared variants and sizes, the reactivity contract, and the complete component index.
- [`references/components/buttons.md`](references/components/buttons.md): Button, IconButton, and CloseButton: variants, sizes, disabled, selected, and per-instance overrides.
- [`references/components/data-viewer.md`](references/components/data-viewer.md): Native Excel workbooks with sheet tabs, CSV and TSV files, and virtualized plain-text viewing.
- [`references/components/display.md`](references/components/display.md): Labels, badges, banners, cards, separators, status bars, and icon previews.
- [`references/components/feedback.md`](references/components/feedback.md): Sonner-style toasts with kinds, positions, actions, loading-to-success updates, and timers.
- [`references/components/icons.md`](references/components/icons.md): The icon cache, Lucide names, custom SVGs, Material file icons, and the responsive icon grid.
- [`references/components/inputs.md`](references/components/inputs.md): Text fields, text areas, switches, checkboxes, selects, steppers, and segmented controls.
- [`references/components/layout.md`](references/components/layout.md): Split panes, empty states, cards, status bars, and the fully composed settings and shell blocks.
- [`references/components/menus.md`](references/components/menus.md): Text menus, menu rows, toggles, icon menu buttons, and navigator tabs.
- [`references/components/navigation.md`](references/components/navigation.md): Tab strips, the explorer tree, and the command palette.
- [`references/components/overlays.md`](references/components/overlays.md): Dialogs, confirm and name prompts, progress windows, and anchored popovers.

**Charts**

- [`references/charts/overview.md`](references/charts/overview.md): Interactive 2D and 3D plot components: series, axes, cameras, interaction, fits, live data, and export.
- [`references/charts/3d.md`](references/charts/3d.md): A complete guide to Plot3D: orbit cameras, perspective and orthographic projection, x/y/z axes, 3D lines, scatter, bars, surfaces, legends, hover, animation, export, and the native example.
- [`references/charts/analysis-and-publication.md`](references/charts/analysis-and-publication.md): New chart families, colormaps, annotations, analysis tools, math text, vector PDF, and the 3D GPU layer.
- [`references/charts/animations.md`](references/charts/animations.md): Thirteen hover animations with user-defined easing, plus the direction arrow for streaming data.
- [`references/charts/axes-and-ticks.md`](references/charts/axes-and-ticks.md): Linear and log scales, automatic and fixed limits, major and minor ticks, grid, and label precision.
- [`references/charts/chart-types.md`](references/charts/chart-types.md): Pie and donut slices, candlesticks, heatmaps, radar polygons, gauges, and waterfalls.
- [`references/charts/configuration.md`](references/charts/configuration.md): PlotConfig field by field, per-series styles, theme colors, sanitizing, and a settings dialog.
- [`references/charts/export.md`](references/charts/export.md): Export the plot to SVG or PNG from the same scene the screen draws, including sizes, transparency, and the toolbar menu.
- [`references/charts/fits.md`](references/charts/fits.md): Least-squares models, equations, R², moving averages, and selecting points inside a plot area.
- [`references/charts/interaction.md`](references/charts/interaction.md): Pan, wheel zoom, box zoom, area selection, hover popups, legend toggles, tools, and annotations.
- [`references/charts/live-data.md`](references/charts/live-data.md): Follow the newest x units, keep a bounded buffer, and repaint only while streaming.
- [`references/charts/performance.md`](references/charts/performance.md): Per-pixel decimation, sorted data, million-point series, and what costs a frame.
- [`references/charts/plot-engine.md`](references/charts/plot-engine.md): GPU data layers, level-of-detail pyramids, streaming series, and time, category, and symmetric-log axes.
- [`references/charts/series.md`](references/charts/series.md): Lines, areas, steps, scatter, bars, candlesticks, heatmaps, waterfalls, pie slices, radar polygons, gauges, functions, reference lines, error bars, and markers.

**Theming**

- [`references/theming/custom-themes.md`](references/theming/custom-themes.md): Build tokens from a brand accent, check contrast, install them with ThemeState, and keep them across theme changes.
- [`references/theming/customization.md`](references/theming/customization.md): Theme colors, global shape, per-instance overrides, custom fonts, icons, and layouts.
- [`references/theming/icons.md`](references/theming/icons.md): The Icons and MaterialIcons caches, custom SVGs, Lucide search, and sharp HiDPI textures.
- [`references/theming/metrics.md`](references/theming/metrics.md): Every chrome size constant in egui_workbench::metrics and where it is used.
- [`references/theming/style-and-density.md`](references/theming/style-and-density.md): WorkbenchStyle, density factors, corner presets, border width, text scale, and what the style does not change.
- [`references/theming/themes.md`](references/theming/themes.md): The nine built-in themes, theme identifiers, follow-system behavior, and live theme switching.
- [`references/theming/tokens.md`](references/theming/tokens.md): Every Theme field, what it paints, how tokens map to egui visuals, and the contrast guarantee.

**Recipes**

- [`references/recipes/brand.md`](references/recipes/brand.md): Persist a brand accent, density, corner preset, and text size; the accent follows theme changes on its own.
- [`references/recipes/editor.md`](references/recipes/editor.md): File tree, tabs, an editor, dirty tracking, saving, and close prompts.
- [`references/recipes/icon-browser.md`](references/recipes/icon-browser.md): Search 2,410 Lucide icons, show a responsive virtualized grid, and track the selection.
- [`references/recipes/ide-layout.md`](references/recipes/ide-layout.md): Compose explorer, editor, inspector, output, and status bar with the runtime panels.
- [`references/recipes/live-plot.md`](references/recipes/live-plot.md): Stream samples into a plot, follow the newest window, select an area, fit it, and export.
- [`references/recipes/long-tasks.md`](references/recipes/long-tasks.md): Run work off the UI thread and report progress with loading, success, and error toasts.
- [`references/recipes/responsive.md`](references/recipes/responsive.md): Patterns that survive 640×480: wrapping rows, two-column thresholds, flexible widths, and stacking splits.
- [`references/recipes/settings-dialog.md`](references/recipes/settings-dialog.md): Build a tabbed settings surface with rows, switches, selects, and an appearance tab.

**Reference**

- [`references/reference/api-map.md`](references/reference/api-map.md): The public surface grouped by module, with links to the guide pages.
- [`references/reference/benchmarks.md`](references/reference/benchmarks.md): The criterion suites for plot decimation, a large tree, a one-million-row CSV, and palette filtering, with the recorded 0.3.1 baseline.
- [`references/reference/design-rules.md`](references/reference/design-rules.md): The principles, interaction rules, accessibility contract, and new-screen checklist.
- [`references/reference/faq.md`](references/reference/faq.md): Common errors and their fixes: egui versions, fonts, wrapping, repaint, snapshots, and docs.
- [`references/reference/migration-0.4.md`](references/reference/migration-0.4.md): The command registry, serde, non-exhaustive enums, and the Cargo features added in 0.4.
- [`references/reference/testing.md`](references/reference/testing.md): egui_kittest, AccessKit queries, snapshots, the harness clock, and end-to-end runs under Xvfb.

<!-- references:end -->
