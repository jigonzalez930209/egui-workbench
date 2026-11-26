//! A small compendium of named molecules and substituents for the editor palette.
//!
//! Every entry carries a SMILES string that the crate's own reader can parse, so the
//! palette behaves exactly like typing the SMILES in the editor. The entries are grouped
//! by chemical type and use no stereochemistry, because the 2D reader does not assign it.

use crate::SmilesGraph;
use crate::smiles::{SmilesError, parse_smiles};

/// How a library entry is meant to be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryKind {
    /// A complete molecule, normally placed on empty canvas.
    Molecule,
    /// A substituent or group, normally attached to an atom.
    Group,
}

/// One named structure in the compendium.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LibraryEntry {
    /// Display name, including a common abbreviation when there is one.
    pub name: &'static str,
    /// SMILES read by [`crate::parse_smiles`].
    pub smiles: &'static str,
    /// Whether it is a free molecule or a group.
    pub kind: LibraryKind,
}

impl LibraryEntry {
    /// Parse this entry's SMILES.
    ///
    /// # Errors
    /// When the entry contains a SMILES the reader does not support. The library test
    /// checks every entry, so this only fails if the data was edited incorrectly.
    pub fn graph(&self) -> Result<SmilesGraph, SmilesError> {
        parse_smiles(self.smiles)
    }
}

/// One group of related entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LibraryGroup {
    /// Display name of the group.
    pub name: &'static str,
    /// Entries in menu order.
    pub entries: &'static [LibraryEntry],
}

const fn molecule(name: &'static str, smiles: &'static str) -> LibraryEntry {
    LibraryEntry {
        name,
        smiles,
        kind: LibraryKind::Molecule,
    }
}

const fn group(name: &'static str, smiles: &'static str) -> LibraryEntry {
    LibraryEntry {
        name,
        smiles,
        kind: LibraryKind::Group,
    }
}

/// The standard amino acids as free molecules.
pub const AMINO_ACIDS: &[LibraryEntry] = &[
    molecule("Glycine (Gly)", "NCC(=O)O"),
    molecule("Alanine (Ala)", "NC(C)C(=O)O"),
    molecule("Valine (Val)", "NC(C(C)C)C(=O)O"),
    molecule("Leucine (Leu)", "NC(CC(C)C)C(=O)O"),
    molecule("Isoleucine (Ile)", "NC(C(C)CC)C(=O)O"),
    molecule("Proline (Pro)", "N1CCCC1C(=O)O"),
    molecule("Phenylalanine (Phe)", "NC(Cc1ccccc1)C(=O)O"),
    molecule("Tyrosine (Tyr)", "NC(Cc1ccc(O)cc1)C(=O)O"),
    molecule("Tryptophan (Trp)", "C1=CC=C2C(=C1)C(=CN2)CC(C(=O)O)N"),
    molecule("Serine (Ser)", "NC(CO)C(=O)O"),
    molecule("Threonine (Thr)", "NC(C(O)C)C(=O)O"),
    molecule("Cysteine (Cys)", "NC(CS)C(=O)O"),
    molecule("Methionine (Met)", "NC(CCSC)C(=O)O"),
    molecule("Asparagine (Asn)", "NC(CC(=O)N)C(=O)O"),
    molecule("Glutamine (Gln)", "NC(CCC(=O)N)C(=O)O"),
    molecule("Aspartic acid (Asp)", "NC(CC(=O)O)C(=O)O"),
    molecule("Glutamic acid (Glu)", "NC(CCC(=O)O)C(=O)O"),
    molecule("Lysine (Lys)", "NC(CCCCN)C(=O)O"),
    molecule("Arginine (Arg)", "NC(CCCNC(=N)N)C(=O)O"),
    molecule("Histidine (His)", "NC(Cc1cnc[nH]1)C(=O)O"),
];

