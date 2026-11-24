//! The registry view of the command palette.

use egui::Context;

use super::{CommandId, CommandRegistry, WhenContext};
use crate::{PaletteItem, PaletteLabels, PaletteState, show_palette};

/// Translated strings for the command palette. The defaults are English.
#[derive(Clone, Copy, Debug)]
pub struct CommandPaletteLabels<'a> {
    /// Title.
    pub title: &'a str,
    /// Filter hint.
    pub hint: &'a str,
    /// Shown when nothing matches.
    pub empty: &'a str,
    /// Close tooltip.
    pub close: &'a str,
}

impl Default for CommandPaletteLabels<'_> {
    fn default() -> Self {
        Self {
            title: "Command Palette",
            hint: "Type a command…",
            empty: "No matching commands",
            close: "Close",
        }
    }
}

impl<'a> From<CommandPaletteLabels<'a>> for PaletteLabels<'a> {
    fn from(labels: CommandPaletteLabels<'a>) -> Self {
        Self {
            title: labels.title,
            hint: labels.hint,
            empty: labels.empty,
            close: labels.close,
        }
    }
}

/// The palette rows for the commands that apply in `context`, recent first.
pub fn command_palette_items(registry: &CommandRegistry, context: &WhenContext) -> Vec<PaletteItem<CommandId>> {
    let mut items: Vec<PaletteItem<CommandId>> = Vec::with_capacity(registry.len());
    for id in registry.recent() {
        if let Some(item) = item_for(registry, context, id) {
            items.push(item);
        }
    }
    for command in registry.iter() {
        if registry.recent().any(|recent| recent == &command.id) {
            continue;
        }
        if let Some(item) = item_for(registry, context, &command.id) {
            items.push(item);
        }
    }
    items
}

fn item_for(registry: &CommandRegistry, context: &WhenContext, id: &CommandId) -> Option<PaletteItem<CommandId>> {
    let command = registry.get(id)?;
    if !command.is_enabled() || !command.applies(context) {
        return None;
    }
    let shortcut = registry.binding(id).map(ToString::to_string).unwrap_or_default();
    let detail = if shortcut.is_empty() {
        command.category.clone()
    } else {
        format!("{} · {shortcut}", command.category)
    };
    Some(PaletteItem {
        label: command.title.clone(),
        detail,
        value: id.clone(),
    })
}

/// Show the command palette and return the command that ran.
pub fn show_command_palette(
    ctx: &Context,
    state: &mut PaletteState,
    labels: CommandPaletteLabels<'_>,
    registry: &mut CommandRegistry,
    context: &WhenContext,
) -> Option<CommandId> {
    let items = command_palette_items(registry, context);
    let chosen = show_palette(ctx, state, labels.into(), &items)?;
    registry.note_used(chosen.clone());
    Some(chosen)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Command, WhenContext};

    fn registry() -> CommandRegistry {
        let mut registry = CommandRegistry::new();
        registry.register(Command::new("file.save", "Save").category("File").shortcut("Ctrl+S"));
        registry.register(Command::new("edit.paste", "Paste").category("Edit").shortcut("Ctrl+V"));
        registry.register(
            Command::new("edit.paste-special", "Paste Special")
                .category("Edit")
                .when("hasSelection"),
        );
        registry
    }

    #[test]
    fn items_respect_context_and_enabled_state() {
        let mut registry = registry();
        let context = WhenContext::new();
        assert_eq!(command_palette_items(&registry, &context).len(), 2);
        registry.set_enabled(&CommandId::new("file.save"), false);
        let items = command_palette_items(&registry, &context);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Paste");
        let selection = WhenContext::new().with("hasSelection");
        assert_eq!(command_palette_items(&registry, &selection).len(), 2);
    }

    #[test]
    fn recent_commands_come_first() {
        let mut registry = registry();
        registry.note_used(CommandId::new("edit.paste"));
        let items = command_palette_items(&registry, &WhenContext::new());
        assert_eq!(items[0].label, "Paste");
        assert!(items[0].detail.contains("Ctrl+V"));
    }
}
