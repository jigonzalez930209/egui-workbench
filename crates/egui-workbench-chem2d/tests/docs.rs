//! The snippets of the chemistry guide, compiled and run.

use egui_workbench_chem_core::{ChemSelection, DocumentId};
use egui_workbench_chem2d::properties::valence_issues;
use egui_workbench_chem2d::{
    ChemDocument, Command, ExportOptions, History, parse_smiles, properties, to_svg, to_v2000,
};

#[test]
fn smiles_become_a_document_with_properties() {
    let graph = parse_smiles("CC(=O)O").expect("acetic acid");
    let molecule = graph.to_molecule();
    let found = properties(molecule.as_core());
    assert_eq!(found.formula, "C2H4O2");
    assert!((found.molecular_weight - 60.05).abs() < 0.05);
    assert!(valence_issues(molecule.as_core()).is_empty());
}

#[test]
fn edits_go_through_a_history() {
    let mut document = ChemDocument::new(DocumentId::new(1), "untitled");
    let mut selection = ChemSelection::new();
    let mut history = History::new();
    history
        .execute(
            &mut document,
            &mut selection,
            &Command::InsertSmiles {
                text: "c1ccccc1".to_owned(),
                center: [0.0, 0.0],
            },
        )
        .expect("benzene");
    assert_eq!(document.molecule.atom_count(), 6);
    assert!(history.undo(&mut document, &mut selection));
    assert_eq!(document.molecule.atom_count(), 0);
}

#[test]
fn a_document_exports_to_svg_and_molfile() {
    let graph = parse_smiles("CCO").expect("ethanol");
    let mut document = ChemDocument::new(DocumentId::new(1), "ethanol");
    document.molecule = graph.to_molecule();
    let svg = to_svg(&document, ExportOptions::default());
    assert!(svg.starts_with("<?xml"));
    assert!(to_v2000(&document.molecule, "ethanol").contains("V2000"));
}
