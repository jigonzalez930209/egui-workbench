//! Formula, mass, elemental analysis, and simple stereochemistry descriptors.

use std::collections::BTreeMap;

use egui_workbench_chem_core::{BondStereo, Element, Molecule};

/// Calculated molecular properties.
#[derive(Clone, Debug, PartialEq)]
pub struct MolecularProperties {
    /// Hill-system formula.
    pub formula: String,
    /// Average molecular weight.
    pub molecular_weight: f64,
    /// Monoisotopic mass.
    pub exact_mass: f64,
    /// Mass percentage by element, sorted by symbol.
    pub elemental_analysis: Vec<(String, f64)>,
    /// Double-bond equivalents.
    pub double_bond_equivalents: f64,
    /// Atom count.
    pub atoms: usize,
    /// Bond count.
    pub bonds: usize,
}

/// Compute the properties of `molecule`.
pub fn properties(molecule: &Molecule) -> MolecularProperties {
    let mut counts = BTreeMap::<String, usize>::new();
    for atom in &molecule.atoms {
        *counts.entry(atom.element.symbol().to_owned()).or_default() += 1;
    }
    let implicit_hydrogens = molecule
        .atoms
        .iter()
        .map(|atom| implicit_hydrogens(molecule, atom.id))
        .sum::<usize>();
    if implicit_hydrogens > 0 {
        *counts.entry("H".to_owned()).or_default() += implicit_hydrogens;
    }
    let formula = hill_formula(&counts);
    let hydrogen_mass = Element::from_symbol("H").map_or(0.0, Element::atomic_mass);
    let molecular_weight = molecule
        .atoms
        .iter()
        .map(|atom| atom.element.atomic_mass())
        .sum::<f64>()
        + implicit_hydrogens as f64 * hydrogen_mass;
    let exact_mass = molecule.atoms.iter().map(|atom| exact_mass(atom.element)).sum::<f64>()
        + implicit_hydrogens as f64 * exact_mass(Element::from_symbol("H").unwrap_or(Element::UNKNOWN));
    let elemental_analysis = counts
        .iter()
        .map(|(symbol, count)| {
            let mass = Element::from_symbol(symbol).map_or(0.0, Element::atomic_mass) * *count as f64;
            let percentage = if molecular_weight > 0.0 {
                mass / molecular_weight * 100.0
            } else {
                0.0
            };
            (symbol.clone(), percentage)
        })
        .collect();
    // Tetravalent Si counts like C, trivalent P like N.
    let carbons = (count(&counts, "C") + count(&counts, "Si")) as f64;
    let hydrogens = count(&counts, "H") as f64;
    let nitrogens = (count(&counts, "N") + count(&counts, "P")) as f64;
    let halogens = ["F", "Cl", "Br", "I"]
        .iter()
        .map(|symbol| count(&counts, symbol) as f64)
        .sum::<f64>();
    MolecularProperties {
        formula,
        molecular_weight,
        exact_mass,
        elemental_analysis,
        double_bond_equivalents: (2.0 * carbons + 2.0 + nitrogens - hydrogens - halogens) / 2.0,
        atoms: molecule.atoms.len(),
        bonds: molecule.bonds.len(),
    }
}

/// A simple stereochemistry descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StereoDescriptor {
    /// Bond id slot.
    pub bond: u32,
    /// `R`, `S`, or `E/Z`-style label.
    pub label: String,
}

/// Report the stereo bonds as `R/S` (wedge/hash) or `E/Z` (crossed) descriptors.///
/// This is a conservative graph label, not a full CIP implementation; it names every
/// bond that carries stereochemistry so a UI can show it and a future CIP pass can
/// replace the label.
pub fn stereo_descriptors(molecule: &Molecule) -> Vec<StereoDescriptor> {
    molecule
        .bonds
        .iter()
        .filter_map(|bond| {
            let label = match bond.stereo {
                BondStereo::Wedge | BondStereo::Hash => "R/S",
                BondStereo::Either | BondStereo::Wavy => "E/Z",
                _ => return None,
            };
            Some(StereoDescriptor {
                bond: bond.id.slot(),
                label: label.to_owned(),
            })
        })
        .collect()
}

fn count(counts: &BTreeMap<String, usize>, symbol: &str) -> usize {
    counts.get(symbol).copied().unwrap_or(0)
}

/// Atoms whose common valence is exceeded by their bond orders.
pub fn valence_issues(molecule: &Molecule) -> Vec<(egui_workbench_chem_core::AtomId, i16)> {
    let mut issues = Vec::new();
    for atom in &molecule.atoms {
        let Some(target) = target_valence(atom.element, atom.charge) else {
            continue;
        };
        let Some(bonded) = bonded_valence(molecule, atom.id, atom.element) else {
            continue;
        };
        if bonded > target {
            issues.push((atom.id, bonded - target));
        }
    }
    issues
}

