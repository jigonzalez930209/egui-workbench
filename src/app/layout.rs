//! Layouts as data: editor groups, splits, tabs, named layouts, and focus tools.
//!
//! The layout is a serializable tree. A host can save it with the workspace, switch
//! named layouts from the palette, and drive every drag with a keyboard action; the UI
//! is a view of this tree, not the source of truth.

mod node;

pub use node::{EditorGroup, EditorGroupId, LayoutNode, LayoutTab, SplitDirection};

use std::collections::VecDeque;
use std::fmt;

/// The whole layout.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Layout {
    /// File format version.
    pub version: u32,
    /// The tree.
    pub root: LayoutNode,
    /// The focused group.
    pub active: EditorGroupId,
    /// A maximized group (the rest are hidden until restored).
    pub maximized: Option<EditorGroupId>,
    /// Zen mode hides the side bars, status bar, and activity bar.
    pub zen: bool,
    /// The last closed tabs, newest first.
    pub closed: VecDeque<LayoutTab>,
    next_id: u64,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            root: LayoutNode::Group(EditorGroup::new(EditorGroupId(0))),
            active: EditorGroupId(0),
            maximized: None,
            zen: false,
            closed: VecDeque::new(),
            next_id: 1,
        }
    }
}

impl Layout {
    /// The current file version.
    pub const VERSION: u32 = 1;

    /// A layout with one empty group.
    pub fn single() -> Self {
        Self::default()
    }

    /// A layout with one group and `tabs`.
    pub fn with_tabs(tabs: Vec<LayoutTab>) -> Self {
        Self {
            root: LayoutNode::Group(EditorGroup::with_tabs(EditorGroupId(0), tabs)),
            ..Self::default()
        }
    }

    /// The group ids.
    pub fn groups(&self) -> Vec<EditorGroupId> {
        self.root.ids()
    }

    /// A group by id.
    pub fn group(&self, id: EditorGroupId) -> Option<&EditorGroup> {
        self.root.group(id)
    }

    /// A group by id, mutably.
    pub fn group_mut(&mut self, id: EditorGroupId) -> Option<&mut EditorGroup> {
        self.root.group_mut(id)
    }

    /// The focused group.
    pub fn active_group(&self) -> Option<&EditorGroup> {
        self.group(self.active)
    }

    /// Focus a group.
    pub fn focus(&mut self, id: EditorGroupId) -> bool {
        if self.group(id).is_some() {
            self.active = id;
            true
        } else {
            false
        }
    }

    /// Split `group` and return the new group id.
    ///
    /// The original group keeps the first half; the new group is empty and focused.
    pub fn split(&mut self, group: EditorGroupId, direction: SplitDirection) -> Option<EditorGroupId> {
        let old = self.root.group(group)?.clone();
        let new_id = EditorGroupId(self.next_id);
        self.next_id += 1;
        self.root = replace_group(
            std::mem::replace(&mut self.root, LayoutNode::Group(EditorGroup::new(EditorGroupId(0)))),
            group,
            LayoutNode::Split {
                direction,
                ratio: 0.5,
                first: Box::new(LayoutNode::Group(old)),
                second: Box::new(LayoutNode::Group(EditorGroup::new(new_id))),
            },
        );
        self.active = new_id;
        Some(new_id)
    }

    /// Close a group, promoting its sibling. The last group cannot be closed.
    pub fn close_group(&mut self, id: EditorGroupId) -> bool {
        if self.groups().len() <= 1 || self.group(id).is_none() {
            return false;
        }
        let removed = self.root.remove_group(id);
        if removed {
            if self.active == id
                && let Some(first) = self.root.first_id()
            {
                self.active = first;
            }
            if self.maximized == Some(id) {
                self.maximized = None;
            }
        }
        removed
    }

    /// Move a tab from one group to another. Returns whether it moved.
    pub fn move_tab(&mut self, from: EditorGroupId, index: usize, to: EditorGroupId, target: usize) -> bool {
        if from == to || !self.groups().contains(&to) {
            return false;
        }
        let Some(tab) = self
            .group_mut(from)
            .and_then(|group| (index < group.tabs.len()).then(|| group.tabs.remove(index)))
        else {
            return false;
        };
        if let Some(group) = self.group_mut(from) {
            group.normalize();
        }
        let Some(group) = self.group_mut(to) else {
            return false;
        };
        let target = target.min(group.tabs.len());
        group.tabs.insert(target, tab);
        group.active = target;
        self.active = to;
        true
    }

    /// Close a tab and remember it for Reopen.
    pub fn close_tab(&mut self, group: EditorGroupId, index: usize) -> Option<LayoutTab> {
        let tab = self
            .group_mut(group)
            .and_then(|group| (index < group.tabs.len()).then(|| group.tabs.remove(index)))?;
        if let Some(group) = self.group_mut(group) {
            group.normalize();
        }
        self.closed.push_front(tab.clone());
        self.closed.truncate(20);
        Some(tab)
    }

    /// Reopen the newest closed tab in the focused group.
    pub fn reopen_closed(&mut self) -> Option<LayoutTab> {
        let tab = self.closed.pop_front()?;
        let Some(group) = self.group_mut(self.active) else {
            self.closed.push_front(tab);
            return None;
        };
        group.tabs.push(tab.clone());
        group.active = group.tabs.len() - 1;
        Some(tab)
    }

