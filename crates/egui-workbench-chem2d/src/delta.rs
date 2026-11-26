//! Serializable document deltas for the agent and the remote client.

use serde::{Deserialize, Serialize};

use egui_workbench_chem_core::{AtomId, BondId, BondOrder, BondStereo, Element};

use crate::Chem2dError;
use crate::annotations::Annotation;
use crate::document::ChemDocument;

/// One document mutation that can travel over the remote protocol.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "delta", content = "value", rename_all = "snake_case")]
#[non_exhaustive]
pub enum DocumentDelta {
    /// Add an atom; the reply carries its id.
    AddAtom {
        /// Element.
        element: Element,
        /// Position.
        position: [f64; 2],
        /// Formal charge.
        #[serde(default)]
        charge: i8,
        /// Optional label.
        #[serde(default)]
        label: Option<String>,
    },
    /// Add a bond.
    AddBond {
        /// First atom.
        a: AtomId,
        /// Second atom.
        b: AtomId,
        /// Order.
        order: BondOrder,
        /// Stereochemistry.
        #[serde(default)]
        stereo: BondStereo,
    },
    /// Remove atoms and their bonds.
    RemoveAtoms(Vec<AtomId>),
    /// Remove bonds.
    RemoveBonds(Vec<BondId>),
    /// Move one atom.
    MoveAtom {
        /// Atom.
        id: AtomId,
        /// New xy position.
        position: [f64; 2],
    },
    /// Change a bond order.
    SetBondOrder {
        /// Bond.
        bond: BondId,
        /// New order.
        order: BondOrder,
    },
    /// Change bond stereochemistry.
    SetStereo {
        /// Bond.
        bond: BondId,
        /// New stereo.
        stereo: BondStereo,
    },
    /// Add a reaction annotation.
    AddAnnotation(Annotation),
    /// Rename the document.
    SetName(String),
}

/// The result of applying a delta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
#[non_exhaustive]
pub enum DeltaResult {
    /// An atom was added.
    Atom(AtomId),
    /// A bond was added.
    Bond(BondId),
    /// The mutation had no new id.
    Ok,
}

/// Most atoms a delta may add to one document. Lookups by id are linear, so an agent or a
/// remote client that never stops adding atoms would also make every later edit slower.
const MAX_ATOMS: usize = 100_000;

/// Largest coordinate a delta may set, in the editor's units. A finite but absurd value
/// (`1e300`) would overflow the camera and the exporters.
const MAX_COORDINATE: f64 = 1.0e6;

fn check_position(position: [f64; 2]) -> Result<(), Chem2dError> {
    if position
        .iter()
        .all(|value| value.is_finite() && value.abs() <= MAX_COORDINATE)
    {
        Ok(())
    } else {
        Err(Chem2dError::Chemistry(format!(
            "a position must be finite and within {MAX_COORDINATE} of the origin"
        )))
    }
}

impl ChemDocument {
    /// Apply one remote delta.
    ///
    /// # Errors
    /// When an atom or bond is unknown or the graph rejects the change.
    pub fn apply_delta(&mut self, delta: &DocumentDelta) -> Result<DeltaResult, Chem2dError> {
        match delta {
            DocumentDelta::AddAtom {
                element,
                position,
                charge,
                label,
            } => {
                check_position(*position)?;
                if self.molecule.atom_count() >= MAX_ATOMS {
                    return Err(Chem2dError::Chemistry(format!(
                        "a document holds at most {MAX_ATOMS} atoms"
                    )));
                }
                let id = self.molecule.add_atom(*element, [position[0], position[1], 0.0]);
                if let Some(atom) = self.molecule.atom_mut(id) {
                    atom.charge = *charge;
                    atom.label.clone_from(label);
                }
                Ok(DeltaResult::Atom(id))
            }
            DocumentDelta::AddBond { a, b, order, stereo } => {
                let id = self.molecule.add_bond(*a, *b, *order)?;
                if let Some(bond) = self.molecule.bonds.iter_mut().find(|bond| bond.id == id) {
                    bond.stereo = *stereo;
                }
                Ok(DeltaResult::Bond(id))
            }
            DocumentDelta::RemoveAtoms(atoms) => {
                for atom in atoms {
                    self.molecule.remove_atom(*atom);
                }
                self.prune_superatoms();
                Ok(DeltaResult::Ok)
            }
            DocumentDelta::RemoveBonds(bonds) => {
                for bond in bonds {
                    self.molecule.remove_bond(*bond);
                }
                Ok(DeltaResult::Ok)
            }
            DocumentDelta::MoveAtom { id, position } => {
                check_position(*position)?;
                if !self.molecule.set_position_2d(*id, *position) {
                    return Err(Chem2dError::UnknownAtom(*id));
                }
                Ok(DeltaResult::Ok)
            }
            DocumentDelta::SetBondOrder { bond, order } => {
                let Some(entry) = self.molecule.bonds.iter_mut().find(|entry| entry.id == *bond) else {
                    return Err(Chem2dError::UnknownBond(*bond));
                };
                entry.order = *order;
                Ok(DeltaResult::Ok)
            }
            DocumentDelta::SetStereo { bond, stereo } => {
                let Some(entry) = self.molecule.bonds.iter_mut().find(|entry| entry.id == *bond) else {
                    return Err(Chem2dError::UnknownBond(*bond));
                };
                entry.stereo = *stereo;
                Ok(DeltaResult::Ok)
            }
            DocumentDelta::AddAnnotation(annotation) => {
                self.annotations.push(annotation.clone());
                Ok(DeltaResult::Ok)
            }
            DocumentDelta::SetName(name) => {
                self.name.clone_from(name);
                Ok(DeltaResult::Ok)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::DocumentId;

    #[test]
    fn deltas_add_move_and_remove() {
        let mut document = ChemDocument::new(DocumentId::new(1), "test");
        let delta = DocumentDelta::AddAtom {
            element: Element::from_symbol("C").expect("C"),
            position: [0.0, 0.0],
            charge: 0,
            label: None,
        };
        let DeltaResult::Atom(id) = document.apply_delta(&delta).expect("add") else {
            panic!("expected atom");
        };
        document
            .apply_delta(&DocumentDelta::MoveAtom {
                id,
                position: [2.0, 3.0],
            })
            .expect("move");
        assert_eq!(document.molecule.position_2d(id), Some([2.0, 3.0]));
        document
            .apply_delta(&DocumentDelta::RemoveAtoms(vec![id]))
            .expect("remove");
        assert!(document.molecule.atoms.is_empty());
    }

    #[test]
    fn a_delta_round_trips_through_json() {
        let delta = DocumentDelta::SetName("benzene".to_owned());
        let json = serde_json::to_string(&delta).expect("serialize");
        let restored: DocumentDelta = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, delta);
    }
}