/// The valence used by one atom in its bonds, or `None` when it cannot be known.
///
/// Aromatic bonds count 1.5, so a carbon shared by two rings (three aromatic bonds) sums
/// to 4.5; rounding that up to 5 reported a valence error on every naphthalene. It uses
/// one sigma bond per neighbour plus the single pi bond a carbon contributes, so the sum
/// is truncated for carbon. Aromatic N, O, S, and Se need a Kekulé structure: `None`.
fn bonded_valence(molecule: &Molecule, id: egui_workbench_chem_core::AtomId, element: Element) -> Option<i16> {
    let mut sum = 0.0;
    let mut aromatic = false;
    for bond in molecule.bonds.iter().filter(|bond| bond.a == id || bond.b == id) {
        sum += f64::from(bond.order.order());
        aromatic |= bond.order == egui_workbench_chem_core::BondOrder::Aromatic;
    }
    if !aromatic {
        return Some(sum.round() as i16);
    }
    if matches!(element.atomic_number(), 7 | 8 | 16 | 34) {
        return None;
    }
    Some(sum.floor() as i16)
}

pub(crate) fn implicit_hydrogens(molecule: &Molecule, id: egui_workbench_chem_core::AtomId) -> usize {
    let Some(atom) = molecule.atom(id) else {
        return 0;
    };
    let Some(target) = target_valence(atom.element, atom.charge) else {
        return 0;
    };
    // An aromatic heteroatom cannot be given a hydrogen count without kekulizing the ring
    // (pyridine N has none, pyrrole N has one), so it gets none rather than a guess.
    let Some(bonded) = bonded_valence(molecule, id, atom.element) else {
        return 0;
    };
    usize::try_from((target - bonded).max(0)).unwrap_or(0)
}

/// The valence an atom of `element` with formal `charge` fills with bonds and hydrogens.
///
/// Charge moves the valence in opposite directions by group: boron gains a bond when it
/// is negative (BH4-), carbon and silicon lose one either way (a carbocation and a
/// carbanion are both trivalent), and the elements of groups 15-17 gain one when positive
/// (NH4+, H3O+) and lose one when negative (NH2-, OH-).
pub(crate) fn target_valence(element: Element, charge: i8) -> Option<i16> {
    let base = common_valence(element)?;
    let charge = i16::from(charge);
    let target = match element.atomic_number() {
        5 => base - charge,
        6 | 14 => base - charge.abs(),
        _ => base + charge,
    };
    Some(target.max(0))
}

fn common_valence(element: Element) -> Option<i16> {
    Some(match element.atomic_number() {
        1 => 1,
        5 => 3,
        6 => 4,
        7 => 3,
        8 => 2,
        9 => 1,
        14 => 4,
        15 => 3,
        16 => 2,
        17 => 1,
        34 => 2,
        35 => 1,
        53 => 1,
        _ => return None,
    })
}

fn hill_formula(counts: &BTreeMap<String, usize>) -> String {
    let mut output = String::new();
    // Hill order: carbon, then hydrogen, then the rest alphabetically. Without carbon,
    // everything including hydrogen is alphabetical (`ClH`, not `HCl`).
    let has_carbon = counts.contains_key("C");
    if has_carbon {
        for symbol in ["C", "H"] {
            if let Some(count) = counts.get(symbol) {
                output.push_str(symbol);
                if *count > 1 {
                    output.push_str(&count.to_string());
                }
            }
        }
    }
    for (symbol, count) in counts {
        if has_carbon && (symbol == "C" || symbol == "H") {
            continue;
        }
        output.push_str(symbol);
        if *count > 1 {
            output.push_str(&count.to_string());
        }
    }
    output
}

