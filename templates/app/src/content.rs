//! Central content.
//!
//! This is the seam to replace. The starter sends the active tab to [`crate::gallery`],
//! which demonstrates every component. Replace that call with the product screen; the
//! shell, close prompts, palette, and status bar stay unchanged.

use egui_workbench::egui;
use egui_workbench::show_empty_state;

use crate::app::App;
use crate::commands::Command;

/// Draw the active tab.
pub fn show(app: &mut App, ui: &mut egui::Ui) -> Vec<Command> {
    if app.tabs.is_empty() {
        if show_empty_state(
            ui,
            "{{app_title}}",
            "A generic egui workbench.",
            "New tab",
            "The starter gallery opens in every new tab.",
        ) {
            return vec![Command::NewTab];
        }
        return Vec::new();
    }
    crate::gallery::show(app, ui)
}
