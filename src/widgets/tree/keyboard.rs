//! Keyboard navigation of a focused tree.
//!
//! Up/Down move the selection; Right expands a folder or steps into its first child;
//! Left collapses a folder or steps to the parent; Enter toggles a folder or opens a
//! leaf; F2 asks to rename. The selection scrolls minimally into view.

use super::{TreeEvent, TreeNode, TreeState};

/// One visible row in display order.
struct Flat<'a, K> {
    key: &'a K,
    folder: bool,
    expanded: bool,
    parent: Option<&'a K>,
}

pub(super) fn navigate<K: Clone + PartialEq>(
    ui: &egui::Ui,
    nodes: &[TreeNode<K>],
    state: &mut TreeState<K>,
    focus_id: egui::Id,
    events: &mut Vec<TreeEvent<K>>,
) {
    let mut flat = Vec::new();
    flatten(nodes, None, &mut flat);
    if flat.is_empty() {
        return;
    }
    let index = state
        .selected
        .as_ref()
        .and_then(|key| flat.iter().position(|row| row.key == key))
        .unwrap_or(0);
    let pressed = |key| ui.input(|input| input.key_pressed(key));
    let row = &flat[index];
    let mut next = index;
    let mut event = None;
    if pressed(egui::Key::ArrowDown) {
        next = (index + 1).min(flat.len() - 1);
    } else if pressed(egui::Key::ArrowUp) {
        next = index.saturating_sub(1);
    } else if pressed(egui::Key::ArrowRight) {
        if row.folder && !row.expanded {
            event = Some(TreeEvent::Toggle(row.key.clone()));
        } else if flat.get(index + 1).is_some_and(|child| child.parent == Some(row.key)) {
            next = index + 1;
        }
    } else if pressed(egui::Key::ArrowLeft) {
        if row.expanded {
            event = Some(TreeEvent::Toggle(row.key.clone()));
        } else if let Some(parent) = row.parent
            && let Some(position) = flat.iter().position(|candidate| candidate.key == parent)
        {
            next = position;
        }
    } else if pressed(egui::Key::Enter) {
        event = Some(if row.folder {
            TreeEvent::Toggle(row.key.clone())
        } else {
            TreeEvent::Open(row.key.clone())
        });
    } else if pressed(egui::Key::F2) {
        event = Some(TreeEvent::Rename(row.key.clone()));
    }
    if next == index && event.is_none() {
        // No key for the tree this frame: leave the selection alone and do not repaint.
        return;
    }
    events.extend(event);
    state.selected = Some(flat[next].key.clone());
    state.scroll_to_selected = true;
    ui.memory_mut(|memory| memory.request_focus(focus_id));
    ui.ctx().request_repaint();
}

fn flatten<'a, K>(nodes: &'a [TreeNode<K>], parent: Option<&'a K>, out: &mut Vec<Flat<'a, K>>) {
    for node in nodes {
        out.push(Flat {
            key: &node.key,
            folder: node.is_folder(),
            expanded: node.expanded,
            parent,
        });
        if node.expanded
            && let Some(children) = &node.children
        {
            flatten(children, Some(&node.key), out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RowIcon;

    #[test]
    fn expanded_nodes_flatten_in_visual_order() {
        let nodes = vec![
            TreeNode::folder(
                "a",
                "a",
                RowIcon::None,
                true,
                vec![TreeNode::leaf("a/one", "one", RowIcon::None)],
            ),
            TreeNode::folder(
                "b",
                "b",
                RowIcon::None,
                false,
                vec![TreeNode::leaf("b/two", "two", RowIcon::None)],
            ),
            TreeNode::leaf("c", "c", RowIcon::None),
        ];
        let mut flat = Vec::new();
        flatten(&nodes, None, &mut flat);
        let keys: Vec<&str> = flat.iter().map(|row| *row.key).collect();
        assert_eq!(keys, ["a", "a/one", "b", "c"]);
        assert_eq!(flat[1].parent, Some(&"a"));
    }
}
