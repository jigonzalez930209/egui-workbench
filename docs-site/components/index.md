---
title: Components overview
description: The builder layer, shared variants and sizes, the reactivity contract, and the complete component index.
---

# Components overview

`egui_workbench::components` is the shadcn-style layer over the workbench primitives.
The root function API still works; this layer adds builders, shared variants, and one
vocabulary for sizes and tones.

```rust
use egui_workbench::components::prelude::*;

if Button::new("Save").variant(ButtonVariant::Primary).show(ui).clicked() {
    save();
}
TextField::new(&mut name).label("Name").width(240.0).show(ui);
if Switch::new(&mut wrap).label("Word wrap").show(ui) {
    dirty = true;
}
```

<LiveDemo
  scene="components"
  height="480"
  alt="Component board with buttons, inputs, switches, badges, and a card"
  caption="Every component family on one page; it follows the site's light/dark switch."
/>

## Reactivity contract

egui is immediate mode, so there is no virtual DOM and no hidden component state.

- Interactive components take `&mut` state and mutate it in the same frame.
- Components return the result: `bool` for changed/toggled, `Option<T>` for a choice,
  or `egui::Response` for position, hover, focus, and clicks.
- The application remains the source of truth. A click does not schedule a hidden
  update; the caller applies the event in its handler.
- Paint code never performs I/O. Commands and file operations stay in the application;
  the data viewer parses once in `open`/`from_bytes` and paints from memory.
- Animations use egui's animation clock and respect reduced motion.

## Two traits

| Trait | Signature | Used by |
|---|---|---|
| `Component` | `fn show(self, ui: &mut Ui) -> Self::Output` | Everything that needs only a `Ui` |
| `IconComponent` | `fn show(self, ui: &mut Ui, icons: &mut Icons) -> Self::Output` | Buttons, menus, tabs, trees, steppers, plots, icons |

Dialogs and the command palette are context-level composition: they take a
`&egui::Context` and paint a modal above the panels.

## Sizes and tones

```rust
pub enum Size { Sm, Md, Lg }                 // 24, 30, 36 pt
pub enum IconButtonSize { Action, Tool, Small } // 28, 26, 21 pt
pub enum Tone { Neutral, Muted, Accent, Warning, Danger, Custom(Color32) }
```

`Size` also carries the text size, minimum width, and padding. `WorkbenchStyle` scales
the height and vertical padding for `Compact` and `Comfortable` densities; the horizontal
padding and text size stay fixed.

## Component index

| Group | Components | Page |
|---|---|---|
| Buttons | `Button`, `IconButton`, `CloseButton` | [Buttons](/components/buttons) |
| Inputs | `TextField`, `TextArea`, `Switch`, `Checkbox`, `Select`, `Stepper`, `SegmentedControl` | [Inputs](/components/inputs) |
| Display | `Label`, `Badge`, `Banner`, `Card`, `Separator`, `StatusBar`, `IconPreview` | [Display](/components/display) |
| Feedback | `Toasts`, `Toast`, `ToastKind`, `ToastPosition` | [Feedback](/components/feedback) |
| Menus | `Menu`, `MenuButton`, `MenuItem`, `MenuToggle`, `MenuIconButton`, `NavTab` | [Menus](/components/menus) |
| Overlays | `Dialog`, `ConfirmDialog`, `NameDialog`, `ProgressDialog`, `Popover` | [Overlays](/components/overlays) |
| Navigation | `TabStrip`, `Tree`, `CommandPalette` | [Navigation](/components/navigation) |
| Icons | `IconGrid`, `IconPreview`, `Icons` | [Icons](/components/icons) |
| Layout | `split_panes`, `show_empty_state`, `Card` composition, blocks | [Layout](/components/layout) |
| Data | `FileViewer`, `Workbook`, `Sheet`, `Table`, `Cell`, `TextFile` | [Data viewer](/components/data-viewer) |

Charts have their own section: [Charts](/charts/overview).

## Composite blocks

`components::blocks` re-exports the larger shell blocks without exposing the internal
module layout:

```rust
pub use crate::{
    KeyboardMenuState, MenuEntry, MenuGroup, STACK_BELOW, SettingsDialog, TitleBar,
    TitleBarOutput, WindowLabels, banner, close_glyph, compact_menu, dialog_footer,
    dialog_frame, dialog_header, editor_scroll_style, menu_tool_button, open_from_keyboard,
    section_header, settings_row, settings_section, settings_switch, show_choice_window,
    show_empty_state, show_keyboard_menu, show_popover, show_progress, show_settings_dialog,
    show_title_bar, split_panes, title_menu, with_editor_scrollbars,
};
```

`CommandPalette`, `Tree`, and `TabStrip` have builders in `components` itself.

## Migration from the function API

The original functions remain public because the shell and tests use them, and because an
application may prefer a direct call for a one-off.

| Function | Builder |
|---|---|
| `primary_button(ui, label, danger)` | `Button::new(label).variant(Primary or Destructive)` |
| `secondary_button(ui, label)` | `Button::new(label).variant(Secondary)` |
| `text_field(ui, &mut value, hint, width)` | `TextField::new(&mut value).hint(hint).width(width)` |
| `switch(ui, &mut value, label)` | `Switch::new(&mut value).label(label)` |
| `combo(ui, label, &mut value, options)` | `Select::new(label, &mut value, options)` |
| `numeric_stepper(…)` | `Stepper::new(&mut value, range, step)` |
| `action_button` / `tool_button` / `small_button` | `IconButton::new(icon, tooltip).size(…)` |
| `menu_item(…)` | `MenuItem::new(icon, label)` |
| `menu_button(…)` | `MenuButton::new(label)` |
| `menu_toggle(…)` | `MenuToggle::new(label, on)` |
| `show_tab_strip(…)` | `TabStrip::new(tabs, active)` |
| `show_status_bar(…)` | `StatusBar::new().left(…).right(…)` |
| `show_palette(…)` | `CommandPalette::new(state, labels, items)` |

New shared screens should use the builders so variants stay consistent.
