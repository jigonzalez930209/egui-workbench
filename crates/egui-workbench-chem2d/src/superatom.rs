//! Superatom abbreviations with reversible expansion.

use serde::{Deserialize, Serialize};

use egui_workbench_chem_core::{AtomId, BondId, BondOrder, Element};

use crate::Chem2dError;
use crate::document::ChemDocument;

/// A library superatom.
#[derive(Clone, Debug)]
pub struct Superatom {
    /// Abbreviation shown on the pseudo atom.
    pub abbreviation: &'static str,
    /// Full name.
    pub name: &'static str,
    /// Fragment atoms in local coordinates.
    pub atoms: Vec<(Element, [f64; 2])>,
    /// Fragment bonds.
    pub bonds: Vec<(usize, usize, BondOrder)>,
    /// Atom that connects to the rest of the molecule.
    pub attachment: usize,
}

/// One expanded superatom.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SuperatomGroup {
    /// Abbreviation.
    pub abbreviation: String,
    /// The pseudo atom that stays in the graph.
    pub pseudo: AtomId,
    /// Fragment atoms.
    pub atoms: Vec<AtomId>,
    /// Fragment bonds.
    pub bonds: Vec<BondId>,
}

/// Every library superatom.
pub fn library() -> Vec<Superatom> {
    let c = Element::from_symbol("C").expect("carbon");
    let n = Element::from_symbol("N").expect("nitrogen");
    let o = Element::from_symbol("O").expect("oxygen");
    let s = Element::from_symbol("S").expect("sulfur");
    let h = Element::from_symbol("H").expect("hydrogen");
    vec![
        Superatom {
            abbreviation: "Ph",
            name: "Phenyl",
            atoms: (0..6)
                .map(|index| {
                    let angle = std::f64::consts::TAU * index as f64 / 6.0;
                    (c, [1.4 * angle.cos(), 1.4 * angle.sin()])
                })
                .collect(),
            bonds: (0..6)
                .map(|index| (index, (index + 1) % 6, BondOrder::Aromatic))
                .collect(),
            attachment: 0,
        },
        Superatom {
            abbreviation: "COOH",
            name: "Carboxylic acid",
            atoms: vec![(c, [0.0, 0.0]), (o, [1.2, 0.7]), (o, [1.2, -0.7]), (h, [1.9, -0.9])],
            bonds: vec![
                (0, 1, BondOrder::Double),
                (0, 2, BondOrder::Single),
                (2, 3, BondOrder::Single),
            ],
            attachment: 0,
        },
        Superatom {
            abbreviation: "Boc",
            name: "tert-Butoxycarbonyl",
            atoms: vec![
                (c, [0.0, 0.0]),
                (o, [1.2, 0.0]),
                (o, [2.0, 0.7]),
                (c, [2.8, 0.0]),
                (c, [3.4, 0.9]),
                (c, [3.4, -0.9]),
                (c, [3.9, 0.0]),
            ],
            bonds: vec![
                (0, 1, BondOrder::Single),
                (1, 2, BondOrder::Double),
                (2, 3, BondOrder::Single),
                (3, 4, BondOrder::Single),
                (3, 5, BondOrder::Single),
                (3, 6, BondOrder::Single),
            ],
            attachment: 0,
        },
        Superatom {
            abbreviation: "OTs",
            name: "Tosylate",
            atoms: vec![
                (o, [0.0, 0.0]),
                (s, [1.2, 0.0]),
                (o, [1.8, 0.9]),
                (o, [1.8, -0.9]),
                (c, [2.6, 0.0]),
            ],
            bonds: vec![
                (0, 1, BondOrder::Single),
                (1, 2, BondOrder::Double),
                (1, 3, BondOrder::Double),
                (1, 4, BondOrder::Single),
            ],
            attachment: 0,
        },
        Superatom {
            abbreviation: "bpy",
            name: "Bipyridine",
            atoms: vec![
                (n, [0.0, 0.0]),
                (c, [0.8, 0.7]),
                (c, [1.8, 0.7]),
                (c, [2.6, 0.0]),
                (n, [3.4, 0.0]),
                (c, [1.8, -0.7]),
            ],
            bonds: vec![
                (0, 1, BondOrder::Aromatic),
                (1, 2, BondOrder::Aromatic),
                (2, 3, BondOrder::Aromatic),
                (3, 4, BondOrder::Aromatic),
                (4, 5, BondOrder::Aromatic),
                (3, 5, BondOrder::Aromatic),
            ],
            attachment: 0,
        },
        Superatom {
            abbreviation: "phen",
            name: "Phenanthroline",
            atoms: vec![
                (n, [0.0, 0.0]),
                (c, [0.8, 0.7]),
                (c, [1.8, 0.7]),
                (c, [2.6, 0.0]),
                (n, [3.4, 0.0]),
                (c, [1.8, -0.7]),
                (c, [2.6, -1.4]),
            ],
            bonds: vec![
                (0, 1, BondOrder::Aromatic),
                (1, 2, BondOrder::Aromatic),
                (2, 3, BondOrder::Aromatic),
                (3, 4, BondOrder::Aromatic),
                (4, 5, BondOrder::Aromatic),
                (3, 5, BondOrder::Aromatic),
                (4, 6, BondOrder::Aromatic),
            ],
            attachment: 0,
        },
        Superatom {
            abbreviation: "EDTA",
            name: "Ethylenediaminetetraacetic acid",
            atoms: vec![
                (n, [-1.0, 0.0]),
                (c, [0.0, 0.0]),
                (c, [1.0, 0.0]),
                (n, [2.0, 0.0]),
                (c, [0.0, 1.0]),
                (c, [0.0, -1.0]),
                (c, [2.0, 1.0]),
                (c, [2.0, -1.0]),
            ],
            bonds: vec![
                (0, 1, BondOrder::Single),
                (1, 2, BondOrder::Single),
                (2, 3, BondOrder::Single),
                (1, 4, BondOrder::Single),
                (1, 5, BondOrder::Single),
                (3, 6, BondOrder::Single),
                (3, 7, BondOrder::Single),
            ],
            attachment: 0,
        },
    ]
}

