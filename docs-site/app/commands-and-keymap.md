---
title: Commands and keymap
description: One Command enum, one handler, ordered shortcuts, the palette, and the keyboard menu.
---

# Commands and keymap

## One enum

Every action is a variant. Menus, the palette, shortcuts, the keyboard menu, and panel
buttons produce the same value:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    NewTab,
    Save,
    CloseTab,
    NextTab,
    PreviousTab,
    ToggleSidebar,
    ToggleInspector,
    ToggleOutput,
    Palette,
    Settings,
    SetTheme(ThemeId),
    Quit,
}
```

A variant can carry data when the payload belongs to the command (`SetTheme(ThemeId)`).

## One handler

```rust
impl App {
    fn handle(&mut self, ctx: &egui::Context, command: Command) {
        match command {
            Command::NewTab => self.new_tab(),
            Command::Save => self.save_active(),
            Command::CloseTab => self.close_tab(self.tabs.active_index()),
            Command::NextTab => {
                self.tabs.select_next();
            }
            Command::PreviousTab => {
                self.tabs.select_previous();
            }
            Command::ToggleSidebar => self.sidebar = !self.sidebar,
            Command::Palette => self.palette.open(),
            Command::Settings => self.settings_open = true,
            Command::SetTheme(id) => {
                self.theme.set_requested(ctx, id);
                self.save_settings();
            }
            Command::Quit => apply_window_action(ctx, WindowAction::Close),
        }
    }
}
```

Menus, the palette, and the keymap return commands; `App::ui` applies them after the
frame. This keeps undo/redo, logging, and tests in one place.

## Keymap

`Keymap<C>` is an ordered list of `(modifiers, key) -> C`. The first match consumes the
key, so longer chords must come first.

```rust
use egui_workbench::app::Keymap;
use egui_workbench::egui::{Key, Modifiers};

let keymap = Keymap::new()
    .bind(Modifiers::COMMAND | Modifiers::SHIFT, Key::P, Command::Palette)
    .bind(Modifiers::COMMAND | Modifiers::SHIFT, Key::Tab, Command::PreviousTab)
    .bind(Modifiers::COMMAND | Modifiers::ALT, Key::B, Command::ToggleInspector)
    .bind(Modifiers::COMMAND, Key::N, Command::NewTab)
    .bind(Modifiers::COMMAND, Key::S, Command::Save)
    .bind(Modifiers::COMMAND, Key::W, Command::CloseTab)
    .bind(Modifiers::COMMAND, Key::B, Command::ToggleSidebar)
    .bind(Modifiers::COMMAND, Key::J, Command::ToggleOutput)
    .bind(Modifiers::COMMAND, Key::Comma, Command::Settings)
    .bind(Modifiers::COMMAND, Key::Q, Command::Quit)
    .bind(Modifiers::CTRL, Key::Tab, Command::NextTab);
```

`Modifiers::COMMAND` means Cmd on macOS and Ctrl elsewhere. `consume_key` prevents text
fields and other widgets from seeing the chord.

| Method | Purpose |
|---|---|
| `new()` | Empty map |
| `bind(modifiers, key, command)` | Builder-style insert |
| `insert(modifiers, key, command)` | In-place insert |
| `contains(modifiers, key)` | Whether a chord is bound |
| `len()` / `is_empty()` / `clear()` | Introspection |
| `command(ctx)` | Command for a press this frame, consuming the key |
| `command_cloned(ctx)` | The same, cloned |

`command` returns a reference, which can conflict with a mutable borrow of the
application. Use `command_cloned` when the command is `Clone` (the template does).

## When to consult it

Call the keymap in `App::logic`, only while no modal is open:

```rust
if !self.overlay_open()
    && let Some(command) = self.keymap.command_cloned(ctx)
{
    self.handle(ctx, command);
}
```

`overlay_open` should include the palette, settings, prompts, the keyboard menu, and any
dialog. Shortcuts must not fire underneath a modal.

## Palette

The command palette reuses the same commands:

```rust
let items = vec![
    PaletteItem { label: "New tab".into(), detail: "Ctrl+N".into(), value: Command::NewTab },
    PaletteItem { label: "Save".into(), detail: "Ctrl+S".into(), value: Command::Save },
    PaletteItem { label: "Settings…".into(), detail: "Ctrl+,".into(), value: Command::Settings },
];
if let Some(command) = CommandPalette::new(&mut self.palette, labels, &items).show(ctx) {
    self.handle(ctx, command);
}
```

Matching is case-insensitive on label and detail, up to 12 rows. The first row is
highlighted and runs on Enter. `Esc`, the ×, or a backdrop click closes it.

## Keyboard menu

The F10/Alt menu is the discoverable, grouped view of the same commands:

```rust
let groups = vec![
    MenuGroup {
        label: "File".to_owned(),
        entries: vec![
            MenuEntry { label: "New tab".to_owned(), value: Command::NewTab },
            MenuEntry { label: "Save".to_owned(), value: Command::Save },
            MenuEntry { label: "Close tab".to_owned(), value: Command::CloseTab },
        ],
    },
    MenuGroup {
        label: "Help".to_owned(),
        entries: vec![MenuEntry { label: "Settings".to_owned(), value: Command::Settings }],
    },
];
if let Some(command) = show_keyboard_menu(ctx, &mut self.keyboard_menu, &groups, "Enter runs · Esc closes") {
    self.handle(ctx, command);
}
```

`open_from_keyboard(ctx, &mut state, &letters)` opens it on F10 or an Alt+letter chord.

## The full shortcut table

| Shortcut | Command |
|---|---|
| `Ctrl+N` / `Ctrl+O` / `Ctrl+Shift+O` | New / Open / Open Folder |
| `Ctrl+S` / `Ctrl+Shift+S` | Save / Save As |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Next / previous tab |
| `Alt+←` / `Alt+→` | Back / forward |
| `Ctrl+P` / `Ctrl+Shift+P` | Quick Open / command palette |
| `Ctrl+F` / `Ctrl+Shift+F` / `Ctrl+H` / `Ctrl+G` | Find / search workspace / replace / go to line |
| `Ctrl+E` / `Ctrl+Shift+E` | Toggle edit/view / split |
| `Ctrl+B` | Toggle sidebar (Bold inside a focused Markdown editor) |
| `Ctrl+Alt+B` | Toggle inspector (right panel) |
| `Ctrl+J` | Toggle output (bottom panel) |
| `Ctrl+,` | Settings |
| `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | Zoom in / out / reset |
| `F2` | Rename |
| `F10`, `Alt+F/E/V/G/H` | Keyboard menu |
| `Ctrl+Q` | Quit through the close request |

The template implements the subset it needs; keep the table in `commands.rs` as the one
source of truth.

## Rules

- Add a command once, then wire all three entry points: menu, palette, shortcut.
- Put longer chords first; the first match consumes the key.
- Do not consult the keymap while a modal is open.
- `Ctrl+Q` should send `WindowAction::Close`, not exit the process, so unsaved changes
  still prompt.
- Menu rows and palette rows are translated at the call site; the enum stays language
  neutral.
