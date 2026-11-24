//! The workbench panels: title bar, navigator, tabs, content, status bar, and overlays.
//!
//! Every panel collects [`Command`] values instead of mutating state. [`App`] applies
//! them after the frame, which keeps undo/redo, logging, and tests in one place. The
//! product screen itself lives in [`crate::content`], not here.

use egui_workbench::app::WorkbenchApp;
use egui_workbench::commands::{command_menu_groups, show_command_menu, show_command_palette};
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, RichText};
use egui_workbench::{
    SettingsDialog, StatusItem, ThemeId, TitleBar, apply_window_action, combo, is_maximized, settings_row,
    settings_section, settings_switch, show_choice_window, show_settings_dialog, show_title_bar, title_menu,
};

use crate::app::{App, CloseChoice, CloseRequest};
use crate::appearance::{Corners, TextSize};
use crate::commands::{self, Command};
use egui_workbench::style::Density;

impl WorkbenchApp for App {
    type Command = Command;

    fn sidebar_visible(&self) -> bool {
        self.sidebar_open
    }

    fn tab_strip_visible(&self) -> bool {
        true
    }

    fn status_bar_visible(&self) -> bool {
        true
    }

    fn title_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let mut command = None;
        let mut right_command = None;
        let tab = self.tabs.active();
        let mut bar = TitleBar::new(tab.map_or("", |tab| tab.title.as_str()));
        bar.dirty = tab.is_some_and(|tab| tab.dirty);
        bar.maximized = is_maximized(ui.ctx());
        let has_tabs = self.tabs.len() > 1;
        let palette_open = self.palette.open;
        let registry = self.registry.clone();
        let context = commands::context();
        let result = show_title_bar(
            ui,
            &mut self.icons,
            &bar,
            |ui, icons| {
                title_menu(ui, "File", 230.0, |ui| {
                    if let Some(id) = show_command_menu(ui, icons, &registry, &context, "File") {
                        command = commands::command_from_id(&id);
                    }
                });
                title_menu(ui, "View", 230.0, |ui| {
                    if let Some(id) = show_command_menu(ui, icons, &registry, &context, "View") {
                        command = commands::command_from_id(&id);
                    }
                });
                title_menu(ui, "Theme", 230.0, |ui| {
                    if let Some(id) = show_command_menu(ui, icons, &registry, &context, "Theme") {
                        command = commands::command_from_id(&id);
                    }
                });
                title_menu(ui, "Help", 210.0, |ui| {
                    if let Some(id) = show_command_menu(ui, icons, &registry, &context, "Help") {
                        command = commands::command_from_id(&id);
                    }
                });
                ui.add_space(6.0);
                if IconButton::new("back", "Previous Tab")
                    .enabled(has_tabs)
                    .show(ui, icons)
                    .clicked()
                {
                    command = Some(Command::PreviousTab);
                }
                if IconButton::new("forward", "Next Tab")
                    .enabled(has_tabs)
                    .show(ui, icons)
                    .clicked()
                {
                    command = Some(Command::NextTab);
                }
            },
            |ui, icons| {
                if IconButton::new("settings", "Settings").show(ui, icons).clicked() {
                    right_command = Some(Command::Settings);
                }
                if IconButton::new("command", "Command Palette")
                    .active(palette_open)
                    .show(ui, icons)
                    .clicked()
                {
                    right_command = Some(Command::Palette);
                }
            },
        );
        if let Some(action) = result.window {
            apply_window_action(ui.ctx(), action);
        }
        command.or(right_command).into_iter().collect()
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let active = self.tabs.active().map(|tab| tab.section);
        let mut commands = Vec::new();
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(10, 0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(5.0);
                ui.label(RichText::new("EXAMPLES").small().strong().weak());
                ui.add_space(2.0);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for section in crate::model::GallerySection::ALL {
                        if NavTab::new(section.label()).active(active == Some(section)).show(ui) {
                            commands.push(Command::OpenExample(section));
                        }
                    }
                });
            });
        commands
    }

    fn tab_strip(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let tabs: Vec<TabItem<'_>> = self
            .tabs
            .items()
            .iter()
            .map(|tab| TabItem {
                dirty: tab.dirty,
                ..TabItem::new(&tab.title, TabIcon::Glyph("files"))
            })
            .collect();
        let mut commands = Vec::new();
        let event = TabStrip::new(&tabs, self.tabs.active_index())
            .close_tooltip("Close")
            .controls_width(76.0)
            .show(ui, &mut self.icons, |ui, icons| {
                if IconButton::new("plus", "New tab")
                    .size(IconButtonSize::Tool)
                    .show(ui, icons)
                    .clicked()
                {
                    commands.push(Command::NewTab);
                }
                if IconButton::new("close", "Close tab")
                    .size(IconButtonSize::Tool)
                    .show(ui, icons)
                    .clicked()
                {
                    commands.push(Command::CloseTab);
                }
            });
        match event {
            Some(TabEvent::Select(index)) => {
                self.tabs.set_active(index);
            }
            Some(TabEvent::Close(index)) => self.close_tab(index),
            None | Some(_) => {}
        }
        commands
    }

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

    fn central(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        crate::content::show(self, ui)
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let tab = self.tabs.active();
        let mut left = vec![
            StatusItem::new(tab.map_or("No tab open".to_owned(), |tab| tab.title.clone())),
            StatusItem::new(if tab.is_some_and(|tab| tab.dirty) {
                "Modified"
            } else {
                "Saved"
            }),
            StatusItem::new(format!("{} tabs", self.tabs.len())),
        ];
        if !self.status.is_empty() {
            left.push(StatusItem::new(self.status.clone()));
        }
        StatusBar::new()
            .left_items(left)
            .right(StatusItem::new(self.theme.requested().label()))
            .right(StatusItem::new(concat!("v", env!("CARGO_PKG_VERSION"))))
            .show(ui);
        Vec::new()
    }

    fn overlays(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let ctx = ui.ctx().clone();
        let mut commands = Vec::new();
        let labels = egui_workbench::commands::CommandPaletteLabels::default();
        let context = commands::context();
        if let Some(id) = show_command_palette(&ctx, &mut self.palette, labels, &mut self.registry, &context)
            && let Some(command) = commands::command_from_id(&id)
        {
            commands.push(command);
        }
        let groups = command_menu_groups(&self.registry, &context);
        if let Some(id) = egui_workbench::show_keyboard_menu(
            &ctx,
            &mut self.keyboard_menu,
            &groups,
            "Arrows navigate · Enter runs · Esc closes",
        ) && let Some(command) = commands::command_from_id(&id)
        {
            commands.push(command);
        }
        self.settings(&ctx);
        self.about(&ctx);
        self.demo_dialogs(&ctx);
        self.unsaved_prompt(&ctx);
        commands
    }
}

