# Core concepts

> The state contract, commands, theming tokens, builders versus functions, accessibility, and repaint behavior. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/concepts.html>

## Immediate mode means caller-owned state

There is no virtual DOM and no hidden component state. Every frame, a component receives
a snapshot and a `&mut` for the value it edits:

```rust
let mut name = String::from("report.md");
if TextField::new(&mut name).label("Name").show(ui).changed() {
    dirty = true;
}
```

The component mutates the string in the same frame and returns an `egui::Response`; it
does not schedule a hidden update. The application remains the source of truth.

| Return | Used by | Meaning |
|---|---|---|
| `bool` | `Switch`, `Checkbox`, `MenuButton`, `MenuToggle`, `CloseButton` | Changed, toggled, or clicked |
| `bool` | `Select`, `Stepper` | The value changed |
| `Option<usize>` | `SegmentedControl`, `IconGrid` | The selected index / clicked cell |
| `Option<T>` | `CommandPalette` | The value of the row that ran |
| `Option<TabEvent>` | `TabStrip` | Select or close a tab |
| `Vec<TreeEvent<K>>` | `Tree` | Toggle, open, or rename a node |
| `Response` | `Button`, `IconButton`, `TextField`, `Label` | Clicks, hover, focus, geometry |
| `Option<PromptAction>` | `ConfirmDialog`, `NameDialog` | Confirm or cancel |

## One command, one handler

Menus, the palette, shortcuts, the keyboard menu, and panel buttons all produce the same
enum. The application applies commands after the frame, in one place:

```rust
enum Command {
    Save,
    CloseTab,
    ToggleSidebar,
}

fn handle(&mut self, ctx: &egui::Context, command: Command) {
    match command {
        Command::Save => self.save_active(),
        Command::CloseTab => self.close_tab(),
        Command::ToggleSidebar => self.sidebar = !self.sidebar,
    }
}
```

For the full pattern, see [Commands and keymap](../app/commands-and-keymap.md).

## Builders and functions

The root crate exports the original function API (`primary_button`, `text_field`,
`switch`, `show_tab_strip`, …). `components` adds builders with shared variants, sizes,
and tones:

```rust
// Function API, still public.
if egui_workbench::primary_button(ui, "Save", false) {
    save();
}

// Builder API, preferred for new screens.
if Button::new("Save").variant(ButtonVariant::Primary).show(ui).clicked() {
    save();
}
```

Both paint the same widgets. New code should use builders so variants do not drift. See
the [migration table](../components/index.md#migration-from-the-function-api).

## Tokens, style, and metrics

Three global layers decide how the shell looks:

1. **`theme::Theme`** — colors. Nine built-in themes, or your own tokens with
   `theme::apply_tokens`. Install through `ThemeState` to keep the preference.
2. **`style::WorkbenchStyle`** — density and shape. `Compact`, `Default`, or
   `Comfortable`; control, item, container, dialog, and pill radii; border width.
   Installed once with `style::set` and preserved across theme changes.
3. **`metrics`** — fixed chrome sizes: title bar 34 pt, tab strip 34 pt, status bar
   24 pt, tree row 21 pt, and so on.

Components read these values; they never define a color or duplicate a chrome size.
See [Color tokens](../theming/tokens.md) and [Style and density](../theming/style-and-density.md).

## Accessibility

- Every custom-painted widget declares an AccessKit role and name.
- Icon-only buttons are named by their tooltip; fields by their visible label; tree rows
  are tree items; tabs and settings rail entries are tabs.
- `egui_kittest` fails a test when a widget has no accessible name, so the tests protect
  the contract.
- Contrast is checked against WCAG 2.2 AA: 4.5:1 for text and 3:1 for the focus ring.
- Every pointer action has a keyboard path. `Esc` closes the top overlay; `Enter` runs the
  primary action.

## Motion and repaint

- Animations use egui's animation clock. Reduced motion sets the animation time to zero.
- The UI does not repaint while idle. A background worker must call
  `ctx.request_repaint()` after it produces a result.
- Toasts wake the frame only for their next timeout or animation step.
- The plot does not repaint while the pointer is still.

## What belongs to the application

| Concern | Where it lives |
|---|---|
| Documents and their data | Your model, wrapped in `Documents<T>` |
| File I/O, network, persistence | `App::handle` or a worker; never a component |
| Translations | The call site; components receive translated strings |
| The `Command` enum | Your crate |
| Window title and close policy | `App::logic` with `set_window_title` and `CloseGuard` |

The library never translates, never reads a file, and never picks a storage format.
