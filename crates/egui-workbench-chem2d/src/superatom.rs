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
        boc(c, o),
        ots(c, o, s),
        bipyridine(c, n),
        phenanthroline(c, n),
        edta(c, n, o),
    ]
}

/// A flat hexagon of six vertices with circumradius `1.4` (the bond length) around `center`,
/// starting at `start` radians and going counterclockwise.
fn hexagon(center: [f64; 2], start: f64) -> [[f64; 2]; 6] {
    std::array::from_fn(|index| {
        let angle = start + std::f64::consts::FRAC_PI_3 * index as f64;
        [center[0] + 1.4 * angle.cos(), center[1] + 1.4 * angle.sin()]
    })
}

/// tert-Butoxycarbonyl, `C(=O)OC(CH3)3`, attached through the carbonyl carbon.
fn boc(c: Element, o: Element) -> Superatom {
    Superatom {
        abbreviation: "Boc",
        name: "tert-Butoxycarbonyl",
        atoms: vec![
            (c, [0.0, 0.0]),
            (o, [0.6, 1.0]),
            (o, [1.2, 0.0]),
            (c, [2.0, 0.0]),
            (c, [2.6, 0.9]),
            (c, [2.6, -0.9]),
            (c, [3.2, 0.0]),
        ],
        bonds: vec![
            (0, 1, BondOrder::Double),
            (0, 2, BondOrder::Single),
            (2, 3, BondOrder::Single),
            (3, 4, BondOrder::Single),
            (3, 5, BondOrder::Single),
            (3, 6, BondOrder::Single),
        ],
        attachment: 0,
    }
}

/// p-Toluenesulfonate, `O-S(=O)(=O)-C6H4-CH3`, attached through the ester oxygen.
fn ots(c: Element, o: Element, s: Element) -> Superatom {
    // The tolyl ring: vertex 0 bonds to sulfur, vertex 3 (para) carries the methyl.
    let ring = hexagon([4.0, 0.0], std::f64::consts::PI);
    let mut atoms = vec![(o, [0.0, 0.0]), (s, [1.2, 0.0]), (o, [1.8, 0.9]), (o, [1.8, -0.9])];
    atoms.extend(ring.iter().map(|point| (c, *point)));
    atoms.push((c, [6.9, 0.0]));
    let mut bonds = vec![
        (0, 1, BondOrder::Single),
        (1, 2, BondOrder::Double),
        (1, 3, BondOrder::Double),
        (1, 4, BondOrder::Single),
        (7, 10, BondOrder::Single),
    ];
    bonds.extend((0..6).map(|index| (4 + index, 4 + (index + 1) % 6, BondOrder::Aromatic)));
    Superatom {
        abbreviation: "OTs",
        name: "Tosylate",
        atoms,
        bonds,
        attachment: 0,
    }
}

/// 2,2'-Bipyridine: two pyridines joined at C2, with both nitrogens on the same side.
/// Attached through the first nitrogen.
fn bipyridine(c: Element, n: Element) -> Superatom {
    let pi3 = std::f64::consts::FRAC_PI_3;
    // Ring A has its link vertex at angle 0 and its nitrogen at +60 degrees; ring B mirrors
    // it with the link vertex at 180 degrees and the nitrogen at 120 degrees.
    let left = hexagon([-2.125, 0.0], pi3);
    let right = hexagon([2.125, 0.0], 2.0 * pi3);
    // Left ring: vertex 0 is the nitrogen and vertex 5 (angle 0) the link carbon. Right
    // ring: vertex 0 is the nitrogen and vertex 1 (angle 180) the link carbon.
    let mut atoms = Vec::new();
    atoms.push((n, left[0]));
    for point in &left[1..] {
        atoms.push((c, *point));
    }
    atoms.push((n, right[0]));
    for point in &right[1..] {
        atoms.push((c, *point));
    }
    let mut bonds: Vec<(usize, usize, BondOrder)> = Vec::new();
    for ring in [0usize, 6] {
        bonds.extend((0..6).map(|index| (ring + index, ring + (index + 1) % 6, BondOrder::Aromatic)));
    }
    // Left vertex 5 and right vertex 1 (atom 7) are the 2 and 2' carbons.
    bonds.push((5, 7, BondOrder::Single));
    Superatom {
        abbreviation: "bpy",
        name: "Bipyridine",
        atoms,
        bonds,
        attachment: 0,
    }
}

