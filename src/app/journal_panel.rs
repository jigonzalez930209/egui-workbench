//! The journal viewer used by the first DevTools slice and the History panel.

use egui::{RichText, Ui};

use super::{CommandJournal, JournalEntry};

/// Translated strings of the journal panel.
#[derive(Clone, Copy, Debug)]
pub struct JournalLabels<'a> {
    /// Panel title.
    pub title: &'a str,
    /// Shown when the journal is empty.
    pub empty: &'a str,
    /// Suffix for the argument preview.
    pub args: &'a str,
}

impl Default for JournalLabels<'_> {
    fn default() -> Self {
        Self {
            title: "Journal",
            empty: "No commands recorded yet",
            args: "args",
        }
    }
}

/// Draw the journal and return the sequence of the row the user clicked.
pub fn show_journal(ui: &mut Ui, journal: &CommandJournal, labels: JournalLabels<'_>) -> Option<u64> {
    ui.horizontal(|ui| {
        ui.label(RichText::new(labels.title).strong());
        ui.label(RichText::new(format!("{} entries", journal.len())).weak());
    });
    ui.add_space(4.0);
    if journal.is_empty() {
        ui.label(RichText::new(labels.empty).weak());
        return None;
    }
    let mut chosen = None;
    egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
        for entry in journal.entries() {
            if ui.selectable_label(false, row_text(entry, labels.args)).clicked() {
                chosen = Some(entry.sequence);
            }
        }
    });
    chosen
}

fn row_text(entry: &JournalEntry, args_label: &str) -> String {
    let mut text = format!("{:>4}  {}", entry.sequence, entry.command);
    if let Some(label) = &entry.label {
        text.push_str("  ");
        text.push_str(label);
    }
    if let Some(args) = &entry.args {
        text.push_str("  ");
        text.push_str(args_label);
        text.push_str(": ");
        let compact = args.to_string();
        let mut characters = compact.chars();
        let preview: String = characters.by_ref().take(77).collect();
        text.push_str(&preview);
        if characters.next().is_some() {
            text.push('…');
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn row_text_includes_the_command_and_arguments() {
        let mut journal = CommandJournal::new();
        journal.append_with("file.save", Value::from("a.rs"), "Save");
        let text = row_text(&journal.entries()[0], "args");
        assert!(text.contains("file.save"));
        assert!(text.contains("Save"));
        assert!(text.contains("a.rs"));
    }

    #[test]
    fn labels_default_to_english() {
        let labels = JournalLabels::default();
        assert_eq!(labels.title, "Journal");
        assert_eq!(labels.empty, "No commands recorded yet");
    }
}
