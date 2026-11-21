---
layout: home

hero:
  name: egui-workbench
  text: A desktop workbench for egui
  tagline: The shell, not the content. Themes, title bar, tabs, explorer, dialogs, charts, and the runtime every desktop app repeats.
  image:
    src: /workbench.png
    alt: The workbench in GitHub Light
  actions:
    - theme: brand
      text: Get started
      link: /guide/introduction
    - theme: alt
      text: Live demo
      link: /guide/live-demo
    - theme: alt
      text: Components
      link: /components/
    - theme: alt
      text: Charts
      link: /charts/overview

features:
  - icon: 🪟
    title: Native chrome, ready to use
    details: Undecorated window, resize bands, title bar and menus, tab strip, explorer, status bar, right and bottom panels.
  - icon: 🎨
    title: Nine themes, one style system
    details: GitHub, Dracula, and One Dark Pro tokens, plus density and corner presets that survive theme changes.
  - icon: 🧩
    title: Builders with shared variants
    details: Button, inputs, menus, cards, badges, dialogs, tree, palette, tabs, toasts, and an icon grid over 2,410 Lucide icons.
  - icon: 📈
    title: Real interactive charts
    details: Lines, areas, steps, scatter, bars, error bars, log axes, fits, live windows, selection, and SVG/PNG export.
  - icon: 🧠
    title: One Command, one handler
    details: Menus, the palette, shortcuts, and panels return events. Only the application mutates state.
  - icon: ✅
    title: Tested, accessible, documented
    details: Every component is driven with egui_kittest, contrast is checked against WCAG AA, and snapshots guard the pixels.
---

## See it

<LiveDemo
  scene="shell"
  height="460"
  alt="The workbench example with title bar, explorer, tabs, editor, and status bar"
  caption="The workbench running in your browser: open files, edit them, switch themes from View."
/>

## The idea

`egui-workbench` packages the visual core of an IDE-like desktop application as a Rust
crate. It draws the shell and returns what the user asked for; it never owns your
documents, performs file I/O, or chooses a storage format.

```rust
use egui_workbench::app::{WorkbenchApp, show_workbench};
use egui_workbench::egui;

fn ui(ui: &mut egui::Ui, app: &mut App) {
    let commands = show_workbench(ui, app);
    for command in commands {
        app.handle(ui.ctx(), command);
    }
}
```

Start with `cargo xtask new`, or read [Build your first app](/guide/first-app) to wire the
runtime by hand.
