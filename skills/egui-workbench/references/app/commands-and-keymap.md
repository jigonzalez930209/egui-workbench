# Commands, contexts, and keybindings

> One registry for menus, the palette, shortcuts, contexts, rebinding, and JSON state. Live page: <https://jigonzalez930209.github.io/egui-workbench/app/commands-and-keymap.html>

The 0.4 command layer replaces hand-wired menus and a bare shortcut list with one
registry. A command has a stable id, a title, a category, an icon, a default shortcut, a
`when` context, enabled/checked state, and an optional argument schema. Menus, the
palette, the keyboard menu, the title bar, and the keybindings editor are views of the
same data.

## One registry

```rust
use egui_workbench::commands::{Command, CommandId, CommandRegistry, WhenContext};

fn main() {
    let mut registry = CommandRegistry::new();
    registry.register(
        Command::new("file.save", "Save")
            .category("File")
            .shortcut("Ctrl+S")
            .icon("save"),
    );
    registry.register(
        Command::new("edit.paste", "Paste")
            .category("Edit")
            .shortcut("Ctrl+V")
            .when("editorFocus && !dialogOpen"),
    );

    let context = WhenContext::new().with("editorFocus");
    assert_eq!(registry.palette_items(&context).len(), 2);
    let save = CommandId::new("file.save");
    assert_eq!(registry.binding(&save).map(ToString::to_string), Some("Ctrl+S".to_owned()));
    assert!(registry.get(&save).is_some_and(|command| command.is_enabled()));
}
```

The registry never runs application code. It returns a `CommandId`; the application maps
it to its own enum or handler, which keeps metadata serializable and testable without a
UI. The starter's `commands.rs` does exactly that.

## Contexts and `when`

`WhenContext` is a set of keys. The standard keys are `editorFocus`, `hasSelection`,
`dialogOpen`, `paletteOpen`, `readOnly`, and `dirty`; an application may add its own.
The expression language is `!`, `&&`, `||`, and parentheses, evaluated with `!` first,
then `&&`, then `||`.

```rust
use egui_workbench::commands::{When, WhenContext};

let when = When::parse("editorFocus && !dialogOpen").expect("valid when");
let context = WhenContext::new().with("editorFocus");
assert!(when.evaluate(&context));
assert!(!when.evaluate(&context.clone().with("dialogOpen")));
```

A parse error carries a message and a byte offset. `Command::try_when` returns it instead
of silently falling back to always-true.

## Rebinding and conflicts

```rust
use egui_workbench::commands::{Command, CommandId, CommandRegistry, KeySequence};

let mut registry = CommandRegistry::new();
registry.register(Command::new("file.save", "Save").shortcut("Ctrl+S"));
let paste = CommandId::new("edit.paste");
registry.register(Command::new("edit.paste", "Paste").shortcut("Ctrl+V"));

// Conflict: file.save already owns Ctrl+S.
assert!(registry.bind(&paste, KeySequence::parse("Ctrl+S").expect("keys")).is_err());

// Chords are space-separated.
registry
    .bind(&paste, KeySequence::parse("Ctrl+K Ctrl+V").expect("keys"))
    .expect("binding is free");
```

`bindings()` lists every active binding, longest sequence first. Default shortcuts that
collide are skipped and reported by `conflicts()`. `unbind` removes a binding,
`reset_binding` returns to the default, and `has_override` tells the editor whether to
offer Reset.

## Keybindings file and VS Code import

The file is versioned JSON:

```json
{
  "version": 1,
  "bindings": [
    { "command": "file.save", "keys": "Ctrl+S" },
    { "command": "edit.paste", "keys": "Ctrl+K Ctrl+V" }
  ]
}
```

```rust
use egui_workbench::commands::{Command, CommandRegistry, KeybindingEntry, Keybindings};

let mut registry = CommandRegistry::new();
registry.register(Command::new("file.save", "Save"));
let file = Keybindings::new().with(KeybindingEntry::new("file.save", "Ctrl+S"));
let errors = file.apply(&mut registry);
assert!(errors.is_empty());
```

