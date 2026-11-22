# Feedback

> Sonner-style toasts with kinds, positions, actions, loading-to-success updates, and timers. Live page: <https://jigonzalez930209.github.io/egui-workbench/components/feedback.html>

`Toasts` is a `sonner`-style notification stack. Keep one in the application state, push
toasts from anywhere, and call `show` once per frame after the panels.

```rust
use egui_workbench::components::{Toast, ToastKind, ToastPosition, Toasts};

struct App {
    toasts: Toasts,
    icons: Icons,
}

impl App {
    fn new() -> Self {
        Self {
            // Clear the 24 pt status bar in the bottom-right corner.
            toasts: Toasts::new().offset(0.0, egui_workbench::metrics::STATUS_HEIGHT),
            icons: Icons::new(),
        }
    }
}
```

## Kinds

| Kind | Icon | Color |
|---|---|---|
| `Info` (default) | info | Link token |
| `Success` | circle check | Success token |
| `Warning` | triangle alert | Warning token |
| `Error` | circle x | Danger token |
| `Loading` | spinner | Muted text; no timeout until updated |

```rust
toasts.success("Saved");
toasts.warning("The file changed on disk");
toasts.error("Could not save");
let id = toasts.loading("Exporting…");
```

## Actions and descriptions

```rust
let id = toasts.add(
    Toast::new(ToastKind::Error, "Could not save")
        .description("Permission denied")
        .action("Retry")
        .duration(8.0),
);
```

A toast with an action has an `Action` button on the right. Clicking it reports
`ToastEvent::Action(id)` and closes the toast.

## Loading to success

`loading` returns a `ToastId`. Replace the toast in place; its timer restarts and it
keeps its position in the stack:

```rust
let id = toasts.loading("Exporting report…");
// ... later, on the worker result:
toasts.update(id, Toast::new(ToastKind::Success, "Exported").description("report.pdf"));
```

`update` returns `false` when the toast is already gone.

## Rendering

```rust
for event in toasts.show(ctx, &mut icons) {
    match event {
        ToastEvent::Action(id) => retry(id),
        ToastEvent::Closed(id) => {}
        ToastEvent::Expired(id) => {}
    }
}
```

| Item | Type | Default | Description |
|---|---|---|---|
| `Toasts::new()` | `Toasts` | bottom-right, 356 pt, 4 visible | An empty stack. |
| `.position(p)` | `ToastPosition` | BottomRight | Corner or edge center. |
| `.set_position(p)` | `ToastPosition` | — | Change the corner at run time. |
| `.width(pt)` | `f32` | 356.0 | Minimum 200 pt; shrinks on narrow windows. |
| `.max_visible(n)` | `usize` | 4 | Older toasts wait in the queue. |
| `.offset(x, y)` | `f32, f32` | 0, 0 | Clear the title bar or status bar. |
| `.add(toast)` | `Toast` | — | Queue a toast and return its id. |
| `.info/success/warning/error(title)` | `&str` | — | Shorthands for the four timed kinds. |
| `.loading(title)` | `&str` | — | Persistent spinner until updated or dismissed. |
| `.update(id, toast)` | `ToastId, Toast` | — | Replace in place; returns false when gone. |
| `.dismiss(id)` | `ToastId` | — | Close with the exit animation. |
| `.dismiss_all()` | `—` | — | Close every toast. |
| `.show(ctx, icons)` | `Vec<ToastEvent>` | — | Paint above every panel and report events. |

## Toast builder

| Method | Default | Effect |
|---|---|---|
| `Toast::new(kind, title)` | 4 s timeout | A notification; `Loading` has no timeout |
| `.description(text)` | None | Second muted line that wraps |
| `.action(label)` | None | Action button |
| `.duration(seconds)` | 4.0 | Clamped to at least 0.5 s |
| `.persistent()` | false | Keep until closed or updated |
| `.closable(bool)` | true | Show or hide the × |

## Behavior

- Default timeout is 4 s. Hovering any toast pauses every timer.
- Enter and exit animations follow reduced motion.
- Nothing repaints while the stack is empty; while toasts are visible, the frame wakes
  only for the next timeout or animation step.
- `max_visible` queues the rest; they appear as slots free up.
- The stack never covers the title bar or status bar when `.offset` is set.
