//! Unit tests.

use super::*;
use egui_workbench_chem_core::DocumentId;

fn max_valence(element: Element) -> f64 {
    match element.symbol() {
        "H" => 1.0,
        "C" => 4.0,
        "N" => 3.0,
        "O" => 2.0,
        "S" => 6.0,
        _ => f64::INFINITY,
    }
}

fn formula(superatom: &Superatom) -> String {
    let mut counts = std::collections::BTreeMap::<&str, usize>::new();
    for (element, _) in &superatom.atoms {
        *counts.entry(element.symbol()).or_default() += 1;
    }
    counts
        .iter()
        .map(|(symbol, count)| format!("{symbol}{count}"))
        .collect()
}

#[test]
fn library_entries_are_chemically_valid() {
    for superatom in library() {
        let name = superatom.abbreviation;
        let mut valence = vec![0.0; superatom.atoms.len()];
        let mut parent: Vec<usize> = (0..superatom.atoms.len()).collect();
        fn root(parent: &mut Vec<usize>, index: usize) -> usize {
            if parent[index] != index {
                let next = parent[index];
                parent[index] = root(parent, next);
            }
            parent[index]
        }
        for (a, b, order) in &superatom.bonds {
            assert!(
                *a < superatom.atoms.len() && *b < superatom.atoms.len(),
                "{name}: bond range"
            );
            assert_ne!(a, b, "{name}: self bond");
            valence[*a] += f64::from(order.order());
            valence[*b] += f64::from(order.order());
            let (ra, rb) = (root(&mut parent, *a), root(&mut parent, *b));
            parent[ra] = rb;
        }
        // One connected fragment.
        let first = root(&mut parent, 0);
        assert!(
            (0..superatom.atoms.len()).all(|index| root(&mut parent, index) == first),
            "{name}: disconnected"
        );
        // Explicit bonds never exceed the element's valence (implicit hydrogens fill
        // the rest, so only an upper bound is checkable). Aromatic counts as 1.5, so a
        // fused-ring carbon may reach 4.5.
        for (index, (element, _)) in superatom.atoms.iter().enumerate() {
            let limit = max_valence(*element) + 0.5;
            assert!(
                valence[index] <= limit,
                "{name}: atom {index} ({element}) has valence {}",
                valence[index]
            );
        }
        // No ring smaller than five atoms: every bond must lie on no triangle.
        for (a, b, _) in &superatom.bonds {
            let neighbours = |atom: usize| -> Vec<usize> {
                superatom
                    .bonds
                    .iter()
                    .filter_map(|(x, y, _)| {
                        if *x == atom {
                            Some(*y)
                        } else if *y == atom {
                            Some(*x)
                        } else {
                            None
                        }
                    })
                    .collect()
            };
            let shared = neighbours(*a)
                .into_iter()
                .filter(|candidate| neighbours(*b).contains(candidate))
                .count();
            assert_eq!(shared, 0, "{name}: three-membered ring");
        }
        // No two atoms closer than a bond could allow.
        for (i, left) in superatom.atoms.iter().enumerate() {
            for (j, right) in superatom.atoms.iter().enumerate().skip(i + 1) {
                let gap = (left.1[0] - right.1[0]).hypot(left.1[1] - right.1[1]);
                assert!(gap > 0.5, "{name}: atoms {i} and {j} overlap");
            }
        }
    }
}

#[test]
fn library_formulas_match_the_named_groups() {
    let heavy = |abbreviation: &str| formula(&find(abbreviation).expect("entry"));
    // Heavy atoms only; hydrogens are implicit except the explicit one of COOH.
    assert_eq!(heavy("Boc"), "C5O2");
    assert_eq!(heavy("OTs"), "C7O3S1");
    assert_eq!(heavy("bpy"), "C10N2");
    assert_eq!(heavy("phen"), "C12N2");
    assert_eq!(heavy("EDTA"), "C10N2O8");
    assert_eq!(heavy("Ph"), "C6");
}

#[test]
fn phenanthroline_nitrogens_face_each_other() {
    let phen = find("phen").expect("phen");
    let nitrogens: Vec<[f64; 2]> = phen
        .atoms
        .iter()
        .filter(|(element, _)| element.symbol() == "N")
        .map(|(_, position)| *position)
        .collect();
    assert_eq!(nitrogens.len(), 2);
    let gap = (nitrogens[0][0] - nitrogens[1][0]).hypot(nitrogens[0][1] - nitrogens[1][1]);
    assert!(gap < 3.0, "the nitrogens are {gap} apart");
    assert_eq!(phen.bonds.len(), 16, "three fused rings have 16 bonds");
}

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
