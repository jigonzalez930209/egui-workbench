# Start a project from this base

## 1. Generate the application

With `cargo-generate` installed, start from the published template without cloning this
repository:

```bash
cargo install cargo-generate
cargo generate --git https://github.com/jigonzalez930209/egui-workbench \
  --subfolder templates/app --name my-app
# prompts for the title; leave the workbench path empty to use crates.io
cd my-app
cargo run
```

From a checkout, the contributor path fills the local dependency automatically:

```bash
git clone https://github.com/jigonzalez930209/egui-workbench
cd egui-workbench
cargo xtask new ../my-app --title "My App"
cd ../my-app
cargo run
```

The generator copies [`templates/app`](../templates/app) and replaces the
`{{project-name}}`, `{{app_title}}`, and `{{workbench_path}}` placeholders, copies the
licenses, and leaves a compiling application. Use `--name` when the directory does not
give a valid crate name. The generated `Cargo.toml` points at this repository by path
when `cargo xtask new` creates it, and at `egui-workbench = "0.3"` otherwise. If you move
the project, update the path or switch to the published crate.

## 2. Rename the product

- `Cargo.toml`: description, version, license, repository.
- `src/main.rs`: the window title and `native_options` title.
- `assets/logo.svg`: the starter ships a neutral 24×24 mark, registered by
  `App::new` as the `logo` glyph. Replace it with the product mark; the workbench draws
  it at 20 pt in the title bar. Full-color SVGs are drawn untinted.
- `README.md` and `src/settings.rs` (the config directory uses `CARGO_PKG_NAME`).

## 3. Replace the model

The starter is intentionally domain-neutral. `src/model.rs` has the only two model types:

```rust
pub struct Tab {
    pub id: u64,
    pub title: String,
    pub dirty: bool,
    pub data: TabData,
}

#[derive(Default)]
pub struct TabData {
    pub demo: DemoState, // replace with the state one tab owns
}
```

- `Tab` is the shell metadata read by the tab strip, title bar, and close guard.
- `TabData` is the product state. Change it freely; `Tab::new` only needs `Default`.
- Keep the `Document` implementation (`title`, `is_dirty`). `Documents<Tab>` already
  handles tab order, selection, and removal.

The starter opens a tabbed gallery: the navigator lists fixed example sections, each
editor tab shows one section and can be closed, and the Icons tab explores Lucide by
name or similarity. It is demo-only: replace `content.rs`, delete `gallery.rs`, and adapt
`DemoState`.

There is **no file I/O in the starter**. `App::save_tab` is a stub that clears the dirty
flag; replace it with the application's persistence (file, database, remote session, …).
Returning `Err` keeps the tab open and surfaces the message. The window and every dirty
tab already go through the shared Save / Discard / Cancel prompt.

If the app has no tabs, set `tab_strip_visible` to `false`, skip `Documents`, and call
the components directly; the rest of the shell still works.

## 4. Add commands in one place

Every feature starts as a `Command` in `src/commands.rs`. Add the variant, then:

1. a shortcut in `keymap()`;
2. a row in `palette_items()`;
3. a menu item in `src/ui.rs` (`title_menu` / `menu_button` / `menu_toggle`);
4. an arm in `App::handle`.

Menu items, palette rows, and shortcuts all produce the same value, so an action is
reachable three ways. The close guard, `content.rs`, and future background workers can
return commands too.

The order matters for chords: put `Ctrl+Shift+X` before `Ctrl+X`. The `Keymap`
consumes the first match, and `App::logic` only consults it while no overlay is open.

## 5. Build the content

`src/content.rs` is the main seam: `central()` calls `content::show(app, ui)`, and the
function returns commands. Replace the placeholder screen there. Build it with
`egui_workbench::components::prelude::*` (`Button`, `TextField`, `Switch`, `Card`,
`Dialog`, …); [`COMPONENTS.md`](COMPONENTS.md) lists every builder, variant, and return
type. Panels stay in `src/ui.rs`:

| Method | Use |
|---|---|
| `title_bar` | `show_title_bar` with the text menus and toggles |
| `sidebar` | A tree (`show_tree`), outline, list, or inspector |
| `tab_strip` | `show_tab_strip`; return `Select`/`Close` events as commands |
| `central` | Delegate to `content::show`; use `with_editor_scrollbars` if needed |
| `status_bar` | `show_status_bar` with left and right `StatusItem`s |
| `overlays` | Palette, settings, prompts, About, progress |

Hide a panel with `sidebar_visible`, `tab_strip_visible`, or `status_bar_visible`.
Keep the panel order from `WORKBENCH.md` §3; do not draw a panel before the title bar.

## 6. Translate at the call site

Source strings are English. Pass the translated `&str` into components; they never
translate. Keep `…` in labels that open dialogs and lay out for +40 % text length.

## 7. Add Material file icons only when needed

The starter uses `default-features = false`: only the bundled UI glyphs are embedded, so
the build stays small and the template has no file assumptions. Enable the default
feature when the app shows files or folders:

```toml
egui-workbench = { path = "…" }
```

Then `Icons::material` is available for `TabIcon::Texture` and `RowIcon::MaterialFile`.
The full specification is in `WORKBENCH.md` §11.

## 8. Add tests

The starter ships six tests (`cargo test` in the generated project): settings
serialization and tab close/save behavior. As you build:

- Unit-test `TabData` and `App::handle` without egui.
- Test reactive components with `egui_kittest`; see `../egui-workbench/tests/components.rs`.
- Extend `src/settings.rs` tests when you add persisted fields.
- Drive each screen with `egui_kittest` harnesses, like `../egui-workbench/tests/`.
- Add PNG snapshots only for layouts that must stay pixel-stable, with a light and a
  dark reference.

Run the same gate as the base:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Checklist

- [ ] `cargo xtask check` passes in the base and `cargo clippy --all-targets -D warnings` in the app.
- [ ] `TabData` holds the product state, not shell state.
- [ ] `save_tab` is real, and every dirty tab and the window use the close prompt.
- [ ] Every command is reachable from a menu, the palette, and a shortcut.
- [ ] New controls use `components` builders; no raw button/frame colors.
- [ ] Colors come from `theme`, sizes from `metrics`.
- [ ] Files stay under 500 lines.
- [ ] Icon-only buttons have tooltips and AccessKit names.
- [ ] `Esc` closes the top overlay; `Enter` runs the primary action.
- [ ] Any direct `egui`/`eframe` dependency is on the same 0.37 minor as `egui-workbench`.
