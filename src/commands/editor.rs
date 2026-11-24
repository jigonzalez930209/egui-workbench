//! A keybindings editor built from the command registry.

use egui::epaint::Shadow;
use egui::{Color32, Context, Frame, Id, Modal, RichText, ScrollArea, Sense};

use super::{CommandId, CommandRegistry, KeySequence};
use crate::{close_glyph, text_field};

/// One change the editor applied to the registry.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeybindingChange {
    /// The command now uses this sequence.
    Bound {
        /// The command.
        command: CommandId,
        /// The new sequence.
        sequence: KeySequence,
    },
    /// The command has no shortcut.
    Unbound {
        /// The command.
        command: CommandId,
    },
    /// The command went back to its default.
    Reset {
        /// The command.
        command: CommandId,
    },
}

/// Persistent editor state.
#[derive(Clone, Debug, Default)]
pub struct KeybindingsEditorState {
    /// Whether the editor is visible.
    pub open: bool,
    /// Filter text.
    pub filter: String,
    /// The command waiting for the next key press.
    pub recording: Option<CommandId>,
    /// The last error, shown under the list.
    pub error: Option<String>,
}

impl KeybindingsEditorState {
    /// Open the editor with an empty filter.
    pub fn open(&mut self) {
        self.open = true;
        self.recording = None;
        self.error = None;
    }

    /// Close the editor.
    pub fn close(&mut self) {
        self.open = false;
        self.recording = None;
        self.error = None;
    }
}

/// Translated strings of the editor.
#[derive(Clone, Copy, Debug)]
pub struct KeybindingsLabels<'a> {
    /// Title.
    pub title: &'a str,
    /// Filter hint.
    pub filter: &'a str,
    /// Rebind button.
    pub rebind: &'a str,
    /// Reset button.
    pub reset: &'a str,
    /// Unbind button.
    pub unbind: &'a str,
    /// Shown while waiting for a key.
    pub recording: &'a str,
    /// Shown when no command matches.
    pub empty: &'a str,
    /// Close tooltip.
    pub close: &'a str,
}

impl Default for KeybindingsLabels<'_> {
    fn default() -> Self {
        Self {
            title: "Keyboard Shortcuts",
            filter: "Filter commands…",
            rebind: "Rebind",
            reset: "Reset",
            unbind: "Unbind",
            recording: "Press a key or chord…",
            empty: "No matching commands",
            close: "Close",
        }
    }
}

