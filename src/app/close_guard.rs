//! The unsaved-changes close guard.
//!
//! eframe reports a close request before the window closes. The guard cancels that
//! request while documents are dirty, so the application can show its own prompt.
//! After the user chooses Save or Discard, [`CloseGuard::allow_close`] lets the next
//! request through.

use egui::{Context, ViewportCommand};

/// Delays window close requests until unsaved documents are resolved.
///
/// ```no_run
/// # use egui_workbench::app::CloseGuard;
/// # struct App { guard: CloseGuard, dirty: bool }
/// # impl App {
/// fn logic(&mut self, ctx: &egui::Context) {
///     if self.guard.should_prompt(ctx, self.dirty) {
///         // Show the Save / Discard / Cancel prompt.
///     }
/// }
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CloseGuard {
    allow: bool,
}

impl CloseGuard {
    /// A guard that prompts while documents are dirty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the next close request will bypass the prompt.
    pub fn is_allowed(&self) -> bool {
        self.allow
    }

    /// Cancel a close request while `has_unsaved` is true. Returns `true` when the
    /// application must show its prompt.
    ///
    /// Call this from `eframe::App::logic`.
    pub fn should_prompt(&mut self, ctx: &Context, has_unsaved: bool) -> bool {
        if self.allow || !has_unsaved || !ctx.input(|input| input.viewport().close_requested()) {
            return false;
        }
        ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        true
    }

    /// Let the next close request through and request the close now.
    ///
    /// Call this after Save succeeded or the user chose Discard.
    pub fn allow_close(&mut self, ctx: &Context) {
        self.allow = true;
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    /// Return to prompting on the next close request.
    pub fn reset(&mut self) {
        self.allow = false;
    }
}

#[cfg(test)]
mod tests {
    use egui::{RawInput, ViewportEvent, ViewportId};

    use super::*;

    fn frame(ctx: &Context, guard: &mut CloseGuard, dirty: bool) -> (bool, Vec<ViewportCommand>) {
        let mut input = RawInput::default();
        input.viewports.entry(ViewportId::ROOT).or_default().events = vec![ViewportEvent::Close];
        let mut prompted = false;
        let mut output = ctx.run_ui(input, |ui| {
            prompted = guard.should_prompt(ui.ctx(), dirty);
        });
        output.textures_delta.clear();
        let commands = output
            .viewport_output
            .values()
            .flat_map(|output| output.commands.iter().cloned())
            .collect();
        (prompted, commands)
    }

    #[test]
    fn starts_closed_and_can_be_reset() {
        let mut guard = CloseGuard::new();
        assert!(!guard.is_allowed());
        guard.allow = true;
        assert!(guard.is_allowed());
        guard.reset();
        assert!(!guard.is_allowed());
    }

    #[test]
    fn cancels_dirty_requests_until_the_application_allows_them() {
        let ctx = Context::default();
        let mut guard = CloseGuard::new();

        let (prompted, commands) = frame(&ctx, &mut guard, false);
        assert!(!prompted);
        assert!(
            !commands
                .iter()
                .any(|command| matches!(command, ViewportCommand::CancelClose))
        );

        let (prompted, commands) = frame(&ctx, &mut guard, true);
        assert!(prompted);
        assert!(
            commands
                .iter()
                .any(|command| matches!(command, ViewportCommand::CancelClose))
        );

        guard.allow_close(&ctx);
        let (prompted, commands) = frame(&ctx, &mut guard, true);
        assert!(!prompted);
        assert!(
            !commands
                .iter()
                .any(|command| matches!(command, ViewportCommand::CancelClose))
        );

        guard.reset();
        let (prompted, _) = frame(&ctx, &mut guard, true);
        assert!(prompted);
    }
}
