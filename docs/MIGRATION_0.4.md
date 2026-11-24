# Migrating from 0.3 to 0.4

0.4 is the breaking window. This page lists every change an application must make.

## Cargo features

The crate now has `plot`, `plot3d`, `viewer`, and `serde` features, plus the existing
`material-icons` and `lucide-icons`. All are on by default, so `egui-workbench = "0.4"`
keeps working. A build that names features must add the ones it uses:

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
rebinding, or palette. New code should register a `Command` and poll the registry:

```rust
use egui_workbench::commands::{Command, CommandRegistry, WhenContext};

let mut registry = CommandRegistry::new();
registry.register(Command::new("file.save", "Save").shortcut("Ctrl+S"));
let context = WhenContext::new();

if let Some(id) = registry.shortcut(ctx, &context) {
    // Map id to your own handler.
}
```

Menus and the palette read the same registry: `show_command_menu`,
`command_menu_groups`, `command_palette_items`, `show_command_palette`, and
`show_keybindings_editor`. The starter application's `commands.rs` is the reference.

## Palette state

`PaletteState` gained `recent: Vec<String>`, and `show_palette` now uses fuzzy ranking and
records the chosen label. Construct it with `..PaletteState::default()` instead of a
literal with only `open` and `query`. `PaletteMode` parses the `>`, `:`, and `@` prefixes;
`PaletteFeed<T>` polls a background result set without blocking.

## Non-exhaustive enums

Public enums such as `TabEvent`, `TreeEvent`, `PromptAction`, `ToastKind`,
`FileFormat`, `ExportFormat`, `PlotTool`, `Size`, and `ThemeId` are `#[non_exhaustive]`.
Add a wildcard arm to exhaustive matches. This is what lets later releases add a variant
without breaking a build.

## Serializable state and events

With `serde`, every built-in state and event round-trips through JSON. The event names are
stable snake_case variants. If you stored a `PaletteState` or `ThemeState` by hand, you
can now let serde do it; `ThemeId` serializes as `github-light`, `dracula`, and so on.

## Crate root

The crate root still re-exports the common functions. The `components` builders remain
the recommended API; the `widgets` functions are unchanged in 0.4 and are scheduled for
deprecation in the 0.17 cleanup. `#[non_exhaustive]` is on the new option structs
(`Command`, `ArgSchema`, `KeybindingEntry`, `Keybindings`, `CommandState`); build them
with the provided constructors and builder methods.

## Removed or renamed

There are no removed items in 0.4. `PromptAction`, `TabEvent`, and `TreeEvent` keep their
variant names; only the serde representation and the non-exhaustive marker are new.
