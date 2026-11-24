//! Transactional undo and redo with a real tree, not a stack.
//!
//! Undo moves to the parent; editing after an undo adds a sibling branch instead of
//! discarding the abandoned one. A group coalesces typing or a drag into one node, and a
//! [`HistoryPanel`](crate::app::show_history_tree) lets the user jump anywhere.

use std::fmt;

/// A node in the undo tree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct NodeId(u64);

impl NodeId {
    /// The numeric id, for a journal or a label.
    pub const fn index(self) -> u64 {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// One state snapshot in the tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HistoryNode<S> {
    /// Stable id.
    pub id: NodeId,
    /// Parent, `None` for the root.
    pub parent: Option<NodeId>,
    /// Children in creation order.
    pub children: Vec<NodeId>,
    /// A short label shown in the History panel.
    pub label: String,
    /// The state at this node.
    pub state: S,
    /// The coalescing group, when this node is still open.
    pub group: Option<u64>,
    /// When the node was recorded, in the application's clock.
    pub timestamp: f64,
}

/// A coalescing group: typing or a drag records one node until the group ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct GroupId(u64);

impl GroupId {
    /// The numeric id.
    pub const fn index(self) -> u64 {
        self.0
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Why a history operation failed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HistoryError {
    /// The node id does not exist.
    UnknownNode(NodeId),
    /// The history is empty.
    Empty,
    /// A group is already open.
    GroupAlreadyOpen(GroupId),
    /// No group is open.
    NoGroup,
}

impl fmt::Display for HistoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode(id) => write!(formatter, "unknown history node {id}"),
            Self::Empty => formatter.write_str("the history is empty"),
            Self::GroupAlreadyOpen(id) => write!(formatter, "group {id} is already open"),
            Self::NoGroup => formatter.write_str("no group is open"),
        }
    }
}

impl std::error::Error for HistoryError {}

/// A tree of state snapshots with a current node.
///
/// ```
/// # use egui_workbench::app::History;
/// let mut history = History::new("a".to_owned());
/// history.record("type b", "ab".to_owned());
/// history.record("type c", "abc".to_owned());
/// assert_eq!(history.state(), "abc");
/// history.undo();
/// assert_eq!(history.state(), "ab");
/// history.record("branch", "a!".to_owned());
/// assert_eq!(history.len(), 4, "the abandoned branch is still there");
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct History<S> {
    nodes: Vec<HistoryNode<S>>,
    current: NodeId,
    next_id: u64,
    next_group: u64,
    active_group: Option<GroupId>,
}

impl<S> History<S> {
    /// A history with one root node.
    pub fn new(initial: S) -> Self {
        Self {
            nodes: vec![HistoryNode {
                id: NodeId(0),
                parent: None,
                children: Vec::new(),
                label: "Initial".to_owned(),
                state: initial,
                group: None,
                timestamp: 0.0,
            }],
            current: NodeId(0),
            next_id: 1,
            next_group: 1,
            active_group: None,
        }
    }

    /// The current node id.
    pub fn current(&self) -> NodeId {
        self.current
    }

    /// The current state.
    pub fn state(&self) -> &S {
        &self.nodes[self.current.0 as usize].state
    }

    /// The current node.
    pub fn current_node(&self) -> &HistoryNode<S> {
        &self.nodes[self.current.0 as usize]
    }

    /// A node by id.
    pub fn node(&self, id: NodeId) -> Option<&HistoryNode<S>> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// Every node, in creation order (the root first).
    pub fn nodes(&self) -> &[HistoryNode<S>] {
        &self.nodes
    }

