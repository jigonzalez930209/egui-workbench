# __APP_TITLE__

A generic desktop workbench built on [`egui-workbench`](__WORKBENCH_PATH__).

```bash
cargo run
cargo test
cargo clippy --all-targets -- -D warnings
```

## What is already here

- Undecorated window, title bar, text menus, tabs, navigator, status bar, and settings.
- One `Command` enum driving menus, the palette, and shortcuts.
- `Documents<Tab>` with an active tab, dirty markers, and Save / Discard / Cancel
  prompts for both the window and every dirty tab.
- Appearance persistence with atomic writes in the platform config directory.
- Nine themes, follow-system, reduced motion, and a neutral `assets/logo.svg` mark.
- 2,410 Lucide icons (ISC), used by the gallery and available by name to every button
  and menu.
- A tabbed `src/gallery.rs`: the left navigator lists fixed examples, the editor tabs
  are closable, and the Icons tab explores all 2,410 icons with search, sizes, and
  grid/list layouts. Replace that file with the product screen.
- Six unit tests. No file I/O, format, renderer, or editor assumption.

## The three seams

| File | Replace |
|---|---|
| `src/model.rs` | `TabData`: the state one tab owns. |
| `src/content.rs` | The placeholder screen in `show`. |
| `src/ui.rs` | The navigator list, menus, and status cells. |

`src/app.rs` is the runtime: state, `Command` handling, close guard, window title. Replace
`App::save_tab` with the product's persistence; the stub only clears the dirty flag.

## Where things live

| File | Responsibility |
|---|---|
| `src/main.rs` | Window, renderer, and `run_native`. |
| `src/app.rs` | `App` state, `Command` handling, tabs, close guard, persistence seam. |
| `src/model.rs` | `Tab` and `TabData`; the per-tab domain model. |
| `src/content.rs` | The active tab's screen; the main product seam. |
| `src/gallery.rs` | Demo-only: every component with live state. Delete when replacing the screen. |
| `src/commands.rs` | The `Command` enum, shortcut table, and palette rows. |
| `src/ui.rs` | Workbench panels and overlays; returns commands only. |
| `src/settings.rs` | Appearance preferences in the platform config directory. |
| `assets/logo.svg` | 24×24 product mark, registered as the workbench `logo` glyph. |

## Rules

- Menus, the palette, and the keymap produce the same `Command`; add an action to all
  three.
- Draw panels through `show_workbench`; do not change the order in `WORKBENCH.md` §3.
- Never hard-code a color or a chrome size: use `egui_workbench::theme` and
  `egui_workbench::metrics`.
- Widgets return events; only `App::handle` mutates state.
- Keep files under 500 lines and translation at the call site.
- The starter enables `lucide-icons` only. Enable `egui-workbench`'s default features
  when the app also needs the Material file and folder icons.
- Keep the gallery out of product code: replace `content.rs` and remove `gallery.rs`.
