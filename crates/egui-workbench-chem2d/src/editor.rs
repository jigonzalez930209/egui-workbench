//! The ready-to-use 2D editor widget.

use egui::{Rect, Response, Ui, pos2, vec2};
use egui_workbench_chem_core::{AtomId, BondOrder, ChemSelection, Element, SelectionMode};

use crate::camera::Camera2D;
use crate::document::ChemDocument;
use crate::history::{Command, History};
use crate::interact::Hit;
use crate::templates::RingTemplate;
use crate::{Annotation, SuperatomGroup};

mod view;
use crate::Chem2dError;

/// The active editing tool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditorTool {
    /// Click to select; drag empty space for a rectangle; drag an atom to move it.
    #[default]
    Select,
    /// Drag to pan.
    Pan,
    /// Click to add an atom of this element.
    AddAtom(Element),
    /// Click two atoms to bond them.
    AddBond,
    /// Click an atom or bond to delete it.
    Erase,
    /// Drag a free-form lasso.
    Lasso,
    /// Click to stamp a ring template.
    AddRing(RingTemplate),
    /// Click to start a carbon chain of this length.
    AddChain {
        /// Number of carbons.
        count: usize,
    },
}

/// What the editor did this frame.
pub struct Chem2dResponse {
    /// The viewport response.
    pub response: Response,
    /// The document changed (including undo/redo).
    pub changed: bool,
    /// The selection changed.
    pub selection_changed: bool,
    /// The atom or bond under the pointer.
    pub hovered: Option<Hit>,
}

enum Drag {
    Pan,
    Move {
        atoms: Vec<(AtomId, [f64; 2])>,
        start: [f64; 2],
        delta: [f64; 2],
    },
    Rect {
        start: [f64; 2],
        current: [f64; 2],
    },
    Lasso {
        points: Vec<[f64; 2]>,
    },
}

/// A self-contained 2D editor: document, camera, selection, history, and tools.
pub struct Chem2dEditor {
    /// The document being edited.
    pub document: ChemDocument,
    /// The camera.
    pub camera: Camera2D,
    /// The shared selection.
    pub selection: ChemSelection,
    /// Undo/redo history.
    pub history: History,
    /// The active tool.
    pub tool: EditorTool,
    /// Whether the 1-2-5 grid draws.
    pub grid: bool,
    /// Bond order used by the bond tool.
    pub bond_order: BondOrder,
    drag: Option<Drag>,
    pending_bond: Option<AtomId>,
}

impl Chem2dEditor {
    /// An editor for `document`, fitted to its atoms.
    pub fn new(document: ChemDocument) -> Self {
        let mut editor = Self {
            document,
            camera: Camera2D::default(),
            selection: ChemSelection::new(),
            history: History::new(),
            tool: EditorTool::Select,
            grid: true,
            bond_order: BondOrder::Single,
            drag: None,
            pending_bond: None,
        };
        editor.fit();
        editor
    }

