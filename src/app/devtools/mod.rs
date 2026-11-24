//! The first DevTools slice: the journal and a JSON state inspector, toggled with F12.

use egui::{Context, Key};
use serde_json::Value;

use crate::foundation::theme::{Theme, apply_tokens};

mod accessibility;
mod inspector;
mod performance;
mod state_tree;
mod theme;

pub use accessibility::show_accessibility;
pub use inspector::show_inspector;
pub use performance::{PerformanceState, show_performance};
pub use state_tree::show_state_tree;
pub use theme::show_theme_tokens;

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
    /// The widget inspector.
    Inspector,
    /// Live theme and style tokens.
    Theme,
    /// Frame time and repaint reasons.
    Performance,
    /// Accessibility audit.
    Accessibility,
}

/// Persistent DevTools state.
#[derive(Clone, Debug)]
pub struct DevToolsState {
    /// Whether the window is open.
    pub open: bool,
    /// The selected tab.
    pub tab: DevToolsTab,
    /// Filter for the state tree.
    pub filter: String,
    /// Live theme tokens.
    pub theme: Theme,
    /// Frame timing.
    pub performance: PerformanceState,
    /// Selected widget path in the inspector.
    pub selected: Option<String>,
}

impl Default for DevToolsState {
    fn default() -> Self {
        Self {
            open: false,
            tab: DevToolsTab::Journal,
            filter: String::new(),
            theme: Theme::dark(),
            performance: PerformanceState::default(),
            selected: None,
        }
    }
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

    /// Open the widget inspector.
    pub fn open_inspector(&mut self) {
        self.open = true;
        self.tab = DevToolsTab::Inspector;
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
    /// Inspector tab.
    pub inspector: &'a str,
    /// Theme tab.
    pub theme: &'a str,
    /// Performance tab.
    pub performance: &'a str,
    /// Accessibility tab.
    pub accessibility: &'a str,
}

impl Default for DevToolsLabels<'_> {
    fn default() -> Self {
        Self {
            title: "DevTools",
            journal: "Journal",
            state: "State",
            filter: "Filter state…",
            inspector: "Inspector",
            theme: "Theme",
            performance: "Performance",
            accessibility: "Accessibility",
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
        .default_size([720.0, 480.0])
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut state.tab, DevToolsTab::Journal, labels.journal);
                ui.selectable_value(&mut state.tab, DevToolsTab::State, labels.state);
                ui.selectable_value(&mut state.tab, DevToolsTab::Inspector, labels.inspector);
                ui.selectable_value(&mut state.tab, DevToolsTab::Theme, labels.theme);
                ui.selectable_value(&mut state.tab, DevToolsTab::Performance, labels.performance);
                ui.selectable_value(&mut state.tab, DevToolsTab::Accessibility, labels.accessibility);
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
                DevToolsTab::Inspector => show_inspector(ui, snapshot, &mut state.selected),
                DevToolsTab::Theme => {
                    if show_theme_tokens(ui, &mut state.theme) {
                        apply_tokens(ctx, &state.theme);
                    }
                }
                DevToolsTab::Performance => {
                    state.performance.update(ctx);
                    show_performance(ui, &state.performance);
                }
                DevToolsTab::Accessibility => show_accessibility(ui, snapshot),
            }
        });
    state.open = open;
}

#[cfg(test)]
mod tests {
    use super::accessibility::accessibility_issues;
    use super::*;
    use crate::foundation::theme::Theme;
    use egui::Context;
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
    fn the_new_tabs_and_audit_are_wired() {
        let ctx = Context::default();
        let snapshot = json!({
            "widgets": [
                {"kind": "button", "id": "ok", "label": "OK"},
                {"kind": "button", "id": "", "label": ""},
                {"kind": "label", "text": ""},
                {"kind": "plot", "id": "p", "title": ""}
            ]
        });
        let mut selected = None;
        let mut theme = Theme::dark();
        let mut performance = PerformanceState::default();
        performance.update(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            show_inspector(ui, &snapshot, &mut selected);
            let _ = show_theme_tokens(ui, &mut theme);
            show_performance(ui, &performance);
            show_accessibility(ui, &snapshot);
        });
        output.textures_delta.clear();
        let issues = accessibility_issues(&snapshot);
        assert!(issues.iter().any(|issue| issue.contains("button has no label")));
        assert!(issues.iter().any(|issue| issue.contains("label is empty")));
        assert!(issues.iter().any(|issue| issue.contains("plot has no title")));
        let mut state = DevToolsState::default();
        state.open_inspector();
        assert_eq!(state.tab, DevToolsTab::Inspector);
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
