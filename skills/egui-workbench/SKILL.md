---
name: egui-workbench
description: Build native desktop applications in Rust with the egui-workbench crate (an IDE-style shell for egui/eframe 0.37) - custom title bar and menus, document tabs, explorer tree, command palette, keymap, settings, dialogs, toasts, nine themes, interactive plots with fits and SVG/PNG export, and shadcn-style components. Use this skill whenever the user wants a Rust desktop GUI, an egui or eframe app, a tool or editor or dashboard with a native window, or mentions egui-workbench, egui_workbench, WorkbenchApp, show_workbench, or the workbench components and plots, even if they only say "make me a desktop app in Rust".
license: MIT OR Apache-2.0
compatibility: Rust 1.95+ (edition 2024) with cargo; Linux needs the usual winit/OpenGL development packages.
metadata:
  crate: egui-workbench
  version: "0.1"
  repository: https://github.com/jigonzalez930209/egui-workbench
  docs: https://jigonzalez930209.github.io/egui-workbench/
---

# egui-workbench

`egui-workbench` is the shell of a desktop application built on egui 0.37: undecorated
window with its own title bar, menus, tab strip, explorer, status bar, command palette,
keymap, close guard, settings dialog, toasts, plots, and a component set with shared sizes
and tones. It owns presentation only. Your application owns state, files, and commands.

## Set up a project

Add the crate and use its re-exports so the project has exactly one copy of egui:

```toml
[dependencies]
egui-workbench = "0.1"
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
| Charts | `components::plot::{Plot, Series, PlotState, Fit}` | `references/charts/overview.md` |
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
- [`references/guide/quick-start.md`](references/guide/quick-start.md): Generate a complete application from templates/app and learn what each generated file is for.

**Application runtime**

- [`references/app/commands-and-keymap.md`](references/app/commands-and-keymap.md): One Command enum, one handler, ordered shortcuts, the palette, and the keyboard menu.
- [`references/app/documents.md`](references/app/documents.md): Documents<T>, the Document trait, dirty tabs, CloseGuard, and the Save / Discard / Cancel prompt.
- [`references/app/i18n.md`](references/app/i18n.md): Translate at the call site, keep ellipses, and lay out for longer text.
- [`references/app/settings-persistence.md`](references/app/settings-persistence.md): Persist theme, motion, density, corners, text size, and brand accent atomically in the platform config directory.
- [`references/app/window.md`](references/app/window.md): Undecorated native options, resize edges, window actions, and title synchronization.
- [`references/app/workbench.md`](references/app/workbench.md): WorkbenchApp, the fixed panel order, optional right and bottom panels, responsive widths, and the command flow.

**Components**

- [`references/components/index.md`](references/components/index.md): The builder layer, shared variants and sizes, the reactivity contract, and the complete component index.
- [`references/components/buttons.md`](references/components/buttons.md): Button, IconButton, and CloseButton: variants, sizes, disabled, selected, and per-instance overrides.
- [`references/components/display.md`](references/components/display.md): Labels, badges, banners, cards, separators, status bars, and icon previews.
- [`references/components/feedback.md`](references/components/feedback.md): Sonner-style toasts with kinds, positions, actions, loading-to-success updates, and timers.
- [`references/components/icons.md`](references/components/icons.md): The icon cache, Lucide names, custom SVGs, Material file icons, and the responsive icon grid.
- [`references/components/inputs.md`](references/components/inputs.md): Text fields, text areas, switches, checkboxes, selects, steppers, and segmented controls.
- [`references/components/layout.md`](references/components/layout.md): Split panes, empty states, cards, status bars, and the fully composed settings and shell blocks.
- [`references/components/menus.md`](references/components/menus.md): Text menus, menu rows, toggles, icon menu buttons, and navigator tabs.
- [`references/components/navigation.md`](references/components/navigation.md): Tab strips, the explorer tree, and the command palette.
- [`references/components/overlays.md`](references/components/overlays.md): Dialogs, confirm and name prompts, progress windows, and anchored popovers.

**Charts**

- [`references/charts/overview.md`](references/charts/overview.md): An interactive 2D plot component: series, axes, interaction, fits, live data, and export.
- [`references/charts/axes-and-ticks.md`](references/charts/axes-and-ticks.md): Linear and log scales, automatic and fixed limits, major and minor ticks, grid, and label precision.
- [`references/charts/configuration.md`](references/charts/configuration.md): PlotConfig field by field, per-series styles, theme colors, sanitizing, and a settings dialog.
- [`references/charts/export.md`](references/charts/export.md): Export the plot to SVG or PNG from the same scene the screen draws, including sizes, transparency, and the toolbar menu.
- [`references/charts/fits.md`](references/charts/fits.md): Least-squares models, equations, R², moving averages, and selecting points inside a plot area.
- [`references/charts/interaction.md`](references/charts/interaction.md): Pan, wheel zoom, box zoom, area selection, hover popups, legend toggles, tools, and annotations.
- [`references/charts/live-data.md`](references/charts/live-data.md): Follow the newest x units, keep a bounded buffer, and repaint only while streaming.
- [`references/charts/performance.md`](references/charts/performance.md): Per-pixel decimation, sorted data, million-point series, and what costs a frame.
- [`references/charts/series.md`](references/charts/series.md): Lines, areas, steps, scatter, bars, functions, reference lines, error bars, and markers.

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
- [`references/reference/design-rules.md`](references/reference/design-rules.md): The principles, interaction rules, accessibility contract, and new-screen checklist.
- [`references/reference/faq.md`](references/reference/faq.md): Common errors and their fixes: egui versions, fonts, wrapping, repaint, snapshots, and docs.
- [`references/reference/testing.md`](references/reference/testing.md): egui_kittest, AccessKit queries, snapshots, the harness clock, and end-to-end runs under Xvfb.

<!-- references:end -->
