---
title: Layout
description: Split panes, empty states, cards, status bars, and the fully composed settings and shell blocks.
---

# Layout

<LiveDemo
  scene="shell"
  height="460"
  alt="A complete workbench layout"
  caption="The shell composes the title bar, explorer, tabs, status bar, and central content."
/>

## split_panes

Two panes with a draggable divider, used for editor/preview. At 620 pt or wider the panes
sit side by side; narrower, they stack with the second pane below. The ratio is the
second pane's share, clamped to `0.2..=0.8`.

```rust
let mut ratio = self.split;
let changed = split_panes(
    ui,
    &mut ratio,
    |ui| editor(ui, &mut self.text),
    |ui| preview(ui, &self.text),
);
self.split = ratio;
```

The divider is a 6 pt drag handle with a resize cursor. Each pane keeps at least 200 pt
(columns) or 120 pt (rows) when the space allows; in a smaller space both panes shrink
evenly and never overflow.

`STACK_BELOW` is the threshold (`620.0`). `split_panes` returns `true` when the divider
moved this frame.

## show_empty_state

A centered heading, subtitle, primary button, and hint. Returns `true` when the button is
clicked.

```rust
if show_empty_state(
    ui,
    "No document is open",
    "Open a file or create a new tab.",
    "New tab",
    "Ctrl+N",
) {
    return vec![Command::NewTab];
}
```

Use it in the central panel when there is nothing to show.

## editor scrollbars

`with_editor_scrollbars(ui, |ui| …)` installs the thin editor scroll style for the scope
and restores it afterwards. `editor_scroll_style(base)` returns the style without
installing it.

```rust
with_editor_scrollbars(ui, |ui| {
    egui::ScrollArea::both()
        .id_salt("editor")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add(egui::TextEdit::multiline(&mut self.text).desired_width(f32::INFINITY));
        });
});
```

## StatusItem and StatusBar

The status bar is a 24 pt row, normally drawn by `show_workbench` in a bottom panel.

```rust
use egui_workbench::StatusItem;

StatusBar::new()
    .left(StatusItem::new("README.md"))
    .left(StatusItem::new("Modified"))
    .right(StatusItem::new("UTF-8"))
    .right(StatusItem::new("LF"))
    .show(ui);
```

`StatusItem::new(text)`, `.tooltip(text)`, and `.warning()` are the whole cell API.
`left_items` / `right_items` replace a side. `ellipsize_start(text, max)` shortens a long
path from the left.

## Section headers

`section_header(ui, icons, title, |ui, icons| …)` draws the explorer header: capitalized
title and right-aligned small icon buttons.

```rust
let mut refresh = false;
section_header(ui, &mut icons, "EXPLORER", |ui, icons| {
    if small_button(ui, icons, "refresh", "Refresh") {
        refresh = true;
    }
});
```

The button functions (`action_button`, `tool_button`, `small_button`) remain available
for exact chrome sizes; `IconButton` is the builder equivalent.

## Settings blocks

`SettingsDialog`, `show_settings_dialog`, `settings_section`, `settings_row`, and
`settings_switch` compose the standard settings surface:

```rust
let dialog = SettingsDialog {
    title: "Settings",
    tabs: &["Appearance", "Shape"],
    note: "My App 0.1.0",
    done: "Done",
    close: "Close",
};
show_settings_dialog(ctx, &mut self.settings_open, &dialog, |ui, tab| {
    if tab == 0 {
        settings_section(ui, "Appearance", "Theme and motion.");
        settings_row(ui, "Theme", "Color family for the whole workbench.", |ui| {
            combo(ui, "Theme", &mut theme, &options);
        });
        settings_switch(ui, "Follow system", "Match the desktop preference.", &mut follow);
    }
});
```

The dialog is 560–620 pt wide (preferred 560, clamped to the window), with a left rail
of tabs and Done/×/Esc behavior. See [Settings dialog](/recipes/settings-dialog).

## Composite blocks

`components::blocks` re-exports:

- `TitleBar`, `show_title_bar`, `title_menu`, `TitleBarOutput`, `WindowLabels`
- `SettingsDialog`, `show_settings_dialog`, `settings_row`, `settings_section`, `settings_switch`
- `KeyboardMenuState`, `show_keyboard_menu`, `open_from_keyboard`
- `split_panes`, `STACK_BELOW`
- `banner`, `show_choice_window`, `show_progress`, `show_popover`
- `dialog_frame`, `dialog_header`, `dialog_footer`
- `show_empty_state`, `section_header`, `compact_menu`, `menu_tool_button`
- `with_editor_scrollbars`, `editor_scroll_style`, `close_glyph`

`CommandPalette`, `Tree`, and `TabStrip` have builders in `components` itself.

## Layout rules

- Draw panels through `show_workbench`; do not change the fixed order.
- Give the central content at least 480 pt; the explorer and right panel yield below
  that.
- Use `ui.available_width()` and `horizontal_wrapped` instead of fixed widths for
  content that must survive 640×480.
- Keep the title bar, tab strip, and status bar at their `metrics` heights; they are
  measured against native window decorations.
