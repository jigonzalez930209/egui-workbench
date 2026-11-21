---
title: Feedback
description: Sonner-style toasts with kinds, positions, actions, loading-to-success updates, and timers.
---

# Feedback

<LiveDemo
  scene="toasts"
  height="380"
  alt="A toast with an action button"
  caption="Every toast kind, an action with undo, positions, and a loading toast updated in place."
/>

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

<ApiTable :rows="[
  { name: 'Toasts::new()', type: 'Toasts', default: 'bottom-right, 356 pt, 4 visible', description: 'An empty stack.' },
  { name: '.position(p)', type: 'ToastPosition', default: 'BottomRight', description: 'Corner or edge center.' },
  { name: '.set_position(p)', type: 'ToastPosition', default: '—', description: 'Change the corner at run time.' },
  { name: '.width(pt)', type: 'f32', default: '356.0', description: 'Minimum 200 pt; shrinks on narrow windows.' },
  { name: '.max_visible(n)', type: 'usize', default: '4', description: 'Older toasts wait in the queue.' },
  { name: '.offset(x, y)', type: 'f32, f32', default: '0, 0', description: 'Clear the title bar or status bar.' },
  { name: '.add(toast)', type: 'Toast', default: '—', description: 'Queue a toast and return its id.' },
  { name: '.info/success/warning/error(title)', type: '&str', default: '—', description: 'Shorthands for the four timed kinds.' },
  { name: '.loading(title)', type: '&str', default: '—', description: 'Persistent spinner until updated or dismissed.' },
  { name: '.update(id, toast)', type: 'ToastId, Toast', default: '—', description: 'Replace in place; returns false when gone.' },
  { name: '.dismiss(id)', type: 'ToastId', default: '—', description: 'Close with the exit animation.' },
  { name: '.dismiss_all()', type: '—', default: '—', description: 'Close every toast.' },
  { name: '.show(ctx, icons)', type: 'Vec&lt;ToastEvent&gt;', default: '—', description: 'Paint above every panel and report events.' }
]" />

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
