//! Commands, their shortcuts, and the registry every chrome surface reads.
//!
//! The application still owns a small `Command` enum for payloads. The registry holds
//! the metadata, shortcuts, categories, and contexts; menus, the palette, the keyboard
//! menu, and the keybindings editor are views of it.

use egui_workbench::ThemeId;
use egui_workbench::commands::{Command as CommandMeta, CommandId, CommandRegistry, WhenContext};

use crate::model::GallerySection;

/// Every action the application can perform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// New tab (opens the Buttons example).
    NewTab,
    /// Open the editor tab for one gallery section.
    OpenExample(GallerySection),
    /// Save the active tab.
    Save,
    /// Close the active tab.
    CloseTab,
    /// Select the next tab.
    NextTab,
    /// Select the previous tab.
    PreviousTab,
    /// Show or hide the navigator.
    ToggleSidebar,
    /// Show or hide the inspector (right panel).
    ToggleInspector,
    /// Show or hide the output (bottom panel).
    ToggleOutput,
    /// Open the command palette.
    Palette,
    /// Open settings.
    Settings,
    /// Open the About window.
    About,
    /// Open the custom dialog demo.
    ShowDialog,
    /// Open the confirm dialog demo.
    ShowConfirm,
    /// Open the name dialog demo.
    ShowName,
    /// Open the progress dialog demo.
    ShowProgress,
    /// Open the keyboard menu demo.
    KeyboardMenu,
    /// Switch theme.
    SetTheme(ThemeId),
    /// Follow the desktop light/dark preference.
    ToggleFollowSystem,
    /// Request window close.
    Quit,
}

/// Stable ids of the commands the application uses outside the menu.
pub mod ids {
    /// New tab.
    pub const NEW_TAB: &str = "file.new";
    /// Save.
    pub const SAVE: &str = "file.save";
    /// Close tab.
    pub const CLOSE_TAB: &str = "file.close";
    /// Quit.
    pub const QUIT: &str = "file.quit";
    /// Next tab.
    pub const NEXT_TAB: &str = "view.next-tab";
    /// Previous tab.
    pub const PREVIOUS_TAB: &str = "view.previous-tab";
    /// Toggle the navigator.
    pub const TOGGLE_SIDEBAR: &str = "view.navigator";
    /// Toggle the inspector.
    pub const TOGGLE_INSPECTOR: &str = "view.inspector";
    /// Toggle the output.
    pub const TOGGLE_OUTPUT: &str = "view.output";
    /// Follow the system theme.
    pub const FOLLOW_SYSTEM: &str = "theme.follow-system";
    /// Settings.
    pub const SETTINGS: &str = "workbench.settings";
    /// About.
    pub const ABOUT: &str = "workbench.about";
    /// The palette itself is opened from the title bar or the shortcut.
    pub const PALETTE: &str = "workbench.palette";
}

/// The registry: metadata, default shortcuts, and categories.
pub fn registry() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    registry.register_all([
        CommandMeta::new(ids::NEW_TAB, "New Tab")
            .category("File")
            .shortcut("Ctrl+N")
            .icon("plus"),
        CommandMeta::new(ids::SAVE, "Save")
            .category("File")
            .shortcut("Ctrl+S")
            .icon("save"),
        CommandMeta::new(ids::CLOSE_TAB, "Close Tab")
            .category("File")
            .shortcut("Ctrl+W")
            .icon("close"),
        CommandMeta::new(ids::QUIT, "Quit")
            .category("File")
            .shortcut("Ctrl+Q")
            .icon("power"),
        CommandMeta::new(ids::NEXT_TAB, "Next Tab")
            .category("View")
            .shortcut("Ctrl+Tab"),
        CommandMeta::new(ids::PREVIOUS_TAB, "Previous Tab")
            .category("View")
            .shortcut("Ctrl+Shift+Tab"),
        CommandMeta::new(ids::TOGGLE_SIDEBAR, "Navigator")
            .category("View")
            .shortcut("Ctrl+B")
            .icon("sidebar"),
        CommandMeta::new(ids::TOGGLE_INSPECTOR, "Inspector")
            .category("View")
            .shortcut("Ctrl+Alt+B")
            .icon("panel-right"),
        CommandMeta::new(ids::TOGGLE_OUTPUT, "Output")
            .category("View")
            .shortcut("Ctrl+J")
            .icon("terminal"),
        CommandMeta::new(ids::FOLLOW_SYSTEM, "Follow System Theme")
            .category("Theme")
            .icon("monitor"),
        CommandMeta::new(ids::SETTINGS, "Settings…")
            .category("Help")
            .shortcut("Ctrl+,")
            .icon("settings"),
        CommandMeta::new(ids::ABOUT, "About").category("Help").icon("info"),
        CommandMeta::new(ids::PALETTE, "Command Palette")
            .category("Help")
            .shortcut("Ctrl+Shift+P")
            .icon("command"),
        CommandMeta::new("demo.dialog", "Dialog Demo").category("Help"),
        CommandMeta::new("demo.confirm", "Confirm Demo").category("Help"),
        CommandMeta::new("demo.name", "Name Demo").category("Help"),
        CommandMeta::new("demo.progress", "Progress Demo").category("Help"),
        CommandMeta::new("demo.keyboard-menu", "Keyboard Menu")
            .category("Help")
            .shortcut("F10"),
    ]);
    for theme in ThemeId::ALL {
        registry.register(
            CommandMeta::new(format!("theme.{}", theme.as_setting()), theme.label())
                .category("Theme")
                .icon("palette"),
        );
    }
    for section in GallerySection::ALL {
        registry.register(
            CommandMeta::new(section.command_id(), format!("Example: {}", section.label())).category("Examples"),
        );
    }
    registry
}

