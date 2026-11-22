# Text editor

> File tree, tabs, an editor, dirty tracking, saving, and close prompts. Live page: <https://jigonzalez930209.github.io/egui-workbench/recipes/editor.html>

This recipe builds the classic workbench: an explorer tree, closable tabs, a text editor,
dirty markers, and Save / Discard / Cancel prompts.

## Model

```rust
use egui_workbench::app::Document;
use std::path::PathBuf;

struct Doc {
    path: PathBuf,
    text: String,
    dirty: bool,
}

impl Document for Doc {
    fn title(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled")
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}
```

Keep the text with `\n` line endings; convert on load/save if the file uses CRLF.

## Explorer

```rust
let view = TreeView {
    id_salt: "explorer",
    active: self.docs.active().map(|doc| &doc.path),
    light: self.theme.is_light(),
};

let events = show_tree(ui, &mut self.icons, &nodes, &mut self.tree, &view, |ui, icons, node| {
    if MenuItem::new("eye", "Open").show(ui, icons) {
        self.context_action = Some(Action::Open(node.key.clone()));
        ui.close();
    }
    MenuItem::new("edit", "Rename…").shortcut("F2").show(ui, icons);
    MenuItem::new("close", "Delete…").danger(true).show(ui, icons);
});

for event in events {
    match event {
        TreeEvent::Open(path) => self.open(path),
        TreeEvent::Toggle(path) => self.toggle(path),
        TreeEvent::Rename(path) => self.rename(path),
    }
}
```

`TreeView::active` follows the active document. Clicking a folder only toggles it; it
does not change the selection of the active file.

## Tabs

```rust
let tabs: Vec<TabItem<'_>> = self
    .docs
    .items()
    .iter()
    .map(|doc| TabItem {
        dirty: doc.dirty,
        ..TabItem::new(doc.title(), TabIcon::Glyph("lucide-file-text"))
    })
    .collect();

let event = TabStrip::new(&tabs, self.docs.active_index())
    .close_tooltip("Close")
    .controls_width(76.0)
    .show(ui, &mut self.icons, |ui, icons| {
        // new tab / close / mode controls
    });
```

Handle `Select` and `Close`. A dirty close goes through `close_tab`, not `remove`.

## Editor

```rust
with_editor_scrollbars(ui, |ui| {
    egui::ScrollArea::both()
        .id_salt("editor")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut doc.text)
                    .code_editor()
                    .frame(egui::Frame::NONE)
                    .desired_width(f32::INFINITY),
            );
            doc.dirty |= response.changed();
        });
});
```

`with_editor_scrollbars` installs the thin scrollbar style and restores it. `Frame::NONE`
removes the field frame so the editor merges with the panel background.

## Save

```rust
fn save_tab(&mut self, index: usize) -> Result<(), String> {
    let Some(doc) = self.docs.get_mut(index) else {
        return Err("No document is open.".to_owned());
    };
    std::fs::write(&doc.path, &doc.text).map_err(|error| error.to_string())?;
    doc.dirty = false;
    Ok(())
}
```

Return `Err` to keep the tab open and surface the message. Do not clear `dirty` on a
failed write.

## Status bar

```rust
StatusBar::new()
    .left(StatusItem::new(doc.title()))
    .left(StatusItem::new(if doc.dirty { "Modified" } else { "Saved" }))
    .left(StatusItem::new(format!("{} lines", doc.text.lines().count())))
    .right(StatusItem::new("UTF-8"))
    .right(StatusItem::new("LF"))
    .right(StatusItem::new("Plain Text"))
    .show(ui);
```

## Close flow

- Closing a clean tab removes it immediately.
- Closing a dirty tab stores `CloseRequest::Tab(index)` and shows the shared prompt.
- A window close request goes through `CloseGuard`.
- Save All saves every dirty tab; if one fails, keep the prompt open and show the error.

See [Documents and close guard](../app/documents.md).

## Preview

For a Markdown product, replace the right pane of `split_panes` with your renderer. The
shell does not care what the content is, and the split ratio is just an `f32` in the
model. The showcase uses a `TextEdit` in the left pane and plain text in the right pane;
the real product plugs its renderer there.

## Checklist

- [ ] Load files with a fallback for invalid UTF-8.
- [ ] Save atomically (temporary file + rename) if a partial write would corrupt a file.
- [ ] Track dirty state from the editor response, not from a timer.
- [ ] Use the shared close prompt for the window and every dirty tab.
- [ ] Keep the explorer selection in sync with the active document, without overwriting
      folder clicks.
