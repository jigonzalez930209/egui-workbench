//! IME composition and HiDPI behavior tests.
//!
//! The desktop matrix runs this on Linux, macOS, and Windows. It exercises the two
//! platform-sensitive paths named by the 0.3.x hardening release: composition events
//! (accents, CJK) and a 2× pixels-per-point frame.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use egui_workbench::egui::{self, Event, ImeEvent, Role};
use egui_workbench::theme;
use egui_workbench::{ThemeId, components::Component, components::TextField};

#[derive(Default)]
struct State {
    text: String,
}

fn harness(pixels_per_point: f32) -> Harness<'static, State> {
    Harness::builder()
        .with_size(egui::vec2(520.0, 220.0))
        .with_pixels_per_point(pixels_per_point)
        .with_max_steps(16)
        .with_context_setup(|ctx| theme::apply_id(ctx, ThemeId::GithubLight))
        .build_ui_state(
            |ui, state: &mut State| {
                TextField::new(&mut state.text)
                    .label("Name")
                    .hint("Type…")
                    .width(360.0)
                    .show(ui);
            },
            State::default(),
        )
}

#[test]
fn ime_preedit_then_commit_inserts_the_composed_text() {
    let mut harness = harness(1.0);
    harness.run();
    harness.get_by_role(Role::TextInput).click();
    harness.run();

    // A keyboard reports the composition, then the committed character.
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "e".to_owned(),
        active_range_chars: Some(0..1),
    }));
    harness.run();
    harness.event(Event::Ime(ImeEvent::Commit("é".to_owned())));
    harness.run();

    assert_eq!(harness.state().text, "é");
}

#[test]
fn two_x_pixels_per_point_keeps_text_and_layout_usable() {
    let mut harness = harness(2.0);
    harness.run();
    assert!(harness.ctx.pixels_per_point() >= 2.0);
    assert!(harness.query_by_label("Name").is_some());
    harness.get_by_role(Role::TextInput).click();
    harness.run();
    harness.event(Event::Text("HiDPI".to_owned()));
    harness.run();
    assert_eq!(harness.state().text, "HiDPI");
}
