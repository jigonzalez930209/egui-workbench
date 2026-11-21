//! Commands, their shortcuts, and their palette rows.
//!
//! Keep every command in this enum. Menus, the palette, the keyboard map, and
//! [`crate::content`] all produce it, so each action is reachable three ways.

use egui_workbench::PaletteItem;
use egui_workbench::ThemeId;
use egui_workbench::app::Keymap;
use egui_workbench::egui::{Key, Modifiers};

use crate::model::GallerySection;

/// Every action the application can perform.
///
/// Add domain variants here, then add a shortcut, a palette row, a menu item, and an
/// arm in `App::handle`. A variant can carry data (`DoSomething(u32)`) when the payload
/// belongs to the command.
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

/// The ordered shortcut table. Longer chords come first.
pub fn keymap() -> Keymap<Command> {
    Keymap::new()
        .bind(Modifiers::COMMAND | Modifiers::SHIFT, Key::P, Command::Palette)
        .bind(Modifiers::COMMAND | Modifiers::SHIFT, Key::Tab, Command::PreviousTab)
        .bind(Modifiers::COMMAND | Modifiers::ALT, Key::B, Command::ToggleInspector)
        .bind(Modifiers::COMMAND, Key::N, Command::NewTab)
        .bind(Modifiers::COMMAND, Key::S, Command::Save)
        .bind(Modifiers::COMMAND, Key::W, Command::CloseTab)
        .bind(Modifiers::COMMAND, Key::B, Command::ToggleSidebar)
        .bind(Modifiers::COMMAND, Key::J, Command::ToggleOutput)
        .bind(Modifiers::COMMAND, Key::Comma, Command::Settings)
        .bind(Modifiers::COMMAND, Key::Q, Command::Quit)
        .bind(Modifiers::CTRL, Key::Tab, Command::NextTab)
}

/// Palette rows: every command and its shortcut.
pub fn palette_items() -> Vec<PaletteItem<Command>> {
    let mut items: Vec<PaletteItem<Command>> = [
        ("New tab", "Ctrl+N", Command::NewTab),
        ("Example: Icons", "", Command::OpenExample(GallerySection::Icons)),
        ("Example: Buttons", "", Command::OpenExample(GallerySection::Buttons)),
        ("Example: Inputs", "", Command::OpenExample(GallerySection::Inputs)),
        ("Example: Feedback", "", Command::OpenExample(GallerySection::Feedback)),
        ("Example: Menus", "", Command::OpenExample(GallerySection::Menus)),
        ("Example: Data", "", Command::OpenExample(GallerySection::Data)),
        ("Example: Charts", "", Command::OpenExample(GallerySection::Charts)),
        ("Example: Actions", "", Command::OpenExample(GallerySection::Actions)),
        ("Save", "Ctrl+S", Command::Save),
        ("Close tab", "Ctrl+W", Command::CloseTab),
        ("Next tab", "Ctrl+Tab", Command::NextTab),
        ("Previous tab", "Ctrl+Shift+Tab", Command::PreviousTab),
        ("Toggle navigator", "Ctrl+B", Command::ToggleSidebar),
        ("Toggle inspector", "Ctrl+Alt+B", Command::ToggleInspector),
        ("Toggle output", "Ctrl+J", Command::ToggleOutput),
        ("Settings…", "Ctrl+,", Command::Settings),
        ("About", "", Command::About),
        ("Dialog demo", "", Command::ShowDialog),
        ("Confirm demo", "", Command::ShowConfirm),
        ("Name demo", "", Command::ShowName),
        ("Progress demo", "", Command::ShowProgress),
        ("Keyboard menu", "F10", Command::KeyboardMenu),
        ("Follow system", "", Command::ToggleFollowSystem),
        ("Quit", "Ctrl+Q", Command::Quit),
    ]
    .into_iter()
    .map(|(label, detail, command)| PaletteItem {
        label: label.to_owned(),
        detail: detail.to_owned(),
        value: command,
    })
    .collect();
    items.extend(ThemeId::ALL.into_iter().map(|id| PaletteItem {
        label: format!("Theme: {}", id.label()),
        detail: String::new(),
        value: Command::SetTheme(id),
    }));
    items
}