    /// Pin or unpin a tab.
    pub fn pin_tab(&mut self, group: EditorGroupId, index: usize, pinned: bool) -> bool {
        let Some(tab) = self.group_mut(group).and_then(|group| group.tabs.get_mut(index)) else {
            return false;
        };
        tab.pinned = pinned;
        if pinned {
            tab.preview = false;
        }
        true
    }

    /// Maximize a group, or restore with `None`.
    pub fn maximize(&mut self, group: Option<EditorGroupId>) -> bool {
        if let Some(group) = group
            && self.group(group).is_none()
        {
            return false;
        }
        self.maximized = group;
        true
    }

    /// Toggle Zen mode.
    pub fn set_zen(&mut self, zen: bool) {
        self.zen = zen;
    }

    /// Clamp active indices and the focused group after a load.
    pub fn normalize(&mut self) {
        for id in self.groups() {
            if let Some(group) = self.group_mut(id) {
                group.normalize();
            }
        }
        if self.group(self.active).is_none()
            && let Some(first) = self.root.first_id()
        {
            self.active = first;
        }
        if let Some(maximized) = self.maximized
            && self.group(maximized).is_none()
        {
            self.maximized = None;
        }
        self.closed.truncate(20);
    }

    /// A JSON representation, for a host or a test.
    ///
    /// # Errors
    /// When serialization fails (it does not for this type).
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> Result<String, LayoutError> {
        serde_json::to_string_pretty(self).map_err(|error| LayoutError::Json(error.to_string()))
    }

    /// Parse a layout, rejecting a newer version.
    ///
    /// # Errors
    /// When the JSON is malformed or the version is newer than [`Self::VERSION`].
    #[cfg(feature = "serde")]
    pub fn from_json(json: &str) -> Result<Self, LayoutError> {
        let mut layout: Self = serde_json::from_str(json).map_err(|error| LayoutError::Json(error.to_string()))?;
        if layout.version > Self::VERSION {
            return Err(LayoutError::UnsupportedVersion(layout.version));
        }
        if layout.version < Self::VERSION {
            layout.version = Self::VERSION;
        }
        layout.normalize();
        Ok(layout)
    }
}

fn replace_group(node: LayoutNode, id: EditorGroupId, replacement: LayoutNode) -> LayoutNode {
    match node {
        LayoutNode::Group(group) if group.id == id => replacement,
        LayoutNode::Group(group) => LayoutNode::Group(group),
        LayoutNode::Split {
            direction,
            ratio,
            first,
            second,
        } => LayoutNode::Split {
            direction,
            ratio,
            first: Box::new(replace_group(*first, id, replacement.clone())),
            second: Box::new(replace_group(*second, id, replacement)),
        },
    }
}

/// A keyboard-drivable layout action. Every drag has one of these equivalents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LayoutAction {
    /// Split the focused group horizontally.
    SplitRight,
    /// Split the focused group vertically.
    SplitDown,
    /// Focus the next group.
    FocusNext,
    /// Focus the previous group.
    FocusPrevious,
    /// Move the active tab to the next group.
    MoveTabToNextGroup,
    /// Close the focused group.
    CloseGroup,
    /// Maximize or restore the focused group.
    ToggleMaximize,
    /// Toggle Zen mode.
    ToggleZen,
    /// Reopen the last closed tab.
    ReopenClosed,
}

impl Layout {
    /// Apply a keyboard action. Returns whether the layout changed.
    pub fn apply(&mut self, action: LayoutAction) -> bool {
        match action {
            LayoutAction::SplitRight => self.split(self.active, SplitDirection::Horizontal).is_some(),
            LayoutAction::SplitDown => self.split(self.active, SplitDirection::Vertical).is_some(),
            LayoutAction::FocusNext => self.cycle_focus(1),
            LayoutAction::FocusPrevious => self.cycle_focus(-1),
            LayoutAction::MoveTabToNextGroup => self.move_active_tab_to_next(),
            LayoutAction::CloseGroup => self.close_group(self.active),
            LayoutAction::ToggleMaximize => self.maximize(if self.maximized.is_some() {
                None
            } else {
                Some(self.active)
            }),
            LayoutAction::ToggleZen => {
                self.zen = !self.zen;
                true
            }
            LayoutAction::ReopenClosed => self.reopen_closed().is_some(),
        }
    }

    fn cycle_focus(&mut self, step: isize) -> bool {
        let groups = self.groups();
        if groups.len() <= 1 {
            return false;
        }
        let Some(current) = groups.iter().position(|id| *id == self.active) else {
            return false;
        };
        let next = (current as isize + step).rem_euclid(groups.len() as isize) as usize;
        self.active = groups[next];
        true
    }

    fn move_active_tab_to_next(&mut self) -> bool {
        let groups = self.groups();
        if groups.len() <= 1 {
            return false;
        }
        let Some(current) = groups.iter().position(|id| *id == self.active) else {
            return false;
        };
        let next = groups[(current + 1) % groups.len()];
        let index = self.active_group().map_or(0, |group| group.active);
        if self.group(self.active).is_none_or(|group| index >= group.tabs.len()) {
            return false;
        }
        self.move_tab(self.active, index, next, 0)
    }
}

/// Why a layout could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LayoutError {
    /// Malformed JSON.
    Json(String),
    /// A newer file version.
    UnsupportedVersion(u32),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid layout JSON: {error}"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported layout version {version}"),
        }
    }
}

impl std::error::Error for LayoutError {}

#[cfg(test)]
mod tests;