/// Amino acid side chains, attached through their first atom.
pub const AMINO_ACID_SIDE_CHAINS: &[LibraryEntry] = &[
    group("Alanine side chain (Me)", "C"),
    group("Valine side chain (iPr)", "C(C)C"),
    group("Leucine side chain (iBu)", "CC(C)C"),
    group("Isoleucine side chain (sBu)", "C(C)CC"),
    group("Phenylalanine side chain (Bn)", "Cc1ccccc1"),
    group("Tyrosine side chain", "Cc1ccc(O)cc1"),
    group("Tryptophan side chain", "Cc1c[nH]c2ccccc12"),
    group("Serine side chain", "CO"),
    group("Threonine side chain", "C(O)C"),
    group("Cysteine side chain", "CS"),
    group("Methionine side chain", "CCSC"),
    group("Asparagine side chain", "CC(=O)N"),
    group("Glutamine side chain", "CCC(=O)N"),
    group("Aspartic acid side chain", "CC(=O)O"),
    group("Glutamic acid side chain", "CCC(=O)O"),
    group("Lysine side chain", "CCCCN"),
    group("Arginine side chain", "CCCNC(=N)N"),
    group("Histidine side chain", "Cc1cnc[nH]1"),
];

/// Sugar structures, mostly in their common ring form.
pub const SUGARS: &[LibraryEntry] = &[
    molecule("D-Glucose (open chain)", "OCC(O)C(O)C(O)C(O)C=O"),
    molecule("D-Glucose (pyranose)", "OCC1OC(O)C(O)C(O)C1O"),
    molecule("D-Fructose (open chain)", "OCC(=O)C(O)C(O)C(O)CO"),
    molecule("D-Ribose (furanose)", "OCC1OC(O)C(O)C1O"),
    molecule("2-Deoxy-D-ribose", "OCC1OC(O)CC1O"),
    molecule("Sucrose", "OCC1OC(CO)(OC2OC(CO)C(O)C(O)C2O)C(O)C1O"),
    molecule("Maltose", "OCC1OC(OC2C(CO)OC(O)C(O)C2O)C(O)C(O)C1O"),
];

/// The common nucleobases.
pub const NUCLEOBASES: &[LibraryEntry] = &[
    molecule("Adenine (A)", "C1=NC2=C(N1)C(=NC=N2)N"),
    molecule("Guanine (G)", "C1=NC2=C(N1)C(=O)NC(=N2)N"),
    molecule("Cytosine (C)", "NC1=NC=CC(=O)N1"),
    molecule("Thymine (T)", "CC1=CNC(=O)NC1=O"),
    molecule("Uracil (U)", "O=C1NC=CC(=O)N1"),
];

/// Rings and heterocycles that are useful as substituents.
pub const HETEROCYCLES: &[LibraryEntry] = &[
    group("Benzene / Phenyl", "c1ccccc1"),
    group("Pyridine", "c1ccncc1"),
    group("Pyrimidine", "c1cncnc1"),
    group("Pyrrole", "c1cc[nH]c1"),
    group("Imidazole", "c1cnc[nH]1"),
    group("Furan", "c1ccoc1"),
    group("Thiophene", "c1ccsc1"),
    group("Indole", "c1ccc2[nH]ccc2c1"),
    group("Naphthalene", "c1ccc2ccccc2c1"),
    group("Cyclohexane", "C1CCCCC1"),
    group("Cyclopentane", "C1CCCC1"),
    group("Piperidine", "C1CCNCC1"),
    group("Tetrahydrofuran ring", "C1CCOC1"),
];

/// Functional groups, attached through their first atom.
pub const FUNCTIONAL_GROUPS: &[LibraryEntry] = &[
    group("Hydroxyl (OH)", "O"),
    group("Amino (NH2)", "N"),
    group("Methyl", "C"),
    group("Ethyl", "CC"),
    group("Phenyl", "c1ccccc1"),
    group("Benzyl", "Cc1ccccc1"),
    group("Carboxyl (COOH)", "C(=O)O"),
    group("Aldehyde (CHO)", "C=O"),
    group("Acetyl", "C(=O)C"),
    group("Ester (COOMe)", "C(=O)OC"),
    group("Amide (CONH2)", "C(=O)N"),
    group("Nitrile (CN)", "C#N"),
    group("Nitro (NO2)", "[N+](=O)[O-]"),
    group("Sulfonic acid (SO3H)", "S(=O)(=O)O"),
    group("Phosphate (PO4H2)", "P(=O)(O)O"),
    group("Methoxy (OMe)", "OC"),
    group("Trifluoromethyl (CF3)", "C(F)(F)F"),
    group("tert-Butyl (tBu)", "C(C)(C)C"),
    group("Boc", "C(=O)OC(C)(C)C"),
    group("Tosyl (Ts)", "S(=O)(=O)c1ccc(C)cc1"),
];

