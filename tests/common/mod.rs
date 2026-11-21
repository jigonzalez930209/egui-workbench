//! Shared helpers for the component tests.

#![allow(dead_code)]

use egui_kittest::Harness;
use egui_workbench::egui::{self, Event, Modifiers, PointerButton, Pos2, ViewportCommand};
use egui_workbench::{ThemeId, theme};

/// A harness of `size` over GitHub Light with `state`.
pub fn harness<'a, S>(size: [f32; 2], state: S, app: impl FnMut(&mut egui::Ui, &mut S) + 'a) -> Harness<'a, S> {
    themed(ThemeId::GithubLight, size, state, app)
}

/// A harness with an explicit theme.
pub fn themed<'a, S>(
    id: ThemeId,
    size: [f32; 2],
    state: S,
    app: impl FnMut(&mut egui::Ui, &mut S) + 'a,
) -> Harness<'a, S> {
    Harness::builder()
        .with_size(egui::vec2(size[0], size[1]))
        .with_max_steps(64)
        .with_context_setup(move |ctx| theme::apply_id(ctx, id))
        .build_ui_state(app, state)
}

/// A harness whose clock advances 1/60 s per step, for double clicks and animations.
pub fn realtime<'a, S>(size: [f32; 2], state: S, app: impl FnMut(&mut egui::Ui, &mut S) + 'a) -> Harness<'a, S> {
    Harness::builder()
        .with_size(egui::vec2(size[0], size[1]))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .with_context_setup(|ctx| theme::apply_id(ctx, ThemeId::GithubLight))
        .build_ui_state(app, state)
}

/// Press and release the primary button at `pos` within one frame.
pub fn click_at<S>(harness: &Harness<'_, S>, pos: Pos2) {
    harness.event(Event::PointerMoved(pos));
    for pressed in [true, false] {
        harness.event(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
}

/// Press the primary button at `pos` and keep it down.
pub fn press_at<S>(harness: &Harness<'_, S>, pos: Pos2) {
    harness.event(Event::PointerMoved(pos));
    harness.event(Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
}

/// Release the primary button at `pos`.
pub fn release_at<S>(harness: &Harness<'_, S>, pos: Pos2) {
    harness.event(Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
}

/// The viewport commands of the last frame.
pub fn commands<S>(harness: &Harness<'_, S>) -> Vec<ViewportCommand> {
    harness
        .output()
        .viewport_output
        .values()
        .flat_map(|output| output.commands.iter().cloned())
        .collect()
}
