//! Buttons, the segmented control, form controls, and menu rows.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::kittest::{NodeT, Queryable};
use egui_workbench::egui::{self, Key, Role, accesskit::Toggled};
use egui_workbench::{IconState, Icons, Segment, ThemeId};

#[derive(Default)]
struct State {
    icons: Icons,
    clicks: Vec<String>,
    mode: usize,
    switch: bool,
    zoom: f32,
    text: String,
    theme: Option<ThemeId>,
}

fn controls<'a>() -> egui_kittest::Harness<'a, State> {
    let state = State {
        zoom: 1.0,
        theme: Some(ThemeId::GithubLight),
        ..State::default()
    };
    common::harness([700.0, 500.0], state, |ui, state: &mut State| {
        let icons = &mut state.icons;
        let clicks = &mut state.clicks;
        ui.horizontal(|ui| {
            if egui_workbench::action_button(ui, icons, "settings", "Settings", IconState::NORMAL) {
                clicks.push("Settings".into());
            }
            if egui_workbench::action_button(ui, icons, "back", "Back", IconState::enabled(false)) {
                clicks.push("Back".into());
            }
            if egui_workbench::tool_button(ui, icons, "plus", "New tab", false) {
                clicks.push("New tab".into());
            }
            if egui_workbench::small_button(ui, icons, "refresh", "Refresh") {
                clicks.push("Refresh".into());
            }
            egui_workbench::menu_tool_button(ui, icons, "more", "Tab actions", 210.0, |ui, icons| {
                if egui_workbench::menu_item(ui, icons, "close", "Close Tab", Some("Ctrl+W"), false) {
                    clicks.push("Close Tab".into());
                    ui.close();
                }
                if egui_workbench::menu_toggle(ui, "Word wrap", true, true) {
                    clicks.push("Word wrap".into());
                }
            });
        });
        let segments = [
            Segment {
                icon: "eye",
                tooltip: "View",
                enabled: true,
            },
            Segment {
                icon: "edit",
                tooltip: "Editor",
                enabled: true,
            },
            Segment {
                icon: "split",
                tooltip: "Split",
                enabled: false,
            },
        ];
        if let Some(index) = egui_workbench::segmented(ui, icons, "mode", &segments, state.mode) {
            state.mode = index;
        }
        egui_workbench::switch(ui, &mut state.switch, "Word wrap switch");
        egui_workbench::numeric_stepper(ui, icons, &mut state.zoom, 0.8..=2.0, 0.1, 112.0, |value| {
            format!("{:.0}%", value * 100.0)
        });
        egui_workbench::text_field(ui, &mut state.text, "Type to filter…", 300.0);
        let mut theme = state.theme.unwrap_or_default();
        let options: Vec<(ThemeId, String)> = ThemeId::ALL.iter().map(|id| (*id, id.label().to_owned())).collect();
        if egui_workbench::combo(ui, "Theme", &mut theme, &options) {
            state.theme = Some(theme);
        }
        ui.horizontal(|ui| {
            if egui_workbench::primary_button(ui, "Rename", false).clicked() {
                clicks.push("Rename".into());
            }
            if egui_workbench::primary_button(ui, "Delete", true).clicked() {
                clicks.push("Delete".into());
            }
            if egui_workbench::secondary_button(ui, "Cancel").clicked() {
                clicks.push("Cancel".into());
            }
            if egui_workbench::close_glyph(ui, 14.0, "Close") {
                clicks.push("×".into());
            }
        });
    })
}

#[test]
fn icon_buttons_report_clicks_and_respect_disabled() {
    let mut harness = controls();
    harness.run();
    for label in ["Settings", "Back", "New tab", "Refresh"] {
        harness.get_by_role_and_label(Role::Button, label).click();
        harness.run();
    }
    assert_eq!(harness.state().clicks, ["Settings", "New tab", "Refresh"]);
    let back = harness.get_by_role_and_label(Role::Button, "Back");
    assert!(back.accesskit_node().is_disabled());
}

#[test]
fn icon_button_sizes_match_the_metrics() {
    let mut harness = controls();
    harness.run();
    let size = |label: &str| harness.get_by_role_and_label(Role::Button, label).rect().size();
    assert_eq!(size("Settings"), egui::vec2(28.0, 28.0));
    assert_eq!(size("New tab"), egui::vec2(26.0, 26.0));
    assert_eq!(size("Refresh"), egui::vec2(21.0, 21.0));
}

