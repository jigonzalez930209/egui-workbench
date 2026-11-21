//! The component layer: variants, reactivity, accessibility, and composition.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::kittest::{NodeT, Queryable};
use egui_workbench::components::Dialog;
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{Key, Role};

#[derive(Default)]
struct State {
    icons: egui_workbench::Icons,
    text: String,
    preview: bool,
    wrap: bool,
    disabled_switch: bool,
    mode: usize,
    value: f32,
    clicks: Vec<&'static str>,
    nav: usize,
    dialog_open: bool,
}

fn catalog<'a>() -> egui_kittest::Harness<'a, State> {
    common::harness([700.0, 600.0], State::default(), |ui, state: &mut State| {
        ui.horizontal(|ui| {
            if Button::new("Primary").show(ui).clicked() {
                state.clicks.push("primary");
            }
            if Button::new("Secondary")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                state.clicks.push("secondary");
            }
            Button::new("Disabled").enabled(false).show(ui);
            if Button::new("Link").variant(ButtonVariant::Link).show(ui).clicked() {
                state.clicks.push("link");
            }
            if IconButton::new("save", "Save").show(ui, &mut state.icons).clicked() {
                state.clicks.push("icon");
            }
        });
        if NavTab::new("One").active(state.nav == 0).show(ui) {
            state.nav = 0;
        }
        if NavTab::new("Two").active(state.nav == 1).show(ui) {
            state.nav = 1;
        }
        IconPreview::new("lucide-heart").size(32.0).show(ui, &mut state.icons);
        TextField::new(&mut state.text).label("Name").width(240.0).show(ui);
        if Switch::new(&mut state.preview).label("Preview").show(ui) {
            state.clicks.push("switch");
        }
        if Checkbox::new(&mut state.wrap).label("Wrap").show(ui) {
            state.clicks.push("checkbox");
        }
        Switch::new(&mut state.disabled_switch)
            .label("Disabled switch")
            .enabled(false)
            .show(ui);
        let options = [(0, "Automatic".to_owned()), (1, "Light".to_owned())];
        if Select::new("Mode", &mut state.mode, &options).show(ui) {
            state.clicks.push("select");
        }
        if Stepper::new(&mut state.value, 0.0..=10.0, 1.0)
            .suffix(" pt")
            .show(ui, &mut state.icons)
        {
            state.clicks.push("stepper");
        }
        Badge::new("Saved").tone(Tone::Neutral).show(ui);
        Card::new().title("Card").show(ui, |ui| {
            ui.label("Body");
        });
        Dialog::new("test_dialog", "Dialog", &mut state.dialog_open).show(ui.ctx(), |ui| {
            ui.label("Dialog body");
        });
    })
}

#[test]
fn buttons_are_reactive_and_disabled_buttons_are_announced() {
    let mut harness = catalog();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Primary").click();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Secondary").click();
    harness.run();
    assert_eq!(harness.state().clicks, ["primary", "secondary"]);
    let disabled = harness.get_by_role_and_label(Role::Button, "Disabled");
    assert!(disabled.accesskit_node().is_disabled());
    disabled.click();
    harness.run();
    assert_eq!(harness.state().clicks, ["primary", "secondary"]);
}

#[test]
fn icon_button_uses_the_shared_cache() {
    let mut harness = catalog();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Save").click();
    harness.run();
    assert_eq!(harness.state().clicks, ["icon"]);
}

#[test]
fn navigator_tabs_are_full_width_and_reactive() {
    let mut harness = catalog();
    harness.run();
    let tab = harness.get_by_role_and_label(Role::Tab, "Two");
    assert!(tab.rect().width() > 100.0, "{:?}", tab.rect());
    tab.click();
    harness.run();
    assert_eq!(harness.state().nav, 1);
}

#[test]
fn icon_preview_uses_the_requested_size() {
    let mut harness = catalog();
    harness.run();
    let icon = harness.get_by_role_and_label(Role::Button, "lucide-heart");
    assert_eq!(icon.rect().size(), egui::vec2(44.0, 44.0));
    icon.click();
    harness.run();
}

#[test]
fn text_field_takes_typing_and_is_labelled() {
    let mut harness = catalog();
    harness.run();
    let field = harness.get_by_role_and_label(Role::TextInput, "Name");
    field.focus();
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Name")
        .type_text("hello");
    harness.run();
    assert_eq!(harness.state().text, "hello");
}

#[test]
fn switch_and_checkbox_toggle_state() {
    let mut harness = catalog();
    harness.run();
    harness.get_by_role_and_label(Role::CheckBox, "Preview").click();
    harness.run();
    assert!(harness.state().preview);
    harness.get_by_role_and_label(Role::CheckBox, "Wrap").click();
    harness.run();
    assert!(harness.state().wrap);
    assert_eq!(harness.state().clicks, ["switch", "checkbox"]);

    let disabled = harness.get_by_role_and_label(Role::CheckBox, "Disabled switch");
    assert!(disabled.accesskit_node().is_disabled());
    disabled.click();
    harness.run();
    assert!(!harness.state().disabled_switch);
}

#[test]
fn select_and_stepper_return_changes() {
    let mut harness = catalog();
    harness.run();
    harness.get_by_role_and_label(Role::ComboBox, "Mode").click();
    harness.run();
    harness.get_by_label("Light").click();
    harness.run();
    assert_eq!(harness.state().mode, 1);
    harness.get_by_role_and_label(Role::Button, "+").click();
    harness.run();
    assert!((harness.state().value - 1.0).abs() < 1e-4);
    assert_eq!(harness.state().clicks, ["select", "stepper"]);
}

#[test]
fn dialog_closes_with_escape_and_updates_its_open_flag() {
    let mut harness = catalog();
    harness.run();
    harness.state_mut().dialog_open = true;
    harness.run();
    assert!(harness.query_by_label("Dialog body").is_some());
    harness.key_press(Key::Escape);
    harness.run();
    assert!(!harness.state().dialog_open);
}

#[test]
fn rows_of_buttons_wrap_inside_the_available_width() {
    // Regression: buttons once set their padding in a child scope, which does not take
    // part in `horizontal_wrapped`, so long rows ran past the right edge.
    for size in [Size::Sm, Size::Md, Size::Lg] {
        for variant in [ButtonVariant::Primary, ButtonVariant::Ghost, ButtonVariant::Outline] {
            let mut harness = common::harness([420.0, 300.0], Vec::new(), move |ui, rects: &mut Vec<egui::Rect>| {
                rects.clear();
                let width = 300.0;
                ui.allocate_ui(egui::vec2(width, 280.0), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for label in ["Command palette", "Settings", "About", "Custom dialog", "Progress"] {
                            rects.push(Button::new(label).size(size).variant(variant).show(ui).rect);
                        }
                    });
                    rects.push(ui.min_rect());
                });
            });
            harness.run();
            let rects = harness.state();
            let left = rects[0].left();
            for rect in &rects[..rects.len() - 1] {
                assert!(
                    rect.right() <= left + 300.0 + 0.5,
                    "{size:?} {variant:?}: {rect:?} past 300 pt"
                );
            }
            assert!(rects[4].top() > rects[0].top(), "{size:?} {variant:?}: the row wrapped");
        }
    }
}
