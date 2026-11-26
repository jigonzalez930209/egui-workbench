//! End-to-end 2D feature checks through the public API.

use egui_workbench_chem2d::annotations::{Annotation, ArrowKind, ConditionBox, ReactionArrow};
use egui_workbench_chem2d::{
    Chem2dEditor, ChemDocument, DeltaResult, DocumentDelta, DocumentId, Element, ExportOptions, RingTemplate,
    from_cdxml, from_molfile, from_rxn, from_sdf, parse_smiles, to_cdxml, to_png, to_rxn, to_sdf, to_v2000, to_v3000,
};

fn editor() -> Chem2dEditor {
    Chem2dEditor::new(ChemDocument::new(DocumentId::new(1), "test"))
}

#[test]
fn benzene_can_be_inserted_measured_and_exported() {
    let mut editor = editor();
    editor.insert_smiles("c1ccccc1", [0.0, 0.0]).expect("smiles");
    assert_eq!(editor.document.molecule.atom_count(), 6);
    assert_eq!(editor.document.molecule.aromatic_rings().len(), 1);
    assert_eq!(editor.properties().formula, "C6H6");
    assert!(editor.validate_valences().is_empty());
    editor.clean_layout().expect("clean");
    let svg = editor.export_svg(ExportOptions::default());
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    let png = to_png(&editor.document, ExportOptions::default(), 300).expect("png");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn templates_and_superatoms_are_reversible() {
    let mut editor = editor();
    editor.add_ring(RingTemplate::Benzene, [0.0, 0.0], 0.0).expect("ring");
    assert_eq!(editor.document.molecule.aromatic_rings().len(), 1);
    let pseudo = editor.document.molecule.add_atom(Element::UNKNOWN, [5.0, 0.0, 0.0]);
    editor.document.molecule.atom_mut(pseudo).expect("atom").label = Some("COOH".to_owned());
    editor.expand_superatom(pseudo).expect("expand");
    assert!(editor.document.molecule.atom_count() > 7);
    let group = editor.document.superatoms[0].clone();
    editor.contract_superatom(group).expect("contract");
    assert_eq!(editor.document.molecule.atom_count(), 7);
}

#[test]
fn molfile_sdf_rxn_and_cdxml_round_trip() {
    let mut editor = editor();
    editor.insert_smiles("CCO", [0.0, 0.0]).expect("smiles");
    let v2000 = to_v2000(&editor.document.molecule, "ethanol");
    assert_eq!(from_molfile(&v2000).expect("v2000").atom_count(), 3);
    let v3000 = to_v3000(&editor.document.molecule, "ethanol");
    assert_eq!(from_molfile(&v3000).expect("v3000").atom_count(), 3);
    let sdf = to_sdf(&[editor.document.clone(), editor.document.clone()]);
    assert_eq!(from_sdf(&sdf).len(), 2);
    let rxn = to_rxn(&[editor.document.molecule.clone()], &[editor.document.molecule.clone()]);
    let (reactants, products) = from_rxn(&rxn).expect("rxn");
    assert_eq!((reactants.len(), products.len()), (1, 1));
    let cdxml = to_cdxml(&editor.document.molecule);
    assert_eq!(from_cdxml(&cdxml).expect("cdxml").atom_count(), 3);
}

#[test]
fn annotations_and_deltas_are_serializable() {
    let mut editor = editor();
    editor
        .add_reaction_step(ReactionArrow::new(ArrowKind::Reaction, [0.0, 0.0], [3.0, 0.0]))
        .expect("arrow");
    editor
        .add_annotation(Annotation::Conditions(ConditionBox::line([0.0, 1.0], "H_2O")))
        .expect("conditions");
    assert_eq!(editor.document.annotations.len(), 2);
    let json = serde_json::to_string(&editor.document).expect("serialize");
    let restored: ChemDocument = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.annotations.len(), 2);

    let mut document = ChemDocument::new(DocumentId::new(2), "delta");
    let DeltaResult::Atom(atom) = document
        .apply_delta(&DocumentDelta::AddAtom {
            element: Element::from_symbol("C").expect("C"),
            position: [1.0, 2.0],
            charge: 0,
            label: None,
        })
        .expect("delta")
    else {
        panic!("atom expected");
    };
    assert_eq!(document.molecule.atom(atom).expect("atom").position[1], 2.0);
}

#[test]
fn smiles_parser_is_available_publicly() {
    let graph = parse_smiles("CC(=O)O").expect("parse");
    assert_eq!(graph.atoms.len(), 4);
    assert_eq!(graph.bonds.len(), 3);
}
