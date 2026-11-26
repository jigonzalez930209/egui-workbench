//! The snippets of the chemistry guide, compiled and run.

use egui_workbench_chem_core::{
    BondOrder, ChemDocument, ChemEvent, ChemSelection, DocumentId, Element, EventQueue, Job, JobContext, JobError,
    JobQueue, SelectionMode,
};

#[test]
fn a_document_holds_atoms_and_bonds() {
    let mut document = ChemDocument::new(DocumentId::new(1), "water");
    let oxygen = document
        .molecule
        .add_atom(Element::from_symbol("O").expect("oxygen"), [0.0, 0.0, 0.0]);
    let hydrogen = document
        .molecule
        .add_atom(Element::from_symbol("H").expect("hydrogen"), [0.96, 0.0, 0.0]);
    document
        .molecule
        .add_bond(oxygen, hydrogen, BondOrder::Single)
        .expect("bond");
    assert_eq!(document.molecule.atoms.len(), 2);
    document.molecule.validate().expect("a consistent graph");
}

#[test]
fn a_selection_and_an_event_queue_connect_two_panels() {
    let mut document = ChemDocument::new(DocumentId::new(1), "water");
    let oxygen = document
        .molecule
        .add_atom(Element::from_symbol("O").expect("oxygen"), [0.0, 0.0, 0.0]);
    let mut selection = ChemSelection::new();
    selection.select_atom(oxygen, SelectionMode::Replace);
    let mut events = EventQueue::default();
    events.push(ChemEvent::SelectionChanged {
        document: document.id,
        selection: selection.clone(),
    });
    assert_eq!(events.drain().count(), 1);
}

struct Count(u32);

impl Job for Count {
    type Output = u32;

    fn run(self, context: &JobContext) -> Result<u32, JobError> {
        for step in 0..self.0 {
            context.check_cancelled()?;
            context.set_progress(step as usize, self.0 as usize, "counting");
        }
        Ok(self.0)
    }
}

#[test]
fn a_background_job_reports_progress_and_a_result() {
    let queue = JobQueue::new();
    let handle = queue.spawn(Count(3));
    assert_eq!(handle.wait().expect("result"), 3);
}
