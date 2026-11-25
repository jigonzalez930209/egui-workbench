//! The 2D document and its molecule view.

use std::ops::{Deref, DerefMut};

use egui_workbench_chem_core::{Atom, Bond, ChemSelection, DocumentId, Molecule};

/// A 2D atom; the shared molecule stores its z coordinate as `0`.
pub type Atom2D = Atom;

/// A 2D bond.
pub type Bond2D = Bond;

/// The shared molecular graph with 2D helpers.
///
/// The molecule is the same type a 3D panel uses, so ids, elements, bond orders,
/// stereochemistry, and selection are shared. The 2D editor ignores the z coordinate.
#[derive(Clone, Debug, Default)]
pub struct Molecule2D(Molecule);

impl Molecule2D {
    /// An empty molecule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Wrap a shared molecule.
    pub fn from_core(molecule: Molecule) -> Self {
        Self(molecule)
    }

    /// The underlying shared molecule.
    pub fn into_core(self) -> Molecule {
        self.0
    }

    /// The underlying shared molecule, borrowed.
    pub fn as_core(&self) -> &Molecule {
        &self.0
    }

    /// The underlying shared molecule, mutably.
    pub fn as_core_mut(&mut self) -> &mut Molecule {
        &mut self.0
    }

    /// The xy position of `id`.
    pub fn position_2d(&self, id: egui_workbench_chem_core::AtomId) -> Option<[f64; 2]> {
        self.0.atom(id).map(|atom| [atom.position[0], atom.position[1]])
    }

    /// Move an atom in the xy plane, preserving z.
    pub fn set_position_2d(&mut self, id: egui_workbench_chem_core::AtomId, position: [f64; 2]) -> bool {
        let Some(atom) = self.0.atom_mut(id) else {
            return false;
        };
        atom.position[0] = position[0];
        atom.position[1] = position[1];
        true
    }

    /// Move every listed atom by `delta` in the xy plane.
    pub fn translate(&mut self, atoms: &[egui_workbench_chem_core::AtomId], delta: [f64; 2]) {
        for id in atoms {
            if let Some(atom) = self.0.atom_mut(*id) {
                atom.position[0] += delta[0];
                atom.position[1] += delta[1];
            }
        }
    }

    /// The bounding box of the atom positions as `[min_x, min_y, max_x, max_y]`.
    pub fn bounds(&self) -> Option<[f64; 4]> {
        let mut bounds: Option<[f64; 4]> = None;
        for atom in &self.0.atoms {
            let [x, y] = [atom.position[0], atom.position[1]];
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            bounds = Some(match bounds {
                Some([min_x, min_y, max_x, max_y]) => [min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y)],
                None => [x, y, x, y],
            });
        }
        bounds
    }

    /// The selection of every atom and bond that still exists.
    pub fn live_selection(&self, selection: &ChemSelection) -> ChemSelection {
        let atoms = self.0.atoms.iter().map(|atom| atom.id).collect::<Vec<_>>();
        let bonds = self.0.bonds.iter().map(|bond| bond.id).collect::<Vec<_>>();
        let mut live = selection.clone();
        live.retain_existing(&atoms, &bonds);
        live
    }

    /// A new molecule with the same topology and fresh ids.
    pub fn duplicate_topology(&self) -> Self {
        let mut copy = Molecule2D::new();
        for atom in &self.0.atoms {
            let id = copy.add_atom(atom.element, atom.position);
            if let Some(target) = copy.atom_mut(id) {
                target.charge = atom.charge;
                target.label.clone_from(&atom.label);
            }
        }
        let ids = self.0.atoms.iter().map(|atom| atom.id).collect::<Vec<_>>();
        for bond in &self.0.bonds {
            let a = ids.iter().position(|id| *id == bond.a);
            let b = ids.iter().position(|id| *id == bond.b);
            if let (Some(a), Some(b)) = (a, b)
                && let (Some(a), Some(b)) = (
                    copy.0.atoms.get(a).map(|atom| atom.id),
                    copy.0.atoms.get(b).map(|atom| atom.id),
                )
            {
                let _ = copy.add_bond(a, b, bond.order);
                if let Some(new_bond) = copy.bonds.last_mut() {
                    new_bond.stereo = bond.stereo;
                }
            }
        }
        copy
    }

    /// Number of atoms.
    pub fn atom_count(&self) -> usize {
        self.0.atoms.len()
    }

    /// Number of bonds.
    pub fn bond_count(&self) -> usize {
        self.0.bonds.len()
    }
}

impl Deref for Molecule2D {
    type Target = Molecule;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Molecule2D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// One 2D chemical document.
#[derive(Clone, Debug)]
pub struct ChemDocument {
    /// Stable document id.
    pub id: DocumentId,
    /// Display name.
    pub name: String,
    /// The shared molecule.
    pub molecule: Molecule2D,
}

impl ChemDocument {
    /// A named, empty document.
    pub fn new(id: DocumentId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            molecule: Molecule2D::new(),
        }
    }

    /// The bounding box of the atom positions.
    pub fn bounds(&self) -> Option<[f64; 4]> {
        self.molecule.bounds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::{BondOrder, Element};

    #[test]
    fn positions_are_xy_and_translate_keeps_z() {
        let mut molecule = Molecule2D::new();
        let id = molecule.add_atom(Element::UNKNOWN, [1.0, 2.0, 3.0]);
        assert_eq!(molecule.position_2d(id), Some([1.0, 2.0]));
        molecule.translate(&[id], [2.0, -1.0]);
        assert_eq!(molecule.atom(id).expect("atom").position, [3.0, 1.0, 3.0]);
        assert_eq!(molecule.bounds(), Some([3.0, 1.0, 3.0, 1.0]));
    }

    #[test]
    fn duplicate_topology_keeps_atoms_and_bonds() {
        let mut molecule = Molecule2D::new();
        let a = molecule.add_atom(Element::from_symbol("C").expect("carbon"), [0.0, 0.0, 0.0]);
        let b = molecule.add_atom(Element::from_symbol("O").expect("oxygen"), [1.2, 0.0, 0.0]);
        molecule.add_bond(a, b, BondOrder::Double).expect("bond");
        let copy = molecule.duplicate_topology();
        assert_eq!(copy.atom_count(), 2);
        assert_eq!(copy.bond_count(), 1);
        copy.validate().expect("valid");
    }
}
