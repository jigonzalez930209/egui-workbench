# Migrating from 0.3 to 0.4

> The command registry, serde, non-exhaustive enums, and the Cargo features added in 0.4. Live page: <https://jigonzalez930209.github.io/egui-workbench/reference/migration-0.4.html>

0.4 is the breaking window. This page lists every change an application must make.

## Cargo features

The crate now has `plot`, `plot3d`, `viewer`, and `serde` features, plus the existing
`material-icons` and `lucide-icons`. All are on by default, so `egui-workbench = "0.4"`
keeps working. A build that names features must add the ones it uses:

`Cargo.toml`:

```toml
[dependencies]
egui-workbench = { version = "0.4", default-features = false, features = [
    "lucide-icons", "plot", "plot3d", "viewer", "serde",
] }
```

`calamine` is behind `viewer`; `resvg`/`usvg` are behind `svg`, which the icon packs and
`plot` enable. A no-icon, no-plot build no longer compiles the rasterizer.

## `Keymap` to `CommandRegistry`

`app::Keymap<C>` still exists, but it has no metadata, contexts, conflicts, chords,
rebinding, or palette. New code should register a `Command` and poll the registry. See
[Commands, contexts, and keybindings](../app/commands-and-keymap.md).

## Palette state

`PaletteState` gained `recent: Vec<String>`, and `show_palette` now uses fuzzy ranking and
records the chosen label. Construct it with `..PaletteState::default()` instead of a
literal with only `open` and `query`. `PaletteMode` parses the `>`, `:`, and `@` prefixes;
`PaletteFeed<T>` polls a background result set without blocking.

## Non-exhaustive enums

Public enums such as `TabEvent`, `TreeEvent`, `PromptAction`, `ToastKind`, `FileFormat`,
`ExportFormat`, `PlotTool`, `Size`, and `ThemeId` are `#[non_exhaustive]`. Add a wildcard
arm to exhaustive matches. This is what lets later releases add a variant without
breaking a build.

## Serializable state and events

With `serde`, every built-in state and event round-trips through JSON. The event names are
stable snake_case variants. `ThemeId` serializes as `github-light`, `dracula`, and so on;
`TabEvent::Select(2)` is `{"select": 2}` and `PromptAction::Submit` is `"submit"`.

## Crate root and widgets

The crate root still re-exports the common functions. The `components` builders remain the
recommended API; the `widgets` functions are unchanged in 0.4. `#[non_exhaustive]` is on
the new option structs (`Command`, `ArgSchema`, `KeybindingEntry`, `Keybindings`,
`CommandState`); build them with the provided constructors.

There are no removed items in 0.4.
