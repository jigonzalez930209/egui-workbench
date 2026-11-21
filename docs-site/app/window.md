---
title: Window
description: Undecorated native options, resize edges, window actions, and title synchronization.
---

# Window

The workbench draws its own title bar, so the OS decorations are off. The window is
moved by dragging the title bar's center, resized from 4 pt bands on the west, east, and
south edges, and controlled through `WindowAction`.

## Native options

```rust
fn main() -> eframe::Result {
    let title = "My App";
    eframe::run_native(
        title,
        egui_workbench::native_options(title),
        Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx)))),
    )
}
```

`native_options(title)` uses:

| Setting | Value |
|---|---|
| Inner size | 980 × 720 |
| Minimum inner size | 640 × 420 |
| Decorations | off |
| Renderer | Glow |
| Title | `title` |

Use `native_options_sized(title, size, min_size)` for a different initial geometry.

## Window actions

```rust
use egui_workbench::{WindowAction, apply_window_action};

apply_window_action(ctx, WindowAction::Minimize);
apply_window_action(ctx, WindowAction::ToggleMaximize);
apply_window_action(ctx, WindowAction::Close);
```

| Action | Viewport command |
|---|---|
| `Minimize` | `Minimized(true)` |
| `ToggleMaximize` | `Maximized(!is_maximized(ctx))` |
| `Close` | `Close` |

`Close` arrives back as a close request, so `CloseGuard` can still cancel it. `is_maximized`
reads the current viewport state.

## Resize edges

```rust
pub fn handle_window_edges(ctx: &Context) {
    // primary press within RESIZE_EDGE of the west/east/south edge
    // sends BeginResize(SouthWest | SouthEast | West | East | South)
}
```

`show_workbench` calls it before drawing the panels. The north edge belongs to the title
bar, which also owns the drag region. `RESIZE_EDGE` is 4 pt.

## Title bar

```rust
let mut bar = TitleBar::new(self.docs.active_title().unwrap_or("My App"));
bar.dirty = self.docs.active().is_some_and(|doc| doc.is_dirty());
bar.maximized = is_maximized(ui.ctx());
bar.logo = "logo"; // default
bar.labels = WindowLabels { minimize: "Minimize", maximize: "Maximize", restore: "Restore", close: "Close" };

let result = show_title_bar(ui, &mut icons, &bar, |ui, icons| {
    // menus and history on the left
}, |ui, icons| {
    // actions on the right
});
if let Some(action) = result.window {
    apply_window_action(ui.ctx(), action);
}
```

The title bar is 34 pt. It paints the panel fill, the product mark, the title with a dirty
bullet, the menus, the drag region, and the window controls. Double-clicking the center
toggles maximize; dragging it starts an OS move; its edges do not start a move.

`TitleBarOutput` also reports `pill_clicked` when the optional `Pill` is clicked. A pill
is useful for "Open Folder" or a workspace name.

## Title synchronization

```rust
fn sync_window_title(&mut self, ctx: &egui::Context) {
    let title = match self.docs.active() {
        Some(doc) => format!("{}{} — My App", doc.title(), if doc.is_dirty() { " •" } else { "" }),
        None => "My App".to_owned(),
    };
    if title != self.window_title {
        set_window_title(ctx, &title);
        self.window_title = title;
    }
}
```

`set_window_title` sends `ViewportCommand::Title`. Cache the last value to avoid a command
every frame.

## Clear color

```rust
fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
    self.theme.clear_color()
}
```

Returning the theme background avoids a black flash while resizing or starting the
window.

## macOS, Windows, and Linux

- `Modifiers::COMMAND` is Cmd on macOS and Ctrl elsewhere.
- The close button hover uses `#C42B1C` with a white glyph, matching Windows/GNOME.
- The window control cell is 44 × 32 pt; the title bar keeps its fixed 34 pt height to
  align with native decorations.
- The app icon is supplied through `eframe::NativeOptions`; the template ships a neutral
  SVG mark for the title bar.

## WebAssembly

`native_options` and `native_options_sized` are native-only (`#[cfg(not(target_arch =
"wasm32"))]`). On the web, use `eframe::WebRunner` with `eframe::WebOptions`; the `web/`
crate in this repository is a complete entry point:

```rust
let runner = eframe::WebRunner::new();
runner.start(canvas, eframe::WebOptions::default(), Box::new(|cc| {
    Ok(Box::new(App::new(&cc.egui_ctx)))
})).await
```

See [Live demo](/guide/live-demo). The shell, themes, and components are the same; only
the window and renderer layer changes.

## Multi-viewport

The crate draws inside the root viewport. For a second window, build a viewport with
`egui::ViewportBuilder`, give it a distinct `ViewportId`, and draw its own root `Ui`.
Panel ids are stable strings, so keep one shell per viewport or namespace the ids in your
own fork.
