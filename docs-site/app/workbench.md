---
title: Workbench layout
description: WorkbenchApp, the fixed panel order, optional right and bottom panels, responsive widths, and the command flow.
---

# Workbench layout

<LiveDemo
  scene="shell"
  height="460"
  alt="The full workbench with title bar, explorer, tabs, editor, and status bar"
  caption="The shell draws the panels; the application draws the content and handles the commands."
/>

## WorkbenchApp

Implement one trait and let `show_workbench` lay out the shell:

```rust
use egui_workbench::app::{WorkbenchApp, show_workbench};
use egui_workbench::egui;

impl WorkbenchApp for App {
    type Command = Command;

    fn sidebar_visible(&self) -> bool { self.sidebar }
    fn tab_strip_visible(&self) -> bool { true }
    fn status_bar_visible(&self) -> bool { true }

    fn title_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> { /* ... */ Vec::new() }
    fn sidebar(&mut self, ui: &mut egui::Ui) -> Vec<Command> { /* ... */ Vec::new() }
    fn tab_strip(&mut self, ui: &mut egui::Ui) -> Vec<Command> { /* ... */ Vec::new() }
    fn central(&mut self, ui: &mut egui::Ui) -> Vec<Command> { /* ... */ Vec::new() }
    fn status_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> { /* ... */ Vec::new() }
    fn overlays(&mut self, ui: &mut egui::Ui) -> Vec<Command> { /* ... */ Vec::new() }
}
```

Every drawing method returns the commands it produced. The shell never applies them.

<ApiTable :rows="[
  { name: 'title_bar_visible()', type: 'bool', default: 'true', description: 'Show the 34 pt title bar.' },
  { name: 'sidebar_visible()', type: 'bool', default: 'false', description: 'Show the left explorer.' },
  { name: 'sidebar_width()', type: 'f32', default: '260.0', description: 'Preferred width; egui remembers later resizes.' },
  { name: 'tab_strip_visible()', type: 'bool', default: 'false', description: 'Show the 34 pt tab strip.' },
  { name: 'status_bar_visible()', type: 'bool', default: 'false', description: 'Show the 24 pt status bar.' },
  { name: 'right_panel_visible()', type: 'bool', default: 'false', description: 'Show the right inspector.' },
  { name: 'right_panel_width()', type: 'f32', default: '280.0', description: 'Preferred right-panel width.' },
  { name: 'bottom_panel_visible()', type: 'bool', default: 'false', description: 'Show the bottom output panel.' },
  { name: 'bottom_panel_height()', type: 'f32', default: '220.0', description: 'Preferred bottom-panel height.' },
  { name: 'title_bar / sidebar / tab_strip / central / status_bar', type: 'fn(&mut Ui) -&gt; Vec&lt;Command&gt;', default: 'empty', description: 'Draw a panel and return its commands.' },
  { name: 'right_panel / bottom_panel', type: 'fn(&mut Ui) -&gt; Vec&lt;Command&gt;', default: 'empty', description: 'Draw the optional panels.' },
  { name: 'overlays', type: 'fn(&mut Ui) -&gt; Vec&lt;Command&gt;', default: 'empty', description: 'Dialogs, palette, prompts, toasts.' }
]" />

## Fixed panel order

`show_workbench` is the only code that knows the order:

1. `handle_window_edges` for the undecorated resize bands.
2. Title bar, full width, 34 pt.
3. Explorer, left, resizable, below the title bar.
4. Right panel, resizable, from the title bar to the bottom.
5. Tab strip, top of the editor column, 34 pt.
6. Status bar, bottom of the editor column, 24 pt.
7. Bottom panel, resizable, above the status bar.
8. Central content.
9. Overlays.

```rust
pub fn show_workbench<A: WorkbenchApp>(ui: &mut Ui, app: &mut A) -> Vec<A::Command> {
    crate::handle_window_edges(ui.ctx());
    let mut commands = Vec::new();
    if app.title_bar_visible() {
        Panel::top(TITLE_BAR_PANEL).exact_size(TITLE_HEIGHT).show(ui, |ui| {
            commands.extend(app.title_bar(ui));
        });
    }
    if app.sidebar_visible() {
        // width: preferred, clamped so the content keeps CONTENT_MIN_WIDTH
        Panel::left(SIDEBAR_PANEL).resizable(true).show(ui, |ui| {
            commands.extend(app.sidebar(ui));
        });
    }
    if app.right_panel_visible() {
        Panel::right(RIGHT_PANEL).resizable(true).show(ui, |ui| {
            commands.extend(app.right_panel(ui));
        });
    }
    // tab strip, status bar, bottom panel, central, overlays
    commands
}
```

Panel ids are stable, so egui remembers resizes across frames:
`TITLE_BAR_PANEL`, `SIDEBAR_PANEL`, `RIGHT_PANEL`, `TAB_STRIP_PANEL`, `STATUS_BAR_PANEL`,
`BOTTOM_PANEL`.

## Responsive behavior

| Rule | Value |
|---|---|
| Explorer minimum | `SIDEBAR_MIN_WIDTH` = 160 pt |
| Content minimum | `CONTENT_MIN_WIDTH` = 480 pt |
| Explorer maximum | Half the window |
| Right panel minimum | `RIGHT_PANEL_MIN_WIDTH` = 180 pt |
| Right panel maximum | Half the window |
| Bottom panel minimum | `BOTTOM_PANEL_MIN_HEIGHT` = 72 pt |
| Bottom panel maximum | 60 % of the editor column |

The explorer and right panel yield to the content on narrow windows. If both are open on
a 640 pt window, the content still gets its minimum before the side panels take space.
The bottom panel never leaves the content without 40 % of the height.

## Right and bottom panels

The template uses them for an inspector and an output log:

```rust
fn right_panel_visible(&self) -> bool {
    self.inspector_open
}

fn right_panel(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
    crate::panels::inspector(self, ui)
}

fn bottom_panel_visible(&self) -> bool {
    self.output_open
}

fn bottom_panel(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
    crate::panels::output(self, ui)
}
```

Both panels get the content's 8 pt inner margin so their contents line up. They are
resizable and their sizes are remembered under their panel ids.

Use the right panel for an inspector, outline, properties, or chat. Use the bottom panel
for a terminal, output, problems, or a timeline.

## Frame logic

```rust
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
        self.appearance.sync_system(ctx, &mut self.theme);
        if self.guard.should_prompt(ctx, self.docs.any_dirty()) {
            self.close_request = Some(CloseRequest::Window);
        }
        if !self.overlay_open()
            && let Some(command) = self.keymap.command_cloned(ctx)
        {
            self.handle(ctx, command);
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

## Skipping the shell

An application can call the panels directly and own the order. It then also owns the
tests that protect it. Most products should keep `show_workbench`: the order is part of
the workbench contract, and the runtime already implements the responsive rules.

## Multiple windows

`show_workbench` draws panels inside the root `Ui`. For a second viewport, use
`egui::ViewportBuilder` and give each viewport its own `Ui` and root layout; the panels
are keyed by stable ids, so they do not collide across viewports.
