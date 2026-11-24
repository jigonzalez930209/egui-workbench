---
title: Testing
description: egui_kittest, AccessKit queries, snapshots, the harness clock, and end-to-end runs under Xvfb.
---

# Testing

The crate is tested at three levels: unit tests in `src/**`, component and runtime tests
in `tests/`, and visual snapshots. All of them run through `egui_kittest` 0.37, the same
egui version as the library.

## Commands

```bash
cargo test
cargo xtask check            # fmt --check, style-lint, clippy -D warnings, docs, tests
cargo xtask template-check   # generate the starter and compile/test it
UPDATE_SNAPSHOTS=1 cargo test --test snapshots
UPDATE_SNAPSHOTS=1 cargo test --test snapshots_components
UPDATE_SNAPSHOTS=1 cargo test --test snapshots_plot
UPDATE_SNAPSHOTS=1 cargo test --test doc_scenes   # first frames of the docs' live demos
```

Review every changed PNG before committing. A snapshot update without an intended visual
change is a bug in the test.

## What the suite covers

| File | Proves |
|---|---|
| `src/**` unit tests | Contrast for all nine themes; palette filtering; tree flattening; path ellipsis; every bundled glyph rasterizes; document index math; keymap matching; theme state and brand accent; close guard; style scaling and text scale |
| `tests/ime_hidpi.rs` | IME pre-edit/commit composition and a 2× pixels-per-point frame, on all three desktops |
| `tests/serde_roundtrip.rs` | Every built-in state and event round-trips through JSON |
| `tests/keybindings_editor.rs` | The registry editor records a chord and applies it |
| `tests/app_runtime.rs` | Panel order, command collection order, hidden panels, right/bottom panel sizing |
| `tests/components.rs` | Every builder variant, disabled state, labels, typing, switch/checkbox, select/stepper, dialog Escape |
| `tests/theme_window.rs` | `apply_id` sets every visual and style; reduced motion; follow system; native options; resize edges; window actions |
| `tests/title_bar.rs` | Close/maximize/restore/minimize; compact pitch; double-click maximize; drag; menus; toggles; pill |
| `tests/controls.rs` | Icon button sizes; disabled state; overflow menu; segmented selection; switch; stepper clamp; text field; combo; dialog geometry |
| `tests/tabs.rs` | Select; close by button and middle click; widths; controls area; scrolling; dirty close |
| `tests/tree.rs` | Header; folder toggle; file open; row geometry; keyboard navigation; selection follows the active document; context menus |
| `tests/dialogs.rs` | Name prompt focus and validation; confirm; choice window; progress; banners; popover; settings rail and switches |
| `tests/palette_menu.rs` | Rows, filtering, Enter/click, no matches, 12-row cap; F10/Alt menu navigation |
| `tests/layout_icons.rs` | Status bar order; empty state; split columns/rows; editor scrollbars; custom glyphs; Material icons |
| `tests/plot.rs`, `tests/plot_export.rs`, `tests/plot_charts.rs` | Plot interaction, fits, sanitize, export; new chart families, direction arrow, and hover animation of every legend effect |
| `tests/viewer.rs` | XLSX sheets and cells, CSV/TSV, text line splitting, binary rejection, and rendering each viewer kind |
| `tests/doc_scenes.rs` | Every docs scene renders at 360 and 1100 pt without panicking (with egui_kittest's accessibility check); every `<LiveDemo>` names a known scene, look, and theme; every embedded demo's first frame equals its reference in `tests/snapshots/doc-scenes/` |
| `tests/snapshots*.rs` | PNG references: the shell, settings, palette, menus; the component board in every base style plus compact + sharp, comfortable + rounded, large text + borderless, and a custom accent; toasts; plots |

## The harness

```rust
use egui_kittest::Harness;

fn harness<'a>() -> Harness<'a, State> {
    Harness::builder()
        .with_size(egui::vec2(760.0, 560.0))
        .with_max_steps(64)
        .with_context_setup(|ctx| {
            egui_workbench::theme::apply_id(ctx, egui_workbench::ThemeId::GithubLight);
        })
        .build_ui_state(draw, State::default())
}
```

| Settings | Use |
|---|---|
| `with_size` | Fixed viewport for geometry and snapshots |
| `with_max_steps(64)` | Animations need more than the default 4 steps |
| `with_step_dt(1.0 / 60.0)` | Double-click tests; the default step is ~0.75 s, longer than egui's 0.3 s double-click window |
| `with_context_setup` | Install a theme, style, or fonts before the first frame |
| `build_ui_state` | A `Ui` harness with state |
| `build_state` | A context harness without a root `Ui` |

## Finding widgets

Query by AccessKit role and name, not by position:

```rust
use egui_kittest::kittest::Queryable;

harness.get_by_label("Save").click();
harness.get_by_role(egui::Role::Tab).click();
```

This is why every custom-painted widget must declare a role and name. A missing name fails
the query and the test.

## Assertions

- Assert the returned event (`bool`, `Option<T>`, `Vec<TreeEvent<K>>`).
- Assert geometry (`response.rect`) for chrome.
- Assert accessibility state (selected, disabled, expanded).
- Assert command order from `show_workbench`.

## Snapshots

```rust
let mut harness = board(ThemeId::GithubLight);
harness.run();
harness.snapshot("components-github-light");
```

The snapshot is written to `tests/snapshots/<name>.png`. Default harness snapshots render
the whole viewport. Review both light and dark references; the flagship themes are GitHub
Light, GitHub Dark, and Dracula. The component board also covers compact + sharp,
comfortable + rounded, and a custom accent.

## End-to-end under Xvfb

For the real binary, use Xvfb and XTest (the MD View repository has `scripts/e2e/x11.py`):

- Start with `env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 DISPLAY=:N`, or winit picks
  Wayland and opens the window on the real desktop.
- Use a temporary `XDG_RUNTIME_DIR` and config dir, so the test never touches user
  settings or forwards to a running instance.
- Focus the window before sending keys.
- Drag slowly: move to the start, press, wait ~200 ms, then move in steps. If press and
  move arrive in one frame, egui hit-tests the moved position and the wrong widget takes
  the drag.
- Kill only the PIDs you started.

## Visual parity

Compare captures with the reference product pixel by pixel at the same window size. The
title-bar and tab-strip button centers must match the compact pitch in `metrics`. A
one-pixel shift in the chrome is worth investigating; a one-pixel shift inside content is
usually font rasterization.

## Docs

```bash
cargo doc --workspace --no-deps --open
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

`cargo xtask check` builds the docs with warnings denied. The VitePress site is separate:

```bash
cd docs-site
npm install
npm run build
npm run build:demo    # wasm catalog for /guide/live-demo
```

The web crate has its own wasm check:

```bash
cargo clippy --manifest-path web/Cargo.toml --target wasm32-unknown-unknown -- -D warnings
```
