//! Overlays of the showcase: palette, Quick Open, search, keyboard menu, settings,
//! prompts, and the unsaved-changes guard.

use egui_workbench::egui::{self, RichText};
use egui_workbench::{
    ConfirmPrompt, NamePrompt, PaletteItem, PaletteLabels, PromptAction, SettingsDialog, ThemeId, combo, settings_row,
    settings_section, settings_switch,
};

use crate::model::{Cmd, file_name};
use crate::{Overlay, Showcase, menus};

/// Draw every overlay and return the commands they produced.
pub fn show(app: &mut Showcase, ctx: &egui::Context) -> Vec<Cmd> {
    let mut out = Vec::new();
    let labels = PaletteLabels {
        title: "Command Palette",
        hint: "Type to filter…",
        empty: "No matches",
        close: "Close",
    };
    out.extend(egui_workbench::show_palette(
        ctx,
        &mut app.palette,
        labels,
        &menus::palette_items(),
    ));
    quick_open(app, ctx);
    out.extend(egui_workbench::show_keyboard_menu(
        ctx,
        &mut app.keyboard_menu,
        &menus::keyboard_groups(),
        "Arrows navigate · Enter runs · Esc closes",
    ));
    search(app, ctx);
    settings(app, ctx);
    prompts(app, ctx);
    out
}

fn quick_open(app: &mut Showcase, ctx: &egui::Context) {
    if !app.quick_open.open {
        return;
    }
    let mut files = Vec::new();
    if let Some(root) = &app.root {
        root.files(&mut files);
    }
    let items: Vec<PaletteItem<std::path::PathBuf>> = files
        .into_iter()
        .map(|path| PaletteItem {
            label: file_name(&path),
            detail: path
                .parent()
                .map(|parent| parent.display().to_string())
                .unwrap_or_default(),
            value: path,
        })
        .collect();
    let labels = PaletteLabels {
        title: "Quick Open",
        hint: "Type a file name…",
        empty: "No matches",
        close: "Close",
    };
    if let Some(path) = egui_workbench::show_palette(ctx, &mut app.quick_open, labels, &items) {
        app.open_path(&path);
    }
}

fn search(app: &mut Showcase, ctx: &egui::Context) {
    let Some(query) = &mut app.search else {
        return;
    };
    let mut close = false;
    egui_workbench::show_popover(ctx, "showcase_search", 560.0, |ui| {
        if egui_workbench::dialog_header(ui, "Search", 13.0, "Close") {
            close = true;
        }
        egui_workbench::text_field(ui, query, "Search in document…", ui.available_width()).request_focus();
        ui.add_space(6.0);
        let doc = app.docs.active();
        let hits = doc.map_or(0, |doc| {
            if query.is_empty() {
                0
            } else {
                doc.text.matches(query.as_str()).count()
            }
        });
        ui.label(RichText::new(format!("{hits} results")).small().weak());
    });
    if close || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        app.search = None;
    }
}

fn settings(app: &mut Showcase, ctx: &egui::Context) {
    let dialog = SettingsDialog {
        title: "Settings",
        tabs: &["Appearance", "Editor", "About"],
        note: concat!("Workbench ", env!("CARGO_PKG_VERSION")),
        done: "Done",
        close: "Close",
    };
    let (mut theme_id, mut follow, mut reduce) = (
        app.theme.requested(),
        app.theme.follow_system(),
        app.theme.reduced_motion(),
    );
    let mut zoom = ctx.zoom_factor();
    let icons = &mut app.icons;
    egui_workbench::show_settings_dialog(ctx, &mut app.settings_open, &dialog, |ui, tab| match tab {
        0 => {
            settings_section(ui, "Appearance", "Theme, motion, and text size.");
            let options: Vec<(ThemeId, String)> = ThemeId::ALL.iter().map(|id| (*id, id.label().to_owned())).collect();
            settings_row(ui, "Theme", "Color family for the whole workbench.", |ui| {
                combo(ui, "Theme", &mut theme_id, &options);
            });
            settings_switch(
                ui,
                "Follow system",
                "Match the desktop light or dark preference.",
                &mut follow,
            );
            settings_switch(ui, "Reduce motion", "Pause non-essential animation.", &mut reduce);
            settings_row(ui, "Text scale", "Scale the whole interface from 80% to 200%.", |ui| {
                egui_workbench::numeric_stepper(ui, icons, &mut zoom, 0.8..=2.0, 0.1, 112.0, |value| {
                    format!("{:.0}%", value * 100.0)
                });
            });
        }
        1 => {
            settings_section(ui, "Editor", "Typing and layout.");
            let mut wrap = true;
            settings_switch(ui, "Word wrap", "Wrap long lines at the window edge.", &mut wrap);
        }
        _ => {
            settings_section(ui, "About", "A reusable egui workbench.");
            ui.label(concat!("Workbench ", env!("CARGO_PKG_VERSION")));
        }
    });
    app.theme.set_requested(ctx, theme_id);
    app.theme.set_follow_system(ctx, follow);
    app.theme.set_reduced_motion(ctx, reduce);
    if (zoom - ctx.zoom_factor()).abs() > f32::EPSILON {
        ctx.set_zoom_factor(zoom);
    }
}

fn prompts(app: &mut Showcase, ctx: &egui::Context) {
    match std::mem::take(&mut app.overlay) {
        Overlay::None => {}
        Overlay::Rename(index, mut name) => {
            let error = name.trim().is_empty().then_some("A name is required.");
            let mut prompt = NamePrompt {
                title: "Rename",
                description: "Give the document a new name.",
                label: "Name",
                value: &mut name,
                confirm: "Rename",
                cancel: "Cancel",
                error,
            };
            match egui_workbench::show_name_prompt(ctx, &mut prompt) {
                Some(PromptAction::Submit) if error.is_none() => {
                    if let Some(doc) = app.docs.get_mut(index) {
                        doc.name = name;
                    }
                }
                Some(PromptAction::Cancel) => {}
                _ => app.overlay = Overlay::Rename(index, name),
            }
        }
        Overlay::Delete(index) => {
            let prompt = ConfirmPrompt {
                title: "Delete file?",
                message: "The document is closed. Files on disk are not touched in this demo.",
                confirm: "Delete",
                cancel: "Cancel",
                danger: true,
            };
            match egui_workbench::show_confirm_prompt(ctx, &prompt) {
                Some(PromptAction::Submit) => app.handle(ctx, Cmd::CloseTab),
                Some(PromptAction::Cancel) => {}
                None => app.overlay = Overlay::Delete(index),
            }
        }
        Overlay::ClosePrompt => {
            let mut open = true;
            let message = "Do you want to save your changes before closing?";
            match egui_workbench::show_choice_window(
                ctx,
                "Save changes?",
                message,
                &["Save", "Discard", "Cancel"],
                &mut open,
            ) {
                Some(0 | 1) => app.guard.allow_close(ctx),
                Some(_) => {}
                None if open => app.overlay = Overlay::ClosePrompt,
                None => {}
            }
        }
        Overlay::About => {
            let mut open = true;
            egui::Window::new("About")
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .default_width(360.0)
                .show(ctx, |ui| {
                    ui.heading("Workbench");
                    ui.label("The MD View shell as reusable egui components.");
                });
            if open && !ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
                app.overlay = Overlay::About;
            }
        }
    }
}