impl App {
    fn settings(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let dialog = SettingsDialog {
            title: "Settings",
            tabs: &["Appearance", "Shape"],
            note: concat!("{{app_title}} ", env!("CARGO_PKG_VERSION")),
            done: "Done",
            close: "Close",
        };
        let (mut theme, mut follow, mut reduce) = (
            self.theme.requested(),
            self.theme.follow_system(),
            self.theme.reduced_motion(),
        );
        let mut appearance = self.appearance;
        show_settings_dialog(ctx, &mut self.settings_open, &dialog, |ui, tab| {
            if tab == 0 {
                settings_section(ui, "Appearance", "Theme and motion.");
                let options: Vec<(ThemeId, String)> =
                    ThemeId::ALL.iter().map(|id| (*id, id.label().to_owned())).collect();
                settings_row(ui, "Theme", "Color family for the whole workbench.", |ui| {
                    combo(ui, "Theme", &mut theme, &options);
                });
                settings_switch(
                    ui,
                    "Follow system",
                    "Match the desktop light or dark preference.",
                    &mut follow,
                );
                settings_switch(ui, "Reduce motion", "Pause non-essential animation.", &mut reduce);
            } else {
                settings_section(ui, "Shape", "Density, corners, text size, and brand color.");
                let densities: Vec<(Density, String)> = Density::ALL
                    .iter()
                    .map(|density| (*density, format!("{density:?}")))
                    .collect();
                settings_row(ui, "Density", "Height of controls and rows.", |ui| {
                    combo(ui, "Density", &mut appearance.density, &densities);
                });
                let corners: Vec<(Corners, String)> = Corners::ALL
                    .iter()
                    .map(|corners| (*corners, corners.label().to_owned()))
                    .collect();
                settings_row(ui, "Corners", "Radius of controls, cards, and dialogs.", |ui| {
                    combo(ui, "Corners", &mut appearance.corners, &corners);
                });
                let sizes: Vec<(TextSize, String)> = TextSize::ALL
                    .iter()
                    .map(|size| (*size, size.label().to_owned()))
                    .collect();
                settings_row(ui, "Text size", "Scale of every label, menu, and title.", |ui| {
                    combo(ui, "Text size", &mut appearance.text_size, &sizes);
                });
                let mut custom = appearance.accent.is_some();
                settings_switch(
                    ui,
                    "Brand accent",
                    "Replace the theme's accent; contrast is kept AA.",
                    &mut custom,
                );
                if custom {
                    let mut color = appearance.accent.unwrap_or(egui::Color32::from_rgb(0xe0, 0x6c, 0x2b));
                    settings_row(ui, "Accent color", "Buttons, links, focus, and selection.", |ui| {
                        egui::color_picker::color_edit_button_srgba(ui, &mut color, egui::color_picker::Alpha::Opaque);
                    });
                    appearance.accent = Some(color);
                } else {
                    appearance.accent = None;
                }
            }
        });
        let theme_changed = (theme, follow, reduce)
            != (
                self.theme.requested(),
                self.theme.follow_system(),
                self.theme.reduced_motion(),
            );
        if theme_changed || appearance != self.appearance {
            self.theme.set_requested(ctx, theme);
            self.theme.set_follow_system(ctx, follow);
            self.theme.set_reduced_motion(ctx, reduce);
            self.appearance = appearance;
            self.appearance.apply(ctx, &mut self.theme);
            self.save_settings();
        }
    }

