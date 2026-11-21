//! The optional right and bottom panels: an inspector and an output log.
//!
//! Demo content. Keep the hooks in `ui.rs` and replace these with your product's
//! properties panel, outline, terminal, problems list, or timeline.

use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, RichText};

use crate::app::App;
use crate::commands::Command;

fn header(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(Size::Sm.height());
        ui.label(RichText::new(title).small().strong().weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
    });
    ui.add_space(4.0);
}

/// The right panel: the active tab and the installed theme tokens.
pub fn inspector(app: &mut App, ui: &mut egui::Ui) -> Vec<Command> {
    let mut commands = Vec::new();
    header(ui, "INSPECTOR", |ui| {
        if IconButton::new("close", "Hide inspector")
            .size(IconButtonSize::Small)
            .show(ui, &mut app.icons)
            .clicked()
        {
            commands.push(Command::ToggleInspector);
        }
    });
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let properties = egui::Grid::new("inspector_tab").num_columns(2).spacing([12.0, 6.0]);
        Label::new("Active tab").strong().show(ui);
        ui.add_space(4.0);
        properties.show(ui, |ui| {
            let tab = app.tabs.active();
            for (name, value) in [
                ("Title", tab.map_or("—".to_owned(), |tab| tab.title.clone())),
                (
                    "Section",
                    tab.map_or("—".to_owned(), |tab| tab.section.label().to_owned()),
                ),
                (
                    "Modified",
                    tab.map_or("—".to_owned(), |tab| if tab.dirty { "yes" } else { "no" }.to_owned()),
                ),
                ("Open tabs", app.tabs.len().to_string()),
            ] {
                ui.label(RichText::new(name).weak());
                ui.add(egui::Label::new(value).truncate());
                ui.end_row();
            }
        });
        ui.add_space(12.0);
        Label::new("Theme tokens").strong().show(ui);
        ui.add_space(4.0);
        let tokens = app.theme.tokens();
        egui::Grid::new("inspector_tokens")
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                for (name, color) in [
                    ("background", tokens.background),
                    ("surface", tokens.surface),
                    ("sunken", tokens.sunken),
                    ("text", tokens.text),
                    ("text_muted", tokens.text_muted),
                    ("accent", tokens.accent),
                    ("selection", tokens.selection),
                    ("focus", tokens.focus),
                    ("link", tokens.link),
                    ("border", tokens.border),
                    ("success", tokens.success),
                    ("warning", tokens.warning),
                    ("danger", tokens.danger),
                ] {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                        ui.painter().rect(
                            rect,
                            3.0,
                            color,
                            egui::Stroke::new(1.0, ui.visuals().window_stroke.color),
                            egui::StrokeKind::Inside,
                        );
                        ui.label(RichText::new(name).small());
                    });
                    let [r, g, b, _] = color.to_array();
                    ui.label(
                        RichText::new(format!("#{r:02x}{g:02x}{b:02x}"))
                            .small()
                            .monospace()
                            .weak(),
                    );
                    ui.end_row();
                }
            });
    });
    commands
}

/// The bottom panel: every command the app handled.
pub fn output(app: &mut App, ui: &mut egui::Ui) -> Vec<Command> {
    let mut commands = Vec::new();
    let mut clear = false;
    header(ui, "OUTPUT", |ui| {
        if IconButton::new("close", "Hide output")
            .size(IconButtonSize::Small)
            .show(ui, &mut app.icons)
            .clicked()
        {
            commands.push(Command::ToggleOutput);
        }
        if Button::new("Clear")
            .size(Size::Sm)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            clear = true;
        }
    });
    if clear {
        app.log.clear();
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if app.log.is_empty() {
                ui.label(RichText::new("Commands you run appear here.").weak());
            }
            for (index, line) in app.log.iter().enumerate() {
                ui.label(
                    RichText::new(format!("{:>4}  {line}", index + 1))
                        .monospace()
                        .size(12.0),
                );
            }
        });
    commands
}
