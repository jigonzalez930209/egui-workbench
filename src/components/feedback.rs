//! Inline feedback: banners above content.

use egui::{Color32, Context, Ui};

use crate::components::style::Tone;
use crate::{banner, show_popover};

/// A full-width banner with a message and optional actions.
///
/// ```
/// # use egui_workbench::components::{Banner, Tone, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// Banner::new("The file changed on disk.").tone(Tone::Warning).show(ui, |ui| {
///     if Button::new("Reload").size(Size::Sm).show(ui).clicked() {
///         // reload
///     }
/// });
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct Banner<'a> {
    message: &'a str,
    color: Option<Color32>,
    tone: Tone,
}

impl<'a> Banner<'a> {
    /// A warning-toned banner.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            color: None,
            tone: Tone::Warning,
        }
    }

    /// Select the semantic color.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Use an explicit theme token, such as `theme.success`.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Show the banner and add actions after the message.
    pub fn show(self, ui: &mut Ui, actions: impl FnOnce(&mut Ui)) {
        let color = self.color.unwrap_or_else(|| self.tone.text(ui.visuals()));
        banner(ui, color, self.message, actions);
    }
}

/// A foreground popover anchored below the window title bar.
pub struct Popover<'a> {
    id: &'a str,
    width: f32,
}

impl<'a> Popover<'a> {
    /// A popover with a stable id.
    pub fn new(id: &'a str) -> Self {
        Self { id, width: 420.0 }
    }

    /// Popover width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Show the popover and return the closure value.
    pub fn show<R>(self, ctx: &Context, add: impl FnOnce(&mut Ui) -> R) -> R {
        show_popover(ctx, self.id, self.width, add)
    }
}