/// Common solvents as free molecules.
pub const SOLVENTS: &[LibraryEntry] = &[
    molecule("Water", "O"),
    molecule("Methanol", "CO"),
    molecule("Ethanol", "CCO"),
    molecule("Acetone", "CC(=O)C"),
    molecule("Acetonitrile", "CC#N"),
    molecule("Dimethyl sulfoxide (DMSO)", "CS(=O)C"),
    molecule("Dimethylformamide (DMF)", "CN(C)C=O"),
    molecule("Tetrahydrofuran (THF)", "C1CCOC1"),
    molecule("Dichloromethane (DCM)", "ClCCl"),
    molecule("Chloroform", "ClC(Cl)Cl"),
    molecule("Toluene", "Cc1ccccc1"),
    molecule("Hexane", "CCCCCC"),
    molecule("Diethyl ether", "CCOCC"),
];

/// Ligands and chelators.
pub const LIGANDS: &[LibraryEntry] = &[
    molecule("Acetylacetone (acac)", "CC(=O)CC(=O)C"),
    molecule("2,2'-Bipyridine (bpy)", "c1ccc(-c2ccccn2)nc1"),
    molecule("1,10-Phenanthroline (phen)", "c1cnc2c(c1)ccc1cccnc12"),
    molecule("EDTA", "OC(=O)CN(CCN(CC(=O)O)CC(=O)O)CC(=O)O"),
    molecule("Cyclopentadienyl anion (Cp-)", "[CH-]1C=CC=C1"),
];

/// Lipids and small building blocks.
pub const LIPIDS: &[LibraryEntry] = &[
    molecule("Glycerol", "OCC(O)CO"),
    molecule("Stearic acid", "CCCCCCCCCCCCCCCCCC(=O)O"),
    molecule("Oleic acid", "CCCCCCCCC=CCCCCCCCC(=O)O"),
    molecule("Palmitic acid", "CCCCCCCCCCCCCCCC(=O)O"),
];

/// The full compendium, in menu order.
pub const LIBRARY: &[LibraryGroup] = &[
    LibraryGroup {
        name: "Amino acids",
        entries: AMINO_ACIDS,
    },
    LibraryGroup {
        name: "Amino acid side chains",
        entries: AMINO_ACID_SIDE_CHAINS,
    },
    LibraryGroup {
        name: "Sugars",
        entries: SUGARS,
    },
    LibraryGroup {
        name: "Nucleobases",
        entries: NUCLEOBASES,
    },
    LibraryGroup {
        name: "Rings and heterocycles",
        entries: HETEROCYCLES,
    },
    LibraryGroup {
        name: "Functional groups",
        entries: FUNCTIONAL_GROUPS,
    },
    LibraryGroup {
        name: "Solvents",
        entries: SOLVENTS,
    },
    LibraryGroup {
        name: "Ligands and chelators",
        entries: LIGANDS,
    },
    LibraryGroup {
        name: "Lipids and building blocks",
        entries: LIPIDS,
    },
];