    /// The number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether there is only the root.
    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1
    }

    /// Whether an undo is possible.
    pub fn can_undo(&self) -> bool {
        self.nodes[self.current.0 as usize].parent.is_some()
    }

    /// Whether a redo is possible.
    pub fn can_redo(&self) -> bool {
        !self.nodes[self.current.0 as usize].children.is_empty()
    }

    /// The open coalescing group.
    pub fn group(&self) -> Option<GroupId> {
        self.active_group
    }

    /// Start coalescing. Recording while a group is open updates the current node in
    /// place instead of adding a child.
    ///
    /// # Errors
    /// When a group is already open.
    pub fn begin_group(&mut self, _label: &str) -> Result<GroupId, HistoryError> {
        if let Some(group) = self.active_group {
            return Err(HistoryError::GroupAlreadyOpen(group));
        }
        let group = GroupId(self.next_group);
        self.next_group += 1;
        self.active_group = Some(group);
        Ok(group)
    }

    /// Stop coalescing.
    ///
    /// # Errors
    /// When no group is open.
    pub fn end_group(&mut self) -> Result<(), HistoryError> {
        if self.active_group.take().is_none() {
            return Err(HistoryError::NoGroup);
        }
        Ok(())
    }

    /// The path from the root to the current node.
    pub fn path(&self) -> Vec<NodeId> {
        let mut path = Vec::new();
        let mut node = Some(self.current);
        while let Some(id) = node {
            path.push(id);
            node = self.node(id).and_then(|node| node.parent);
        }
        path.reverse();
        path
    }

    /// Undo one step and return the new state.
    pub fn undo(&mut self) -> Option<&S> {
        let parent = self.nodes[self.current.0 as usize].parent?;
        self.active_group = None;
        self.current = parent;
        Some(self.state())
    }

    /// Redo the most recently created child.
    pub fn redo(&mut self) -> Option<&S> {
        let children = &self.nodes[self.current.0 as usize].children;
        let child = *children.last()?;
        self.active_group = None;
        self.current = child;
        Some(self.state())
    }

    /// Jump to a node.
    ///
    /// # Errors
    /// When the node id does not exist.
    pub fn jump_to(&mut self, id: NodeId) -> Result<&S, HistoryError> {
        if self.node(id).is_none() {
            return Err(HistoryError::UnknownNode(id));
        }
        self.active_group = None;
        self.current = id;
        Ok(self.state())
    }
}

impl<S: Clone + PartialEq> History<S> {
    /// Record `state` as a child of the current node, or update the current node when a
    /// group is open and already owns it.
    pub fn record(&mut self, label: impl Into<String>, state: S) -> NodeId {
        let label = label.into();
        if let Some(group) = self.active_group
            && self.current != NodeId(0)
            && self.current_node().group == Some(group.index())
        {
            let node = &mut self.nodes[self.current.0 as usize];
            node.state = state;
            node.label = label;
            return node.id;
        }
        if self.state() == &state {
            return self.current;
        }
        self.record_with_group(label, state, self.active_group.map(GroupId::index), 0.0)
    }

    /// Record a node with an explicit timestamp.
    pub fn record_at(&mut self, label: impl Into<String>, state: S, timestamp: f64) -> NodeId {
        self.record_with_group(label.into(), state, self.active_group.map(GroupId::index), timestamp)
    }

    fn record_with_group(&mut self, label: String, state: S, group: Option<u64>, timestamp: f64) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.nodes.push(HistoryNode {
            id,
            parent: Some(self.current),
            children: Vec::new(),
            label,
            state,
            group,
            timestamp,
        });
        self.nodes[self.current.0 as usize].children.push(id);
        self.current = id;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_and_branch_keep_the_abandoned_path() {
        let mut history = History::new("a".to_owned());
        let second = history.record("b", "ab".to_owned());
        let third = history.record("c", "abc".to_owned());
        assert!(history.can_undo());
        assert!(!history.can_redo());
        history.undo();
        assert_eq!(history.state(), "ab");
        let branch = history.record("!", "a!".to_owned());
        assert_eq!(history.node(third).map(|node| node.label.as_str()), Some("c"));
        assert_eq!(history.node(second).map(|node| node.children.len()), Some(2));
        assert_eq!(history.node(branch).map(|node| node.state.as_str()), Some("a!"));
        assert_eq!(history.len(), 4);
    }

    #[test]
    fn redo_follows_the_last_child_and_jump_time_travels() {
        let mut history = History::new(0);
        history.record("one", 1);
        let two = history.record("two", 2);
        history.undo();
        history.undo();
        assert_eq!(history.state(), &0);
        history.redo();
        assert_eq!(history.state(), &1);
        history.jump_to(two).expect("node exists");
        assert_eq!(history.state(), &2);
        assert!(history.jump_to(NodeId(99)).is_err());
    }

    #[test]
    fn a_group_coalesces_typing() {
        let mut history = History::new(String::new());
        history.begin_group("typing").expect("group");
        let first = history.record("t", "t".to_owned());
        let second = history.record("typing", "typing".to_owned());
        assert_eq!(first, second, "the open group updates the same node");
        history.end_group().expect("end");
        history.record("!", "typing!".to_owned());
        assert_eq!(history.len(), 3);
        assert!(history.end_group().is_err());
    }

    #[test]
    fn no_op_records_do_not_add_nodes() {
        let mut history = History::new(1);
        history.record("same", 1);
        assert_eq!(history.len(), 1);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn the_undo_tree_survives_json() {
        let mut history = History::new("a".to_owned());
        history.record("b", "ab".to_owned());
        history.record("c", "abc".to_owned());
        history.undo();
        history.record("!", "a!".to_owned());
        let json = serde_json::to_string(&history).expect("serialize");
        let back: History<String> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.state(), "a!");
        assert_eq!(back.len(), history.len());
        assert_eq!(back.current(), history.current());
    }
}
