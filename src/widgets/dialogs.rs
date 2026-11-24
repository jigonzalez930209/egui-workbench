//! Modal prompts, popover and dialog frames, banners, and progress.
//!
//! Short in-app decisions use these modals, so the workbench keeps one visual language:
//! a 10 pt radius frame on the raised surface with a soft shadow, a 17 pt bold title
//! with a "×" close glyph, sunken text fields, and a right-aligned footer with the
//! primary action on the far right. Enter submits, Esc and a backdrop click cancel.

use egui::epaint::Shadow;
use egui::{Color32, Context, Frame, Id, Key, Margin, Modal, RichText, Stroke, Ui};

use crate::{close_glyph, primary_button, secondary_button, text_field};

/// What the user chose in a prompt.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PromptAction {
    /// Confirm the primary action.
    Submit,
    /// Dismiss without changing anything.
    Cancel,
}

/// The frame of modal dialogs: 10 pt radius, 18 pt padding, shadow 0/8/24 at 60 alpha.
pub fn dialog_frame(ctx: &Context) -> Frame {
    let visuals = ctx.style_of(ctx.theme()).visuals.clone();
    Frame::new()
        .fill(visuals.window_fill)
        .stroke(Stroke::new(
            crate::style::get(ctx).border_width,
            visuals.window_stroke.color,
        ))
        .corner_radius(crate::style::get(ctx).dialog())
        .inner_margin(Margin::same(18))
        .shadow(Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(60),
        })
}

/// The frame of floating popovers (search, keyboard menu): 8 pt radius, 10 pt padding,
/// shadow 0/6/18 at 48 alpha.
pub fn popover_frame(ui: &Ui) -> Frame {
    Frame::new()
        .fill(ui.visuals().window_fill)
        .stroke(crate::style::get(ui.ctx()).border(ui.visuals().window_stroke.color))
        .corner_radius(crate::style::get(ui.ctx()).container())
        .inner_margin(Margin::same(10))
        .shadow(Shadow {
            offset: [0, 6],
            blur: 18,
            spread: 0,
            color: Color32::from_black_alpha(48),
        })
}

/// A popover anchored 40 pt below the top of the window, centered, `width` wide (at
/// most the window width minus 24 pt), above every panel.
pub fn show_popover<R>(ctx: &Context, id_salt: &str, width: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let screen = ctx.content_rect();
    let width = width.min(screen.width() - 24.0);
    let position = egui::pos2(
        (screen.center().x - width * 0.5).max(screen.left() + 12.0),
        screen.top() + 40.0,
    );
    egui::Area::new(Id::unique(id_salt))
        .order(egui::Order::Foreground)
        .fixed_pos(position)
        .show(ctx, |ui| {
            popover_frame(ui)
                .show(ui, |ui| {
                    ui.set_width(width - 20.0);
                    add(ui)
                })
                .inner
        })
        .inner
}

/// The header row of a dialog: a bold title and a "×" that cancels.
pub fn dialog_header(ui: &mut Ui, title: &str, size: f32, close_tooltip: &str) -> bool {
    let mut close = false;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(title)
                .size(size)
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            close = close_glyph(ui, 14.0, close_tooltip);
        });
    });
    ui.add_space(8.0);
    close
}

/// The footer row: the primary button on the right, the cancel button 8 pt to its left.
pub fn dialog_footer(ui: &mut Ui, confirm: &str, cancel: &str, danger: bool) -> Option<PromptAction> {
    let mut action = None;
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if primary_button(ui, confirm, danger).clicked() {
            action = Some(PromptAction::Submit);
        }
        ui.add_space(8.0);
        if secondary_button(ui, cancel).clicked() {
            action = Some(PromptAction::Cancel);
        }
    });
    action
}

/// A single-line name prompt, for creating and renaming entries.
pub struct NamePrompt<'a> {
    /// Dialog title.
    pub title: &'a str,
    /// Short explanation under the title; empty for none.
    pub description: &'a str,
    /// Field label.
    pub label: &'a str,
    /// Editable value.
    pub value: &'a mut String,
    /// Primary button label.
    pub confirm: &'a str,
    /// Cancel button label.
    pub cancel: &'a str,
    /// Validation message in the error color; `None` hides it.
    pub error: Option<&'a str>,
}

