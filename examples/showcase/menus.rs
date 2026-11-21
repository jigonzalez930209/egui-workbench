//! Title-bar menus, palette entries, and keyboard-menu groups, all built from one list
//! so every command is reachable by pointer, palette, and keyboard.

use egui_workbench::KeyboardMenuState;
use egui_workbench::egui::{self, Key, Modifiers, Ui};
use egui_workbench::{MenuEntry, MenuGroup, PaletteItem, ThemeId, menu_button, menu_toggle, title_menu};

use crate::model::Cmd;

/// A labelled command with its shortcut text.
pub struct Item {
    pub label: &'static str,
    pub shortcut: &'static str,
    pub cmd: Cmd,
}

const fn item(label: &'static str, shortcut: &'static str, cmd: Cmd) -> Item {
    Item { label, shortcut, cmd }
}

/// The menu groups; `None` is a separator.
pub fn groups() -> Vec<(&'static str, Vec<Option<Item>>)> {
    vec![
        (
            "File",
            vec![
                Some(item("New", "Ctrl+N", Cmd::New)),
                Some(item("Save", "Ctrl+S", Cmd::Save)),
                None,
                Some(item("Rename File…", "F2", Cmd::Rename)),
                Some(item("Delete File…", "", Cmd::Delete)),
                None,
                Some(item("Close Tab", "Ctrl+W", Cmd::CloseTab)),
                Some(item("Close Other Tabs", "", Cmd::CloseOthers)),
                None,
                Some(item("Quit", "Ctrl+Q", Cmd::Quit)),
            ],
        ),
        (
            "Edit",
            vec![
                Some(item("Search", "Ctrl+F", Cmd::Search)),
                None,
                Some(item("Command Palette…", "Ctrl+Shift+P", Cmd::Palette)),
            ],
        ),
        (
            "View",
            vec![
                Some(item("Reading view", "", Cmd::SetMode(0))),
                Some(item("Edit mode", "Ctrl+E", Cmd::SetMode(1))),
                Some(item("Split view", "Ctrl+Shift+E", Cmd::SetMode(2))),
                None,
                Some(item("Explorer", "Ctrl+B", Cmd::ToggleSidebar)),
                Some(item("Quick Open…", "Ctrl+P", Cmd::QuickOpen)),
            ],
        ),
        (
            "Go",
            vec![
                Some(item("Next Tab", "Ctrl+Tab", Cmd::NextTab)),
                Some(item("Previous Tab", "Ctrl+Shift+Tab", Cmd::PreviousTab)),
            ],
        ),
        (
            "Help",
            vec![
                Some(item("Settings…", "Ctrl+,", Cmd::Settings)),
                Some(item("About", "", Cmd::About)),
            ],
        ),
    ]
}

/// Minimum dropdown widths, matching MD View.
const WIDTHS: [f32; 5] = [230.0, 220.0, 230.0, 200.0, 210.0];

/// The title-bar text menus. The View menu adds the theme submenu.
pub fn title_menus(ui: &mut Ui, mode: usize, sidebar: bool, theme: ThemeId, follow: bool, out: &mut Option<Cmd>) {
    for ((label, entries), width) in groups().into_iter().zip(WIDTHS) {
        title_menu(ui, label, width, |ui| {
            for entry in entries {
                let Some(entry) = entry else {
                    ui.separator();
                    continue;
                };
                let clicked = match entry.cmd {
                    Cmd::SetMode(index) => menu_toggle(ui, entry.label, mode == index, true),
                    Cmd::ToggleSidebar => menu_toggle(ui, entry.label, sidebar, true),
                    _ => menu_button(ui, entry.label),
                };
                if clicked {
                    *out = Some(entry.cmd);
                }
            }
            if label == "View" {
                ui.separator();
                ui.menu_button("Appearance", |ui| {
                    egui_workbench::compact_menu(ui);
                    if menu_toggle(ui, "Follow system", follow, true) {
                        *out = Some(Cmd::ToggleFollowSystem);
                    }
                    ui.separator();
                    for family in ["GitHub", "Dracula", "One Dark Pro"] {
                        ui.menu_button(family, |ui| {
                            egui_workbench::compact_menu(ui);
                            for id in ThemeId::ALL.into_iter().filter(|id| id.family() == family) {
                                if menu_toggle(ui, id.label(), id == theme, true) {
                                    *out = Some(Cmd::SetTheme(id));
                                }
                            }
                        });
                    }
                });
            }
        });
    }
}

/// Palette rows: every command, its shortcut as the detail.
pub fn palette_items() -> Vec<PaletteItem<Cmd>> {
    groups()
        .into_iter()
        .flat_map(|(_, entries)| entries.into_iter().flatten())
        .filter(|entry| entry.cmd != Cmd::Palette)
        .map(|entry| PaletteItem {
            label: entry.label.to_owned(),
            detail: entry.shortcut.to_owned(),
            value: entry.cmd,
        })
        .collect()
}

/// Keyboard-menu groups, opened with F10 or Alt+F/E/V/G/H.
pub fn keyboard_groups() -> Vec<MenuGroup<Cmd>> {
    groups()
        .into_iter()
        .map(|(label, entries)| MenuGroup {
            label: label.to_owned(),
            entries: entries
                .into_iter()
                .flatten()
                .map(|entry| MenuEntry {
                    label: entry.label.to_owned(),
                    value: entry.cmd,
                })
                .collect(),
        })
        .collect()
}

/// The command bound to a key pressed this frame, consuming it. F10 and Alt+letter
/// open the keyboard menu instead.
pub fn shortcut(ctx: &egui::Context, keyboard_menu: &mut KeyboardMenuState, mode: usize) -> Option<Cmd> {
    if egui_workbench::open_from_keyboard(ctx, keyboard_menu, &[Key::F, Key::E, Key::V, Key::G, Key::H]) {
        return None;
    }
    let cmd = Modifiers::COMMAND;
    let shift = Modifiers::COMMAND | Modifiers::SHIFT;
    let table = [
        (shift, Key::P, Cmd::Palette),
        (cmd, Key::P, Cmd::QuickOpen),
        (shift, Key::E, Cmd::SetMode(2)),
        (cmd, Key::E, Cmd::SetMode(if mode == 1 { 0 } else { 1 })),
        (cmd, Key::N, Cmd::New),
        (cmd, Key::S, Cmd::Save),
        (cmd, Key::W, Cmd::CloseTab),
        (cmd, Key::B, Cmd::ToggleSidebar),
        (cmd, Key::F, Cmd::Search),
        (cmd, Key::Comma, Cmd::Settings),
        (cmd, Key::Q, Cmd::Quit),
        (Modifiers::CTRL | Modifiers::SHIFT, Key::Tab, Cmd::PreviousTab),
        (Modifiers::CTRL, Key::Tab, Cmd::NextTab),
        (Modifiers::NONE, Key::F2, Cmd::Rename),
    ];
    table
        .into_iter()
        .find(|(modifiers, key, _)| ctx.input_mut(|input| input.consume_key(*modifiers, *key)))
        .map(|(_, _, cmd)| cmd)
}