    fn about(&mut self, ctx: &egui::Context) {
        if !self.about_open {
            return;
        }
        egui::Window::new("About")
            .open(&mut self.about_open)
            .resizable(false)
            .collapsible(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.heading("{{app_title}}");
                ui.label(concat!("Version ", env!("CARGO_PKG_VERSION")));
                ui.label("Built on egui-workbench.");
            });
    }

    fn demo_dialogs(&mut self, ctx: &egui::Context) {
        if self.demo_dialog {
            let mut open = self.demo_dialog;
            let mut close = false;
            Dialog::new("gallery_dialog", "Custom dialog", &mut open)
                .width(420.0)
                .show(ctx, |ui| {
                    Label::new("Dialog body").strong().show(ui);
                    ui.label("Compose the body with the same component builders.");
                    ui.add_space(12.0);
                    if Button::new("Close").show(ui).clicked() {
                        close = true;
                    }
                });
            if close {
                open = false;
            }
            self.demo_dialog = open;
        }
        if self.demo_confirm
            && ConfirmDialog::new("Delete item?", "This action cannot be undone.")
                .confirm("Delete")
                .danger(true)
                .show(ctx)
                .is_some()
        {
            self.demo_confirm = false;
        }
        if self.demo_name
            && NameDialog::new("Rename", "Name", &mut self.demo_name_value)
                .show(ctx)
                .is_some()
        {
            self.demo_name = false;
        }
        if self.demo_progress && ProgressDialog::new("Loading", "Preparing the sample workspace…", 0.65).show(ctx) {
            self.demo_progress = false;
        }
    }

    fn unsaved_prompt(&mut self, ctx: &egui::Context) {
        let Some(request) = self.close_request else {
            return;
        };
        let message = match request {
            CloseRequest::Window => "Do you want to save your changes before closing?".to_owned(),
            CloseRequest::Tab(index) => format!(
                "Save changes to {} before closing it?",
                self.tabs.get(index).map_or("this tab", |tab| tab.title.as_str())
            ),
        };
        let mut open = true;
        match show_choice_window(
            ctx,
            "Save changes?",
            &message,
            &["Save", "Discard", "Cancel"],
            &mut open,
        ) {
            Some(0) => self.resolve_close(ctx, CloseChoice::Save),
            Some(1) => self.resolve_close(ctx, CloseChoice::Discard),
            Some(_) => self.resolve_close(ctx, CloseChoice::Cancel),
            None if !open => self.resolve_close(ctx, CloseChoice::Cancel),
            None => {}
        }
    }
}
