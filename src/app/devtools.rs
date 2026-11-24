//! The first DevTools slice: the journal and a JSON state inspector, toggled with F12.

use egui::{Context, Key, RichText, Ui};
use serde_json::Value;

use super::journal::CommandJournal;
use super::journal_panel::{JournalLabels, show_journal};

/// Which DevTools tab is open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum DevToolsTab {
    /// The command journal.
    #[default]
    Journal,
    /// The state inspector.
    State,
}

/// Persistent DevTools state.
#[derive(Clone, Debug, Default)]
pub struct DevToolsState {
    /// Whether the window is open.
    pub open: bool,
    /// The selected tab.
    pub tab: DevToolsTab,
    /// Filter for the state tree.
    pub filter: String,
}

impl DevToolsState {
    /// Toggle on F12. Returns whether the window is now open.
    pub fn toggle_on_f12(&mut self, ctx: &Context) -> bool {
        if ctx.input(|input| input.key_pressed(Key::F12)) {
            self.open = !self.open;
        }
        self.open
    }

    /// Open the journal tab.
    pub fn open_journal(&mut self) {
        self.open = true;
        self.tab = DevToolsTab::Journal;
    }

    /// Open the state tab.
    pub fn open_state(&mut self) {
        self.open = true;
        self.tab = DevToolsTab::State;
    }
}

/// Translated strings of the DevTools window.
#[derive(Clone, Copy, Debug)]
pub struct DevToolsLabels<'a> {
    /// Window title.
    pub title: &'a str,
    /// Journal tab.
    pub journal: &'a str,
    /// State tab.
    pub state: &'a str,
    /// Filter hint.
    pub filter: &'a str,
}

impl Default for DevToolsLabels<'_> {
    fn default() -> Self {
        Self {
            title: "DevTools",
            journal: "Journal",
            state: "State",
            filter: "Filter state…",
        }
    }
}

/// Show the DevTools window. `snapshot` is any serializable state the application wants
/// to inspect, usually `serde_json::to_value(&app_state)`.
pub fn show_devtools(
    ctx: &Context,
    state: &mut DevToolsState,
    journal: &CommandJournal,
    snapshot: &Value,
    labels: DevToolsLabels<'_>,
) {
    if !state.open {
        return;
    }
    let mut open = state.open;
    egui::Window::new(labels.title)
        .id(egui::Id::unique("workbench_devtools"))
        .open(&mut open)
        .default_size([620.0, 420.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.tab, DevToolsTab::Journal, labels.journal);
                ui.selectable_value(&mut state.tab, DevToolsTab::State, labels.state);
            });
            ui.separator();
            match state.tab {
                DevToolsTab::Journal => {
                    let _ = show_journal(ui, journal, JournalLabels::default());
                }
                DevToolsTab::State => {
                    ui.add(egui::TextEdit::singleline(&mut state.filter).hint_text(labels.filter));
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        show_state_tree(ui, snapshot, &state.filter);
                    });
                }
            }
        });
    state.open = open;
}

/// Draw a JSON value as a collapsible tree. Keys that contain `filter` (case-insensitive)
/// are highlighted; an empty filter shows everything.
pub fn show_state_tree(ui: &mut Ui, value: &Value, filter: &str) {
    tree(ui, "", value, filter);
}

fn tree(ui: &mut Ui, key: &str, value: &Value, filter: &str) {
    let matches = filter.is_empty() || key.to_lowercase().contains(&filter.to_lowercase());
    match value {
        Value::Object(map) => {
            let label = if key.is_empty() {
                format!("object ({})", map.len())
            } else {
                format!("{key}  {{ {} }}", map.len())
            };
            let header = egui::CollapsingHeader::new(label).default_open(key.is_empty() || matches);
            header.show(ui, |ui| {
                for (child_key, child) in map {
                    tree(ui, child_key, child, filter);
                }
            });
        }
        Value::Array(items) => {
            let label = if key.is_empty() {
                format!("array ({})", items.len())
            } else {
                format!("{key}  [ {} ]", items.len())
            };
            egui::CollapsingHeader::new(label)
                .default_open(key.is_empty() || matches)
                .show(ui, |ui| {
                    for (index, item) in items.iter().enumerate() {
                        tree(ui, &format!("[{index}]"), item, filter);
                    }
                });
        }
        Value::String(text) => {
            ui.label(RichText::new(format!("{key}: \"{text}\"")).color(ui.visuals().hyperlink_color));
        }
        Value::Number(number) => {
            ui.label(format!("{key}: {number}"));
        }
        Value::Bool(boolean) => {
            ui.label(format!("{key}: {boolean}"));
        }
        Value::Null => {
            ui.label(RichText::new(format!("{key}: null")).weak());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn f12_toggles_and_tabs_open() {
        let ctx = Context::default();
        let mut state = DevToolsState::default();
        assert!(!state.toggle_on_f12(&ctx));
        state.open_state();
        assert!(state.open);
        assert_eq!(state.tab, DevToolsTab::State);
        state.open_journal();
        assert_eq!(state.tab, DevToolsTab::Journal);
    }

    #[test]
    fn labels_default_to_english() {
        let labels = DevToolsLabels::default();
        assert_eq!(labels.title, "DevTools");
        assert_eq!(labels.journal, "Journal");
    }

    #[test]
    fn the_state_tree_accepts_every_json_kind() {
        let ctx = Context::default();
        let value = json!({"a": 1, "b": [true, null, "text", {"c": 2.5}]});
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            show_state_tree(ui, &value, "c");
        });
        output.textures_delta.clear();
    }
}
