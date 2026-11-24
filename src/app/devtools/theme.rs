//! Live theme tokens with WCAG AA feedback.

use egui::{RichText, Ui};

use crate::foundation::theme::Theme;

pub fn show_theme_tokens(ui: &mut Ui, theme: &mut Theme) -> bool {
    let mut changed = false;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (label, color) in [
            ("Background", &mut theme.background),
            ("Surface", &mut theme.surface),
            ("Raised", &mut theme.surface_raised),
            ("Sunken", &mut theme.sunken),
            ("Text", &mut theme.text),
            ("Muted text", &mut theme.text_muted),
            ("Accent", &mut theme.accent),
            ("Accent text", &mut theme.accent_text),
            ("Border", &mut theme.border),
            ("Selection", &mut theme.selection),
            ("Hover", &mut theme.hover),
            ("Danger", &mut theme.danger),
            ("Warning", &mut theme.warning),
            ("Success", &mut theme.success),
            ("Done", &mut theme.done),
        ] {
            ui.horizontal(|ui| {
                ui.label(label);
                let mut value = *color;
                if ui.color_edit_button_srgba(&mut value).changed() {
                    *color = value;
                    changed = true;
                }
            });
        }
        ui.separator();
        let issues = theme.accessibility_issues();
        if issues.is_empty() {
            ui.label(RichText::new("All text pairs pass WCAG AA.").color(theme.success));
        } else {
            for (pair, ratio) in issues {
                ui.label(RichText::new(format!("{pair}: {ratio:.2}:1")).color(theme.danger));
            }
        }
    });
    changed
}