/// Show the keybindings editor. Changes are applied to `registry` immediately and
/// returned for the application's journal or settings file.
pub fn show_keybindings_editor(
    ctx: &Context,
    state: &mut KeybindingsEditorState,
    registry: &mut CommandRegistry,
    labels: KeybindingsLabels<'_>,
) -> Vec<KeybindingChange> {
    let mut changes = Vec::new();
    if !state.open {
        return changes;
    }
    if let Some(command) = state.recording.clone()
        && let Some(chord) = CommandRegistry::pressed_chord(ctx)
    {
        // The recording consumes the key so it cannot also trigger a command.
        ctx.input_mut(|input| input.events.retain(|event| !matches!(event, egui::Event::Key { .. })));
        if chord.key == egui::Key::Escape {
            state.recording = None;
            state.error = None;
        } else {
            let sequence = KeySequence::chord(chord);
            match registry.bind(&command, sequence.clone()) {
                Ok(()) => {
                    state.recording = None;
                    state.error = None;
                    changes.push(KeybindingChange::Bound { command, sequence });
                }
                Err(error) => state.error = Some(error.to_string()),
            }
        }
    }

    let mut keep_open = true;
    let visuals = ctx.style_of(ctx.theme()).visuals.clone();
    let frame = Frame::new()
        .fill(visuals.window_fill)
        .stroke(crate::style::get(ctx).border(visuals.window_stroke.color))
        .corner_radius(crate::style::get(ctx).dialog())
        .inner_margin(egui::Margin::same(16))
        .shadow(Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(60),
        });
    let modal = Modal::new(Id::unique("workbench_keybindings"))
        .frame(frame)
        .show(ctx, |ui| {
            ui.set_width(620.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(labels.title)
                        .size(crate::style::get(ui.ctx()).font_size(15.0))
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if close_glyph(ui, 14.0, labels.close) {
                        keep_open = false;
                    }
                });
            });
            ui.add_space(8.0);
            text_field(ui, &mut state.filter, labels.filter, ui.available_width());
            ui.add_space(6.0);
            let filter = state.filter.to_lowercase();
            let rows: Vec<(CommandId, String, String, bool)> = registry
                .iter()
                .filter(|command| {
                    filter.is_empty()
                        || command.title.to_lowercase().contains(&filter)
                        || command.id.as_str().to_lowercase().contains(&filter)
                })
                .map(|command| {
                    (
                        command.id.clone(),
                        command.title.clone(),
                        registry
                            .binding(&command.id)
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                        registry.has_override(&command.id),
                    )
                })
                .collect();
            ScrollArea::vertical()
                .max_height(360.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    if rows.is_empty() {
                        ui.label(RichText::new(labels.empty).weak());
                    }
                    for (id, title, shortcut, has_override) in rows {
                        let recording = state.recording.as_ref() == Some(&id);
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), crate::style::get(ui.ctx()).list_row_height()),
                            Sense::hover(),
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + 4.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &title,
                            crate::style::get(ui.ctx()).font(12.0),
                            ui.visuals().text_color(),
                        );
                        let hint = if recording {
                            labels.recording.to_owned()
                        } else if shortcut.is_empty() {
                            "—".to_owned()
                        } else {
                            shortcut
                        };
                        ui.painter().text(
                            egui::pos2(rect.left() + 300.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &hint,
                            crate::style::get(ui.ctx()).font(11.0),
                            if recording {
                                ui.visuals().warn_fg_color
                            } else {
                                ui.visuals().weak_text_color()
                            },
                        );
                        ui.scope_builder(
                            egui::UiBuilder::new()
                                .max_rect(rect)
                                .layout(egui::Layout::right_to_left(egui::Align::Center)),
                            |ui| {
                                if ui.small_button(labels.rebind).clicked() {
                                    state.recording = Some(id.clone());
                                    state.error = None;
                                }
                                if has_override && ui.small_button(labels.reset).clicked() {
                                    registry.reset_binding(&id);
                                    changes.push(KeybindingChange::Reset { command: id.clone() });
                                }
                                if ui.small_button(labels.unbind).clicked() {
                                    registry.unbind(&id);
                                    changes.push(KeybindingChange::Unbound { command: id.clone() });
                                }
                            },
                        );
                    }
                });
            if let Some(error) = &state.error {
                ui.add_space(4.0);
                ui.label(RichText::new(error).color(ui.visuals().error_fg_color));
            }
        });
    if modal.backdrop_response.clicked()
        || (ctx.input(|input| input.key_pressed(egui::Key::Escape)) && state.recording.is_none())
    {
        keep_open = false;
    }
    if !keep_open {
        state.close();
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Command;

    #[test]
    fn state_opens_and_closes_cleanly() {
        let mut state = KeybindingsEditorState::default();
        state.open();
        assert!(state.open);
        state.recording = Some(CommandId::new("file.save"));
        state.close();
        assert!(!state.open);
        assert!(state.recording.is_none());
    }

    #[test]
    fn labels_default_to_english() {
        let labels = KeybindingsLabels::default();
        assert_eq!(labels.title, "Keyboard Shortcuts");
        assert_eq!(labels.rebind, "Rebind");
        let mut registry = CommandRegistry::new();
        registry.register(Command::new("file.save", "Save").shortcut("Ctrl+S"));
        assert!(registry.get(&CommandId::new("file.save")).is_some());
    }
}
