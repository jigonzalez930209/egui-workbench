//! Command registry views for menus and the keyboard menu.

use egui::Ui;

use super::{CommandId, CommandRegistry, WhenContext};
use crate::{Icons, MenuEntry, MenuGroup, menu_item};

/// Draw one menu category from the registry and return the command that ran.
///
/// The caller normally opens this inside [`crate::title_menu`] or a context menu.
pub fn show_command_menu(
    ui: &mut Ui,
    icons: &mut Icons,
    registry: &CommandRegistry,
    context: &WhenContext,
    category: &str,
) -> Option<CommandId> {
    let mut chosen = None;
    for command in registry.iter() {
        if command.category != category || !command.applies(context) {
            continue;
        }
        let shortcut = registry
            .binding(&command.id)
            .map(ToString::to_string)
            .unwrap_or_default();
        let icon = command.icon.as_deref().unwrap_or("");
        ui.add_enabled_ui(command.is_enabled(), |ui| {
            if menu_item(
                ui,
                icons,
                icon,
                &command.title,
                Some(shortcut.as_str()).filter(|text| !text.is_empty()),
                false,
            ) {
                chosen = Some(command.id.clone());
                ui.close();
            }
        });
    }
    chosen
}

/// The categories and commands for the keyboard menu overlay.
pub fn command_menu_groups(registry: &CommandRegistry, context: &WhenContext) -> Vec<MenuGroup<CommandId>> {
    let mut groups: Vec<MenuGroup<CommandId>> = Vec::new();
    for category in registry.categories() {
        let entries: Vec<MenuEntry<CommandId>> = registry
            .iter()
            .filter(|command| command.category == category && command.applies(context))
            .map(|command| {
                let label = match registry.binding(&command.id) {
                    Some(shortcut) => format!("{}    {shortcut}", command.title),
                    None => command.title.clone(),
                };
                MenuEntry {
                    label,
                    value: command.id.clone(),
                }
            })
            .collect();
        if !entries.is_empty() {
            groups.push(MenuGroup {
                label: category.to_owned(),
                entries,
            });
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Command;

    #[test]
    fn groups_follow_categories_and_context() {
        let mut registry = CommandRegistry::new();
        registry.register(Command::new("file.save", "Save").category("File").shortcut("Ctrl+S"));
        registry.register(Command::new("edit.paste", "Paste").category("Edit").shortcut("Ctrl+V"));
        registry.register(
            Command::new("edit.special", "Special")
                .category("Edit")
                .when("hasSelection"),
        );
        let groups = command_menu_groups(&registry, &WhenContext::new());
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].label, "Edit");
        assert_eq!(groups[0].entries.len(), 1);
        assert_eq!(groups[1].label, "File");
        assert!(groups[1].entries[0].label.contains("Ctrl+S"));
    }
}