/// Find a library superatom by abbreviation (case-insensitive).
pub fn find(abbreviation: &str) -> Option<Superatom> {
    library()
        .into_iter()
        .find(|superatom| superatom.abbreviation.eq_ignore_ascii_case(abbreviation))
}

impl ChemDocument {
    /// Expand the pseudo atom `pseudo` into its library fragment.
    ///
    /// The pseudo atom stays connected to the fragment; contracting it restores the
    /// original graph, including any external bonds on the pseudo atom.
    ///
    /// # Errors
    /// When the atom is unknown, has no known abbreviation, or is already expanded.
    pub fn expand_superatom(&mut self, pseudo: AtomId) -> Result<SuperatomGroup, Chem2dError> {
        if self.superatoms.iter().any(|group| group.pseudo == pseudo) {
            return Err(Chem2dError::Chemistry("the superatom is already expanded".to_owned()));
        }
        let atom = self.molecule.atom(pseudo).ok_or(Chem2dError::UnknownAtom(pseudo))?;
        let abbreviation = atom.label.clone().unwrap_or_default();
        let superatom =
            find(&abbreviation).ok_or_else(|| Chem2dError::Chemistry(format!("unknown superatom `{abbreviation}`")))?;
        let origin = [atom.position[0], atom.position[1]];
        let mut ids = Vec::with_capacity(superatom.atoms.len());
        for (element, [x, y]) in &superatom.atoms {
            ids.push(
                self.molecule
                    .add_atom(*element, [origin[0] + x, origin[1] + y - 1.8, 0.0]),
            );
        }
        let mut bonds = Vec::new();
        for (a, b, order) in &superatom.bonds {
            if let (Some(a), Some(b)) = (ids.get(*a), ids.get(*b))
                && let Ok(id) = self.molecule.add_bond(*a, *b, *order)
            {
                bonds.push(id);
            }
        }
        if let Some(attachment) = ids.get(superatom.attachment)
            && let Ok(id) = self.molecule.add_bond(pseudo, *attachment, BondOrder::Single)
        {
            bonds.push(id);
        }
        let group = SuperatomGroup {
            abbreviation: superatom.abbreviation.to_owned(),
            pseudo,
            atoms: ids,
            bonds,
        };
        self.superatoms.push(group.clone());
        Ok(group)
    }

    /// Collapse an expanded superatom back to its pseudo atom.
    ///
    /// # Errors
    /// When the group is not in this document.
    pub fn contract_superatom(&mut self, group: &SuperatomGroup) -> Result<(), Chem2dError> {
        let Some(position) = self.superatoms.iter().position(|entry| entry.pseudo == group.pseudo) else {
            return Err(Chem2dError::Chemistry("the superatom is not expanded".to_owned()));
        };
        for bond in &group.bonds {
            self.molecule.remove_bond(*bond);
        }
        for atom in &group.atoms {
            self.molecule.remove_atom(*atom);
        }
        self.superatoms.remove(position);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::DocumentId;

    #[test]
    fn every_superatom_expands_and_contracts_reversibly() {
        for superatom in library() {
            let mut document = ChemDocument::new(DocumentId::new(1), "test");
            let pseudo = document.molecule.add_atom(Element::UNKNOWN, [0.0, 0.0, 0.0]);
            document.molecule.atom_mut(pseudo).expect("atom").label = Some(superatom.abbreviation.to_owned());
            let external = document.molecule.add_atom(Element::UNKNOWN, [0.0, 2.0, 0.0]);
            document
                .molecule
                .add_bond(pseudo, external, BondOrder::Single)
                .expect("external");
            let baseline = document.molecule.atom_count();
            let group = document.expand_superatom(pseudo).expect("expand");
            assert!(document.molecule.atom_count() > baseline);
            assert_eq!(document.molecule.atom(external).expect("external").id, external);
            document.contract_superatom(&group).expect("contract");
            assert_eq!(document.molecule.atom_count(), baseline);
            assert!(document.molecule.bond_between(pseudo, external).is_some());
        }
    }
}