/// The context keys this application publishes.
pub fn context() -> WhenContext {
    WhenContext::new()
}

/// The command for a registry id.
pub fn command_from_id(id: &CommandId) -> Option<Command> {
    let id = id.as_str();
    if let Some(theme) = id.strip_prefix("theme.") {
        return Some(Command::SetTheme(ThemeId::from_setting(theme)));
    }
    if let Some(example) = id.strip_prefix("example.") {
        return GallerySection::ALL
            .into_iter()
            .find(|section| section.command_id() == id)
            .map(Command::OpenExample)
            .or_else(|| {
                GallerySection::ALL
                    .into_iter()
                    .find(|section| section.label().eq_ignore_ascii_case(example))
                    .map(Command::OpenExample)
            });
    }
    Some(match id {
        ids::NEW_TAB => Command::NewTab,
        ids::SAVE => Command::Save,
        ids::CLOSE_TAB => Command::CloseTab,
        ids::QUIT => Command::Quit,
        ids::NEXT_TAB => Command::NextTab,
        ids::PREVIOUS_TAB => Command::PreviousTab,
        ids::TOGGLE_SIDEBAR => Command::ToggleSidebar,
        ids::TOGGLE_INSPECTOR => Command::ToggleInspector,
        ids::TOGGLE_OUTPUT => Command::ToggleOutput,
        ids::FOLLOW_SYSTEM => Command::ToggleFollowSystem,
        ids::SETTINGS => Command::Settings,
        ids::ABOUT => Command::About,
        ids::PALETTE => Command::Palette,
        "demo.dialog" => Command::ShowDialog,
        "demo.confirm" => Command::ShowConfirm,
        "demo.name" => Command::ShowName,
        "demo.progress" => Command::ShowProgress,
        "demo.keyboard-menu" => Command::KeyboardMenu,
        _ => return None,
    })
}

/// The registry id of a command, when it has one.
pub fn id_for(command: Command) -> Option<CommandId> {
    let id = match command {
        Command::NewTab => ids::NEW_TAB,
        Command::Save => ids::SAVE,
        Command::CloseTab => ids::CLOSE_TAB,
        Command::Quit => ids::QUIT,
        Command::NextTab => ids::NEXT_TAB,
        Command::PreviousTab => ids::PREVIOUS_TAB,
        Command::ToggleSidebar => ids::TOGGLE_SIDEBAR,
        Command::ToggleInspector => ids::TOGGLE_INSPECTOR,
        Command::ToggleOutput => ids::TOGGLE_OUTPUT,
        Command::ToggleFollowSystem => ids::FOLLOW_SYSTEM,
        Command::Settings => ids::SETTINGS,
        Command::About => ids::ABOUT,
        Command::Palette => ids::PALETTE,
        Command::ShowDialog => "demo.dialog",
        Command::ShowConfirm => "demo.confirm",
        Command::ShowName => "demo.name",
        Command::ShowProgress => "demo.progress",
        Command::KeyboardMenu => "demo.keyboard-menu",
        Command::SetTheme(theme) => return Some(CommandId::new(format!("theme.{}", theme.as_setting()))),
        Command::OpenExample(section) => return Some(CommandId::new(section.command_id())),
    };
    Some(CommandId::new(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_id_maps_to_a_command() {
        let registry = registry();
        for command in registry.iter() {
            assert!(command_from_id(&command.id).is_some(), "{} has no handler", command.id);
        }
    }

    #[test]
    fn commands_keep_their_default_shortcuts() {
        let registry = registry();
        assert_eq!(
            registry.binding(&CommandId::new(ids::SAVE)).map(ToString::to_string),
            Some("Ctrl+S".to_owned())
        );
    }
}
