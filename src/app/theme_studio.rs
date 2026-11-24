//! Theme Studio: import VS Code themes, edit tokens with AA feedback, and export JSON or
//! CSS custom properties.

use egui::{RichText, Ui};

use crate::foundation::theme::Theme;

use super::devtools::show_theme_tokens;

/// Theme Studio state.
#[derive(Clone, Debug)]
pub struct ThemeStudioState {
    /// The theme being edited.
    pub theme: Theme,
    /// VS Code JSON pasted by the user.
    pub import_text: String,
    /// Import/export feedback.
    pub message: Option<String>,
    /// Last exported text (JSON or CSS).
    pub export_text: String,
}

impl Default for ThemeStudioState {
    fn default() -> Self {
        Self {
            theme: Theme::dark(),
            import_text: String::new(),
            message: None,
            export_text: String::new(),
        }
    }
}

impl ThemeStudioState {
    /// Import a VS Code theme. Returns whether the theme changed.
    pub fn import_vscode(&mut self, text: &str) -> bool {
        match Theme::from_vscode_json(text) {
            Ok(theme) => {
                self.theme = theme;
                self.message = Some("Imported and adjusted to WCAG AA".to_owned());
                true
            }
            Err(error) => {
                self.message = Some(error.to_string());
                false
            }
        }
    }

    /// Export the theme as JSON tokens.
    pub fn export_json(&mut self) {
        self.export_text = serde_json::to_string_pretty(&self.theme.to_json()).unwrap_or_default();
        self.message = Some("Exported JSON tokens".to_owned());
    }

    /// Export the theme as CSS custom properties.
    pub fn export_css(&mut self) {
        self.export_text = self.theme.to_css_variables();
        self.message = Some("Exported CSS variables".to_owned());
    }
}

/// Show Theme Studio. Returns whether the theme changed this frame.
pub fn show_theme_studio(ui: &mut Ui, state: &mut ThemeStudioState) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Theme Studio").strong());
        if ui.button("Light").clicked() {
            state.theme = Theme::light();
            changed = true;
        }
        if ui.button("Dark").clicked() {
            state.theme = Theme::dark();
            changed = true;
        }
        if ui.button("High contrast").clicked() {
            state.theme = state.theme.high_contrast();
            changed = true;
        }
        if ui.button("Export JSON").clicked() {
            state.export_json();
        }
        if ui.button("Export CSS").clicked() {
            state.export_css();
        }
    });
    if let Some(message) = &state.message {
        ui.label(RichText::new(message).small().weak());
    }
    ui.separator();
    ui.columns(2, |columns| {
        egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
            ui.label(RichText::new("VS Code theme JSON").strong());
            ui.add(
                egui::TextEdit::multiline(&mut state.import_text)
                    .hint_text("{ \"colors\": { \"editor.background\": \"#1e1e1e\", … } }")
                    .desired_rows(8),
            );
            if ui.button("Import VS Code theme").clicked() {
                changed |= state.import_vscode(&state.import_text.clone());
            }
            ui.separator();
            changed |= show_theme_tokens(ui, &mut state.theme);
        });
        egui::ScrollArea::vertical().show(&mut columns[1], |ui| {
            ui.label(RichText::new("Export").strong());
            ui.add(
                egui::TextEdit::multiline(&mut state.export_text)
                    .desired_rows(20)
                    .interactive(false),
            );
        });
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_a_vscode_theme_and_exports() {
        let ctx = egui::Context::default();
        let mut state = ThemeStudioState::default();
        let text = r##"{"colors":{"editor.background":"#1e1e1e","foreground":"#d4d4d4","focusBorder":"#007acc"}}"##;
        assert!(state.import_vscode(text));
        assert!(state.message.as_deref().unwrap_or_default().contains("Imported"));
        state.export_json();
        assert!(state.export_text.contains("#1e1e1e"));
        state.export_css();
        assert!(state.export_text.contains("--wb-background"));
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let _ = show_theme_studio(ui, &mut state);
        });
        output.textures_delta.clear();
        assert!(!state.import_vscode("{}"));
    }
}
