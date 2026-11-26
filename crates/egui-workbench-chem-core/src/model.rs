//! The molecular graph, its bonds, stereochemistry, and crystal cells.

use serde::{Deserialize, Serialize};

use crate::ChemError;
use crate::element::Element;
use crate::id::{AtomId, BondId, DocumentId, IdPool};

/// One atom with a 3D position.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Atom {
    /// Stable id.
    pub id: AtomId,
    /// Element.
    pub element: Element,
    /// Position in Angstrom.
    pub position: [f64; 3],
    /// Formal charge.
    pub charge: i8,
    /// Optional display label (a residue name, a custom tag).
    pub label: Option<String>,
}

/// Bond order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BondOrder {
    /// One line.
    #[default]
    Single,
    /// Two lines.
    Double,
    /// Three lines.
    Triple,
    /// An aromatic ring bond.
    Aromatic,
    /// A dative/coordination bond.
    Dative,
}

impl BondOrder {
    /// Number of strokes used to draw the bond.
    pub const fn strokes(self) -> u8 {
        match self {
            Self::Single | Self::Dative => 1,
            Self::Double => 2,
            Self::Triple => 3,
            Self::Aromatic => 1,
        }
    }

    /// Display order (aromatic rings use 1.5).
    pub const fn order(self) -> f32 {
        match self {
            Self::Single | Self::Dative => 1.0,
            Self::Double => 2.0,
            Self::Triple => 3.0,
            Self::Aromatic => 1.5,
        }
    }
}

/// 2D/3D bond stereochemistry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BondStereo {
    /// No special drawing.
    #[default]
    None,
    /// A filled wedge toward the viewer.
    Wedge,
    /// A hashed wedge away from the viewer.
    Hash,
    /// A wavy bond of unknown or mixed stereochemistry.
    Wavy,
    /// A crossed double bond with undefined E/Z.
    Either,
    /// A dotted bond for a weak or non-covalent interaction.
    Dotted,
}

/// One bond between two atoms.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bond {
    /// Stable id.
    pub id: BondId,
    /// First atom.
    pub a: AtomId,
    /// Second atom.
    pub b: AtomId,
    /// Bond order.
    pub order: BondOrder,
    /// Stereochemistry.
    pub stereo: BondStereo,
}

/// A molecular graph.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Molecule {
    /// Atoms in insertion order.
    pub atoms: Vec<Atom>,
    /// Bonds in insertion order.
    pub bonds: Vec<Bond>,
    atom_ids: IdPool,
    bond_ids: IdPool,
}

impl Molecule {
    /// An empty molecule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an atom and return its id.
    pub fn add_atom(&mut self, element: Element, position: [f64; 3]) -> AtomId {
        let (slot, generation) = self.atom_ids.allocate();
        let id = AtomId::new(slot, generation);
        self.atoms.push(Atom {
            id,
            element,
            position,
            charge: 0,
            label: None,
        });
        id
    }

    /// Add a bond, checking that both atoms exist, are distinct, and are not already
    /// bonded.
    ///
    /// # Errors
    /// When an atom is unknown, the atoms are equal, or the pair already has a bond.
    pub fn add_bond(&mut self, a: AtomId, b: AtomId, order: BondOrder) -> Result<BondId, ChemError> {
        if self.atom(a).is_none() {
            return Err(ChemError::UnknownAtom(a));
        }
        if self.atom(b).is_none() {
            return Err(ChemError::UnknownAtom(b));
        }
        if a == b {
            return Err(ChemError::SelfBond(a));
        }
        if self.bond_between(a, b).is_some() {
            return Err(ChemError::DuplicateBond { a, b });
        }
        let (slot, generation) = self.bond_ids.allocate();
        let id = BondId::new(slot, generation);
        self.bonds.push(Bond {
            id,
            a,
            b,
            order,
            stereo: BondStereo::None,
        });
        Ok(id)
    }

    /// Remove an atom and every bond attached to it.
    pub fn remove_atom(&mut self, id: AtomId) -> Option<Atom> {
        let position = self.atoms.iter().position(|atom| atom.id == id)?;
        let atom = self.atoms.remove(position);
        self.bonds.retain(|bond| bond.a != id && bond.b != id);
        self.atom_ids.release(id.slot());
        Some(atom)
    }

