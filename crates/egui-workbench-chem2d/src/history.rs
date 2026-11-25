//! Deterministic editing commands and undo/redo.

use egui_workbench_chem_core::{AtomId, BondId, BondOrder, BondStereo, ChemSelection, Element};

use crate::Chem2dError;
use crate::document::ChemDocument;

/// One atomic edit.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Command {
    /// Add an atom at a 2D position and select it.
    AddAtom {
        /// Element.
        element: Element,
        /// Position in Angstrom.
        position: [f64; 2],
    },
    /// Add a bond and select it.
    AddBond {
        /// First atom.
        a: AtomId,
        /// Second atom.
        b: AtomId,
        /// Bond order.
        order: BondOrder,
    },
    /// Remove every selected atom and bond.
    DeleteSelection,
    /// Change a bond order.
    SetBondOrder {
        /// Bond to edit.
        bond: BondId,
        /// New order.
        order: BondOrder,
    },
    /// Change bond stereochemistry.
    SetStereo {
        /// Bond to edit.
        bond: BondId,
        /// New stereo.
        stereo: BondStereo,
    },
    /// Move a fragment.
    TranslateFragment {
        /// Atoms to move.
        atoms: Vec<AtomId>,
        /// xy displacement in Angstrom.
        delta: [f64; 2],
    },
}

impl Command {
    /// Apply the command to `document` and update the selection.
    ///
    /// # Errors
    /// When an atom or bond is unknown or the document rejects the change.
    pub fn apply(&self, document: &mut ChemDocument, selection: &mut ChemSelection) -> Result<(), Chem2dError> {
        match self {
            Self::AddAtom { element, position } => {
                let id = document.molecule.add_atom(*element, [position[0], position[1], 0.0]);
                selection.select_atom(id, egui_workbench_chem_core::SelectionMode::Replace);
            }
            Self::AddBond { a, b, order } => {
                let id = document.molecule.add_bond(*a, *b, *order)?;
                selection.select_bond(id, egui_workbench_chem_core::SelectionMode::Replace);
            }
            Self::DeleteSelection => {
                let atoms = selection.atoms().iter().copied().collect::<Vec<_>>();
                let bonds = selection.bonds().iter().copied().collect::<Vec<_>>();
                for bond in bonds {
                    document.molecule.remove_bond(bond);
                }
                for atom in atoms {
                    document.molecule.remove_atom(atom);
                }
                selection.clear();
            }
            Self::SetBondOrder { bond, order } => {
                let Some(entry) = document.molecule.bonds.iter_mut().find(|entry| entry.id == *bond) else {
                    return Err(Chem2dError::UnknownBond(*bond));
                };
                entry.order = *order;
            }
            Self::SetStereo { bond, stereo } => {
                let Some(entry) = document.molecule.bonds.iter_mut().find(|entry| entry.id == *bond) else {
                    return Err(Chem2dError::UnknownBond(*bond));
                };
                entry.stereo = *stereo;
            }
            Self::TranslateFragment { atoms, delta } => {
                for atom in atoms {
                    if document.molecule.atom(*atom).is_none() {
                        return Err(Chem2dError::UnknownAtom(*atom));
                    }
                }
                document.molecule.translate(atoms, *delta);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Snapshot {
    document: ChemDocument,
    selection: ChemSelection,
}

/// Undo/redo history of whole-document snapshots.
///
/// Snapshots are simple and deterministic; a 2D document is small enough that this is
/// cheap. Use [`History::with_limit`] to cap memory in long editing sessions.
#[derive(Clone, Debug, Default)]
pub struct History {
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    limit: Option<usize>,
}

impl History {
    /// An unlimited history.
    pub fn new() -> Self {
        Self::default()
    }

    /// A history that keeps at most `limit` undo steps.
    pub fn with_limit(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: Some(limit.max(1)),
        }
    }

    /// Apply `command`, recording the previous state.
    ///
    /// # Errors
    /// When the command fails; the document is not changed in that case.
    pub fn execute(
        &mut self,
        document: &mut ChemDocument,
        selection: &mut ChemSelection,
        command: &Command,
    ) -> Result<(), Chem2dError> {
        let before = Snapshot {
            document: document.clone(),
            selection: selection.clone(),
        };
        command.apply(document, selection)?;
        self.undo.push(before);
        if let Some(limit) = self.limit
            && self.undo.len() > limit
        {
            self.undo.remove(0);
        }
        self.redo.clear();
        Ok(())
    }

    /// Undo the last command. Returns whether anything changed.
    pub fn undo(&mut self, document: &mut ChemDocument, selection: &mut ChemSelection) -> bool {
        let Some(snapshot) = self.undo.pop() else {
            return false;
        };
        self.redo.push(Snapshot {
            document: document.clone(),
            selection: selection.clone(),
        });
        *document = snapshot.document;
        *selection = snapshot.selection;
        true
    }

    /// Redo the last undone command. Returns whether anything changed.
    pub fn redo(&mut self, document: &mut ChemDocument, selection: &mut ChemSelection) -> bool {
        let Some(snapshot) = self.redo.pop() else {
            return false;
        };
        self.undo.push(Snapshot {
            document: document.clone(),
            selection: selection.clone(),
        });
        *document = snapshot.document;
        *selection = snapshot.selection;
        true
    }

    /// Whether an undo step exists.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether a redo step exists.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Number of undo steps.
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Drop both stacks.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ChemDocument;
    use egui_workbench_chem_core::DocumentId;

    fn document() -> ChemDocument {
        ChemDocument::new(DocumentId::new(1), "test")
    }

    #[test]
    fn add_atom_undo_and_redo_restore_the_graph() {
        let mut document = document();
        let mut selection = ChemSelection::new();
        let mut history = History::new();
        history
            .execute(
                &mut document,
                &mut selection,
                &Command::AddAtom {
                    element: Element::UNKNOWN,
                    position: [1.0, 2.0],
                },
            )
            .expect("add");
        assert_eq!(document.molecule.atom_count(), 1);
        assert!(history.can_undo());
        assert!(history.undo(&mut document, &mut selection));
        assert_eq!(document.molecule.atom_count(), 0);
        assert!(history.can_redo());
        assert!(history.redo(&mut document, &mut selection));
        assert_eq!(document.molecule.atom_count(), 1);
    }

    #[test]
    fn a_failed_command_does_not_change_the_document() {
        let mut document = document();
        let mut selection = ChemSelection::new();
        let mut history = History::new();
        let result = history.execute(
            &mut document,
            &mut selection,
            &Command::TranslateFragment {
                atoms: vec![AtomId::new(9, 9)],
                delta: [1.0, 1.0],
            },
        );
        assert!(result.is_err());
        assert!(!history.can_undo());
    }

    #[test]
    fn delete_selection_removes_atoms_and_bonds() {
        let mut document = document();
        let a = document.molecule.add_atom(Element::UNKNOWN, [0.0, 0.0, 0.0]);
        let b = document.molecule.add_atom(Element::UNKNOWN, [1.0, 0.0, 0.0]);
        document.molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        let mut selection = ChemSelection::new();
        selection.select_atom(a, egui_workbench_chem_core::SelectionMode::Add);
        let mut history = History::new();
        history
            .execute(&mut document, &mut selection, &Command::DeleteSelection)
            .expect("delete");
        assert_eq!(document.molecule.atom_count(), 1);
        assert_eq!(document.molecule.bond_count(), 0);
    }
}
