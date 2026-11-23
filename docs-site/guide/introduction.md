---
title: Introduction
description: What egui-workbench is, what it deliberately leaves out, and how its layers fit together.
---

# Introduction

`egui-workbench` is a Rust library for building desktop applications with the visual
language of an IDE: a custom title bar, menus, document tabs, an explorer, status bars,
dialogs, a command palette, and a theme system. It is built on [`egui`](https://github.com/emilk/egui)
0.37 and `eframe` from crates.io, with no forks or patches.

The crate is the shell, not the content. The model, the file format, the editor, and the
renderer stay in your application. That boundary is what makes the shell reusable: a
Markdown editor, a log viewer, a data tool, and a plot dashboard can share every pixel
of chrome and nothing of their domain.

## What you get

- **Window and chrome.** An undecorated window with resize edges, a 34 pt title bar with
  text menus and window controls, a document tab strip, a status bar, and an explorer.
- **Optional panels.** A resizable inspector on the right and output/terminal panel at
  the bottom, both responsive on narrow windows.
- **Components.** A shadcn-style builder layer: buttons, icon buttons, text fields,
  text areas, switches, checkboxes, selects, steppers, segmented controls, labels,
  badges, banners, cards, separators, menus, popovers, tab strips, trees, palettes,
  dialogs, toasts, and an icon grid.
- **Charts.** Interactive 2D and 3D plots with series, axes, fits, selection, live
  windows, SVG/PNG export, pie/donut, candlesticks, heatmaps, radar, gauges, waterfalls,
  13 legend hover animations, and a streaming direction arrow. `Plot3D` adds an orbit
  camera, surfaces, and 3D bars.
- **Data viewer.** Excel workbooks with sheet tabs, CSV/TSV, and virtualized plain-text
  reading through `components::viewer::FileViewer`.
- **A runtime.** `ThemeState`, `Documents<T>`, `Keymap<C>`, `CloseGuard`, and
  `show_workbench` implement the plumbing every workbench repeats.
- **Assets.** 2,410 Lucide icons and the VS Code Material file/folder icon set, resolved
  by name through one tinted texture cache.
- **Nine themes.** GitHub Light, GitHub Dark, GitHub Dark Dimmed, Dracula, Dracula Soft,
  Alucard, One Dark Pro, One Dark Pro Darker, and One Dark Pro Light.

<LiveDemo
  scene="components"
  height="480"
  alt="Component board with buttons, fields, switches, badges, and a card"
  caption="The component board, running live. Everything is clickable."
/>

## What it is not

- **Not a framework.** There is no virtual DOM, no retained widget tree, and no hidden
  component state. egui is immediate mode; components receive state every frame.
- **Not an editor.** The crate has no editor format, syntax highlighter, or document
  renderer. The starter's editor is a `TextEdit` and its preview is plain text. The data
  viewer reads tabular files, but it is a viewer, not a spreadsheet editor.
- **Not a storage layer.** `app` has no I/O and picks no config format. The template
  shows one settings implementation you can replace.
- **Not opinionated about commands.** `Command` is your enum. `PaletteItem<T>`,
  `MenuGroup<T>`, `TreeNode<K>`, `Keymap<C>`, and `Documents<T>` are generic over it.

## Core principles

1. **The shell owns presentation only.** Components take translated strings and state
   snapshots, and return events. File operations, persistence, and lifecycle decisions
   belong to the application.
2. **One `Command` enum, one handler.** Menus, the command palette, the keyboard menu,
   shortcuts, and panel buttons produce the same command. Every action is reachable by
   pointer, keyboard, and search.
3. **No literal colors or sizes.** Colors come from `theme` tokens or egui visuals;
   chrome sizes come from `metrics`. The two documented exceptions are the red
   close-button hover and the white icon tint.
4. **Accessible by construction.** Every custom-painted widget declares an AccessKit role
   and name. Contrast follows WCAG 2.2 AA, and the tests enforce it.
5. **No idle repaint.** The UI repaints on input, animation, or an explicit wake from a
   worker.

## A frame in the runtime

eframe 0.37 splits each frame into `logic` and `ui`:

1. `App::logic` syncs the theme, checks the close guard, updates the window title, and
   consumes a shortcut when no modal is open.
2. `App::ui` calls `show_workbench`, which draws the panels in a fixed order and collects
   one `Vec<Command>`.
3. The application applies every command in a single `handle` function.
4. Toasts and other overlays are painted last, above the shell.

```rust
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
        if self.guard.should_prompt(ctx, self.docs.any_dirty()) {
            self.close_prompt = true;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        for command in show_workbench(ui, self) {
            self.handle(&ctx, command);
        }
    }
}
```

## Next steps

- Try the real component catalog in the browser: [Live demo](/guide/live-demo) is the
  same Rust app compiled to WebAssembly.
- [Install the crate](/guide/installation): one line in `Cargo.toml`.
- Generate a working application with [Quick start](/guide/quick-start).
- Learn the contract in [Core concepts](/guide/concepts), then browse the
  [component reference](/components/).
