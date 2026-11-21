//! The title bar: window controls, drag region, menus, action buttons, and the pill.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_workbench::egui::{self, Role, ViewportCommand, pos2};
use egui_workbench::metrics::TITLE_HEIGHT;
use egui_workbench::{IconState, Icons, Pill, TitleBar, TitleBarOutput, WindowAction};

#[derive(Default)]
struct State {
    icons: Icons,
    maximized: bool,
    pill: bool,
    history: bool,
    output: Vec<TitleBarOutput>,
    ran: Vec<&'static str>,
}

fn title_bar<'a>() -> Harness<'a, State> {
    build(false)
}

fn build<'a>(realtime: bool) -> Harness<'a, State> {
    if realtime {
        common::realtime([980.0, 200.0], State::default(), draw)
    } else {
        common::harness([980.0, 200.0], State::default(), draw)
    }
}

fn draw(ui: &mut egui::Ui, state: &mut State) {
    {
        let mut bar = TitleBar::new("Document.md");
        bar.dirty = true;
        bar.maximized = state.maximized;
        if state.pill {
            bar.pill = Some(Pill {
                icon: "folder-open",
                label: "Open Folder",
                tooltip: "No folder",
            });
        }
        let history = state.history;
        let mut ran = Vec::new();
        let mut right_ran = Vec::new();
        egui::Panel::top("title").exact_size(TITLE_HEIGHT).show(ui, |ui| {
            let output = egui_workbench::show_title_bar(
                ui,
                &mut state.icons,
                &bar,
                |ui, icons| {
                    egui_workbench::title_menu(ui, "File", 230.0, |ui| {
                        if egui_workbench::menu_button(ui, "Save") {
                            ran.push("Save");
                        }
                    });
                    if egui_workbench::action_button(ui, icons, "back", "Back", IconState::enabled(history)) {
                        ran.push("Back");
                    }
                },
                |ui, icons| {
                    if egui_workbench::action_button(ui, icons, "settings", "Settings", IconState::NORMAL) {
                        right_ran.push("Settings");
                    }
                    if egui_workbench::action_button(ui, icons, "search", "Search", IconState::active(true)) {
                        right_ran.push("Search");
                    }
                },
            );
            if output != TitleBarOutput::default() {
                state.output.push(output);
            }
        });
        state.ran.extend(ran);
        state.ran.extend(right_ran);
    }
}

fn last_window_action(harness: &Harness<'_, State>) -> Option<WindowAction> {
    harness.state().output.iter().rev().find_map(|output| output.window)
}

#[test]
fn window_controls_report_their_actions() {
    let mut harness = title_bar();
    harness.run();
    for (label, action) in [
        ("Close", WindowAction::Close),
        ("Maximize", WindowAction::ToggleMaximize),
        ("Minimize", WindowAction::Minimize),
    ] {
        harness.get_by_role_and_label(Role::Button, label).click();
        harness.run();
        assert_eq!(last_window_action(&harness), Some(action), "{label}");
    }
}

#[test]
fn a_maximized_window_offers_restore() {
    let mut harness = title_bar();
    harness.state_mut().maximized = true;
    harness.run();
    assert!(harness.query_by_role_and_label(Role::Button, "Maximize").is_none());
    harness.get_by_role_and_label(Role::Button, "Restore").click();
    harness.run();
    assert_eq!(last_window_action(&harness), Some(WindowAction::ToggleMaximize));
}

#[test]
fn window_controls_keep_the_compact_pitch() {
    let mut harness = title_bar();
    harness.run();
    let center = |label: &str| harness.get_by_role_and_label(Role::Button, label).rect().center().x;
    let (close, maximize, minimize) = (center("Close"), center("Maximize"), center("Minimize"));
    assert!((close - maximize - 31.0).abs() <= 1.0, "{close} {maximize}");
    assert!((maximize - minimize - 31.0).abs() <= 1.0, "{maximize} {minimize}");
    assert!(close > 940.0, "the close button sits at the right edge: {close}");
    let (settings, search) = (center("Settings"), center("Search"));
    assert!((settings - search - 24.0).abs() <= 1.0, "{settings} {search}");
}

#[test]
fn double_clicking_the_center_toggles_maximize() {
    // The default harness clock advances more than egui's 0.3 s double-click window
    // per step.
    let mut harness = build(true);
    harness.run();
    let center = pos2(490.0, 17.0);
    for _ in 0..2 {
        common::click_at(&harness, center);
        harness.step();
    }
    harness.run();
    assert_eq!(last_window_action(&harness), Some(WindowAction::ToggleMaximize));
}

#[test]
fn dragging_the_center_moves_the_window() {
    let mut harness = title_bar();
    harness.run();
    common::press_at(&harness, pos2(490.0, 17.0));
    harness.step();
    harness.event(egui::Event::PointerMoved(pos2(520.0, 20.0)));
    harness.step();
    let mut started = common::commands(&harness).contains(&ViewportCommand::StartDrag);
    if !started {
        harness.event(egui::Event::PointerMoved(pos2(540.0, 22.0)));
        harness.step();
        started = common::commands(&harness).contains(&ViewportCommand::StartDrag);
    }
    assert!(started);
}

#[test]
fn the_edges_of_the_bar_do_not_drag() {
    let mut harness = title_bar();
    harness.run();
    common::press_at(&harness, pos2(420.0, 17.0));
    harness.step();
    harness.event(egui::Event::PointerMoved(pos2(450.0, 20.0)));
    harness.step();
    harness.event(egui::Event::PointerMoved(pos2(470.0, 20.0)));
    harness.step();
    assert!(!common::commands(&harness).contains(&ViewportCommand::StartDrag));
}

#[test]
fn menus_open_and_run_their_items() {
    let mut harness = title_bar();
    harness.run();
    harness.get_by_label("File").click();
    harness.run();
    harness.get_by_label("Save").click();
    harness.run();
    assert_eq!(harness.state().ran, ["Save"]);
    assert!(
        harness.query_by_label("Save").is_none(),
        "the menu closes after an item runs"
    );
}

#[test]
fn disabled_actions_ignore_clicks_and_enabled_ones_run() {
    let mut harness = title_bar();
    harness.run();
    let back = harness.get_by_role_and_label(Role::Button, "Back");
    assert!(back.accesskit_node().is_disabled());
    back.click();
    harness.run();
    assert!(harness.state().ran.is_empty());

    harness.state_mut().history = true;
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Back").click();
    harness.get_by_role_and_label(Role::Button, "Settings").click();
    harness.run();
    assert_eq!(harness.state().ran, ["Back", "Settings"]);
}

#[test]
fn toggled_actions_are_announced_as_selected() {
    let mut harness = title_bar();
    harness.run();
    let search = harness.get_by_role_and_label(Role::Button, "Search");
    assert_eq!(search.accesskit_node().toggled(), Some(egui::accesskit::Toggled::True));
}

#[test]
fn the_pill_reports_clicks() {
    let mut harness = title_bar();
    harness.state_mut().pill = true;
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Open Folder").click();
    harness.run();
    assert!(harness.state().output.iter().any(|output| output.pill_clicked));
}
