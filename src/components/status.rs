//! Status bar composition.

use egui::Ui;

use crate::components::Component;
use crate::{StatusItem, show_status_bar};

/// A status bar built from left and right cells.
///
/// ```
/// # use egui_workbench::components::{StatusBar, prelude::*};
/// # use egui_workbench::StatusItem;
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// StatusBar::new()
///     .left(StatusItem::new("3 tabs"))
///     .right(StatusItem::new("UTF-8"))
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Clone, Debug, Default)]
pub struct StatusBar {
    left: Vec<StatusItem>,
    right: Vec<StatusItem>,
}

impl StatusBar {
    /// An empty status bar.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a left cell.
    pub fn left(mut self, item: StatusItem) -> Self {
        self.left.push(item);
        self
    }

    /// Add a right cell.
    pub fn right(mut self, item: StatusItem) -> Self {
        self.right.push(item);
        self
    }

    /// Replace the left cells.
    pub fn left_items(mut self, items: Vec<StatusItem>) -> Self {
        self.left = items;
        self
    }

    /// Replace the right cells.
    pub fn right_items(mut self, items: Vec<StatusItem>) -> Self {
        self.right = items;
        self
    }
}

impl Component for StatusBar {
    type Output = ();

    fn show(self, ui: &mut Ui) {
        show_status_bar(ui, &self.left, &self.right);
    }
}