/// Find an entry by its display name.
pub fn find(name: &str) -> Option<&'static LibraryEntry> {
    LIBRARY
        .iter()
        .flat_map(|group| group.entries.iter())
        .find(|entry| entry.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn every_library_entry_parses_and_has_a_unique_name() {
        let mut names = BTreeSet::new();
        for group in LIBRARY {
            assert!(!group.entries.is_empty(), "{} is empty", group.name);
            for entry in group.entries {
                entry.graph().unwrap_or_else(|error| panic!("{}: {error}", entry.name));
                assert!(names.insert(entry.name), "duplicate entry {}", entry.name);
            }
        }
    }

    #[test]
    fn entries_have_a_usable_size() {
        for group in LIBRARY {
            for entry in group.entries {
                let graph = entry.graph().expect("parsed");
                assert!(graph.atoms.len() <= 100, "{} is too large", entry.name);
            }
        }
    }

    #[test]
    fn lookup_is_case_insensitive() {
        assert_eq!(find("alanine (ala)").map(|entry| entry.name), Some("Alanine (Ala)"));
        assert!(find("not-a-molecule").is_none());
    }

    #[test]
    fn every_entry_has_an_optimized_idempotent_layout() {
        for group in LIBRARY {
            for entry in group.entries {
                let mut molecule = entry.graph().expect("parsed").to_molecule();
                let first = molecule.atoms.iter().map(|atom| atom.position).collect::<Vec<_>>();
                crate::layout::optimize_layout(&mut molecule);
                let moved = first
                    .iter()
                    .zip(&molecule.atoms)
                    .map(|(before, after)| (before[0] - after.position[0]).hypot(before[1] - after.position[1]))
                    .fold(0.0_f64, f64::max);
                assert!(moved < 1.0e-9, "{} moved {moved} on a second pass", entry.name);
                for bond in &molecule.bonds {
                    let (Some(a), Some(b)) = (molecule.atom(bond.a), molecule.atom(bond.b)) else {
                        panic!("{} has a dangling bond", entry.name);
                    };
                    let length = (a.position[0] - b.position[0]).hypot(a.position[1] - b.position[1]);
                    let expected = if a.element.atomic_number() == 1 || b.element.atomic_number() == 1 {
                        1.2
                    } else {
                        1.5
                    };
                    assert!((length - expected).abs() < 0.12, "{} bond length {length}", entry.name);
                }
                for (left, a) in molecule.atoms.iter().enumerate() {
                    for b in molecule.atoms.iter().skip(left + 1) {
                        if molecule.bond_between(a.id, b.id).is_some() {
                            continue;
                        }
                        let distance = (a.position[0] - b.position[0]).hypot(a.position[1] - b.position[1]);
                        assert!(
                            distance > 0.75,
                            "{} atoms {} {} and {} {} overlap ({distance})",
                            entry.name,
                            a.element.symbol(),
                            a.id.slot(),
                            b.element.symbol(),
                            b.id.slot()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn named_structures_have_the_expected_formula() {
        let formula = |name: &str| {
            let entry = find(name).unwrap_or_else(|| panic!("missing {name}"));
            let molecule = entry.graph().expect("parsed").to_molecule();
            crate::properties::properties(molecule.as_core()).formula
        };
        for (name, expected) in [
            ("Glycine (Gly)", "C2H5NO2"),
            ("Alanine (Ala)", "C3H7NO2"),
            ("Proline (Pro)", "C5H9NO2"),
            ("Phenylalanine (Phe)", "C9H11NO2"),
            ("Tryptophan (Trp)", "C11H12N2O2"),
            ("D-Glucose (open chain)", "C6H12O6"),
            ("D-Glucose (pyranose)", "C6H12O6"),
            ("D-Fructose (open chain)", "C6H12O6"),
            ("D-Ribose (furanose)", "C5H10O5"),
            ("2-Deoxy-D-ribose", "C5H10O4"),
            ("Sucrose", "C12H22O11"),
            ("Maltose", "C12H22O11"),
            ("Adenine (A)", "C5H5N5"),
            ("Guanine (G)", "C5H5N5O"),
            ("Cytosine (C)", "C4H5N3O"),
            ("Thymine (T)", "C5H6N2O2"),
            ("Uracil (U)", "C4H4N2O2"),
            ("Water", "H2O"),
            ("Ethanol", "C2H6O"),
            ("Acetone", "C3H6O"),
            ("Acetylacetone (acac)", "C5H8O2"),
            ("2,2'-Bipyridine (bpy)", "C10H8N2"),
            ("1,10-Phenanthroline (phen)", "C12H8N2"),
            ("EDTA", "C10H16N2O8"),
            ("Glycerol", "C3H8O3"),
            ("Stearic acid", "C18H36O2"),
            ("Oleic acid", "C18H34O2"),
            ("Palmitic acid", "C16H32O2"),
        ] {
            assert_eq!(formula(name), expected, "{name}");
        }
    }
}