/// 1,10-Phenanthroline: three fused six-membered rings with the two nitrogens facing each
/// other across the bay. Attached through the first nitrogen.
fn phenanthroline(c: Element, n: Element) -> Superatom {
    use std::f64::consts::FRAC_PI_3;
    let distance = 3.0_f64.sqrt() * 1.4;
    let first = [0.0, 0.0];
    let second = [distance, 0.0];
    let third = [
        second[0] + distance * FRAC_PI_3.cos(),
        second[1] + distance * FRAC_PI_3.sin(),
    ];
    let rings = [first, second, third].map(|center| hexagon(center, FRAC_PI_3 / 2.0));
    // Merge the vertices the fused rings share.
    let mut points: Vec<[f64; 2]> = Vec::new();
    let mut membership: Vec<[bool; 3]> = Vec::new();
    let mut cycles: Vec<[usize; 6]> = Vec::new();
    for (ring_index, ring) in rings.iter().enumerate() {
        let mut cycle = [0usize; 6];
        for (slot, point) in ring.iter().enumerate() {
            let found = points
                .iter()
                .position(|known| (known[0] - point[0]).hypot(known[1] - point[1]) < 1.0e-6);
            let index = found.unwrap_or_else(|| {
                points.push(*point);
                membership.push([false; 3]);
                points.len() - 1
            });
            membership[index][ring_index] = true;
            cycle[slot] = index;
        }
        cycles.push(cycle);
    }
    // The nitrogens are the pair of atoms, one only in the first ring and one only in the
    // third, that sit closest to each other.
    let only = |ring: usize| -> Vec<usize> {
        (0..points.len())
            .filter(|index| membership[*index][ring] && membership[*index].iter().filter(|flag| **flag).count() == 1)
            .collect()
    };
    let mut best = (f64::INFINITY, 0, 0);
    for a in only(0) {
        for b in only(2) {
            let gap = (points[a][0] - points[b][0]).hypot(points[a][1] - points[b][1]);
            if gap < best.0 {
                best = (gap, a, b);
            }
        }
    }
    let (_, nitrogen_a, nitrogen_b) = best;
    // Put the first nitrogen at index 0 so it is the attachment atom.
    let mut order: Vec<usize> = vec![nitrogen_a];
    order.extend((0..points.len()).filter(|index| *index != nitrogen_a));
    let position = |old: usize| order.iter().position(|candidate| *candidate == old).unwrap_or(0);
    let atoms = order
        .iter()
        .map(|old| {
            (
                if *old == nitrogen_a || *old == nitrogen_b { n } else { c },
                points[*old],
            )
        })
        .collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut bonds = Vec::new();
    for cycle in &cycles {
        for slot in 0..6 {
            let (a, b) = (position(cycle[slot]), position(cycle[(slot + 1) % 6]));
            if seen.insert((a.min(b), a.max(b))) {
                bonds.push((a, b, BondOrder::Aromatic));
            }
        }
    }
    Superatom {
        abbreviation: "phen",
        name: "Phenanthroline",
        atoms,
        bonds,
        attachment: 0,
    }
}

/// One acetate arm of EDTA: the nitrogen it hangs from, then the positions of the
/// methylene, carboxyl carbon, carbonyl oxygen, and hydroxyl oxygen.
type Arm = (usize, [f64; 2], [f64; 2], [f64; 2], [f64; 2]);

/// Ethylenediaminetetraacetic acid: `(HOOC-CH2)2N-CH2-CH2-N(CH2-COOH)2`, attached through
/// the first nitrogen.
fn edta(c: Element, n: Element, o: Element) -> Superatom {
    let mut atoms = vec![(n, [0.0, 0.0]), (c, [1.3, 0.75]), (c, [2.6, 0.0]), (n, [3.9, 0.75])];
    let mut bonds = vec![
        (0, 1, BondOrder::Single),
        (1, 2, BondOrder::Single),
        (2, 3, BondOrder::Single),
    ];
    // Each arm is CH2, then a carboxyl carbon with a carbonyl oxygen and a hydroxyl.
    let arms: [Arm; 4] = [
        (0, [-1.2, 0.7], [-2.5, 1.45], [-2.5, 2.65], [-3.6, 0.85]),
        (0, [0.0, -1.4], [0.0, -2.9], [1.1, -3.5], [-1.1, -3.5]),
        (3, [5.1, 1.45], [6.4, 2.2], [6.4, 3.4], [7.5, 1.6]),
        (3, [3.9, -0.65], [3.9, -2.15], [5.0, -2.75], [2.8, -2.75]),
    ];
    for (nitrogen, methylene, carboxyl, carbonyl, hydroxyl) in arms {
        let base = atoms.len();
        atoms.push((c, methylene));
        atoms.push((c, carboxyl));
        atoms.push((o, carbonyl));
        atoms.push((o, hydroxyl));
        bonds.push((nitrogen, base, BondOrder::Single));
        bonds.push((base, base + 1, BondOrder::Single));
        bonds.push((base + 1, base + 2, BondOrder::Double));
        bonds.push((base + 1, base + 3, BondOrder::Single));
    }
    Superatom {
        abbreviation: "EDTA",
        name: "Ethylenediaminetetraacetic acid",
        atoms,
        bonds,
        attachment: 0,
    }
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

    /// Forget expanded groups whose pseudo atom no longer exists and drop deleted atoms and
    /// bonds from the groups that remain, so a delete cannot leave stale ids behind.
    pub fn prune_superatoms(&mut self) {
        let molecule = &self.molecule;
        self.superatoms.retain(|group| molecule.atom(group.pseudo).is_some());
        for group in &mut self.superatoms {
            group.atoms.retain(|id| molecule.atom(*id).is_some());
            group.bonds.retain(|id| molecule.bond(*id).is_some());
        }
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
mod tests;