    /// Remove one bond.
    pub fn remove_bond(&mut self, id: BondId) -> Option<Bond> {
        let position = self.bonds.iter().position(|bond| bond.id == id)?;
        let bond = self.bonds.remove(position);
        self.bond_ids.release(id.slot());
        Some(bond)
    }

    /// The atom with `id`.
    pub fn atom(&self, id: AtomId) -> Option<&Atom> {
        self.atoms.iter().find(|atom| atom.id == id)
    }

    /// The atom with `id`, mutably.
    pub fn atom_mut(&mut self, id: AtomId) -> Option<&mut Atom> {
        self.atoms.iter_mut().find(|atom| atom.id == id)
    }

    /// The bond with `id`.
    pub fn bond(&self, id: BondId) -> Option<&Bond> {
        self.bonds.iter().find(|bond| bond.id == id)
    }

    /// The bond between `a` and `b`, in either order.
    pub fn bond_between(&self, a: AtomId, b: AtomId) -> Option<&Bond> {
        self.bonds
            .iter()
            .find(|bond| (bond.a == a && bond.b == b) || (bond.a == b && bond.b == a))
    }

    /// Every atom bonded to `id`.
    pub fn neighbors(&self, id: AtomId) -> Vec<AtomId> {
        self.bonds
            .iter()
            .filter_map(|bond| {
                if bond.a == id {
                    Some(bond.b)
                } else if bond.b == id {
                    Some(bond.a)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Whether every coordinate is finite and every bond refers to live atoms.
    ///
    /// # Errors
    /// When the graph or a coordinate is invalid.
    pub fn validate(&self) -> Result<(), ChemError> {
        for atom in &self.atoms {
            if !atom.position.iter().all(|value| value.is_finite()) {
                return Err(ChemError::InvalidGeometry(format!(
                    "atom {} has a non-finite coordinate",
                    atom.id
                )));
            }
            if !self.atom_ids.contains(atom.id.slot(), atom.id.generation()) {
                return Err(ChemError::UnknownAtom(atom.id));
            }
        }
        for bond in &self.bonds {
            if bond.a == bond.b {
                return Err(ChemError::SelfBond(bond.a));
            }
            if self.atom(bond.a).is_none() {
                return Err(ChemError::UnknownAtom(bond.a));
            }
            if self.atom(bond.b).is_none() {
                return Err(ChemError::UnknownAtom(bond.b));
            }
            if !self.bond_ids.contains(bond.id.slot(), bond.id.generation()) {
                return Err(ChemError::UnknownBond(bond.id));
            }
        }
        Ok(())
    }
}

/// A crystal unit cell in Angstrom and degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitCell {
    /// a edge length.
    pub a: f64,
    /// b edge length.
    pub b: f64,
    /// c edge length.
    pub c: f64,
    /// Angle between b and c.
    pub alpha: f64,
    /// Angle between a and c.
    pub beta: f64,
    /// Angle between a and b.
    pub gamma: f64,
}

impl UnitCell {
    /// A cell from edge lengths and angles in degrees.
    ///
    /// # Errors
    /// When a length is not positive, an angle is not in `(0, 180)`, or the cell has no
    /// volume.
    pub fn new(a: f64, b: f64, c: f64, alpha: f64, beta: f64, gamma: f64) -> Result<Self, ChemError> {
        let cell = Self {
            a,
            b,
            c,
            alpha,
            beta,
            gamma,
        };
        if !a.is_finite()
            || !b.is_finite()
            || !c.is_finite()
            || a <= 0.0
            || b <= 0.0
            || c <= 0.0
            || !(0.0..180.0).contains(&alpha)
            || !(0.0..180.0).contains(&beta)
            || !(0.0..180.0).contains(&gamma)
        {
            return Err(ChemError::InvalidGeometry("invalid unit cell parameters".to_owned()));
        }
        if !cell.volume().is_finite() || cell.volume() <= 0.0 {
            return Err(ChemError::InvalidGeometry("the unit cell has no volume".to_owned()));
        }
        Ok(cell)
    }

    /// Cell volume in cubic Angstrom.
    pub fn volume(self) -> f64 {
        let alpha = self.alpha.to_radians();
        let beta = self.beta.to_radians();
        let gamma = self.gamma.to_radians();
        self.a
            * self.b
            * self.c
            * (1.0 - alpha.cos().powi(2) - beta.cos().powi(2) - gamma.cos().powi(2)
                + 2.0 * alpha.cos() * beta.cos() * gamma.cos())
            .max(0.0)
            .sqrt()
    }

    /// Convert fractional coordinates to Cartesian coordinates.
    pub fn fractional_to_cartesian(self, fractional: [f64; 3]) -> [f64; 3] {
        let alpha = self.alpha.to_radians();
        let beta = self.beta.to_radians();
        let gamma = self.gamma.to_radians();
        let sin_gamma = gamma.sin();
        let a = [self.a, 0.0, 0.0];
        let b = [self.b * gamma.cos(), self.b * sin_gamma, 0.0];
        let c_x = self.c * beta.cos();
        let c_y = if sin_gamma.abs() < f64::EPSILON {
            0.0
        } else {
            self.c * (alpha.cos() - beta.cos() * gamma.cos()) / sin_gamma
        };
        let c_z = (self.c * self.c - c_x * c_x - c_y * c_y).max(0.0).sqrt();
        let c = [c_x, c_y, c_z];
        [
            fractional[0] * a[0] + fractional[1] * b[0] + fractional[2] * c[0],
            fractional[0] * a[1] + fractional[1] * b[1] + fractional[2] * c[1],
            fractional[0] * a[2] + fractional[1] * b[2] + fractional[2] * c[2],
        ]
    }
}

/// One chemical document.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChemDocument {
    /// Stable document id.
    pub id: DocumentId,
    /// Display name.
    pub name: String,
    /// The molecular graph.
    pub molecule: Molecule,
}

impl ChemDocument {
    /// A named, empty document.
    pub fn new(id: DocumentId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            molecule: Molecule::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atoms_and_bonds_round_trip_and_validate() {
        let mut molecule = Molecule::new();
        let carbon = molecule.add_atom(Element::from_symbol("C").expect("carbon"), [0.0, 0.0, 0.0]);
        let oxygen = molecule.add_atom(Element::from_symbol("O").expect("oxygen"), [1.2, 0.0, 0.0]);
        let bond = molecule.add_bond(carbon, oxygen, BondOrder::Double).expect("bond");
        assert_eq!(molecule.bond_between(oxygen, carbon).map(|bond| bond.id), Some(bond));
        assert_eq!(molecule.neighbors(carbon), vec![oxygen]);
        molecule.validate().expect("valid");
    }

    #[test]
    fn duplicate_and_self_bonds_are_rejected() {
        let mut molecule = Molecule::new();
        let carbon = molecule.add_atom(Element::UNKNOWN, [0.0, 0.0, 0.0]);
        let oxygen = molecule.add_atom(Element::UNKNOWN, [1.0, 0.0, 0.0]);
        assert_eq!(
            molecule.add_bond(carbon, carbon, BondOrder::Single),
            Err(ChemError::SelfBond(carbon))
        );
        molecule.add_bond(carbon, oxygen, BondOrder::Single).expect("bond");
        assert_eq!(
            molecule.add_bond(oxygen, carbon, BondOrder::Single),
            Err(ChemError::DuplicateBond { a: oxygen, b: carbon })
        );
    }

    #[test]
    fn removing_an_atom_removes_its_bonds() {
        let mut molecule = Molecule::new();
        let carbon = molecule.add_atom(Element::UNKNOWN, [0.0, 0.0, 0.0]);
        let oxygen = molecule.add_atom(Element::UNKNOWN, [1.0, 0.0, 0.0]);
        molecule.add_bond(carbon, oxygen, BondOrder::Single).expect("bond");
        molecule.remove_atom(carbon).expect("atom");
        assert!(molecule.bonds.is_empty());
        assert_eq!(molecule.atoms.len(), 1);
    }

    #[test]
    fn a_unit_cell_converts_fractional_coordinates() {
        let cell = UnitCell::new(10.0, 10.0, 10.0, 90.0, 90.0, 90.0).expect("cell");
        assert!((cell.volume() - 1000.0).abs() < 1e-9);
        assert_eq!(cell.fractional_to_cartesian([1.0, 0.0, 0.0]), [10.0, 0.0, 0.0]);
        assert!(UnitCell::new(0.0, 1.0, 1.0, 90.0, 90.0, 90.0).is_err());
    }
}