#[test]
fn the_overflow_button_opens_its_menu() {
    let mut harness = controls();
    harness.run();
    assert!(harness.query_by_label("Close Tab").is_none());
    harness.get_by_role_and_label(Role::Button, "Tab actions").click();
    harness.run();
    let row = harness.get_by_role_and_label(Role::MenuItem, "Close Tab");
    assert_eq!(row.rect().height(), 22.0);
    row.click();
    harness.run();
    assert_eq!(harness.state().clicks, ["Close Tab"]);
    assert!(harness.query_by_label("Close Tab").is_none(), "the menu closes");
}

#[test]
fn menu_toggles_run_and_close_the_menu() {
    let mut harness = controls();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Tab actions").click();
    harness.run();
    harness.get_by_label("Word wrap").click();
    harness.run();
    assert_eq!(harness.state().clicks, ["Word wrap"]);
    assert!(harness.query_by_label("Word wrap").is_none());
}

#[test]
fn the_segmented_control_selects_enabled_segments_only() {
    let mut harness = controls();
    harness.run();
    let toggled = |harness: &egui_kittest::Harness<'_, State>, label: &str| {
        harness
            .get_by_role_and_label(Role::RadioButton, label)
            .accesskit_node()
            .toggled()
    };
    assert_eq!(toggled(&harness, "View"), Some(Toggled::True));
    harness.get_by_role_and_label(Role::RadioButton, "Editor").click();
    harness.run();
    assert_eq!(harness.state().mode, 1);
    assert_eq!(toggled(&harness, "Editor"), Some(Toggled::True));
    harness.get_by_role_and_label(Role::RadioButton, "Split").click();
    harness.run();
    assert_eq!(harness.state().mode, 1, "a disabled segment ignores clicks");
    let width = harness.get_by_role_and_label(Role::RadioButton, "View").rect().width();
    assert_eq!(width, 34.0, "36 pt cells inset 1 pt per side");
}

#[test]
fn the_switch_toggles_and_announces_its_state() {
    let mut harness = controls();
    harness.run();
    let switch = harness.get_by_role_and_label(Role::CheckBox, "Word wrap switch");
    assert_eq!(switch.rect().size(), egui::vec2(36.0, 19.0));
    assert_eq!(switch.accesskit_node().toggled(), Some(Toggled::False));
    switch.click();
    harness.run();
    assert!(harness.state().switch);
    let switch = harness.get_by_role_and_label(Role::CheckBox, "Word wrap switch");
    assert_eq!(switch.accesskit_node().toggled(), Some(Toggled::True));
}

#[test]
fn the_stepper_steps_and_clamps() {
    let mut harness = controls();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "+").click();
    harness.run();
    assert!((harness.state().zoom - 1.1).abs() < 1e-4);
    for _ in 0..12 {
        harness.get_by_role_and_label(Role::Button, "+").click();
        harness.run();
    }
    assert_eq!(harness.state().zoom, 2.0);
    for _ in 0..20 {
        harness.get_by_role_and_label(Role::Button, "−").click();
        harness.run();
    }
    assert!((harness.state().zoom - 0.8).abs() < 1e-4);
}

#[test]
fn the_text_field_takes_typing_and_has_the_control_height() {
    let mut harness = controls();
    harness.run();
    let field = harness.get_by_role(Role::TextInput);
    assert_eq!(field.rect().height(), egui_workbench::metrics::CONTROL_HEIGHT);
    field.focus();
    harness.run();
    harness.get_by_role(Role::TextInput).type_text("hello");
    harness.run();
    assert_eq!(harness.state().text, "hello");
    harness.key_press(Key::Backspace);
    harness.run();
    assert_eq!(harness.state().text, "hell");
}

#[test]
fn the_combo_box_changes_the_value() {
    let mut harness = controls();
    harness.run();
    harness.get_by_role_and_label(Role::ComboBox, "Theme").click();
    harness.run();
    harness.get_by_label("Dracula").click();
    harness.run();
    assert_eq!(harness.state().theme, Some(ThemeId::Dracula));
}

#[test]
fn dialog_buttons_share_their_geometry() {
    let mut harness = controls();
    harness.run();
    for label in ["Rename", "Delete", "Cancel", "×"] {
        harness.get_by_role_and_label(Role::Button, label).click();
        harness.run();
    }
    assert_eq!(harness.state().clicks, ["Rename", "Delete", "Cancel", "×"]);
    let rect = |label: &str| harness.get_by_role_and_label(Role::Button, label).rect();
    assert_eq!(rect("Rename").height(), 30.0);
    assert!(rect("Rename").width() >= 92.0);
    assert!(rect("Cancel").width() >= 88.0);
    assert_eq!(rect("Rename").center().y, rect("Cancel").center().y);
}
