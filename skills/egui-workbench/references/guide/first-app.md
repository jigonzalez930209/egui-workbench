# Build your first app

> Build a small workbench from scratch: window, title bar, tabs, content, status bar, palette, shortcuts, and close guard. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/first-app.html>

This tutorial builds a minimal text workbench without the template. It has one document
model, a title bar and menus, tabs, an explorer, a status bar, a command palette,
shortcuts, and a close guard. The complete program is at the end.

> **Start from the template instead?**
>
> `cargo xtask new ../my-app` gives you all of this plus settings persistence and a demo
> gallery. Read [Quick start](../guide/quick-start.md). This page explains the pieces.

## 1. Dependency

`Cargo.toml`:

```toml
[package]
name = "first-app"
version = "0.1.0"
edition = "2024"

[dependencies]
egui-workbench = "0.3"
```

## 2. The model

Keep the tab metadata the shell needs and put your domain state beside it:

```rust
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

`Documents<Doc>` handles order, selection, and removal. It performs no I/O.

## 3. Commands and shortcuts

Every action is one enum value:

```rust
#[derive(Clone, Copy, Debug)]
enum Command {
    New,
    Save,
    CloseTab,
    ToggleSidebar,
    Palette,
    Quit,
}
```

The keymap is an ordered list. Longer chords must come first because the first match
consumes the key:

```rust
fn keymap() -> Keymap<Command> {
    Keymap::new()
        .bind(Modifiers::COMMAND | Modifiers::SHIFT, Key::S, Command::Save)
        .bind(Modifiers::COMMAND, Key::N, Command::New)
        .bind(Modifiers::COMMAND, Key::W, Command::CloseTab)
        .bind(Modifiers::COMMAND, Key::B, Command::ToggleSidebar)
        .bind(Modifiers::COMMAND, Key::P, Command::Palette)
        .bind(Modifiers::COMMAND, Key::Q, Command::Quit)
}
```

## 4. The application state

```rust
struct App {
    icons: Icons,
    theme: ThemeState,
    docs: Documents<Doc>,
    keymap: Keymap<Command>,
    guard: CloseGuard,
    palette: PaletteState,
    sidebar: bool,
    close_prompt: bool,
    next_id: usize,
    window_title: String,
}
```

`ThemeState` installs a theme and keeps the requested and effective values. `CloseGuard`
turns an eframe close request into a prompt. `PaletteState` is the filter and visibility
of the command palette.

## 5. Panels

Implement `WorkbenchApp`. Each method returns the commands it produced; the shell never
applies them itself.

```rust
impl WorkbenchApp for App {
    type Command = Command;

    fn sidebar_visible(&self) -> bool {
        self.sidebar
    }

    fn tab_strip_visible(&self) -> bool {
        true
    }

    fn status_bar_visible(&self) -> bool {
        true
    }

