# IDE layout

> Compose explorer, editor, inspector, output, and status bar with the runtime panels. Live page: <https://jigonzalez930209.github.io/egui-workbench/recipes/ide-layout.html>

## Composition

| Surface | WorkbenchApp method | Suggested content |
|---|---|---|
| Title bar | `title_bar` | Menus, history, settings, palette |
| Explorer | `sidebar` | File tree, outline, search results |
| Editor | `central` | Tabs + active document |
| Inspector | `right_panel` | Properties, metadata, chat |
| Output | `bottom_panel` | Terminal, problems, logs |
| Status bar | `status_bar` | Path, dirty state, position, language |
| Overlays | `overlays` | Palette, settings, prompts, toasts |

## Visibility flags

```rust
impl WorkbenchApp for App {
    type Command = Command;

    fn sidebar_visible(&self) -> bool { self.explorer_open }
    fn right_panel_visible(&self) -> bool { self.inspector_open }
    fn bottom_panel_visible(&self) -> bool { self.output_open }
    fn tab_strip_visible(&self) -> bool { true }
    fn status_bar_visible(&self) -> bool { true }
    // ...
}
```

Toggling a panel changes the flag; egui remembers the resized width/height under the panel
id. Do not rebuild the panel or change its id when toggling.

## Right panel

```rust
fn right_panel(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
    let mut commands = Vec::new();
    ui.horizontal(|ui| {
        ui.label(RichText::new("INSPECTOR").small().strong().weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if IconButton::new("close", "Hide inspector")
                .size(IconButtonSize::Small)
                .show(ui, &mut self.icons)
                .clicked()
            {
                commands.push(Command::ToggleInspector);
            }
        });
    });
    egui::ScrollArea::vertical().show(ui, |ui| {
        // properties grid, metadata, actions
    });
    commands
}
```

The panel already has an 8 pt inner margin from the shell; do not add another frame unless
you need a card.

## Bottom panel

```rust
fn bottom_panel(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
    let mut commands = Vec::new();
    ui.horizontal(|ui| {
        ui.label(RichText::new("OUTPUT").small().strong().weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::new("Clear").size(Size::Sm).variant(ButtonVariant::Ghost).show(ui).clicked() {
                self.log.clear();
            }
            if IconButton::new("close", "Hide output")
                .size(IconButtonSize::Small)
                .show(ui, &mut self.icons)
                .clicked()
            {
                commands.push(Command::ToggleOutput);
            }
        });
    });
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in &self.log {
                ui.label(RichText::new(line).monospace().size(12.0));
            }
        });
    commands
}
```

Use `.stick_to_bottom(true)` for logs and terminals so new lines are visible without
stealing the scroll position when the user scrolled up.

## Commands

```rust
Command::ToggleSidebar => self.explorer_open = !self.explorer_open,
Command::ToggleInspector => self.inspector_open = !self.inspector_open,
Command::ToggleOutput => self.output_open = !self.output_open,
```

Wire them to View menu toggles, palette rows, and shortcuts (`Ctrl+B`, `Ctrl+Alt+B`,
`Ctrl+J`). A panel should never be toggled by mutating a widget's internal state; the
application owns the flags.

## Sizing rules

- Explorer: min 160 pt, content keeps 480 pt, max half the window.
- Right panel: min 180 pt, max half the window.
- Bottom panel: min 72 pt, max 60 % of the editor column.
- The shell enforces these; do not duplicate them in the panel content.

## Focus and keyboard

- Put the explorer tree first in the tab order; it handles Up/Down/Left/Right/Enter/F2.
- The editor takes focus when a tab is selected.
- `Ctrl+J` toggles the output; `Esc` closes the top overlay, not the panel.
- Panel close buttons are icon buttons with tooltips and AccessKit names.

## Checklist

- [ ] Panel visibility lives in `App`, not in egui memory.
- [ ] Panel ids stay stable across toggles.
- [ ] Every panel is resizable and remembers its size.
- [ ] Content keeps 480 pt on narrow windows.
- [ ] Logs stick to the bottom only when the user is already there.
- [ ] Commands are reachable from menu, palette, and shortcut.
