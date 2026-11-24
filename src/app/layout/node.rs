//! Editor groups, tabs, and the layout tree.

use std::fmt;

/// A stable editor-group id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct EditorGroupId(pub u64);

impl fmt::Display for EditorGroupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// How a split lays out its two children.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum SplitDirection {
    /// Two columns.
    #[default]
    Horizontal,
    /// Two rows.
    Vertical,
}

/// One tab in an editor group.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct LayoutTab {
    /// Stable application id.
    pub id: String,
    /// Tab title.
    pub title: String,
    /// Pinned tabs survive "close others" and move with the group.
    pub pinned: bool,
    /// A preview tab is replaced by the next preview open.
    pub preview: bool,
}

impl LayoutTab {
    /// A regular tab.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            pinned: false,
            preview: false,
        }
    }

    /// A pinned tab.
    pub fn pinned(mut self) -> Self {
        self.pinned = true;
        self.preview = false;
        self
    }

    /// A preview tab.
    pub fn preview(mut self) -> Self {
        self.preview = true;
        self.pinned = false;
        self
    }
}

/// One editor group.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct EditorGroup {
    /// Stable id.
    pub id: EditorGroupId,
    /// Tabs in order.
    pub tabs: Vec<LayoutTab>,
    /// Active tab index.
    pub active: usize,
}

impl EditorGroup {
    /// An empty group.
    pub fn new(id: EditorGroupId) -> Self {
        Self {
            id,
            tabs: Vec::new(),
            active: 0,
        }
    }

    /// A group with tabs, the first active.
    pub fn with_tabs(id: EditorGroupId, tabs: Vec<LayoutTab>) -> Self {
        Self { id, tabs, active: 0 }
    }

    /// The active tab.
    pub fn active_tab(&self) -> Option<&LayoutTab> {
        self.tabs.get(self.active)
    }

    /// Clamp the active index after a removal or a load.
    pub fn normalize(&mut self) {
        if self.tabs.is_empty() {
            self.active = 0;
        } else {
            self.active = self.active.min(self.tabs.len() - 1);
        }
    }
}

/// A node of the layout tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "snake_case"))]
pub enum LayoutNode {
    /// A leaf with an editor group.
    Group(EditorGroup),
    /// Two children side by side or stacked.
    Split {
        /// Direction.
        direction: SplitDirection,
        /// Fraction of the space given to `first`, 0.1–0.9.
        ratio: f32,
        /// First child.
        first: Box<LayoutNode>,
        /// Second child.
        second: Box<LayoutNode>,
    },
}

impl LayoutNode {
    /// The group with `id`, if present.
    pub fn group(&self, id: EditorGroupId) -> Option<&EditorGroup> {
        match self {
            Self::Group(group) => (group.id == id).then_some(group),
            Self::Split { first, second, .. } => first.group(id).or_else(|| second.group(id)),
        }
    }

    /// The group with `id`, mutably.
    pub fn group_mut(&mut self, id: EditorGroupId) -> Option<&mut EditorGroup> {
        match self {
            Self::Group(group) => (group.id == id).then_some(group),
            Self::Split { first, second, .. } => first.group_mut(id).or_else(|| second.group_mut(id)),
        }
    }

    /// The group ids in depth-first order.
    pub fn ids(&self) -> Vec<EditorGroupId> {
        let mut ids = Vec::new();
        self.collect_ids(&mut ids);
        ids
    }

    fn collect_ids(&self, ids: &mut Vec<EditorGroupId>) {
        match self {
            Self::Group(group) => ids.push(group.id),
            Self::Split { first, second, .. } => {
                first.collect_ids(ids);
                second.collect_ids(ids);
            }
        }
    }

    /// The first group id, used when splitting an unknown group.
    pub fn first_id(&self) -> Option<EditorGroupId> {
        self.ids().first().copied()
    }

    pub(super) fn remove_group(&mut self, id: EditorGroupId) -> bool {
        match self {
            Self::Group(_) => false,
            Self::Split { first, second, .. } => {
                if matches!(&**first, Self::Group(group) if group.id == id) {
                    let replacement = std::mem::replace(&mut **second, Self::Group(EditorGroup::new(id)));
                    *self = replacement;
                    return true;
                }
                if matches!(&**second, Self::Group(group) if group.id == id) {
                    let replacement = std::mem::replace(&mut **first, Self::Group(EditorGroup::new(id)));
                    *self = replacement;
                    return true;
                }
                first.remove_group(id) || second.remove_group(id)
            }
        }
    }
}
