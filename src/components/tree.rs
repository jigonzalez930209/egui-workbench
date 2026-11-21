//! The explorer tree component.

use std::fmt::Debug;
use std::hash::Hash;

use egui::Ui;

use crate::Icons;
use crate::widgets::tree::{TreeView, show_tree};

pub use crate::widgets::tree::{RowIcon, TreeEvent, TreeNode, TreeState};

/// A keyboard-navigable explorer tree.
pub struct Tree<'a, K> {
    nodes: &'a [TreeNode<K>],
    state: &'a mut TreeState<K>,
    id_salt: &'a str,
    active: Option<&'a K>,
    light: bool,
}

impl<'a, K> Tree<'a, K> {
    /// A tree over `nodes`.
    pub fn new(nodes: &'a [TreeNode<K>], state: &'a mut TreeState<K>, id_salt: &'a str) -> Self {
        Self {
            nodes,
            state,
            id_salt,
            active: None,
            light: false,
        }
    }

    /// Highlight and follow the active document key.
    pub fn active(mut self, active: Option<&'a K>) -> Self {
        self.active = active;
        self
    }

    /// Use the light Material icon variants.
    pub fn light(mut self, light: bool) -> Self {
        self.light = light;
        self
    }

    /// Paint the tree and return its events.
    pub fn show(
        self,
        ui: &mut Ui,
        icons: &mut Icons,
        context_menu: impl FnMut(&mut Ui, &mut Icons, &TreeNode<K>),
    ) -> Vec<TreeEvent<K>>
    where
        K: Clone + PartialEq + Hash + Debug,
    {
        let view = TreeView {
            id_salt: self.id_salt,
            active: self.active,
            light: self.light,
        };
        show_tree(ui, icons, self.nodes, self.state, &view, context_menu)
    }
}