`Keybindings::import_vscode(json)` reads VS Code's array of `{ "key", "command",
"when" }` objects, normalizes the key syntax, and skips keys it cannot parse. Unknown
command ids are reported by `apply`; the `when` text is preserved for round-tripping but
the command-level context is what the registry evaluates.

## The keybindings editor

`show_keybindings_editor` is a modal list built from the registry. Clicking Rebind records
the next key or chord, applies it, and returns a `KeybindingChange` for your journal or
settings file. It shows the current shortcut, Reset (when overridden), and Unbind.

```rust
use egui_workbench::commands::{CommandRegistry, KeybindingsEditorState, KeybindingsLabels};

fn editor(ctx: &egui::Context, registry: &mut CommandRegistry, state: &mut KeybindingsEditorState) {
    let _changes = egui_workbench::commands::show_keybindings_editor(
        ctx,
        state,
        registry,
        KeybindingsLabels::default(),
    );
}
```

## Shortcut polling

Call `registry.shortcut(ctx, &context)` in `App::logic` while no overlay is open. It
consumes the key so text fields do not see it, and it records the command in the recent
list. Chord sequences start a pending sequence and complete within 1.5 s.

```rust
if !self.overlay_open()
    && let Some(id) = self.registry.shortcut(ctx, &context)
    && let Some(command) = self.command_from_id(&id)
{
    self.handle(ctx, command);
}
```

Do not call it underneath a modal. Include the palette, settings, prompts, the keyboard
menu, and dialogs in `overlay_open`.

## Palette v2

`command_palette_items` turns the registry into rows; `show_command_palette` runs one and
marks it recent. Ranking is fuzzy: exact, then prefix, then substring, then subsequence,
with label matches above detail matches. The `>` prefix is the commands mode; `:` and `@`
are available to a provider that supplies lines or symbols. `PaletteState::mode()` returns
the prefix and the text after it.

```rust
use egui_workbench::commands::CommandPaletteLabels;
use egui_workbench::components::PaletteState;

fn palette(ctx: &egui::Context, state: &mut PaletteState, registry: &mut egui_workbench::commands::CommandRegistry) {
    let context = egui_workbench::commands::WhenContext::new();
    if let Some(id) = egui_workbench::commands::show_command_palette(
        ctx,
        state,
        CommandPaletteLabels::default(),
        registry,
        &context,
    ) {
        // Map the id to the application command and run it.
        let _ = id;
    }
}
```

Results from a background provider use `PaletteFeed`: submit a `Receiver<Vec<PaletteItem<T>>>`,
call `poll()` once per frame, and pass `items()` to the palette. `poll` uses `try_recv`, so
an unfinished request never blocks the frame; the previous rows stay visible until the new
ones arrive.

## Keyboard menu

`command_menu_groups` groups the same registry by category, with the shortcut appended to
each row. Feed the groups to `show_keyboard_menu`; the selected value is a `CommandId`.

```rust
use egui_workbench::commands::{command_menu_groups, WhenContext};
use egui_workbench::{KeyboardMenuState, show_keyboard_menu};

fn keyboard_menu(
    ctx: &egui::Context,
    registry: &egui_workbench::commands::CommandRegistry,
    state: &mut KeyboardMenuState,
) -> Option<egui_workbench::commands::CommandId> {
    let groups = command_menu_groups(registry, &WhenContext::new());
    show_keyboard_menu(ctx, state, &groups, "Enter runs · Esc closes")
}
```

## Serializable state and events

With the `serde` feature, `ThemeState`, `Documents`, `TreeState`, `PaletteState`,
`PlotState`, `PlotConfig`, `Plot3DState`, the viewer data, and the event types round-trip
through JSON. Events use stable snake_case names (`TabEvent::Select` is `{"select": 2}`,
`PromptAction::Submit` is `"submit"`). Add `_` arms when matching the non-exhaustive event
enums.

```rust
#[cfg(feature = "serde")]
{
    let ids = egui_workbench::commands::CommandId::new("file.save");
    let json = serde_json::to_string(&ids).expect("serialize");
    assert_eq!(json, "\"file.save\"");
}
```

## The old `Keymap`

`app::Keymap<C>` remains for applications that only need an ordered shortcut list without
metadata. New code should use `CommandRegistry`: it adds categories, contexts, conflicts,
rebinding, chords, the palette, the editor, and serialization.
