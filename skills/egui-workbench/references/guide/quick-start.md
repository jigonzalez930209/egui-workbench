# Quick start

> Generate a complete application from templates/app and learn what each generated file is for. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/quick-start.html>

## Generate the application

```bash
git clone https://github.com/jigonzalez930209/egui-workbench
cd egui-workbench
cargo xtask new ../my-app --name my-app --title "My App"
cd ../my-app
cargo run
```

`xtask new` copies `templates/app`, replaces `__APP_NAME__`, `__APP_TITLE__`, and
`__WORKBENCH_PATH__`, and copies both licenses. The result is a standalone Cargo
workspace that compiles without changes.

```text
my-app/
├── Cargo.toml              # dependency and lints
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── src/
│   ├── main.rs             # window and run_native
│   ├── app.rs              # App state, handle(Command), close guard, save_tab
│   ├── appearance.rs       # density, corners, text size, brand accent
│   ├── commands.rs         # Command, keymap, palette rows
│   ├── model.rs            # Tab + TabData, the per-tab model
│   ├── content.rs          # the product screen seam
│   ├── ui.rs               # WorkbenchApp panels and overlays
│   ├── panels.rs           # demo inspector and output panels
│   ├── settings.rs         # persisted appearance preferences
│   ├── gallery.rs          # demo-only component gallery
│   ├── charts.rs           # demo-only charts
│   ├── chart_settings/     # demo-only plot configuration UI
│   └── icon_browser.rs     # demo-only icon browser
└── assets/
    ├── logo.svg            # product mark
    ├── icons/              # app icon sizes (when you add them)
    └── ui-icons/           # extra 24×24 monochrome glyphs
```

## What runs

- An undecorated window with a custom title bar, File/View/Help menus, and window
  controls.
- A navigator listing eight example sections; each opens a closable tab.
- An editor column with tabs, a placeholder gallery, and a status bar.
- Settings with **Appearance** (theme, follow system, reduced motion) and **Shape**
  (density, corners, text size, brand accent).
- A command palette, keyboard menu, demo dialogs, toasts, and Save / Discard / Cancel
  prompts for both the window and dirty tabs.
- An optional inspector (right) and output panel (bottom), toggled from View.
- Appearance preferences stored atomically in the platform config directory.

## Replace the demo

The starter is deliberately domain-neutral. Three seams matter:

| File | Replace |
|---|---|
| `src/model.rs` | `TabData`: the state one tab owns. |
| `src/content.rs` | The placeholder screen: call your product UI here. |
| `App::save_tab` in `src/app.rs` | Persistence; the stub only clears the dirty flag. |

Delete `gallery.rs`, `charts.rs`, `chart_settings/`, and `icon_browser.rs` when they are
no longer useful, and adapt `DemoState` in `model.rs`.

> **Keep the shell**
>
> `ui.rs` already implements `WorkbenchApp`. You can replace everything inside the
> `central` method without touching the title bar, tabs, palette, settings, or close
> guard.

## Add a command

Every feature starts as a command:

1. Add a variant to `Command` in `src/commands.rs`.
2. Add a shortcut to `keymap()` (longer chords first).
3. Add a row to `palette_items()`.
4. Add a menu item in `src/ui.rs` under `title_bar`.
5. Add an arm to `App::handle` in `src/app.rs`.

The same command is then reachable by pointer, keyboard, and search.

## Run the checks

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

From the workbench repository, `cargo xtask check` runs the same gate for the library
and `cargo xtask template-check` regenerates this application and compiles it.

Next: [Build your first app](../guide/first-app.md) explains the runtime by building it
without the template.