/// Show a 380 pt name prompt; returns the user's choice once.
pub fn show_name_prompt(ctx: &Context, prompt: &mut NamePrompt<'_>) -> Option<PromptAction> {
    let mut action = None;
    let dialog = Modal::new(Id::unique("workbench_name_prompt"))
        .frame(dialog_frame(ctx))
        .show(ctx, |ui| {
            ui.set_width(380.0);
            if dialog_header(ui, prompt.title, 17.0, prompt.cancel) {
                action = Some(PromptAction::Cancel);
            }
            if !prompt.description.is_empty() {
                ui.label(RichText::new(prompt.description).small().weak());
                ui.add_space(10.0);
            }
            let label = ui.label(
                RichText::new(prompt.label)
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(3.0);
            // The visible label names the field for screen readers.
            let response = text_field(ui, prompt.value, "", 344.0).labelled_by(label.id);
            if ctx.memory(|memory| memory.focused()).is_none() {
                response.request_focus();
            }
            if let Some(error) = prompt.error {
                ui.add_space(5.0);
                ui.label(RichText::new(error).small().color(ui.visuals().error_fg_color));
            }
            ui.add_space(14.0);
            action = action.or(dialog_footer(ui, prompt.confirm, prompt.cancel, false));
            if ui.input(|input| input.key_pressed(Key::Enter)) {
                action = Some(PromptAction::Submit);
            }
        });
    if dialog.backdrop_response.clicked() || ctx.input(|input| input.key_pressed(Key::Escape)) {
        action = Some(PromptAction::Cancel);
    }
    action
}

/// A compact confirmation.
pub struct ConfirmPrompt<'a> {
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

/// Show a 340 pt confirmation; returns the user's choice once.
pub fn show_confirm_prompt(ctx: &Context, prompt: &ConfirmPrompt<'_>) -> Option<PromptAction> {
    let mut action = None;
    let dialog = Modal::new(Id::unique("workbench_confirm_prompt"))
        .frame(dialog_frame(ctx))
        .show(ctx, |ui| {
            ui.set_width(340.0);
            if dialog_header(ui, prompt.title, 17.0, prompt.cancel) {
                action = Some(PromptAction::Cancel);
            }
            ui.label(prompt.message);
            ui.add_space(14.0);
            action = action.or(dialog_footer(ui, prompt.confirm, prompt.cancel, prompt.danger));
        });
    if dialog.backdrop_response.clicked() || ctx.input(|input| input.key_pressed(Key::Escape)) {
        action = Some(PromptAction::Cancel);
    }
    action
}

/// A three-way choice for closing unsaved work. Returns the index of the clicked button
/// in `buttons` (for example Save, Discard, Cancel); Esc returns the last one.
pub fn show_choice_window(
    ctx: &Context,
    title: &str,
    message: &str,
    buttons: &[&str],
    open: &mut bool,
) -> Option<usize> {
    let mut choice = None;
    if ctx.input(|input| input.key_pressed(Key::Escape)) && !buttons.is_empty() {
        choice = Some(buttons.len() - 1);
    }
    if !*open {
        return choice;
    }
    Modal::new(Id::unique(("workbench_choice_window", title)))
        .frame(dialog_frame(ctx))
        .show(ctx, |ui| {
            ui.set_width(dialog_width(ctx, 380.0));
            if dialog_header(ui, title, 17.0, "Close") {
                *open = false;
            }
            ui.add(egui::Label::new(message).wrap());
            ui.add_space(16.0);
            // Right-aligned like every dialog footer: the first button is the primary
            // action on the far right, the rest follow to its left (Cancel leftmost).
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                for (index, label) in buttons.iter().enumerate() {
                    let response = if index == 0 {
                        primary_button(ui, label, false)
                    } else {
                        secondary_button(ui, label)
                    };
                    if response.clicked() {
                        choice = Some(index);
                    }
                }
            });
        });
    choice
}

/// A centered, modal progress dialog with a cancel button; returns true on cancel.
///
/// Same frame and header as the other dialogs, a full-width 6 pt bar in the selection
/// color with the percentage on the right, and the cancel button right-aligned.
pub fn show_progress(ctx: &Context, title: &str, detail: &str, progress: f32, cancel: &str) -> bool {
    let mut cancelled = false;
    let progress = progress.clamp(0.0, 1.0);
    Modal::new(Id::unique(("workbench_progress", title)))
        .frame(dialog_frame(ctx))
        .show(ctx, |ui| {
            let width = dialog_width(ctx, 400.0);
            ui.set_width(width);
            ui.label(
                RichText::new(title)
                    .size(crate::style::get(ui.ctx()).font_size(17.0))
                    .strong()
                    .color(ui.visuals().text_color()),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let percent = format!("{:.0}%", progress * 100.0);
                ui.add(egui::Label::new(RichText::new(detail).weak()).truncate());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(percent).small().weak());
                });
            });
            ui.add_space(6.0);
            ui.scope(|ui| {
                // The unfilled track uses the border token so the full length reads in
                // every theme (the sunken color matches the light surfaces).
                let track = ui.visuals().widgets.inactive.bg_stroke.color;
                ui.visuals_mut().extreme_bg_color = track;
                let fill = ui.visuals().widgets.active.bg_fill;
                ui.add(
                    egui::ProgressBar::new(progress)
                        .desired_width(width)
                        .desired_height(6.0)
                        .corner_radius(crate::style::get(ui.ctx()).tight())
                        .fill(fill),
                );
            });
            ui.add_space(16.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancelled = secondary_button(ui, cancel).clicked();
            });
        });
    cancelled
}

/// `preferred` points, but never wider than the window minus the dialog padding and a
/// 16 pt gutter on each side.
fn dialog_width(ctx: &Context, preferred: f32) -> f32 {
    preferred
        .min(ctx.content_rect().width() - 2.0 * (18.0 + 16.0))
        .max(160.0)
}

/// A full-width banner above the document, tinted with 12 % of `color` (use
/// `ui.visuals().warn_fg_color` for conflicts). `actions` adds its buttons after the text.
pub fn banner(ui: &mut Ui, color: Color32, message: &str, actions: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(color.gamma_multiply(0.12))
        .stroke(crate::style::get(ui.ctx()).border(color.gamma_multiply(0.35)))
        .corner_radius(crate::style::get(ui.ctx()).control())
        .inner_margin(Margin::symmetric(10, 5))
        .show(ui, |ui| {
            // Span the full width of the content column, like an editor notification bar.
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.set_min_height(24.0);
                ui.label(RichText::new(message).color(color));
                ui.add_space(4.0);
                actions(ui);
            });
        });
}