fn exact_mass(element: Element) -> f64 {
    match element.atomic_number() {
        1 => 1.007_825,
        5 => 11.009_305,
        6 => 12.0,
        7 => 14.003_074,
        8 => 15.994_915,
        9 => 18.998_403,
        11 => 22.989_770,
        12 => 23.985_042,
        14 => 27.976_927,
        15 => 30.973_762,
        16 => 31.972_071,
        17 => 34.968_853,
        19 => 38.963_707,
        20 => 39.962_591,
        26 => 55.934_938,
        29 => 62.929_598,
        30 => 63.929_142,
        35 => 78.918_338,
        53 => 126.904_473,
        _ => element.atomic_mass(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Molecule2D;
    use egui_workbench_chem_core::{BondOrder, Element};

    #[test]
    fn water_has_the_expected_formula_and_mass() {
        let mut molecule = Molecule2D::new();
        let o = molecule.add_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
        let h1 = molecule.add_atom(Element::from_symbol("H").expect("H"), [1.0, 0.0, 0.0]);
        let h2 = molecule.add_atom(Element::from_symbol("H").expect("H"), [-1.0, 0.0, 0.0]);
        molecule.add_bond(o, h1, BondOrder::Single).expect("bond");
        molecule.add_bond(o, h2, BondOrder::Single).expect("bond");
        let properties = properties(&molecule);
        assert_eq!(properties.formula, "H2O");
        assert!((properties.molecular_weight - 18.015).abs() < 0.01);
        assert!((properties.exact_mass - 18.010_565).abs() < 0.001);
        assert_eq!(properties.double_bond_equivalents, 0.0);
    }

    #[test]
    fn benzene_has_four_double_bond_equivalents() {
        let mut molecule = Molecule2D::new();
        let ids = (0..6)
            .map(|index| {
                let angle = std::f64::consts::TAU * index as f64 / 6.0;
                molecule.add_atom(Element::from_symbol("C").expect("C"), [angle.cos(), angle.sin(), 0.0])
            })
            .collect::<Vec<_>>();
        for index in 0..6 {
            molecule
                .add_bond(ids[index], ids[(index + 1) % 6], BondOrder::Aromatic)
                .expect("bond");
        }
        let properties = properties(&molecule);
        assert_eq!(implicit_hydrogens(&molecule, ids[0]), 1);
        assert_eq!(properties.formula, "C6H6");
        assert_eq!(properties.double_bond_equivalents, 4.0);
    }

    #[test]
    fn stereo_bonds_are_reported() {
        let mut molecule = Molecule2D::new();
        let a = molecule.add_atom(Element::UNKNOWN, [0.0, 0.0, 0.0]);
        let b = molecule.add_atom(Element::UNKNOWN, [1.0, 0.0, 0.0]);
        let id = molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        molecule
            .bonds
            .iter_mut()
            .find(|bond| bond.id == id)
            .expect("bond")
            .stereo = BondStereo::Wedge;
        assert_eq!(stereo_descriptors(&molecule).len(), 1);
    }
}

#[cfg(test)]
mod chemistry_tests {
    use super::*;
    use crate::document::Molecule2D;
    use egui_workbench_chem_core::{BondOrder, Element};

    fn element(symbol: &str) -> Element {
        Element::from_symbol(symbol).expect("element")
    }

    #[test]
    fn the_hill_formula_orders_carbon_free_compounds_alphabetically() {
        let mut molecule = Molecule2D::new();
        let h = molecule.add_atom(element("H"), [0.0; 3]);
        let cl = molecule.add_atom(element("Cl"), [1.0, 0.0, 0.0]);
        molecule.add_bond(h, cl, BondOrder::Single).expect("bond");
        assert_eq!(properties(&molecule).formula, "ClH");
    }

    #[test]
    fn methane_keeps_carbon_then_hydrogen() {
        let mut molecule = Molecule2D::new();
        molecule.add_atom(element("C"), [0.0; 3]);
        assert_eq!(properties(&molecule).formula, "CH4");
    }

    #[test]
    fn naphthalene_has_no_valence_issue_and_eight_hydrogens() {
        // Two fused hexagons sharing the edge between atoms 5 and 0.
        let mut molecule = Molecule2D::new();
        let ids: Vec<_> = (0..10)
            .map(|index| molecule.add_atom(element("C"), [f64::from(index), 0.0, 0.0]))
            .collect();
        let rings: [[usize; 6]; 2] = [[0, 1, 2, 3, 4, 5], [5, 0, 6, 7, 8, 9]];
        for ring in rings {
            for slot in 0..6 {
                let _ = molecule.add_bond(ids[ring[slot]], ids[ring[(slot + 1) % 6]], BondOrder::Aromatic);
            }
        }
        // Atoms 0 and 5 are shared: three aromatic bonds each.
        assert!(valence_issues(&molecule).is_empty(), "{:?}", valence_issues(&molecule));
        assert_eq!(properties(&molecule).formula, "C10H8");
    }
}

#[cfg(test)]
mod charge_tests {
    use super::*;
    use crate::document::Molecule2D;
    use egui_workbench_chem_core::Element;

    fn lone(symbol: &str, charge: i8) -> Molecule2D {
        let mut molecule = Molecule2D::new();
        let id = molecule.add_atom(Element::from_symbol(symbol).expect("element"), [0.0; 3]);
        molecule.atom_mut(id).expect("atom").charge = charge;
        molecule
    }

    #[test]
    fn charged_atoms_get_the_hydrogens_their_group_implies() {
        // (element, charge, hydrogens): CH3+, CH3-, NH4+, NH2-, OH-, H3O+, BH4-.
        for (symbol, charge, expected) in [
            ("C", 1, 3),
            ("C", -1, 3),
            ("N", 1, 4),
            ("N", -1, 2),
            ("O", -1, 1),
            ("O", 1, 3),
            ("B", -1, 4),
        ] {
            let molecule = lone(symbol, charge);
            let id = molecule.atoms[0].id;
            assert_eq!(implicit_hydrogens(&molecule, id), expected, "{symbol}{charge:+}");
        }
    }
}
