use super::*;

#[test]
fn ethanol_parses_into_three_atoms() {
    let graph = parse_smiles("CCO").expect("parse");
    assert_eq!(graph.atoms.len(), 3);
    assert_eq!(graph.bonds.len(), 2);
}

#[test]
fn benzene_parses_as_one_aromatic_ring() {
    let graph = parse_smiles("c1ccccc1").expect("parse");
    let molecule = graph.to_molecule();
    assert_eq!(molecule.rings().len(), 1);
    assert_eq!(molecule.aromatic_rings().len(), 1);
}

#[test]
fn charges_and_hydrogens_are_read() {
    let graph = parse_smiles("[NH4+]").expect("parse");
    assert_eq!(graph.charges[0], 1);
    assert_eq!(graph.hydrogens[0], 4);
}

#[test]
fn invalid_smiles_is_rejected() {
    assert!(parse_smiles("C1CC").is_err());
    assert!(parse_smiles("C)").is_err());
    assert!(parse_smiles("").is_err());
}

#[test]
fn insertion_offsets_the_graph() {
    let graph = parse_smiles("CCO").expect("parse");
    let mut document = ChemDocument::new(egui_workbench_chem_core::DocumentId::new(1), "test");
    let ids = graph.insert(&mut document, [10.0, 5.0]);
    assert_eq!(ids.len(), 3);
    assert!(document.molecule.atom(ids[0]).expect("atom").position[0] >= 8.0);
}

#[test]
fn written_smiles_parses_back() {
    for source in ["CCO", "c1ccccc1", "C1CCCCC1", "CC(=O)O"] {
        let graph = parse_smiles(source).expect("parse");
        let molecule = graph.to_molecule();
        let written = to_smiles(&molecule);
        let reparsed = parse_smiles(&written).unwrap_or_else(|error| panic!("{source} -> {written}: {error}"));
        assert_eq!(reparsed.atoms.len(), graph.atoms.len(), "{source} -> {written}");
        assert_eq!(reparsed.bonds.len(), graph.bonds.len(), "{source} -> {written}");
    }
}
