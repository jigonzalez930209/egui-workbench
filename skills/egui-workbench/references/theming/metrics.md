# Size metrics

> Every chrome size constant in egui_workbench::metrics and where it is used. Live page: <https://jigonzalez930209.github.io/egui-workbench/theming/metrics.html>

Chrome sizes live in one module so the title bar, tabs, explorer, dialogs, and status bar
cannot drift apart. Every value is in egui points (1 pt = 1 px at 100 % scale).

```rust
use egui_workbench::metrics::*;
```

## Heights

| Constant | Value | Used by |
|---|---|---|
| `TITLE_HEIGHT` | 34.0 | Title bar panel |
| `TAB_HEIGHT` | 34.0 | Tab strip panel and `TabStrip` |
| `STATUS_HEIGHT` | 24.0 | Status bar panel |
| `TREE_ROW_HEIGHT` | 21.0 | Explorer rows |
| `CONTROL_HEIGHT` | 30.0 | Text fields, dialog buttons, combos, steppers |

`WorkbenchStyle` scales `TREE_ROW_HEIGHT`, `CONTROL_HEIGHT`, `MENU_ROW_HEIGHT`, and
`LIST_ROW_HEIGHT` by the density factor. Title bar, tab strip, and status bar keep their
fixed heights; they match native window decorations and hit areas.

## Widths and indentation

| Constant | Value | Used by |
|---|---|---|
| `TAB_STRIP_PADDING` | 8.0 | Left inset of the first tab |
| `SIDEBAR_WIDTH` | 260.0 | Default explorer width |
| `TREE_INDENT` | 12.0 | Indentation per tree depth |
| `CHEVRON_SIZE` | 14.0 | Expand/collapse slot |
| `WINDOW_BUTTON_WIDTH` | 44.0 | Minimize / maximize / close cell |
| `WINDOW_BUTTON_HEIGHT` | 32.0 | Window control cell |

`SIDEBAR_MIN_WIDTH` (160.0) and `CONTENT_MIN_WIDTH` (480.0) live in `app`; the shell
shrinks the explorer to keep the content usable and never lets the explorer exceed half
the window. `RIGHT_PANEL_MIN_WIDTH` is 180.0 and `BOTTOM_PANEL_MIN_HEIGHT` is 72.0.

## Buttons and rows

| Constant | Value | Used by |
|---|---|---|
| `ACTION_BUTTON` | 28.0 | Title-bar action buttons |
| `TOOL_BUTTON` | 26.0 | Tab-strip tool buttons |
| `SMALL_BUTTON` | 21.0 | Explorer header buttons |
| `MENU_ROW_HEIGHT` | 22.0 | Context-menu rows |
| `LIST_ROW_HEIGHT` | 26.0 | Palette and keyboard-menu rows |
| `ICON_SMALL` | 14.0 | Dense menus, steppers, window buttons |
| `ICON_MEDIUM` | 16.0 | Title-bar actions, tabs, tree rows |
| `ICON_LARGE` | 20.0 | Product mark |
| `RESIZE_EDGE` | 4.0 | Resize band of the undecorated window |

## Builder sizes

`components::Size` is the control scale:

| Size | Height | Text | Min width | Padding |
|---|---|---|---|---|
| `Sm` | 24.0 | 11.5 | 0.0 | 8 × 2 |
| `Md` | 30.0 | 13.0 | 88.0 | 12 × 4 |
| `Lg` | 36.0 | 14.0 | 104.0 | 16 × 6 |

`IconButtonSize` is the square icon scale:

| Size | Side | Glyph | Radius |
|---|---|---|---|
| `Action` | 28.0 | 16.0 | 5 |
| `Tool` | 26.0 | 18.0 | 5 |
| `Small` | 21.0 | 16.0 | 4 |

## Style-derived heights

`WorkbenchStyle` methods return the effective values:

```rust
let style = style::get(ctx);
style.control_height();          // scaled 30
style.size_height(Size::Lg);     // scaled 36
style.menu_row_height();         // scaled 22
style.list_row_height();         // scaled 26
style.tree_row_height();         // scaled 21
```

## Why the chrome does not scale

Density changes controls and rows, not the title bar, tab strip, or status bar. Those
three surfaces align with the operating system's window buttons and with the visual
rhythm of the app icon; scaling them independently makes the window controls float or
clip. If you need a different chrome scale, fork the panel heights rather than changing
`metrics` at runtime.
