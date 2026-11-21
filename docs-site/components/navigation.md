---
title: Navigation
description: Tab strips, the explorer tree, and the command palette.
---

# Navigation

<LiveDemo
  scene="shell"
  height="420"
  alt="The workbench with its explorer tree, tab strip, and command palette"
  caption="Click files in the tree, switch and close tabs, use arrow keys in the tree, and press Ctrl+P for the palette."
/>

## TabStrip

`TabStrip` paints its whole `Ui::max_rect()`, so use it inside a fixed-height panel or a
sized container. In `show_workbench` it already runs in a 34 pt top panel.

```rust
let tabs: Vec<TabItem<'_>> = docs
    .items()
    .iter()
    .map(|doc| TabItem {
        dirty: doc.dirty,
        ..TabItem::new(&doc.name, TabIcon::Glyph("files"))
    })
    .collect();

let event = TabStrip::new(&tabs, docs.active_index())
    .close_tooltip("Close")
    .controls_width(276.0)
    .show(ui, &mut icons, |ui, icons| {
        if IconButton::new("plus", "New tab")
            .size(IconButtonSize::Tool)
            .show(ui, icons)
            .clicked()
        {
            // record the intent for the caller
        }
    });

match event {
    Some(TabEvent::Select(index)) => docs.set_active(index),
    Some(TabEvent::Close(index)) => { docs.remove(index); }
    None => {}
}
```

| Feature | Behavior |
|---|---|
| Width | 96–212 pt (label + 58 pt) |
| Icon | 16 pt at x + 15; `TabIcon::{None, Glyph, Texture}` |
| Label | Starts at x + 28, truncated with `…` |
| Dirty | 3 pt dot when inactive; close button when active or hovered |
| Warning / missing | Warning color / strike-through |
| Scrolling | Mouse wheel scrolls horizontally; the active tab is revealed |
| Close | Close button or middle click |

`controls_width` is the space reserved on the right for your controls (the template uses
76 pt). The closure runs right-to-left.

## Tree

A keyboard-navigable explorer tree. Rows are 21 pt with 12 pt indentation; each row has a
chevron slot, a 16 pt icon, and a 12.5 pt label.

```rust
use egui_workbench::components::{RowIcon, Tree, TreeNode, TreeState};

let nodes = vec![
    TreeNode::folder(
        "src",
        "src",
        RowIcon::Glyph("folder".into()),
        true,
        vec![
            TreeNode::leaf("src/main.rs", "main.rs", RowIcon::Glyph("lucide-file-code".into())),
            TreeNode::leaf("src/lib.rs", "lib.rs", RowIcon::Glyph("lucide-file-code".into())),
        ],
    ),
    TreeNode::leaf("README.md", "README.md", RowIcon::Glyph("lucide-file-text".into())),
];

let events = Tree::new(&nodes, &mut self.tree, "explorer")
    .active(self.docs.active().map(|doc| &doc.path))
    .light(self.is_light())
    .show(ui, &mut self.icons, |ui, icons, node| {
        if MenuItem::new("eye", "Open").show(ui, icons) {
            // record the chosen node and close the menu
        }
        ui.separator();
        if MenuItem::new("edit", "Rename…").show(ui, icons) {
            // record
        }
    });

for event in events {
    match event {
        TreeEvent::Toggle(key) => self.toggle(key),
        TreeEvent::Open(key) => self.open(key),
        TreeEvent::Rename(key) => self.rename(key),
    }
}
```

| Event | When |
|---|---|
| `Toggle(key)` | Click, Enter, Left, or Right on a folder |
| `Open(key)` | Click or Enter on a leaf |
| `Rename(key)` | F2 on the focused row |

State: `TreeState<K>` holds `selected`, `scroll_to_selected`, and the last active key. The
selection follows a *change* of active document, so clicking a folder is not overwritten
on the next frame.

`RowIcon` alternatives: `None`, `Glyph(String)`, `Texture(TextureId)`,
`MaterialFile(PathBuf)`, and `MaterialFolder(String)` when the `material-icons` feature is
enabled.

## CommandPalette

A 560 pt modal with a filter field, up to 12 rows, and a highlighted first match.
Matching is a case-insensitive substring test on label and detail.

```rust
let labels = PaletteLabels {
    title: "Command Palette",
    hint: "Type to filter…",
    empty: "No matches",
    close: "Close",
};
let items = vec![
    PaletteItem { label: "Save".into(), detail: "Ctrl+S".into(), value: Command::Save },
    PaletteItem { label: "Close tab".into(), detail: "Ctrl+W".into(), value: Command::CloseTab },
];
let mut state = PaletteState::default();
state.open();
if let Some(command) = CommandPalette::new(&mut state, labels, &items).show(ctx) {
    handle(command);
}
```

`PaletteState` is `{ open: bool, query: String }`; `open()` sets the flag and clears the
query. `filter_palette(items, query)` is public if you need the list elsewhere.

## Keyboard menu

`show_keyboard_menu` is the F10/Alt menu, with grouped entries. It is a separate,
context-level overlay:

```rust
let groups = vec![MenuGroup {
    label: "File".to_owned(),
    entries: vec![
        MenuEntry { label: "New tab".to_owned(), value: Command::NewTab },
        MenuEntry { label: "Save".to_owned(), value: Command::Save },
    ],
}];
if let Some(command) = show_keyboard_menu(ctx, &mut state, &groups, "Enter runs · Esc closes") {
    handle(command);
}
```

`open_from_keyboard(ctx, &mut state, &letters)` opens it when F10 or an Alt+letter chord
is pressed.

## Accessibility

- Tabs announce `Role::Tab` and their label; close buttons include the tab name.
- Tree rows are tree items with expanded state; the active document is announced.
- Palette rows are list options; the first is highlighted and runs on Enter.
- Every navigation surface is usable with arrows, Enter, and Escape.
