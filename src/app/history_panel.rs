//! The History panel: the undo tree with time-travel jumps.

use egui::{RichText, Ui};

use super::History;

/// Translated strings of the History panel.
#[derive(Clone, Copy, Debug)]
pub struct HistoryLabels<'a> {
    /// Panel title.
    pub title: &'a str,
    /// Marker shown next to the current node.
    pub current: &'a str,
    /// Shown when only the root exists.
    pub empty: &'a str,
}

impl Default for HistoryLabels<'_> {
    fn default() -> Self {
        Self {
            title: "History",
            current: "current",
            empty: "No edits yet",
        }
    }
}

/// Draw the undo tree and return the node the user clicked.
///
/// The rows are in creation order, indented by depth. The current node is selected; a
/// click calls [`History::jump_to`] in the caller and re-renders from that state.
pub fn show_history_tree<S>(
    ui: &mut Ui,
    history: &History<S>,
    labels: HistoryLabels<'_>,
) -> Option<super::history::NodeId> {
    ui.label(RichText::new(labels.title).strong());
    ui.add_space(4.0);
    if history.is_empty() {
        ui.label(RichText::new(labels.empty).weak());
        return None;
    }
    let mut chosen = None;
    for node in history.nodes() {
        let depth = depth_of(history, node.id);
        ui.horizontal(|ui| {
            ui.add_space(depth as f32 * 12.0);
            let current = node.id == history.current();
            let label = if current {
                format!("{} · {}", node.label, labels.current)
            } else {
                node.label.clone()
            };
            if ui.selectable_label(current, label).clicked() && !current {
                chosen = Some(node.id);
            }
        });
    }
    chosen
}

fn depth_of<S>(history: &History<S>, id: super::history::NodeId) -> usize {
    let mut depth = 0;
    let mut node = history.node(id).and_then(|node| node.parent);
    while let Some(parent) = node {
        depth += 1;
        node = history.node(parent).and_then(|node| node.parent);
    }
    depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_default_to_english() {
        let labels = HistoryLabels::default();
        assert_eq!(labels.title, "History");
        assert_eq!(labels.current, "current");
    }

    #[test]
    fn depth_follows_the_parent_chain() {
        let mut history = History::new(0);
        let one = history.record("one", 1);
        let two = history.record("two", 2);
        assert_eq!(depth_of(&history, history.current()), 2);
        assert_eq!(depth_of(&history, one), 1);
        assert_eq!(depth_of(&history, two), 2);
    }
}
