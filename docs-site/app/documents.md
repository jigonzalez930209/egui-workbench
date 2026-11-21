---
title: Documents and close guard
description: Documents<T>, the Document trait, dirty tabs, CloseGuard, and the Save / Discard / Cancel prompt.
---

# Documents and close guard

## Document

Implement one trait for your tab model:

```rust
use egui_workbench::app::Document;

struct Doc {
    title: String,
    body: String,
    dirty: bool,
}

impl Document for Doc {
    fn title(&self) -> &str {
        &self.title
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}
```

`is_dirty` defaults to `false`. The shell reads only `title()` and `is_dirty()`; the rest
of the model is yours.

## Documents

`Documents<T>` is a vector plus an active index. It performs no I/O and knows nothing
about paths.

```rust
let mut docs = Documents::with_items(vec![Doc::new("a"), Doc::new("b")]);
assert_eq!(docs.active_title(), Some("a"));
docs.select_next();
assert_eq!(docs.active_title(), Some("b"));
docs.remove(0);
assert_eq!(docs.active_index(), 0);
```

| Method | Behavior |
|---|---|
| `new()` / `with_items(vec)` | Empty list / list with index 0 active |
| `len()` / `is_empty()` | Number of documents |
| `items()` / `items_mut()` | The whole slice in tab order |
| `active_index()` | Active index; `0` when empty |
| `active()` / `active_mut()` | The active document |
| `get(i)` / `get_mut(i)` | By index |
| `set_active(i)` | Returns `false` when out of range |
| `push(item)` | Append without selecting |
| `push_active(item)` | Append and select |
| `insert(i, item)` | Insert without selecting |
| `remove(i)` | Remove and keep a sensible active document |
| `close_active()` | Remove the active document |
| `select_next()` / `select_previous()` | Wrap around |
| `select_by(predicate)` | Select the first match |
| `clear()` | Remove everything |
| `active_title()` | `title()` of the active document |
| `any_dirty()` | Any `is_dirty()` |

### Removal semantics

- Removing **before** the active document shifts the index, so the same document stays
  active.
- Removing the **active** document selects its successor, or the new last document.
- Removing the last document leaves the list empty and the active index at 0.

### Tab order

`items()` is the tab order. Render it directly in the tab strip and the navigator; do not
keep a second order list.

## Dirty state

The application decides what dirties a document:

```rust
let response = TextArea::new(&mut doc.body).show(ui);
doc.dirty |= response.changed();
```

When saving succeeds, clear the flag. When saving fails, keep it set and surface the
error (status bar, toast, or close prompt).

## CloseGuard

eframe reports a close request before the window closes. `CloseGuard` cancels it while
documents are dirty, so the application can show its own prompt.

```rust
pub struct App {
    guard: CloseGuard,
    close_request: Option<CloseRequest>,
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.guard.should_prompt(ctx, self.docs.any_dirty()) {
            self.close_request = Some(CloseRequest::Window);
        }
    }
}
```

| Method | Purpose |
|---|---|
| `new()` | A guard that prompts while dirty |
| `is_allowed()` | Whether the next close request bypasses the prompt |
| `should_prompt(ctx, has_unsaved)` | Cancel the request and return `true` when the app must prompt |
| `allow_close(ctx)` | Let the next request through and close now |
| `reset()` | Return to prompting |

`should_prompt` is the only place that sends `CancelClose`.

## The prompt

`show_choice_window` draws the standard Save / Discard / Cancel prompt:

```rust
let mut open = true;
match show_choice_window(
    ctx,
    "Save changes?",
    "Do you want to save before closing?",
    &["Save", "Discard", "Cancel"],
    &mut open,
) {
    Some(0) => {
        if self.save_all() {
            self.guard.allow_close(ctx);
        }
    }
    Some(1) => self.guard.allow_close(ctx),
    Some(_) => self.guard.reset(),
    None if !open => self.guard.reset(),
    None => {}
}
```

Escape maps to the last button (Cancel). A backdrop click closes the window without a
choice, so `None` with `open == false` must reset the guard.

## Per-tab close

The same pattern works for one tab. Keep a request enum:

```rust
enum CloseRequest {
    Window,
    Tab(usize),
}
```

When the user closes a dirty tab, store `CloseRequest::Tab(index)` and show the prompt.
On Save, save that tab and remove it; on Discard, remove it; on Cancel, reset the guard.
On a failed save, keep the prompt open and show the error.

## Window title

```rust
let title = match self.docs.active() {
    Some(doc) => format!("{}{} — My App", doc.title(), if doc.is_dirty() { " •" } else { "" }),
    None => "My App".to_owned(),
};
if title != self.window_title {
    set_window_title(ctx, &title);
    self.window_title = title;
}
```

Cache the last title; `set_window_title` sends a viewport command every time it is called.

## Rules

- Keep the close policy in the application; the library never decides lifecycle.
- Do not call `std::process::exit` from a shortcut; send `WindowAction::Close`.
- Do not clear dirty flags on save failure.
- Reset the guard when the prompt is cancelled or dismissed, or the next close request
  will pass through.
