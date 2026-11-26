//! Regression tests for hostile or unusual input to the readers.

use egui_workbench_chem2d::{BondOrder, from_cdxml, from_molfile, from_v3000, parse_smiles, to_v3000};

const COUNTS: &str = "t\n  x\n\n  1  0  0  0  0  0  0  0  0  0999 V2000\n";

#[test]
fn a_multibyte_character_in_a_column_is_an_error_not_a_panic() {
    let text = format!("{COUNTS}   0.0000    0.0000é   0.0000 C   0  0\nM  END\n");
    assert!(from_molfile(&text).is_err());
}

#[test]
fn a_bare_chg_line_is_ignored() {
    let text = format!("{COUNTS}    0.0000    0.0000    0.0000 C   0  0\nM  CHG\nM  CHG  1   1 999\nM  END\n");
    let molecule = from_molfile(&text).expect("molfile");
    assert_eq!(molecule.atoms[0].charge, 0);
}

#[test]
fn a_truncated_atom_block_is_rejected() {
    let text = "t\n  x\n\n  3  0  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 C   0  0\nM  END\n";
    assert!(from_molfile(text).is_err());
}

#[test]
fn bond_atom_zero_does_not_bond_the_first_atom() {
    let text = "t\n  x\n\n  2  1  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 C   0  0\n    1.0000    0.0000    0.0000 C   0  0\n  0  2  1  0\nM  END\n";
    assert!(from_molfile(text).expect("molfile").bonds.is_empty());
}

#[test]
fn v3000_charges_round_trip_through_the_chg_property() {
    let molecule = from_molfile(&format!(
        "{COUNTS}    0.0000    0.0000    0.0000 N   0  0\nM  CHG  1   1   1\nM  END\n"
    ))
    .expect("molfile");
    assert_eq!(molecule.atoms[0].charge, 1);
    let text = to_v3000(&molecule, "n");
    assert!(text.contains("CHG=1"), "{text}");
    assert_eq!(from_v3000(&text).expect("v3000").atoms[0].charge, 1);
}

#[test]
fn cdxml_nodes_without_an_element_are_carbon() {
    let text = r#"<CDXML><page><fragment><n id="1" p="0 0"/><n id="2" p="10 0" Element="8"/><b id="3" B="1" E="2" Order="2"/></fragment></page></CDXML>"#;
    let molecule = from_cdxml(text).expect("cdxml");
    assert_eq!(molecule.atoms[0].element.atomic_number(), 6);
    assert_eq!(molecule.bonds[0].order, BondOrder::Double);
}

#[test]
fn smiles_hydrogen_counts_are_bounded() {
    assert!(parse_smiles("[CH99999999]").is_err());
    assert!(parse_smiles("[CH3]").is_ok());
}

#[test]
fn repeated_signs_set_the_charge() {
    let graph = parse_smiles("[Fe++]").expect("smiles");
    assert_eq!(graph.charges, vec![2]);
    let graph = parse_smiles("[O--]").expect("smiles");
    assert_eq!(graph.charges, vec![-2]);
}

mod chemistry {
    use egui_workbench_chem_core::{ChemSelection, DocumentId};
    use egui_workbench_chem2d::{
        ChemDocument, Command, DocumentDelta, Element, History, RingTemplate, add_chain, add_ring, parse_smiles,
        properties,
    };

    fn document() -> ChemDocument {
        ChemDocument::new(DocumentId::new(1), "t")
    }

    #[test]
    fn a_chain_has_uniform_bonds_and_a_120_degree_zigzag() {
        let mut document = document();
        let ids = add_chain(&mut document.molecule, [0.0, 0.0], 6, 0.0);
        let points: Vec<[f64; 2]> = ids
            .iter()
            .map(|id| {
                let position = document.molecule.atom(*id).expect("atom").position;
                [position[0], position[1]]
            })
            .collect();
        for pair in points.windows(2) {
            let length = (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]);
            assert!((length - 1.5).abs() < 1.0e-9, "bond length {length}");
        }
        for triple in points.windows(3) {
            let (a, b) = (
                [triple[0][0] - triple[1][0], triple[0][1] - triple[1][1]],
                [triple[2][0] - triple[1][0], triple[2][1] - triple[1][1]],
            );
            let angle = ((a[0] * b[0] + a[1] * b[1]) / (a[0].hypot(a[1]) * b[0].hypot(b[1])))
                .acos()
                .to_degrees();
            assert!((angle - 120.0).abs() < 1.0e-6, "bond angle {angle}");
        }
    }

    #[test]
    fn a_huge_chain_request_is_capped() {
        let mut document = document();
        assert_eq!(
            add_chain(&mut document.molecule, [0.0, 0.0], usize::MAX, 0.0).len(),
            1_000
        );
    }

    #[test]
    fn pyrrole_has_a_nitrogen_hydrogen_and_is_aromatic() {
        let mut document = document();
        add_ring(&mut document.molecule, RingTemplate::Pyrrole, [0.0, 0.0], 0.0);
        assert_eq!(properties(document.molecule.as_core()).formula, "C4H5N");
        assert_eq!(document.molecule.aromatic_rings().len(), 1);
    }

    #[test]
    fn cyclopentadiene_is_not_aromatic() {
        let mut document = document();
        add_ring(&mut document.molecule, RingTemplate::Cyclopentadiene, [0.0, 0.0], 0.0);
        assert!(document.molecule.aromatic_rings().is_empty());
        assert_eq!(properties(document.molecule.as_core()).formula, "C5H6");
    }

    #[test]
    fn a_failed_command_leaves_nothing_half_applied() {
        let mut document = document();
        let ids = add_chain(&mut document.molecule, [0.0, 0.0], 3, 0.0);
        let before: Vec<[f64; 3]> = ids
            .iter()
            .map(|id| document.molecule.atom(*id).expect("atom").position)
            .collect();
        let stale = {
            let mut other = egui_workbench_chem2d::Molecule2D::new();
            other.add_atom(Element::UNKNOWN, [0.0; 3])
        };
        let mut atoms = ids.clone();
        // The unknown id comes last, after the earlier atoms were already rotated.
        atoms.push(egui_workbench_chem_core::AtomId::new(stale.slot() + 50, 9));
        let mut history = History::new();
        let mut selection = ChemSelection::new();
        let result = history.execute(
            &mut document,
            &mut selection,
            &Command::RotateFragment {
                atoms,
                center: [0.0, 0.0],
                radians: 1.0,
            },
        );
        assert!(result.is_err());
        let after: Vec<[f64; 3]> = ids
            .iter()
            .map(|id| document.molecule.atom(*id).expect("atom").position)
            .collect();
        assert_eq!(before, after);
        assert!(!history.can_undo());
    }

    #[test]
    fn deltas_reject_absurd_positions() {
        let mut document = document();
        let add = |position: [f64; 2]| DocumentDelta::AddAtom {
            element: Element::from_symbol("C").expect("C"),
            position,
            charge: 0,
            label: None,
        };
        assert!(document.apply_delta(&add([1.0e300, 0.0])).is_err());
        assert!(document.apply_delta(&add([0.0, 0.0])).is_ok());
    }

    #[test]
    fn relaxing_a_large_smiles_chain_finishes_quickly() {
        // 400 carbons used to take a cubic number of id lookups.
        let smiles = "C".repeat(400);
        let started = std::time::Instant::now();
        let molecule = parse_smiles(&smiles).expect("smiles").to_molecule();
        assert_eq!(molecule.atom_count(), 400);
        assert!(
            started.elapsed() < std::time::Duration::from_secs(20),
            "{:?}",
            started.elapsed()
        );
    }
}
