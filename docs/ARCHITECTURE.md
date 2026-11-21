# Architecture

The crate is one dependency with five layers. Dependencies point down only.

```text
application (template or your crate)
        │  implements WorkbenchApp, owns Command and state
        ▼
app/                 ThemeState · Documents · Keymap · CloseGuard · show_workbench
        │
        ▼
components/          builders · variants · tones · composite blocks
        │
        ├──────────────┬──────────────┐
        ▼              ▼              ▼
chrome/            widgets/       (both use foundation)
window, title,     buttons, controls,
tabs, status,      dialogs, palette,
split              settings, tree
        │              │
        └──────┬───────┘
               ▼
foundation/          metrics · theme · icons · material
```

## Rules per layer

| Layer | Owns | Must not |
|---|---|---|
| `foundation` | Sizes, color tokens, egui style application, SVG caches | Draw application content or read files |
| `chrome` | Window edges, title bar, tab strip, status bar, split | Own a document model or apply commands |
| `widgets` | The function API for inputs, buttons, dialogs, overlays, tree rows | Know an application's `Command` type |
| `components` | Builders, shared variants/tones, and composite blocks | Own application state or invent tokens |
| `app` | Repeated application plumbing and the fixed panel order | Do file I/O, translate text, or pick a storage format |
| Template | State, `Command`, persistence, translations | Reimplement chrome or hard-code tokens |

Components are generic where an application value is needed: `PaletteItem<T>`,
`MenuGroup<T>`, `TreeNode<K>`, `Keymap<C>`, and `Documents<T>`. The crate never depends
on an application's command enum.

## Presentation and state

Every component is a function that takes a `Ui` plus a state snapshot and returns an
event or command. It does not mutate application state. `show_workbench` follows the
same rule: it draws the panels and returns a `Vec<Command>` for the application to apply
after the frame.

This keeps three things possible:

1. **One handler.** Menus, the palette, the keymap, and panel buttons produce the same
   `Command`, so every action has one implementation.
2. **Testable panels.** `egui_kittest` can drive a panel, inspect the returned command,
   and compare a snapshot without a running application.
3. **Undo/redo and logging.** State changes happen in one place, not inside widget code.

## The component layer

`components` is the shadcn-style surface. It gives every primitive a builder, shared
`Size`/`ButtonVariant`/`Tone` values, and a consistent reactivity contract:

- interactive builders take `&mut` state and return the change (`bool`, `Option<T>`, or
  `Response`);
- paint code never performs I/O;
- widgets declare AccessKit metadata;
- colors and sizes come from `ui.visuals()`, `theme`, or `metrics`.

The original `widgets` functions remain public. New screens should prefer the builders so
variants do not drift; the shell and the runtime keep using the functions internally.
Composite blocks (title bar, settings, keyboard menu, split, palette, tree) are re-exported
from `components::blocks` or have a builder (`TabStrip`, `CommandPalette`, `Tree`).

## The fixed panel order

`show_workbench` is the only code that knows the order from `WORKBENCH.md` §3:

1. `handle_window_edges` for the undecorated resize bands.
2. Title bar, full width, 34 pt.
3. Explorer, left, resizable, below the title bar.
4. Tab strip, top of the editor column, 34 pt.
5. Status bar, bottom of the editor column, 24 pt.
6. Central content.
7. Overlays.

Applications can skip the shell and call these panels themselves; they then own the
order and the tests that protect it.

## Runtime pieces

### `ThemeState`

Holds the requested theme, follow-system, reduced motion, and the effective theme
currently installed in egui. It re-applies styles only when something changes, so
`sync_system(ctx)` can run every frame. `clear_color()` feeds
`eframe::App::clear_color`, which avoids a black flash while resizing.

### `Documents<T>`

A vector plus an active index. `remove` keeps the same document active when possible:
removing before the active one shifts the index; removing the active one selects its
successor. Implement `Document` for `title()` and `is_dirty()`; the rest is generic.

### `Keymap<C>`

An ordered list of `(modifiers, key) -> C`. First match consumes the key, so longer
chords must come first (`Ctrl+Shift+P` before `Ctrl+P`). Only consult it while no modal
is open.

### `CloseGuard`

Cancels an eframe close request while documents are dirty and returns `true` so the
application shows its own Save / Discard / Cancel prompt. `allow_close` lets the next
request through. This is the only place that sends `CancelClose`.

### `WorkbenchApp`

The application side of the shell. `sidebar_visible`, `tab_strip_visible`, and
`status_bar_visible` compose the layout; each `show_*` method returns commands.
`show_workbench` calls `handle_window_edges` for you.

## Adding a component

1. Put it in the layer it belongs to: `foundation` if it only defines a value,
   `widgets` if it is an input or overlay, `chrome` if it is part of the fixed shell.
2. Take sizes from `metrics` and colors from `theme` or `ui.visuals()`.
3. Return an event or `Option<Command>`; do not mutate application state.
4. Declare an AccessKit role and name with `response.widget_info(...)`.
5. Add a test in `tests/` and, for a visible component, a snapshot.
6. Re-export it from `src/lib.rs` and add it to the API map in `WORKBENCH.md` §13.1.

## Adding a theme

Add the variant to `ThemeId`, its tokens to `foundation/theme/palette.rs`, and its
`as_setting` / `label` / `family` / `mode` arms. The contrast tests iterate
`ThemeId::ALL`, so they cover it automatically. Add a snapshot if the theme is meant to
be a flagship.

## Versioning the base

`egui-workbench` follows semantic versioning on crates.io (0.x: a minor bump may break
the API; `CHANGELOG.md` says how to migrate). It depends on `egui`/`eframe` 0.37 from
crates.io; moving to a new egui minor is a breaking release of this crate. Projects
generated from a checkout use a path dependency, so the repository revision is their
version until they switch to `egui-workbench = "0.1"`.
