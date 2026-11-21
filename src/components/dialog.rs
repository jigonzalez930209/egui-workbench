//! Modal dialogs: a generic shell plus the name and confirm prompts.

use egui::{Context, Id, Key, Modal, Ui};

use crate::widgets::dialogs::{
    ConfirmPrompt, NamePrompt, dialog_frame, dialog_header, show_confirm_prompt, show_name_prompt, show_progress,
};

/// A custom modal with the workbench frame, title, and close glyph.
///
/// The body draws the content and the footer; use `dialog_footer` for the standard
/// primary/cancel row. Returns the body value, or `None` while the dialog is closed.
///
/// ```
/// # use egui_workbench::components::{Dialog, prelude::*};
/// # use egui_workbench::egui;
/// # let ctx = egui::Context::default();
/// let mut open = true;
/// let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
///     Dialog::new("demo", "Rename", &mut open).show(ui.ctx(), |ui| {
///         ui.label("Body");
///     });
/// });
/// output.textures_delta.clear();
/// ```
pub struct Dialog<'a> {
    id: &'a str,
    title: &'a str,
    open: &'a mut bool,
    width: f32,
    close: &'a str,
    body_size: f32,
}

impl<'a> Dialog<'a> {
    /// A modal controlled by `open`.
    pub fn new(id: &'a str, title: &'a str, open: &'a mut bool) -> Self {
        Self {
            id,
            title,
            open,
            width: 420.0,
            close: "Close",
            body_size: 17.0,
        }
    }

    /// Body width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Close-button tooltip.
    pub fn close(mut self, close: &'a str) -> Self {
        self.close = close;
        self
    }

    /// Title size in points.
    pub fn title_size(mut self, size: f32) -> Self {
        self.body_size = size;
        self
    }

    /// Show the dialog while `open` is true. Returns the body value.
    pub fn show<R>(self, ctx: &Context, body: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        if !*self.open {
            return None;
        }
        let mut keep_open = true;
        let mut result = None;
        let modal = Modal::new(Id::unique(self.id))
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                // Never wider than the window minus the frame padding and a 16 pt gutter.
                let fit = (ctx.content_rect().width() - 2.0 * (18.0 + 16.0)).max(200.0);
                ui.set_width(self.width.min(fit));
                if dialog_header(ui, self.title, self.body_size, self.close) {
                    keep_open = false;
                }
                result = Some(body(ui));
            });
        if modal.backdrop_response.clicked() || ctx.input(|input| input.key_pressed(Key::Escape)) {
            keep_open = false;
        }
        *self.open = keep_open;
        result
    }
}

/// A confirmation dialog.
pub struct ConfirmDialog<'a> {
    /// Dialog title.
    pub title: &'a str,
    /// Question or description.
    pub message: &'a str,
    /// Primary button label.
    pub confirm: &'a str,
    /// Cancel button label.
    pub cancel: &'a str,
    /// Paint the primary action as destructive.
    pub danger: bool,
}

impl<'a> ConfirmDialog<'a> {
    /// A non-destructive confirmation.
    pub fn new(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            confirm: "OK",
            cancel: "Cancel",
            danger: false,
        }
    }

    /// The primary button label.
    pub fn confirm(mut self, confirm: &'a str) -> Self {
        self.confirm = confirm;
        self
    }

    /// The cancel button label.
    pub fn cancel(mut self, cancel: &'a str) -> Self {
        self.cancel = cancel;
        self
    }

    /// Paint the primary action as destructive.
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// Show the dialog.
    pub fn show(self, ctx: &Context) -> Option<PromptAction> {
        show_confirm_prompt(
            ctx,
            &ConfirmPrompt {
                title: self.title,
                message: self.message,
                confirm: self.confirm,
                cancel: self.cancel,
                danger: self.danger,
            },
        )
    }
}

/// A name prompt: one field, validation text, and a primary/cancel footer.
pub struct NameDialog<'a> {
    /// Dialog title.
    pub title: &'a str,
    /// Optional description.
    pub description: &'a str,
    /// Field label.
    pub label: &'a str,
    /// Editable value.
    pub value: &'a mut String,
    /// Primary button label.
    pub confirm: &'a str,
    /// Cancel button label.
    pub cancel: &'a str,
    /// Validation message.
    pub error: Option<&'a str>,
}

impl<'a> NameDialog<'a> {
    /// A prompt bound to `value`.
    pub fn new(title: &'a str, label: &'a str, value: &'a mut String) -> Self {
        Self {
            title,
            description: "",
            label,
            value,
            confirm: "OK",
            cancel: "Cancel",
            error: None,
        }
    }

    /// Add a description under the title.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    /// The primary button label.
    pub fn confirm(mut self, confirm: &'a str) -> Self {
        self.confirm = confirm;
        self
    }

    /// The cancel button label.
    pub fn cancel(mut self, cancel: &'a str) -> Self {
        self.cancel = cancel;
        self
    }

    /// Validation text under the field.
    pub fn error(mut self, error: Option<&'a str>) -> Self {
        self.error = error;
        self
    }

    /// Show the prompt.
    pub fn show(self, ctx: &Context) -> Option<PromptAction> {
        let mut prompt = NamePrompt {
            title: self.title,
            description: self.description,
            label: self.label,
            value: self.value,
            confirm: self.confirm,
            cancel: self.cancel,
            error: self.error,
        };
        show_name_prompt(ctx, &mut prompt)
    }
}

/// Re-exported prompt result for callers that only import `components`.
pub use crate::widgets::dialogs::PromptAction;

/// A centered progress window with a cancel action.
pub struct ProgressDialog<'a> {
    /// Window title.
    pub title: &'a str,
    /// Detail line.
    pub detail: &'a str,
    /// Progress from 0.0 to 1.0.
    pub progress: f32,
    /// Cancel button label.
    pub cancel: &'a str,
}

impl<'a> ProgressDialog<'a> {
    /// A progress window.
    pub fn new(title: &'a str, detail: &'a str, progress: f32) -> Self {
        Self {
            title,
            detail,
            progress,
            cancel: "Cancel",
        }
    }

    /// The cancel button label.
    pub fn cancel(mut self, cancel: &'a str) -> Self {
        self.cancel = cancel;
        self
    }

    /// Show the window; returns true when Cancel was clicked.
    pub fn show(self, ctx: &Context) -> bool {
        show_progress(ctx, self.title, self.detail, self.progress, self.cancel)
    }
}