    fn central(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let Some(doc) = self.docs.active_mut() else {
            return Vec::new();
        };
        let response = TextArea::new(&mut doc.body)
            .rows(20)
            .width(ui.available_width())
            .show(ui);
        doc.dirty |= response.changed();
        Vec::new()
    }
}
```

`show_workbench` calls the title bar, explorer, tab strip, status bar, central content,
and overlays in the fixed order. See [Workbench layout](../app/workbench.md) for the optional
right and bottom panels.

## 6. Title bar and menus

`show_title_bar` paints the window controls, the drag region, and the double-click
maximize behavior. The left and right closures add your widgets.

```rust
fn title_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
    let mut command = None;
    let mut bar = TitleBar::new(self.docs.active_title().unwrap_or("First app"));
    bar.maximized = is_maximized(ui.ctx());
    let result = show_title_bar(
        ui,
        &mut self.icons,
        &bar,
        |ui, _icons| {
            title_menu(ui, "File", 220.0, |ui| {
                if MenuButton::new("New").show(ui) {
                    command = Some(Command::New);
                }
                if MenuButton::new("Save").show(ui) {
                    command = Some(Command::Save);
                }
                ui.separator();
                if MenuButton::new("Quit").show(ui) {
                    command = Some(Command::Quit);
                }
            });
            title_menu(ui, "View", 220.0, |ui| {
                if MenuToggle::new("Explorer", self.sidebar).show(ui) {
                    command = Some(Command::ToggleSidebar);
                }
            });
        },
        |_ui, _icons| {},
    );
    if let Some(action) = result.window {
        apply_window_action(ui.ctx(), action);
    }
    command.into_iter().collect()
}
```

## 7. Tabs

Build one `TabItem` per document and handle the returned `TabEvent`:

```rust
fn tab_strip(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
    let tabs: Vec<TabItem<'_>> = self
        .docs
        .items()
        .iter()
        .map(|doc| TabItem {
            dirty: doc.dirty,
            ..TabItem::new(&doc.title, TabIcon::Glyph("files"))
        })
        .collect();
    let event = TabStrip::new(&tabs, self.docs.active_index()).show(ui, &mut self.icons, |ui, icons| {
        if IconButton::new("plus", "New tab")
            .size(IconButtonSize::Tool)
            .show(ui, icons)
            .clicked()
        {
            // The closure cannot borrow `self` again here; record the intent instead.
            ui.ctx().data_mut(|data| data.insert_temp(egui::Id::new("first-app-new"), true));
        }
    });
    match event {
        Some(TabEvent::Select(index)) => {
            self.docs.set_active(index);
        }
        Some(TabEvent::Close(index)) => {
            self.docs.remove(index);
        }
        None => {}
    }
    Vec::new()
}
```

> **Borrowing in closures**
>
> The `controls` closure runs while `tabs` borrows `self.docs`. Record a flag in egui data
> or in a local `Cell`, then push the command after the closure returns. The template uses
> a local `Vec<Command>` for this reason.

## 8. Palette, close prompt, and overlays

```rust
fn overlays(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
    let ctx = ui.ctx().clone();
    let mut commands = Vec::new();
    let labels = PaletteLabels {
        title: "Command Palette",
        hint: "Type to filter…",
        empty: "No matches",
        close: "Close",
    };
    let items = vec![
        PaletteItem { label: "New".into(), detail: "Ctrl+N".into(), value: Command::New },
        PaletteItem { label: "Save".into(), detail: "Ctrl+Shift+S".into(), value: Command::Save },
        PaletteItem { label: "Close tab".into(), detail: "Ctrl+W".into(), value: Command::CloseTab },
        PaletteItem { label: "Toggle explorer".into(), detail: "Ctrl+B".into(), value: Command::ToggleSidebar },
    ];
    if let Some(command) = CommandPalette::new(&mut self.palette, labels, &items).show(&ctx) {
        commands.push(command);
    }
    if self.close_prompt {
        let mut open = true;
        match show_choice_window(
            &ctx,
            "Save changes?",
            "Do you want to save before closing?",
            &["Save", "Discard", "Cancel"],
            &mut open,
        ) {
            Some(0) => {
                for doc in self.docs.items_mut() {
                    doc.dirty = false;
                }
                self.guard.allow_close(&ctx);
                self.close_prompt = false;
            }
            Some(1) => {
                self.guard.allow_close(&ctx);
                self.close_prompt = false;
            }
            Some(_) => {
                self.guard.reset();
                self.close_prompt = false;
            }
            None if !open => {
                self.guard.reset();
                self.close_prompt = false;
            }
            None => {}
        }
    }
    commands
}
```

## 9. Frame logic

`logic` runs before `ui`. Sync the theme, ask the guard whether to prompt, keep the window
title current, and consume a shortcut only while no overlay is open:

```rust
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
        if self.guard.should_prompt(ctx, self.docs.any_dirty()) {
            self.close_prompt = true;
        }
        let title = format!("{} — First app", self.docs.active_title().unwrap_or("Untitled"));
        if title != self.window_title {
            set_window_title(ctx, &title);
            self.window_title = title;
        }
        if !self.palette.open
            && !self.close_prompt
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

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}
```

## 10. Complete program

`src/main.rs`:

```rust
use egui_workbench::app::{
    CloseGuard, Document, Documents, Keymap, ThemeState, WorkbenchApp, show_workbench,
};
use egui_workbench::components::prelude::*;
use egui_workbench::eframe;
use egui_workbench::egui::{self, Key, Modifiers};
use egui_workbench::{
    Icons, StatusItem, ThemeId, TitleBar, WindowAction, apply_window_action, is_maximized,
    set_window_title, show_choice_window, show_empty_state, show_title_bar, title_menu,
};