    /// Fit the camera to the document.
    pub fn fit(&mut self) {
        if let Some(bounds) = self.document.bounds() {
            let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 420.0));
            self.camera.fit(bounds, viewport, 40.0);
        } else {
            self.camera = Camera2D::default();
        }
    }

    /// Add an atom through the history.
    ///
    /// # Errors
    /// When the document rejects the atom.
    pub fn add_atom(&mut self, element: Element, position: [f64; 2]) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::AddAtom { element, position },
        )
    }

    /// Add a bond through the history.
    ///
    /// # Errors
    /// When either atom is unknown or the bond already exists.
    pub fn add_bond(&mut self, a: AtomId, b: AtomId, order: BondOrder) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::AddBond { a, b, order },
        )
    }

    /// Delete the selection through the history.
    ///
    /// # Errors
    /// When the document rejects the change.
    pub fn delete_selection(&mut self) -> Result<(), Chem2dError> {
        self.history
            .execute(&mut self.document, &mut self.selection, &Command::DeleteSelection)
    }

    /// Undo one command.
    pub fn undo(&mut self) -> bool {
        self.history.undo(&mut self.document, &mut self.selection)
    }

    /// Redo one command.
    pub fn redo(&mut self) -> bool {
        self.history.redo(&mut self.document, &mut self.selection)
    }

    /// Stamp a ring template through the history.
    ///
    /// # Errors
    /// When the document rejects the template.
    pub fn add_ring(&mut self, template: RingTemplate, center: [f64; 2], rotation: f64) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::AddRing {
                template,
                center,
                rotation,
            },
        )
    }

    /// Add a carbon chain through the history.
    ///
    /// # Errors
    /// When the document rejects the chain.
    pub fn add_chain(&mut self, start: [f64; 2], count: usize, angle: f64) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::AddChain { start, count, angle },
        )
    }

    /// Rotate the selected atoms around the selection centroid.
    ///
    /// # Errors
    /// When the selection is empty.
    pub fn rotate_selection(&mut self, radians: f64) -> Result<(), Chem2dError> {
        let atoms = self.selection.atoms().iter().copied().collect::<Vec<_>>();
        if atoms.is_empty() {
            return Err(Chem2dError::Chemistry("the selection is empty".to_owned()));
        }
        let center = centroid(&self.document, &atoms)?;
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::RotateFragment { atoms, center, radians },
        )
    }

    /// Mirror the selected atoms across their centroid.
    ///
    /// # Errors
    /// When the selection is empty.
    pub fn mirror_selection(&mut self, vertical: bool) -> Result<(), Chem2dError> {
        let atoms = self.selection.atoms().iter().copied().collect::<Vec<_>>();
        if atoms.is_empty() {
            return Err(Chem2dError::Chemistry("the selection is empty".to_owned()));
        }
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::MirrorFragment { atoms, vertical },
        )
    }

    /// Expand a labeled pseudo atom into its superatom fragment.
    ///
    /// # Errors
    /// When the atom has no known superatom label.
    pub fn expand_superatom(&mut self, pseudo: AtomId) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::ExpandSuperatom { pseudo },
        )
    }

    /// Collapse an expanded superatom.
    ///
    /// # Errors
    /// When the group is not expanded.
    pub fn contract_superatom(&mut self, group: SuperatomGroup) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::ContractSuperatom { group },
        )
    }

    /// Add a reaction arrow, conditions box, or polymer bracket.
    ///
    /// # Errors
    /// When the history cannot record the annotation.
    pub fn add_annotation(&mut self, annotation: Annotation) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::AddAnnotation(annotation),
        )
    }

    /// Insert a SMILES graph through the history.
    ///
    /// # Errors
    /// When the SMILES is invalid.
    pub fn insert_smiles(&mut self, smiles: &str, center: [f64; 2]) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::InsertSmiles {
                text: smiles.to_owned(),
                center,
            },
        )
    }

    /// Normalize bond lengths and overlap through the history.
    ///
    /// # Errors
    /// When the history cannot record the change.
    pub fn clean_layout(&mut self) -> Result<(), Chem2dError> {
        self.history
            .execute(&mut self.document, &mut self.selection, &Command::CleanLayout)
    }

    /// The calculated properties of the current document.
    pub fn properties(&self) -> crate::properties::MolecularProperties {
        crate::properties::properties(&self.document.molecule)
    }

    /// Validate the common valences and return the over-bonded atoms.
    pub fn validate_valences(&self) -> Vec<(AtomId, i16)> {
        crate::properties::valence_issues(&self.document.molecule)
    }

    /// Add a reaction arrow as a reaction step.
    ///
    /// # Errors
    /// When the history cannot record the annotation.
    pub fn add_reaction_step(&mut self, arrow: crate::annotations::ReactionArrow) -> Result<(), Chem2dError> {
        self.add_annotation(crate::annotations::Annotation::Arrow(arrow))
    }

    /// Export the current document as SVG.
    pub fn export_svg(&self, options: crate::export::ExportOptions) -> String {
        crate::export::to_svg(&self.document, options)
    }
}

fn selection_mode(ui: &Ui) -> SelectionMode {
    let (shift, command) = ui.input(|input| (input.modifiers.shift, input.modifiers.command));
    if command {
        SelectionMode::Toggle
    } else if shift {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    }
}

fn centroid(document: &ChemDocument, atoms: &[AtomId]) -> Result<[f64; 2], Chem2dError> {
    let mut sum = [0.0; 2];
    let mut count = 0.0;
    for id in atoms {
        let atom = document.molecule.atom(*id).ok_or(Chem2dError::UnknownAtom(*id))?;
        sum[0] += atom.position[0];
        sum[1] += atom.position[1];
        count += 1.0;
    }
    if count == 0.0 {
        return Err(Chem2dError::Chemistry("no atoms".to_owned()));
    }
    Ok([sum[0] / count, sum[1] / count])
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::{DocumentId, Element};

    fn editor() -> Chem2dEditor {
        Chem2dEditor::new(ChemDocument::new(DocumentId::new(1), "test"))
    }

    #[test]
    fn editor_commands_go_through_history() {
        let mut editor = editor();
        editor
            .add_atom(Element::from_symbol("C").expect("carbon"), [0.0, 0.0])
            .expect("add");
        editor
            .add_atom(Element::from_symbol("O").expect("oxygen"), [1.2, 0.0])
            .expect("add");
        let ids = editor
            .document
            .molecule
            .atoms
            .iter()
            .map(|atom| atom.id)
            .collect::<Vec<_>>();
        editor.add_bond(ids[0], ids[1], BondOrder::Single).expect("bond");
        assert_eq!(editor.document.molecule.bond_count(), 1);
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.bond_count(), 0);
        assert!(editor.redo());
        assert_eq!(editor.document.molecule.bond_count(), 1);
    }

    #[test]
    fn the_editor_paints_without_a_panic() {
        let mut editor = editor();
        editor
            .add_atom(Element::from_symbol("C").expect("carbon"), [0.0, 0.0])
            .expect("add");
        let context = egui::Context::default();
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            editor.show(ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn rings_chains_rotation_and_mirror_go_through_history() {
        let mut editor = editor();
        editor.add_ring(RingTemplate::Benzene, [0.0, 0.0], 0.0).expect("ring");
        assert_eq!(editor.document.molecule.atom_count(), 6);
        assert_eq!(editor.document.molecule.aromatic_rings().len(), 1);
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.atom_count(), 0);

        editor.add_chain([0.0, 0.0], 4, 0.0).expect("chain");
        assert_eq!(editor.document.molecule.atom_count(), 4);
        let before = editor.document.molecule.atoms[0].position;
        editor.rotate_selection(0.5).expect("rotate");
        assert_ne!(editor.document.molecule.atoms[0].position, before);
        editor.mirror_selection(true).expect("mirror");
        assert!(editor.undo());
    }
}
