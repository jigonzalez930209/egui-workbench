//! The selection shared by the 2D and 3D panels.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::id::{AtomId, BondId};

/// How a selection update combines with the current selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SelectionMode {
    /// Replace the current selection.
    #[default]
    Replace,
    /// Add the incoming ids.
    Add,
    /// Remove the incoming ids.
    Subtract,
    /// Flip the incoming ids.
    Toggle,
}

/// The atoms and bonds selected in one document.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChemSelection {
    atoms: BTreeSet<AtomId>,
    bonds: BTreeSet<BondId>,
}

impl ChemSelection {
    /// An empty selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// The selected atoms.
    pub fn atoms(&self) -> &BTreeSet<AtomId> {
        &self.atoms
    }

    /// The selected bonds.
    pub fn bonds(&self) -> &BTreeSet<BondId> {
        &self.bonds
    }

    /// Whether nothing is selected.
    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty() && self.bonds.is_empty()
    }

    /// Number of selected atoms and bonds.
    pub fn len(&self) -> usize {
        self.atoms.len() + self.bonds.len()
    }

    /// Select one atom according to `mode`.
    pub fn select_atom(&mut self, id: AtomId, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                self.atoms.clear();
                self.bonds.clear();
                self.atoms.insert(id);
            }
            SelectionMode::Add => {
                self.atoms.insert(id);
            }
            SelectionMode::Subtract => {
                self.atoms.remove(&id);
            }
            SelectionMode::Toggle => {
                if !self.atoms.remove(&id) {
                    self.atoms.insert(id);
                }
            }
        }
    }

    /// Select one bond according to `mode`.
    pub fn select_bond(&mut self, id: BondId, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                self.atoms.clear();
                self.bonds.clear();
                self.bonds.insert(id);
            }
            SelectionMode::Add => {
                self.bonds.insert(id);
            }
            SelectionMode::Subtract => {
                self.bonds.remove(&id);
            }
            SelectionMode::Toggle => {
                if !self.bonds.remove(&id) {
                    self.bonds.insert(id);
                }
            }
        }
    }

    /// Apply a whole selection, for synchronizing a panel or a peer.
    pub fn apply(&mut self, incoming: &Self, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => *self = incoming.clone(),
            SelectionMode::Add => {
                self.atoms.extend(incoming.atoms.iter().copied());
                self.bonds.extend(incoming.bonds.iter().copied());
            }
            SelectionMode::Subtract => {
                for atom in &incoming.atoms {
                    self.atoms.remove(atom);
                }
                for bond in &incoming.bonds {
                    self.bonds.remove(bond);
                }
            }
            SelectionMode::Toggle => {
                for atom in &incoming.atoms {
                    if !self.atoms.remove(atom) {
                        self.atoms.insert(*atom);
                    }
                }
                for bond in &incoming.bonds {
                    if !self.bonds.remove(bond) {
                        self.bonds.insert(*bond);
                    }
                }
            }
        }
    }

    /// Drop every selected id that is no longer in the document.
    pub fn retain_existing(&mut self, atoms: &[AtomId], bonds: &[BondId]) {
        self.atoms.retain(|id| atoms.contains(id));
        self.bonds.retain(|id| bonds.contains(id));
    }

    /// Remove everything.
    pub fn clear(&mut self) {
        self.atoms.clear();
        self.bonds.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_add_subtract_and_toggle_combine_selections() {
        let a = AtomId::new(0, 1);
        let b = AtomId::new(1, 1);
        let mut selection = ChemSelection::new();
        selection.select_atom(a, SelectionMode::Replace);
        selection.select_atom(b, SelectionMode::Add);
        assert_eq!(selection.atoms().len(), 2);
        selection.select_atom(a, SelectionMode::Toggle);
        assert!(!selection.atoms().contains(&a));
        selection.select_atom(b, SelectionMode::Subtract);
        assert!(selection.is_empty());
    }

    #[test]
    fn apply_replaces_and_retains_only_live_ids() {
        let a = AtomId::new(0, 1);
        let b = AtomId::new(1, 1);
        let bond = BondId::new(0, 1);
        let mut incoming = ChemSelection::new();
        incoming.select_atom(a, SelectionMode::Add);
        incoming.select_bond(bond, SelectionMode::Add);
        let mut selection = ChemSelection::new();
        selection.select_atom(b, SelectionMode::Add);
        selection.apply(&incoming, SelectionMode::Replace);
        assert_eq!(selection.atoms().len(), 1);
        selection.retain_existing(&[b], &[]);
        assert!(selection.is_empty());
    }
}
