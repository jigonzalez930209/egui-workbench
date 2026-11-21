//! Modal prompts, the choice window, progress, banners, popovers, and settings.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_workbench::egui::{Key, Role, accesskit::Toggled};
use egui_workbench::{ConfirmPrompt, NamePrompt, PromptAction, SettingsDialog};

#[derive(Default)]
struct Prompt {
    value: String,
    error: Option<&'static str>,
    actions: Vec<PromptAction>,
}

fn name_prompt<'a>() -> Harness<'a, Prompt> {
    let state = Prompt {
        value: "untitled.md".into(),
        ..Prompt::default()
    };
    common::harness([800.0, 600.0], state, |ui, state: &mut Prompt| {
        let mut prompt = NamePrompt {
            title: "New file",
            description: "Create a file in the workspace.",
            label: "Name",
            value: &mut state.value,
            confirm: "Create",
            cancel: "Cancel",
            error: state.error,
        };
        if let Some(action) = egui_workbench::show_name_prompt(ui.ctx(), &mut prompt) {
            state.actions.push(action);
        }
    })
}

#[test]
fn the_name_prompt_focuses_its_field_and_submits_with_enter() {
    let mut harness = name_prompt();
    harness.run();
    let field = harness.get_by_role(Role::TextInput);
    assert!(field.is_focused(), "the field takes focus");
    harness.key_press(Key::A);
    harness.run();
    harness.get_by_role(Role::TextInput).type_text("notes.md");
    harness.run();
    assert!(harness.state().value.ends_with("notes.md"));
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().actions.last(), Some(&PromptAction::Submit));
}

#[test]
fn the_name_prompt_buttons_and_escape_report_choices() {
    let mut harness = name_prompt();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Create").click();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Cancel").click();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "×").click();
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(
        harness.state().actions,
        [
            PromptAction::Submit,
            PromptAction::Cancel,
            PromptAction::Cancel,
            PromptAction::Cancel
        ]
    );
}

#[test]
fn the_name_prompt_shows_validation_errors() {
    let mut harness = name_prompt();
    harness.state_mut().error = Some("A file with this name exists.");
    harness.run();
    assert!(harness.query_by_label("A file with this name exists.").is_some());
    assert!(harness.query_by_label("New file").is_some());
    assert!(harness.query_by_label("Create a file in the workspace.").is_some());
}

#[test]
fn the_footer_puts_the_primary_action_on_the_right() {
    let mut harness = name_prompt();
    harness.run();
    let create = harness.get_by_role_and_label(Role::Button, "Create").rect();
    let cancel = harness.get_by_role_and_label(Role::Button, "Cancel").rect();
    assert!(create.left() > cancel.right());
    assert_eq!(create.center().y, cancel.center().y);
}

#[test]
fn the_confirm_prompt_reports_choices() {
    let mut harness = common::harness([800.0, 600.0], Vec::new(), |ui, actions: &mut Vec<PromptAction>| {
        let prompt = ConfirmPrompt {
            title: "Delete file?",
            message: "notes.md will be moved to the trash.",
            confirm: "Delete",
            cancel: "Keep",
            danger: true,
        };
        actions.extend(egui_workbench::show_confirm_prompt(ui.ctx(), &prompt));
    });
    harness.run();
    assert!(harness.query_by_label("notes.md will be moved to the trash.").is_some());
    harness.get_by_role_and_label(Role::Button, "Delete").click();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Keep").click();
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(
        *harness.state(),
        [PromptAction::Submit, PromptAction::Cancel, PromptAction::Cancel]
    );
}

#[test]
fn the_choice_window_returns_the_button_index_and_escape_means_the_last() {
    let mut harness = common::harness([800.0, 600.0], Vec::new(), |ui, choices: &mut Vec<usize>| {
        let mut open = true;
        let message = "Do you want to save your changes before closing?";
        choices.extend(egui_workbench::show_choice_window(
            ui.ctx(),
            "Save changes?",
            message,
            &["Save", "Discard", "Cancel"],
            &mut open,
        ));
    });
    harness.run();
    for label in ["Discard", "Save"] {
        harness.get_by_role_and_label(Role::Button, label).click();
        harness.run();
    }
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(*harness.state(), [1, 0, 2]);
}

#[test]
fn the_progress_window_shows_progress_and_cancels() {
    let mut harness = common::harness([800.0, 600.0], 0, |ui, cancels: &mut usize| {
        if egui_workbench::show_progress(ui.ctx(), "Opening file…", "/tmp/big.txt", 0.42, "Cancel") {
            *cancels += 1;
        }
    });
    harness.run();
    assert!(harness.query_by_label("/tmp/big.txt").is_some());
    assert!(harness.query_by_role(Role::ProgressIndicator).is_some());
    harness.get_by_role_and_label(Role::Button, "Cancel").click();
    harness.run();
    assert_eq!(*harness.state(), 1);
}

