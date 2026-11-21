//! The command palette and the keyboard menu.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_workbench::egui::{Key, Modifiers, Role, accesskit::Toggled};
use egui_workbench::{KeyboardMenuState, MenuEntry, MenuGroup, PaletteItem, PaletteLabels, PaletteState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cmd {
    Save,
    Open,
    Quit,
    Find,
    Palette,
}

fn items() -> Vec<PaletteItem<Cmd>> {
    [
        ("Save", "Ctrl+S", Cmd::Save),
        ("Open Folder…", "Ctrl+Shift+O", Cmd::Open),
        ("Quit", "Ctrl+Q", Cmd::Quit),
        ("Find", "Ctrl+F", Cmd::Find),
    ]
    .into_iter()
    .map(|(label, detail, value)| PaletteItem {
        label: label.into(),
        detail: detail.into(),
        value,
    })
    .collect()
}

#[derive(Default)]
struct Palette {
    state: PaletteState,
    ran: Vec<Cmd>,
}

fn palette<'a>() -> Harness<'a, Palette> {
    let mut state = Palette::default();
    state.state.open();
    common::harness([800.0, 600.0], state, |ui, palette: &mut Palette| {
        let labels = PaletteLabels {
            title: "Command Palette",
            hint: "Type to filter…",
            empty: "No matches",
            close: "Close",
        };
        palette.ran.extend(egui_workbench::show_palette(
            ui.ctx(),
            &mut palette.state,
            labels,
            &items(),
        ));
    })
}

fn rows(harness: &Harness<'_, Palette>) -> Vec<String> {
    harness
        .query_all_by_role(Role::ListBoxOption)
        .map(|node| node.accesskit_node().label().unwrap_or_default())
        .collect()
}

#[test]
fn the_palette_lists_everything_and_highlights_the_first_row() {
    let mut harness = palette();
    harness.run();
    assert_eq!(rows(&harness), ["Save", "Open Folder…", "Quit", "Find"]);
    let first = harness.get_by_role_and_label(Role::ListBoxOption, "Save");
    assert_eq!(first.accesskit_node().toggled(), Some(Toggled::True));
    assert_eq!(first.rect().height(), egui_workbench::metrics::LIST_ROW_HEIGHT);
    assert!(harness.get_by_role(Role::TextInput).is_focused());
}

#[test]
fn typing_filters_and_enter_runs_the_first_match() {
    let mut harness = palette();
    harness.run();
    harness.get_by_role(Role::TextInput).type_text("ctrl+shift");
    harness.run();
    assert_eq!(rows(&harness), ["Open Folder…"], "the detail is searched too");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().ran, [Cmd::Open]);
    assert!(!harness.state().state.open, "running a row closes the palette");
}

#[test]
fn clicking_any_row_runs_it() {
    let mut harness = palette();
    harness.run();
    harness.get_by_role_and_label(Role::ListBoxOption, "Quit").click();
    harness.run();
    assert_eq!(harness.state().ran, [Cmd::Quit]);
    assert!(!harness.state().state.open);
}

#[test]
fn an_unmatched_query_says_so_and_escape_closes() {
    let mut harness = palette();
    harness.run();
    harness.get_by_role(Role::TextInput).type_text("zzz");
    harness.run();
    assert!(rows(&harness).is_empty());
    assert!(harness.query_by_label("No matches").is_some());
    harness.key_press(Key::Enter);
    harness.run();
    assert!(harness.state().ran.is_empty(), "Enter with no match does nothing");
    harness.key_press(Key::Escape);
    harness.run();
    assert!(!harness.state().state.open);
}

#[test]
fn reopening_clears_the_query() {
    let mut state = PaletteState {
        open: false,
        query: "old".into(),
    };
    state.open();
    assert!(state.open && state.query.is_empty());
}

#[test]
fn the_palette_shows_at_most_twelve_rows() {
    let many: Vec<PaletteItem<usize>> = (0..30)
        .map(|index| PaletteItem {
            label: format!("Command {index}"),
            detail: String::new(),
            value: index,
        })
        .collect();
    assert_eq!(egui_workbench::filter_palette(&many, "").len(), 12);
    assert_eq!(egui_workbench::filter_palette(&many, "command 2").len(), 11);
}

fn groups() -> Vec<MenuGroup<Cmd>> {
    let entry = |label: &str, value| MenuEntry {
        label: label.into(),
        value,
    };
    vec![
        MenuGroup {
            label: "File".into(),
            entries: vec![entry("Save", Cmd::Save), entry("Quit", Cmd::Quit)],
        },
        MenuGroup {
            label: "Edit".into(),
            entries: vec![entry("Find", Cmd::Find), entry("Command Palette…", Cmd::Palette)],
        },
    ]
}

#[derive(Default)]
struct Menu {
    state: KeyboardMenuState,
    ran: Vec<Cmd>,
}

fn keyboard_menu<'a>() -> Harness<'a, Menu> {
    common::harness([800.0, 600.0], Menu::default(), |ui, menu: &mut Menu| {
        egui_workbench::open_from_keyboard(ui.ctx(), &mut menu.state, &[Key::F, Key::E]);
        menu.ran.extend(egui_workbench::show_keyboard_menu(
            ui.ctx(),
            &mut menu.state,
            &groups(),
            "Arrows navigate · Enter runs · Esc closes",
        ));
    })
}

fn selected_item(harness: &Harness<'_, Menu>) -> Option<String> {
    harness
        .get_all_by_role(Role::MenuItem)
        .find(|node| node.accesskit_node().toggled() == Some(Toggled::True))
        .and_then(|node| node.accesskit_node().label())
}

#[test]
fn f10_opens_the_first_group_and_arrows_navigate() {
    let mut harness = keyboard_menu();
    harness.run();
    assert!(harness.query_by_label("Save").is_none());
    harness.key_press(Key::F10);
    harness.run();
    assert!(harness.state().state.open);
    assert_eq!(selected_item(&harness).as_deref(), Some("Save"));
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert_eq!(selected_item(&harness).as_deref(), Some("Quit"));
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert_eq!(
        selected_item(&harness).as_deref(),
        Some("Quit"),
        "Down stops at the last item"
    );
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(selected_item(&harness).as_deref(), Some("Find"), "Right switches group");
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(selected_item(&harness).as_deref(), Some("Save"), "groups wrap around");
    harness.key_press(Key::ArrowLeft);
    harness.run();
    harness.key_press(Key::ArrowDown);
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().ran, [Cmd::Palette]);
    assert!(!harness.state().state.open, "running an item closes the menu");
}

#[test]
fn alt_letters_open_their_group_and_escape_closes() {
    let mut harness = keyboard_menu();
    harness.run();
    harness.key_press_modifiers(Modifiers::ALT, Key::E);
    harness.run();
    assert_eq!(harness.state().state.group, 1);
    assert_eq!(selected_item(&harness).as_deref(), Some("Find"));
    harness.key_press(Key::Escape);
    harness.run();
    assert!(!harness.state().state.open);
    assert!(harness.state().ran.is_empty());
}

#[test]
fn clicking_an_item_runs_it() {
    let mut harness = keyboard_menu();
    harness.run();
    harness.key_press(Key::F10);
    harness.run();
    harness.get_by_role_and_label(Role::MenuItem, "Quit").click();
    harness.run();
    assert_eq!(harness.state().ran, [Cmd::Quit]);
    assert!(
        harness
            .query_by_label("Arrows navigate · Enter runs · Esc closes")
            .is_none()
    );
}
