# Overlays

> Dialogs, confirm and name prompts, progress windows, and anchored popovers. Live page: <https://jigonzalez930209.github.io/egui-workbench/components/overlays.html>

Dialogs are context-level composition: they take `&egui::Context`, paint a modal above
the panels, and own the frame, header, close glyph, backdrop, and Escape handling. The
body owns the footer.

## Dialog

```rust
let mut open = true;
let mut close = false;
Dialog::new("rename", "Rename document", &mut open).show(ctx, |ui| {
    TextField::new(&mut self.name).label("Name").show(ui);
    // Right-aligned primary + cancel; returns Submit or Cancel.
    if let Some(action) = dialog_footer(ui, "Rename", "Cancel", false) {
        if action == PromptAction::Submit {
            self.rename();
        }
        close = true;
    }
});
if close {
    open = false;
}
```

`show` returns the body value, or `None` while the dialog is closed. It closes on the ×,
Escape, or a backdrop click. The width is clamped to the window minus the frame padding
and a 16 pt gutter, so dialogs fit small windows.

| Item | Type | Default | Description |
|---|---|---|---|
| `Dialog::new(id, title, &mut open)` | `Dialog` | 420 pt | A modal controlled by the open flag. |
| `.width(pt)` | `f32` | 420.0 | Body width; clamped to the window. |
| `.close(tooltip)` | `&str` | "Close" | Close-button tooltip. |
| `.title_size(pt)` | `f32` | 17.0 | Title size. |
| `.show(ctx, body)` | `Option<R>` | — | Paint while open; returns the body value. |

## ConfirmDialog

A two-button confirmation. It returns `Option<PromptAction>`; `None` means neither button
was pressed.

```rust
if let Some(action) = ConfirmDialog::new("Delete item?", "This cannot be undone.")
    .confirm("Delete")
    .cancel("Cancel")
    .danger(true)
    .show(ctx)
    && action == PromptAction::Submit
{
    self.delete();
}
```

Defaults: confirm `"OK"`, cancel `"Cancel"`, `danger: false`. The primary button is on the
right. Escape cancels.

## NameDialog

A single-field prompt with optional description and validation text.

```rust
if let Some(action) = NameDialog::new("Rename", "Name", &mut self.name)
    .description("Use a short, unique name.")
    .confirm("Rename")
    .error(self.name_error.as_deref())
    .show(ctx)
    && action == PromptAction::Submit
{
    self.rename();
}
```

The field takes focus when the dialog opens; Enter confirms and Escape cancels.

## ProgressDialog

A centered progress window with a cancel action. Returns `true` when Cancel was clicked.

```rust
if ProgressDialog::new("Exporting", "Rendering the report…", self.progress)
    .cancel("Cancel")
    .show(ctx)
{
    self.cancel_export();
}
```

`progress` is clamped to `0.0..=1.0` by the renderer.

## Popover

A foreground panel anchored below the title bar, for search, quick settings, or the
keyboard menu.

```rust
Popover::new("search").width(420.0).show(ctx, |ui| {
    TextField::new(&mut self.query).hint("Search…").width(ui.available_width()).show(ui);
});
```

`show` returns the closure value. It is not modal; clicks outside close it.

## Standard frames

`components::blocks` exposes the pieces used by the shell:

- `dialog_frame(ctx)`, `dialog_header(ui, title, size, close)`,
  `dialog_footer(ui, confirm, cancel, danger) -> Option<PromptAction>`
- `show_choice_window(ctx, title, message, buttons, &mut open)`
- `show_progress(ctx, title, detail, progress, cancel)`
- `show_popover(ctx, id, width, body)`
- `banner(ui, color, message, actions)`

Use them when you need the exact workbench treatment without a builder.

## Behavior and accessibility

- `Esc` closes the top overlay; `Enter` runs the primary action.
- Focus starts in the first field of a name prompt.
- Every dialog has an accessible title and named buttons.
- The backdrop blocks interaction with the panels behind it.
- Destructive primaries use the danger token and readable text.