#[test]
fn banners_show_their_message_and_actions() {
    let mut harness = common::harness([800.0, 200.0], Vec::new(), |ui, clicked: &mut Vec<&'static str>| {
        let warn = ui.visuals().warn_fg_color;
        egui_workbench::banner(ui, warn, "The file changed on disk.", |ui| {
            if ui.button("Reload").clicked() {
                clicked.push("Reload");
            }
            if ui.button("Keep mine").clicked() {
                clicked.push("Keep mine");
            }
        });
    });
    harness.run();
    assert!(harness.query_by_label("The file changed on disk.").is_some());
    harness.get_by_role_and_label(Role::Button, "Keep mine").click();
    harness.run();
    assert_eq!(*harness.state(), ["Keep mine"]);
}

#[test]
fn popovers_sit_centered_below_the_title_bar() {
    let mut harness = common::harness([1000.0, 600.0], String::new(), |ui, query: &mut String| {
        egui_workbench::show_popover(ui.ctx(), "search", 560.0, |ui| {
            egui_workbench::dialog_header(ui, "Search", 13.0, "Close");
            egui_workbench::text_field(ui, query, "Search in document…", ui.available_width());
        });
    });
    harness.run();
    let field = harness.get_by_role(Role::TextInput).rect();
    assert!((field.center().x - 500.0).abs() <= 4.0, "{field:?}");
    assert!(field.top() > 40.0 && field.top() < 120.0, "{field:?}");
}

#[derive(Default)]
struct Settings {
    open: bool,
    tab_seen: Vec<usize>,
    wrap: bool,
}

fn settings<'a>() -> Harness<'a, Settings> {
    let state = Settings {
        open: true,
        ..Settings::default()
    };
    common::harness([900.0, 640.0], state, |ui, state: &mut Settings| {
        let dialog = SettingsDialog {
            title: "Settings",
            tabs: &["Appearance", "Editor", "About"],
            note: "App 1.0.0",
            done: "Done",
            close: "Close",
        };
        let wrap = &mut state.wrap;
        let seen = egui_workbench::show_settings_dialog(ui.ctx(), &mut state.open, &dialog, |ui, tab| {
            match tab {
                0 => egui_workbench::settings_section(ui, "Appearance", "Theme and text size."),
                1 => {
                    egui_workbench::settings_section(ui, "Editor", "Typing and layout.");
                    egui_workbench::settings_switch(ui, "Word wrap", "Wrap long lines.", wrap);
                }
                _ => egui_workbench::settings_section(ui, "About", "Version and logs."),
            }
            tab
        });
        state.tab_seen.extend(seen);
    })
}

#[test]
fn the_settings_rail_switches_tabs() {
    let mut harness = settings();
    harness.run();
    assert_eq!(harness.state().tab_seen.last(), Some(&0));
    let tab = |harness: &Harness<'_, Settings>, label: &str| {
        harness
            .get_by_role_and_label(Role::Tab, label)
            .accesskit_node()
            .toggled()
    };
    assert_eq!(tab(&harness, "Appearance"), Some(Toggled::True));
    harness.get_by_role_and_label(Role::Tab, "Editor").click();
    harness.run();
    assert_eq!(harness.state().tab_seen.last(), Some(&1));
    assert_eq!(tab(&harness, "Editor"), Some(Toggled::True));
    assert_eq!(tab(&harness, "Appearance"), Some(Toggled::False));
    harness.get_by_role_and_label(Role::CheckBox, "Word wrap").click();
    harness.run();
    assert!(harness.state().wrap);
    assert!(harness.query_by_label("App 1.0.0").is_some());
}

#[test]
fn the_settings_dialog_closes_with_done_close_and_escape() {
    for close in ["Done", "×", "Escape"] {
        let mut harness = settings();
        harness.run();
        if close == "Escape" {
            harness.key_press(Key::Escape);
        } else {
            harness.get_by_role_and_label(Role::Button, close).click();
        }
        harness.run();
        assert!(!harness.state().open, "{close}");
        assert!(harness.query_by_label("Settings").is_none(), "{close}");
    }
}

#[test]
fn the_settings_dialog_keeps_its_width_bounds() {
    let mut harness = settings();
    harness.run();
    let done = harness.get_by_role_and_label(Role::Button, "Done").rect();
    let rail = harness.get_by_role_and_label(Role::Tab, "Appearance").rect();
    let width = done.right() - rail.left();
    assert!((560.0..=640.0).contains(&width), "{width}");
    assert_eq!(rail.height(), 28.0);
}
