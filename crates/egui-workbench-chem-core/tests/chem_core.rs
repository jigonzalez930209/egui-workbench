//! The public contract used by the 2D and 3D crates.

use egui_workbench_chem_core::model::BondOrder;
use egui_workbench_chem_core::{
    AtomId, ChemDocument, ChemEvent, ChemSelection, DocumentId, Element, EventQueue, SelectionMode,
};

#[test]
fn a_document_round_trips_through_json_with_ids_intact() {
    let mut document = ChemDocument::new(DocumentId::new(7), "benzene");
    let carbon = document
        .molecule
        .add_atom(Element::from_symbol("C").expect("carbon"), [0.0; 3]);
    let hydrogen = document
        .molecule
        .add_atom(Element::from_symbol("H").expect("hydrogen"), [1.09, 0.0, 0.0]);
    document
        .molecule
        .add_bond(carbon, hydrogen, BondOrder::Single)
        .expect("bond");

    let json = serde_json::to_string(&document).expect("serialize");
    let restored: ChemDocument = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.id, DocumentId::new(7));
    assert_eq!(restored.name, "benzene");
    assert_eq!(restored.molecule.atoms.len(), 2);
    assert_eq!(restored.molecule.atoms[0].id, carbon);
    assert_eq!(restored.molecule.atoms[1].id, hydrogen);
    assert_eq!(restored.molecule.bonds.len(), 1);
    restored.molecule.validate().expect("valid");
}

#[test]
fn a_selection_and_event_serialize_for_the_remote_bus() {
    let document = DocumentId::new(1);
    let mut selection = ChemSelection::new();
    selection.select_atom(AtomId::new(0, 1), SelectionMode::Replace);
    let event = ChemEvent::SelectionChanged {
        document,
        selection: selection.clone(),
    };
    let json = serde_json::to_string(&event).expect("serialize");
    let restored: ChemEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored, event);
}

#[test]
fn an_event_queue_drains_in_order() {
    let document = DocumentId::new(1);
    let mut queue = EventQueue::default();
    queue.push(ChemEvent::Request3DConformers { document });
    queue.push(ChemEvent::GeometryUpdated {
        document,
        atom: Some(AtomId::new(0, 1)),
    });
    assert_eq!(queue.len(), 2);
    let events = queue.drain().collect::<Vec<_>>();
    assert!(matches!(events[0], ChemEvent::Request3DConformers { .. }));
    assert!(matches!(events[1], ChemEvent::GeometryUpdated { .. }));
}
