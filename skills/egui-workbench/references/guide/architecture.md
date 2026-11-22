# Architecture

> The five layers of egui-workbench, the fixed panel order, and the flow of one frame. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/architecture.html>

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
foundation/          metrics · theme · style · icons · material
```

## Rules per layer

| Layer | Owns | Must not |
|---|---|---|
| `foundation` | Sizes, color tokens, shape/density, egui style application, SVG caches | Draw application content or read files |
| `chrome` | Window edges, title bar, tab strip, status bar, split | Own a document model or apply commands |
| `widgets` | The function API for inputs, buttons, dialogs, overlays, tree rows | Know an application's `Command` type |
| `components` | Builders, shared variants/tones, composite blocks, plots, toasts | Own application state or invent tokens |
| `app` | Repeated application plumbing and the fixed panel order | Do file I/O, translate text, or pick a storage format |
| Template | State, `Command`, persistence, translations | Reimplement chrome or hard-code tokens |

Components are generic where an application value is needed: `PaletteItem<T>`,
`MenuGroup<T>`, `TreeNode<K>`, `Keymap<C>`, and `Documents<T>`. The crate never depends
on an application's command enum.

## Presentation and state

Every component is a function or builder that takes a `Ui` plus a state snapshot and
returns an event or command. It does not mutate application state. `show_workbench`
follows the same rule: it draws the panels and returns `Vec<Command>` for the application
to apply after the frame.

This keeps three things possible:

1. **One handler.** Menus, the palette, the keymap, and panel buttons produce the same
   command, so every action has one implementation.
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
- colors and sizes come from `ui.visuals()`, `theme`, `style`, or `metrics`.

The original `widgets` functions remain public. New screens should prefer the builders so
variants do not drift; the shell and runtime use the functions internally.

## The fixed panel order

`show_workbench` is the only code that knows the order from `WORKBENCH.md` §3:

1. `handle_window_edges` for the undecorated resize bands.
2. Title bar, full width, 34 pt.
3. Explorer, left, resizable, below the title bar.
4. Right panel, resizable, from the title bar to the bottom.
5. Tab strip, top of the editor column, 34 pt.
6. Status bar, bottom of the editor column, 24 pt.
7. Bottom panel, resizable, above the status bar.
8. Central content.
9. Overlays.

The explorer and right panel yield to the content on narrow windows. The right panel is
never wider than half the window; the bottom panel never taller than 60 % of the editor
column.

Applications can skip the shell and call these panels themselves; they then own the
order and the tests that protect it.

## One frame

```text
input ──► App::logic
            theme.sync_system
            guard.should_prompt
            keymap.command_cloned (only without a modal)
          │
          ▼
        App::ui
            show_workbench
              title_bar        ─┐
              sidebar           │
              right_panel       ├─ each returns Vec<Command>
              tab_strip         │
              status_bar        │
              bottom_panel      │
              central           │
              overlays         ─┘
          │
          ▼
        App::handle(each command) ──► state changes ──► next frame
```

## Where values come from

| Value | Source |
|---|---|
| Colors | `theme::Theme` tokens, `ui.visuals()`, or `theme::current(ctx)` for roles egui lacks (`success`, `done`) |
| Shape | `style::WorkbenchStyle` (density, radii, border width) |
| Chrome sizes | `egui_workbench::metrics` |
| Control sizes | `components::Size` and `IconButtonSize` |
| Icons | `Icons` (UI glyphs, Lucide, custom SVGs) and `Icons::material` |

No component defines a literal color. The only documented exceptions are the close
button's red hover (`#C42B1C`) and the white tint used to colorize monochrome SVGs.

## Adding a component

1. Put it in the layer it belongs to: `foundation` if it only defines a value,
   `widgets` if it is an input or overlay, `chrome` if it is part of the fixed shell.
2. Take sizes from `metrics`/`style` and colors from `theme` or `ui.visuals()`.
3. Return an event or `Option<Command>`; do not mutate application state.
4. Declare an AccessKit role and name with `response.widget_info(...)`.
5. Add a test in `tests/` and, for a visible component, a snapshot.
6. Re-export it from `src/lib.rs` and add it to [the API map](../reference/api-map.md).

## Versioning the base

`egui-workbench` follows semantic versioning on crates.io (0.x: a minor bump may break
the API; `CHANGELOG.md` says how to migrate). It depends on `egui`/`eframe` 0.37 from
crates.io; moving to a new egui minor is a breaking release of this crate. Projects
generated from a checkout use a path dependency, so the repository revision is their
version until they switch to `egui-workbench = "0.2"`.
