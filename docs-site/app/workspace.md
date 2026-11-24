---
title: Workspace layouts
description: Serialized editor groups, splits, tabs, named layouts, focus tools, and the activity bar.
---

# Workspace layouts

0.6 turns the fixed five-region shell into layouts as data. `Layout` is a serializable
tree of editor groups and splits; the window, a palette command, or a host reads the same
tree.

## The tree

```rust
use egui_workbench::app::{EditorGroupId, Layout, LayoutTab, SplitDirection};

let mut layout = Layout::with_tabs(vec![
    LayoutTab::new("a", "analysis.rs"),
    LayoutTab::new("b", "notes.md").pinned(),
]);
let right = layout.split(EditorGroupId(0), SplitDirection::Horizontal).expect("split");
assert_eq!(layout.groups(), vec![EditorGroupId(0), right]);
assert_eq!(layout.active, right);
```

A `LayoutNode` is a `Group(EditorGroup)` or a `Split { direction, ratio, first, second }`.
`ratio` is the fraction given to `first`; the UI drags it.

## Tabs and groups

| Operation | API |
|---|---|
| Move a tab between groups | `move_tab(from, index, to, target)` |
| Close a tab and remember it | `close_tab(group, index)` |
| Reopen the last closed tab | `reopen_closed()` |
| Pin a tab | `pin_tab(group, index, true)` |
| Split | `split(group, direction)` |
| Close a group | `close_group(id)` |
| Focus | `focus(id)` |
| Maximize / restore | `maximize(Some(id))` / `maximize(None)` |
| Zen mode | `set_zen(true)` |

`closed` keeps the last 20 tabs; the overflow menu can show them. The last group cannot be
closed.

## Keyboard for every drag

`LayoutAction` is the keyboard equivalent of every pointer operation. Bind these to the
0.4 registry commands:

```rust
use egui_workbench::app::{Layout, LayoutAction};

let mut layout = Layout::single();
assert!(layout.apply(LayoutAction::SplitRight));
assert!(layout.apply(LayoutAction::FocusNext));
assert!(layout.apply(LayoutAction::ToggleZen));
```

The actions are `SplitRight`, `SplitDown`, `FocusNext`, `FocusPrevious`,
`MoveTabToNextGroup`, `CloseGroup`, `ToggleMaximize`, `ToggleZen`, and `ReopenClosed`.
AccessKit roles are `Tab` for the group tabs and the activity-bar items.

## Named layouts and persistence

`Workspace` stores the current `layout`, `named_layouts`, and `active_layout` next to the
documents and panel sizes. Save a layout under a name and switch to it from the palette:

```rust
use egui_workbench::app::Workspace;

let mut workspace = Workspace::new();
workspace.save_layout("Review");
assert!(workspace.switch_layout("Review"));
```

`Workspace::from_json` migrates a 0.5 file (which had no layout) to a single empty group
and fills the fields with `serde(default)`. The layout file has its own version and
`Layout::from_json` rejects a newer one.

## Floating and detached

The example `examples/workspace` builds three editor groups from one `Layout`, opens a
floating `egui::Window`, and opens a native detached viewport with
`egui::Context::show_viewport_immediate`. In the browser, the viewport is embedded in the
canvas. `CloseGuard` still owns the close request for every window.

## Activity bar

`ActivityBar` is a 32 pt vertical rail: one icon per view group, a 2 pt active marker, an
optional badge circle, tooltips, and AccessKit `Tab` roles.

```rust
use egui_workbench::components::{ActivityBar, ActivityBarItem};

let items = [
    ActivityBarItem::new("folder", "Explorer"),
    ActivityBarItem::new("search", "Search"),
    ActivityBarItem::new("settings", "Settings").badge("1"),
];
// let selected = ActivityBar::new(&items, 0).show(ui, &mut icons);
```

## Compatibility

`show_workbench` and `WorkbenchApp` keep their five-region one-group behavior. The layout
API is additive; an existing application keeps compiling and can adopt groups one screen
at a time.