#[derive(Clone, Copy, Debug)]
enum Command {
    New,
    Save,
    CloseTab,
    ToggleSidebar,
    Quit,
}

struct Doc {
    title: String,
    body: String,
    dirty: bool,
}

impl Doc {
    fn new(id: usize) -> Self {
        Self {
            title: format!("Untitled {id}"),
            body: String::new(),
            dirty: false,
        }
    }
}

impl Document for Doc {
    fn title(&self) -> &str {
        &self.title
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

struct App {
    icons: Icons,
    theme: ThemeState,
    docs: Documents<Doc>,
    keymap: Keymap<Command>,
    guard: CloseGuard,
    palette: PaletteState,
    sidebar: bool,
    close_prompt: bool,
    next_id: usize,
    window_title: String,
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        let mut docs = Documents::new();
        docs.push_active(Doc::new(1));
        Self {
            icons: Icons::new(),
            theme: ThemeState::new(ctx, ThemeId::GithubDark, false),
            docs,
            keymap: Keymap::new()
                .bind(Modifiers::COMMAND | Modifiers::SHIFT, Key::S, Command::Save)
                .bind(Modifiers::COMMAND, Key::N, Command::New)
                .bind(Modifiers::COMMAND, Key::W, Command::CloseTab)
                .bind(Modifiers::COMMAND, Key::B, Command::ToggleSidebar)
                .bind(Modifiers::COMMAND, Key::Q, Command::Quit),
            guard: CloseGuard::new(),
            palette: PaletteState::default(),
            sidebar: true,
            close_prompt: false,
            next_id: 2,
            window_title: String::new(),
        }
    }

    fn handle(&mut self, ctx: &egui::Context, command: Command) {
        match command {
            Command::New => {
                let id = self.next_id;
                self.next_id += 1;
                self.docs.push_active(Doc::new(id));
            }
            Command::Save => {
                if let Some(doc) = self.docs.active_mut() {
                    doc.dirty = false;
                }
            }
            Command::CloseTab => {
                let index = self.docs.active_index();
                self.docs.remove(index);
            }
            Command::ToggleSidebar => self.sidebar = !self.sidebar,
            Command::Quit => apply_window_action(ctx, WindowAction::Close),
        }
    }
}

impl WorkbenchApp for App {
    type Command = Command;

    fn sidebar_visible(&self) -> bool {
        self.sidebar
    }

    fn tab_strip_visible(&self) -> bool {
        true
    }

    fn status_bar_visible(&self) -> bool {
        true
    }

    fn title_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let mut command = None;
        let mut bar = TitleBar::new(self.docs.active_title().unwrap_or("First app"));
        bar.maximized = is_maximized(ui.ctx());
        let result = show_title_bar(
            ui,
            &mut self.icons,
            &bar,
            |ui, _icons| {
                title_menu(ui, "File", 220.0, |ui| {
                    if MenuButton::new("New").show(ui) {
                        command = Some(Command::New);
                    }
                    if MenuButton::new("Save").show(ui) {
                        command = Some(Command::Save);
                    }
                    ui.separator();
                    if MenuButton::new("Quit").show(ui) {
                        command = Some(Command::Quit);
                    }
                });
                title_menu(ui, "View", 220.0, |ui| {
                    if MenuToggle::new("Explorer", self.sidebar).show(ui) {
                        command = Some(Command::ToggleSidebar);
                    }
                });
            },
            |_ui, _icons| {},
        );
        if let Some(action) = result.window {
            apply_window_action(ui.ctx(), action);
        }
        command.into_iter().collect()
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        ui.add_space(8.0);
        Label::new("EXPLORER").size(Size::Sm).tone(Tone::Muted).show(ui);
        ui.add_space(4.0);
        let active = self.docs.active_index();
        let mut selected = None;
        for (index, doc) in self.docs.items().iter().enumerate() {
            if NavTab::new(&doc.title).active(index == active).show(ui) {
                selected = Some(index);
            }
        }
        if let Some(index) = selected {
            self.docs.set_active(index);
        }
        Vec::new()
    }

