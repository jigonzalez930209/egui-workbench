//! The tab strip: selection, closing, geometry, scrolling, and the controls area.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_workbench::egui::{self, PointerButton, Role, accesskit::Toggled};
use egui_workbench::metrics::{TAB_HEIGHT, TAB_STRIP_PADDING};
use egui_workbench::{Icons, TabEvent, TabIcon, TabItem};

struct State {
    icons: Icons,
    names: Vec<String>,
    dirty: Vec<bool>,
    active: usize,
    events: Vec<TabEvent>,
    new_tab: usize,
}

fn strip<'a>(names: &[&str], width: f32) -> Harness<'a, State> {
    let state = State {
        icons: Icons::new(),
        names: names.iter().map(|name| (*name).to_owned()).collect(),
        dirty: vec![false; names.len()],
        active: 0,
        events: Vec::new(),
        new_tab: 0,
    };
    common::harness([width, 200.0], state, |ui, state: &mut State| {
        let tabs: Vec<TabItem<'_>> = state
            .names
            .iter()
            .zip(&state.dirty)
            .map(|(name, dirty)| TabItem {
                dirty: *dirty,
                ..TabItem::new(name, TabIcon::Glyph("files"))
            })
            .collect();
        let mut new_tab = false;
        let mut event = None;
        egui::Panel::top("tabs").exact_size(TAB_HEIGHT).show(ui, |ui| {
            event = egui_workbench::show_tab_strip(
                ui,
                &mut state.icons,
                &tabs,
                state.active,
                "Close",
                276.0,
                |ui, icons| {
                    new_tab = egui_workbench::tool_button(ui, icons, "plus", "New tab", false);
                },
            );
        });
        if new_tab {
            state.new_tab += 1;
        }
        if let Some(event) = event {
            state.events.push(event);
            match event {
                TabEvent::Select(index) => state.active = index,
                TabEvent::Close(index) => {
                    state.names.remove(index);
                    state.dirty.remove(index);
                    state.active = state.active.min(state.names.len().saturating_sub(1));
                }
                _ => {}
            }
        }
    })
}

#[test]
fn clicking_a_tab_selects_it() {
    let mut harness = strip(&["README.md", "main.rs", "Cargo.toml"], 980.0);
    harness.run();
    let selected = |harness: &Harness<'_, State>, label: &str| {
        harness
            .get_by_role_and_label(Role::Tab, label)
            .accesskit_node()
            .toggled()
    };
    assert_eq!(selected(&harness, "README.md"), Some(Toggled::True));
    harness.get_by_role_and_label(Role::Tab, "main.rs").click();
    harness.run();
    assert_eq!(harness.state().events, [TabEvent::Select(1)]);
    assert_eq!(selected(&harness, "main.rs"), Some(Toggled::True));
    assert_eq!(selected(&harness, "README.md"), Some(Toggled::False));
}

#[test]
fn the_close_button_and_middle_click_close_tabs() {
    let mut harness = strip(&["a.md", "b.md", "c.md"], 980.0);
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Close b.md").click();
    harness.run();
    assert_eq!(harness.state().events, [TabEvent::Close(1)]);
    assert_eq!(harness.state().names, ["a.md", "c.md"]);
    harness
        .get_by_role_and_label(Role::Tab, "c.md")
        .click_button(PointerButton::Middle);
    harness.run();
    assert_eq!(harness.state().names, ["a.md"]);
}

#[test]
fn tab_widths_follow_the_label_within_bounds() {
    let long = "a-very-long-file-name-that-does-not-fit-in-a-tab.md";
    let mut harness = strip(&["a", "README.md", long], 1400.0);
    harness.run();
    let width = |label: &str| harness.get_by_role_and_label(Role::Tab, label).rect().width();
    assert_eq!(width("a"), 96.0, "short labels get the minimum width");
    assert!(width("README.md") > 96.0 && width("README.md") < 212.0);
    assert_eq!(width(long), 212.0, "long labels are truncated at the maximum");
    assert_eq!(
        harness.get_by_role_and_label(Role::Tab, "a").rect().height(),
        TAB_HEIGHT
    );
}

#[test]
fn the_controls_area_runs_its_buttons() {
    let mut harness = strip(&["a.md"], 980.0);
    harness.run();
    let button = harness.get_by_role_and_label(Role::Button, "New tab");
    assert!(button.rect().right() > 950.0, "controls sit at the right edge");
    button.click();
    harness.run();
    assert_eq!(harness.state().new_tab, 1);
}

#[test]
fn many_tabs_scroll_and_the_active_one_is_revealed() {
    let names: Vec<String> = (0..20).map(|index| format!("document-{index:02}.md")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut harness = strip(&refs, 700.0);
    harness.run();
    let visible_right = 700.0 - 276.0;
    let last = harness.get_by_role_and_label(Role::Tab, "document-19.md").rect();
    assert!(last.left() > visible_right, "the last tab starts hidden: {last:?}");

    harness.state_mut().active = 19;
    harness.run();
    let last = harness.get_by_role_and_label(Role::Tab, "document-19.md").rect();
    assert!(
        last.right() <= visible_right + 1.0,
        "the active tab is scrolled into view: {last:?}"
    );

    // The wheel scrolls back to the start.
    harness.event(egui::Event::PointerMoved(egui::pos2(200.0, 17.0)));
    harness.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, 200.0),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
    });
    harness.run();
    let first = harness.get_by_role_and_label(Role::Tab, "document-00.md").rect();
    assert!(
        first.left() >= TAB_STRIP_PADDING && first.left() < 40.0,
        "the strip starts with a visible left inset: {first:?}"
    );
}

#[test]
fn a_dirty_tab_keeps_its_close_button_reachable() {
    let mut harness = strip(&["a.md", "b.md"], 980.0);
    harness.state_mut().dirty[1] = true;
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Close b.md").click();
    harness.run();
    assert_eq!(harness.state().names, ["a.md"]);
}
