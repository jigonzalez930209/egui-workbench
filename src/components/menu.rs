//! Menu rows, toggles, and a text menu button.

use egui::Ui;

use crate::Icons;
use crate::components::{Component, IconComponent};
use crate::{compact_menu, menu_button, menu_item, menu_toggle, menu_tool_button};

/// A 22 pt context-menu row with an icon, label, and optional shortcut.
pub struct MenuItem<'a> {
    icon: &'a str,
    label: &'a str,
    shortcut: Option<&'a str>,
    danger: bool,
    enabled: bool,
}

impl<'a> MenuItem<'a> {
    /// A menu row.
    pub fn new(icon: &'a str, label: &'a str) -> Self {
        Self {
            icon,
            label,
            shortcut: None,
            danger: false,
            enabled: true,
        }
    }

    /// Right-aligned shortcut text.
    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Paint with the error color.
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// Enable or disable the row.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl IconComponent for MenuItem<'_> {
    type Output = bool;

    fn show(self, ui: &mut Ui, icons: &mut Icons) -> bool {
        ui.add_enabled_ui(self.enabled, |ui| {
            menu_item(ui, icons, self.icon, self.label, self.shortcut, self.danger)
        })
        .inner
    }
}

/// A selectable menu row that closes the menu when clicked.
pub struct MenuToggle<'a> {
    label: &'a str,
    on: bool,
    enabled: bool,
}

impl<'a> MenuToggle<'a> {
    /// A toggle row.
    pub fn new(label: &'a str, on: bool) -> Self {
        Self {
            label,
            on,
            enabled: true,
        }
    }

    /// Enable or disable the row.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Component for MenuToggle<'_> {
    type Output = bool;

    fn show(self, ui: &mut Ui) -> bool {
        menu_toggle(ui, self.label, self.on, self.enabled)
    }
}

/// A text menu button. The closure draws the compact menu contents.
pub struct Menu<'a> {
    label: &'a str,
    width: f32,
}

impl<'a> Menu<'a> {
    /// A menu with a minimum popup width.
    pub fn new(label: &'a str) -> Self {
        Self { label, width: 180.0 }
    }

    /// Set the popup width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Show the menu and run `add` inside the popup.
    pub fn show(self, ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
        ui.menu_button(self.label, |ui| {
            compact_menu(ui);
            ui.set_min_width(self.width);
            add(ui);
        });
    }
}

/// A plain menu button that closes its menu when clicked.
pub struct MenuButton<'a> {
    label: &'a str,
}

impl<'a> MenuButton<'a> {
    /// A menu action row.
    pub fn new(label: &'a str) -> Self {
        Self { label }
    }
}

impl Component for MenuButton<'_> {
    type Output = bool;

    fn show(self, ui: &mut Ui) -> bool {
        menu_button(ui, self.label)
    }
}

/// The compact "…" icon button that opens a popup menu.
pub struct MenuIconButton<'a> {
    icon: &'a str,
    tooltip: &'a str,
    width: f32,
}

impl<'a> MenuIconButton<'a> {
    /// A menu button with a minimum popup width.
    pub fn new(icon: &'a str, tooltip: &'a str) -> Self {
        Self {
            icon,
            tooltip,
            width: 180.0,
        }
    }

    /// Set the popup width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Show the button and its menu.
    pub fn show(self, ui: &mut Ui, icons: &mut Icons, add: impl FnOnce(&mut Ui, &mut Icons)) {
        menu_tool_button(ui, icons, self.icon, self.tooltip, self.width, add);
    }
}