    fn tab_strip(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let tabs: Vec<TabItem<'_>> = self
            .docs
            .items()
            .iter()
            .map(|doc| TabItem {
                dirty: doc.dirty,
                ..TabItem::new(&doc.title, TabIcon::Glyph("files"))
            })
            .collect();
        let mut commands = Vec::new();
        let event = TabStrip::new(&tabs, self.docs.active_index()).show(ui, &mut self.icons, |ui, icons| {
            if IconButton::new("plus", "New tab")
                .size(IconButtonSize::Tool)
                .show(ui, icons)
                .clicked()
            {
                commands.push(Command::New);
            }
        });
        match event {
            Some(TabEvent::Select(index)) => {
                self.docs.set_active(index);
            }
            Some(TabEvent::Close(index)) => {
                self.docs.remove(index);
            }
            None => {}
        }
        commands
    }

    fn central(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let Some(doc) = self.docs.active_mut() else {
            if show_empty_state(ui, "First app", "No document is open.", "New", "Ctrl+N") {
                return vec![Command::New];
            }
            return Vec::new();
        };
        let response = TextArea::new(&mut doc.body)
            .rows(20)
            .width(ui.available_width())
            .show(ui);
        doc.dirty |= response.changed();
        Vec::new()
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let title = self.docs.active_title().unwrap_or("No document");
        StatusBar::new()
            .left(StatusItem::new(title))
            .right(StatusItem::new(format!("{} tabs", self.docs.len())))
            .show(ui);
        Vec::new()
    }

    fn overlays(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let ctx = ui.ctx().clone();
        let mut commands = Vec::new();
        let labels = PaletteLabels {
            title: "Command Palette",
            hint: "Type to filter…",
            empty: "No matches",
            close: "Close",
        };
        let items = vec![
            PaletteItem {
                label: "New".into(),
                detail: "Ctrl+N".into(),
                value: Command::New,
            },
            PaletteItem {
                label: "Save".into(),
                detail: "Ctrl+Shift+S".into(),
                value: Command::Save,
            },
            PaletteItem {
                label: "Close tab".into(),
                detail: "Ctrl+W".into(),
                value: Command::CloseTab,
            },
            PaletteItem {
                label: "Toggle explorer".into(),
                detail: "Ctrl+B".into(),
                value: Command::ToggleSidebar,
            },
        ];
        if let Some(command) = CommandPalette::new(&mut self.palette, labels, &items).show(&ctx) {
            commands.push(command);
        }
        if self.close_prompt {
            let mut open = true;
            match show_choice_window(
                &ctx,
                "Save changes?",
                "Do you want to save before closing?",
                &["Save", "Discard", "Cancel"],
                &mut open,
            ) {
                Some(0) => {
                    for doc in self.docs.items_mut() {
                        doc.dirty = false;
                    }
                    self.guard.allow_close(&ctx);
                    self.close_prompt = false;
                }
                Some(1) => {
                    self.guard.allow_close(&ctx);
                    self.close_prompt = false;
                }
                Some(_) => {
                    self.guard.reset();
                    self.close_prompt = false;
                }
                None if !open => {
                    self.guard.reset();
                    self.close_prompt = false;
                }
                None => {}
            }
        }
        commands
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
        if self.guard.should_prompt(ctx, self.docs.any_dirty()) {
            self.close_prompt = true;
        }
        let title = format!("{} — First app", self.docs.active_title().unwrap_or("Untitled"));
        if title != self.window_title {
            set_window_title(ctx, &title);
            self.window_title = title;
        }
        if self.close_prompt {
            return;
        }
        if !self.palette.open
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

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "First app",
        egui_workbench::native_options("First app"),
        Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx)))),
    )
}
```

## Next

- Add settings persistence: [Settings persistence](../app/settings-persistence.md).
- Add a right inspector and a bottom output panel: [Workbench layout](../app/workbench.md).
- Replace the placeholder with product UI: [Components](../components/index.md).
