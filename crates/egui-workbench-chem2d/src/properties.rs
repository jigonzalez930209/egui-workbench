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
    let carbons = count(&counts, "C") as f64;
    let hydrogens = count(&counts, "H") as f64;
    let nitrogens = count(&counts, "N") as f64;
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
        let Some(base) = common_valence(atom.element) else {
            continue;
        };
        let target = (base + i16::from(atom.charge)).max(0);
        let bonded = molecule
            .bonds
            .iter()
            .filter(|bond| bond.a == atom.id || bond.b == atom.id)
            .map(|bond| f64::from(bond.order.order()))
            .sum::<f64>()
            .round() as i16;
        if bonded > target {
            issues.push((atom.id, bonded - target));
        }
    }
    issues
}

fn implicit_hydrogens(molecule: &Molecule, id: egui_workbench_chem_core::AtomId) -> usize {
    let Some(atom) = molecule.atom(id) else {
        return 0;
    };
    let Some(base) = common_valence(atom.element) else {
        return 0;
    };
    let target = (base + i16::from(atom.charge)).max(0);
    let bonded = molecule
        .bonds
        .iter()
        .filter(|bond| bond.a == id || bond.b == id)
        .map(|bond| f64::from(bond.order.order()))
        .sum::<f64>()
        .round() as i16;
    usize::try_from((target - bonded).max(0)).unwrap_or(0)
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
    for symbol in ["C", "H"] {
        if let Some(count) = counts.get(symbol) {
            output.push_str(symbol);
            if *count > 1 {
                output.push_str(&count.to_string());
            }
        }
    }
    for (symbol, count) in counts {
        if symbol == "C" || symbol == "H" {
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
