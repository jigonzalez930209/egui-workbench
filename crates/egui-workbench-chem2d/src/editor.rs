//! The ready-to-use 2D editor widget.

use egui::{Rect, Response, Ui, pos2, vec2};
use egui_workbench_chem_core::{AtomId, BondOrder, BondStereo, ChemSelection, Element, SelectionMode};

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
    /// Click to add an atom of this element; click an existing atom to change it.
    AddAtom(Element),
    /// Click empty space for the first bond, click an atom to sprout the next bond, or
    /// drag to choose the direction explicitly.
    ///
    /// Click empty space to create two atoms. A click on an atom grows the chain at 120°;
    /// dragging from an atom ends at a new atom (or connects to an existing one), and a
    /// drag from empty space creates both atoms. The endpoint uses
    /// [`Chem2dEditor::draw_element`], and the bond uses [`Chem2dEditor::bond_order`] and
    /// [`Chem2dEditor::bond_stereo`].
    AddBond,
    /// Click an atom to add `delta` to its formal charge.
    Charge {
        /// Charge change, usually `1` or `-1`.
        delta: i8,
    },
    /// Click an atom to attach [`Chem2dEditor::fragment_smiles`] there.
    AttachFragment,
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
    Rotate {
        atoms: Vec<(AtomId, [f64; 2])>,
        center: [f64; 2],
        last_angle: f64,
        angle: f64,
    },
    Rect {
        start: [f64; 2],
        current: [f64; 2],
    },
    Lasso {
        points: Vec<[f64; 2]>,
    },
    DrawBond {
        from: Option<AtomId>,
        from_position: [f64; 2],
        to: Option<AtomId>,
        current: [f64; 2],
        moved: bool,
    },
}

#[derive(Clone, Debug)]
struct AtomEditorState {
    atom: AtomId,
    symbol: String,
    charge: i32,
    label: String,
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
    /// Bond stereochemistry used by the bond tool.
    pub bond_stereo: BondStereo,
    /// Element created when a bond drag ends in empty space.
    pub draw_element: Element,
    /// SMILES attached when [`EditorTool::AttachFragment`] clicks an atom.
    pub fragment_smiles: String,
    drag: Option<Drag>,
    atom_editor: Option<AtomEditorState>,
    element_buffer: String,
    element_buffer_time: f64,
    viewport: Rect,
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
            bond_stereo: BondStereo::None,
            draw_element: Element::from_symbol("C").expect("carbon"),
            fragment_smiles: String::new(),
            drag: None,
            atom_editor: None,
            element_buffer: String::new(),
            element_buffer_time: 0.0,
            viewport: Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 420.0)),
        };
        editor.fit();
        editor
    }

    /// Fit the camera to the document.
    pub fn fit(&mut self) {
        if let Some(bounds) = self.document.bounds() {
            self.camera.fit(bounds, self.viewport, 40.0);
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

    /// Apply an arbitrary command through the history.
    ///
    /// This is the escape hatch for an application toolbar that edits atoms, bonds, or
    /// superatoms without going through a tool.
    ///
    /// # Errors
    /// When the document rejects the change; the document is not changed in that case.
    pub fn execute(&mut self, command: &Command) -> Result<(), Chem2dError> {
        self.history.execute(&mut self.document, &mut self.selection, command)
    }

    /// Remove every atom, bond, superatom, and annotation, as the trash tool does.
    ///
    /// # Errors
    /// When the history cannot record the change.
    pub fn clear_document(&mut self) -> Result<(), Chem2dError> {
        self.execute(&Command::ClearDocument)
    }

    /// Change every selected atom to `element` and clear their custom labels.
    ///
    /// Returns whether anything changed. With an empty selection this does nothing, so a
    /// caller can fall back to changing the element tool.
    ///
    /// # Errors
    /// When the document rejects the change.
    pub fn set_selected_element(&mut self, element: Element) -> Result<bool, Chem2dError> {
        let atoms = self
            .selection
            .atoms()
            .iter()
            .copied()
            .filter(|id| {
                self.document
                    .molecule
                    .atom(*id)
                    .is_some_and(|atom| atom.element != element || atom.label.is_some())
            })
            .collect::<Vec<_>>();
        if atoms.is_empty() {
            return Ok(false);
        }
        self.execute(&Command::SetAtomElements { atoms, element })?;
        Ok(true)
    }

    /// Add `delta` to the formal charge of every selected atom.
    ///
    /// Returns whether anything changed.
    ///
    /// # Errors
    /// When the document rejects the change.
    pub fn adjust_selected_charge(&mut self, delta: i8) -> Result<bool, Chem2dError> {
        if delta == 0 {
            return Ok(false);
        }
        let atoms = self
            .selection
            .atoms()
            .iter()
            .copied()
            .filter(|id| {
                self.document
                    .molecule
                    .atom(*id)
                    .is_some_and(|atom| atom.charge.saturating_add(delta) != atom.charge)
            })
            .collect::<Vec<_>>();
        if atoms.is_empty() {
            return Ok(false);
        }
        self.execute(&Command::AdjustAtomCharges { atoms, delta })?;
        Ok(true)
    }

    /// Set or clear the custom label of every selected atom.
    ///
    /// Returns whether anything changed.
    ///
    /// # Errors
    /// When the document rejects the change.
    pub fn set_selected_label(&mut self, label: Option<&str>) -> Result<bool, Chem2dError> {
        let label = normalized_label(label);
        let atoms = self
            .selection
            .atoms()
            .iter()
            .copied()
            .filter(|id| {
                self.document
                    .molecule
                    .atom(*id)
                    .is_some_and(|atom| normalized_label(atom.label.as_deref()) != label)
            })
            .collect::<Vec<_>>();
        if atoms.is_empty() {
            return Ok(false);
        }
        self.execute(&Command::SetAtomLabels { atoms, label })?;
        Ok(true)
    }

    /// Change the order of every selected bond.
    ///
    /// Returns whether anything changed.
    ///
    /// # Errors
    /// When the document rejects the change.
    pub fn set_selected_bond_order(&mut self, order: BondOrder) -> Result<bool, Chem2dError> {
        let bonds = self
            .selection
            .bonds()
            .iter()
            .copied()
            .filter(|id| {
                self.document
                    .molecule
                    .bonds
                    .iter()
                    .any(|bond| bond.id == *id && bond.order != order)
            })
            .collect::<Vec<_>>();
        if bonds.is_empty() {
            return Ok(false);
        }
        self.execute(&Command::SetBondOrders { bonds, order })?;
        Ok(true)
    }

    /// Change the stereochemistry of every selected bond.
    ///
    /// Returns whether anything changed.
    ///
    /// # Errors
    /// When the document rejects the change.
    pub fn set_selected_bond_stereo(&mut self, stereo: BondStereo) -> Result<bool, Chem2dError> {
        let bonds = self
            .selection
            .bonds()
            .iter()
            .copied()
            .filter(|id| {
                self.document
                    .molecule
                    .bonds
                    .iter()
                    .any(|bond| bond.id == *id && bond.stereo != stereo)
            })
            .collect::<Vec<_>>();
        if bonds.is_empty() {
            return Ok(false);
        }
        self.execute(&Command::SetBondStereos { bonds, stereo })?;
        Ok(true)
    }

    /// Move every selected atom by `delta` Angstrom.
    ///
    /// Returns whether anything changed.
    ///
    /// # Errors
    /// When the document rejects the change.
    pub fn nudge_selection(&mut self, delta: [f64; 2]) -> Result<bool, Chem2dError> {
        if delta == [0.0, 0.0] {
            return Ok(false);
        }
        let atoms = self
            .selection
            .atoms()
            .iter()
            .copied()
            .filter(|id| self.document.molecule.atom(*id).is_some())
            .collect::<Vec<_>>();
        if atoms.is_empty() {
            return Ok(false);
        }
        self.execute(&Command::TranslateFragment { atoms, delta })?;
        Ok(true)
    }

    /// Select every atom and bond. Returns whether the selection changed.
    pub fn select_all(&mut self) -> bool {
        let before = self.selection.clone();
        self.selection.clear();
        for atom in &self.document.molecule.atoms {
            self.selection.select_atom(atom.id, SelectionMode::Add);
        }
        for bond in &self.document.molecule.bonds {
            self.selection.select_bond(bond.id, SelectionMode::Add);
        }
        self.selection != before
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

    /// Attach a parsed SMILES graph to `anchor` with one new bond.
    ///
    /// The fragment is laid out outward from the anchor, so a complex ring or branch can
    /// be grown from any atom with a single click in the
    /// [`EditorTool::AttachFragment`] tool.
    ///
    /// # Errors
    /// When the SMILES is invalid or the anchor is unknown.
    pub fn attach_smiles(&mut self, smiles: &str, anchor: AtomId) -> Result<(), Chem2dError> {
        self.execute(&Command::AttachSmiles {
            text: smiles.to_owned(),
            anchor,
        })
    }

    /// Attach a ring template to `anchor` with one new bond.
    ///
    /// # Errors
    /// When the anchor is unknown or the ring cannot be attached.
    pub fn attach_ring(&mut self, template: RingTemplate, anchor: AtomId) -> Result<(), Chem2dError> {
        self.execute(&Command::AttachRing { template, atom: anchor })
    }

    /// Optimize bond lengths, bond angles, ring shapes, and overlap through the history.
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

fn normalized_label(label: Option<&str>) -> Option<String> {
    label
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
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
    fn selection_edits_change_atoms_and_bonds_without_a_dialog() {
        let mut editor = editor();
        let a = editor
            .document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let b = editor
            .document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [1.5, 0.0, 0.0]);
        let bond = editor
            .document
            .molecule
            .add_bond(a, b, BondOrder::Single)
            .expect("bond");
        editor.selection.clear();
        editor.selection.select_atom(a, SelectionMode::Add);
        editor.selection.select_atom(b, SelectionMode::Add);
        assert!(
            editor
                .set_selected_element(Element::from_symbol("O").expect("O"))
                .expect("element")
        );
        assert!(editor.adjust_selected_charge(-1).expect("charge"));
        assert!(editor.set_selected_label(Some("OH")).expect("label"));
        assert_eq!(
            editor.document.molecule.atom(a).expect("atom").element,
            Element::from_symbol("O").expect("O")
        );
        assert_eq!(editor.document.molecule.atom(b).expect("atom").charge, -1);
        assert_eq!(
            editor.document.molecule.atom(a).expect("atom").label.as_deref(),
            Some("OH")
        );

        editor.selection.clear();
        editor.selection.select_bond(bond, SelectionMode::Add);
        assert!(editor.set_selected_bond_order(BondOrder::Double).expect("order"));
        assert!(editor.set_selected_bond_stereo(BondStereo::Hash).expect("stereo"));
        assert_eq!(editor.document.molecule.bonds[0].order, BondOrder::Double);
        assert_eq!(editor.document.molecule.bonds[0].stereo, BondStereo::Hash);
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.bonds[0].stereo, BondStereo::None);
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.bonds[0].order, BondOrder::Single);
    }

    #[test]
    fn clean_layout_optimizes_geometry_and_undoes() {
        let mut editor = editor();
        editor.add_chain([0.0, 0.0], 5, 0.0).expect("chain");
        for atom in &mut editor.document.molecule.atoms {
            atom.position[0] *= 0.55;
            atom.position[1] *= 0.3;
        }
        let distorted = editor.document.molecule.atoms[0].position;
        editor.clean_layout().expect("clean");
        let first = editor.document.molecule.atoms[0].position;
        let second = editor.document.molecule.atoms[1].position;
        let length = (first[0] - second[0]).hypot(first[1] - second[1]);
        assert!((length - 1.5).abs() < 0.12, "{length}");
        let cleaned = editor
            .document
            .molecule
            .atoms
            .iter()
            .map(|atom| atom.position)
            .collect::<Vec<_>>();
        editor.clean_layout().expect("second clean");
        let twice = editor
            .document
            .molecule
            .atoms
            .iter()
            .map(|atom| atom.position)
            .collect::<Vec<_>>();
        let moved = cleaned
            .iter()
            .zip(&twice)
            .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
            .fold(0.0_f64, f64::max);
        assert!(moved < 1.0e-9, "second clean moved {moved}");
        assert!(editor.undo());
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.atoms[0].position, distorted);
    }

    #[test]
    fn select_all_and_nudge_follow_the_document() {
        let mut editor = editor();
        editor.add_chain([0.0, 0.0], 3, 0.0).expect("chain");
        assert!(editor.select_all());
        assert_eq!(editor.selection.atoms().len(), 3);
        assert_eq!(editor.selection.bonds().len(), 2);
        let before = editor.document.molecule.atoms[0].position;
        assert!(editor.nudge_selection([0.1, 0.0]).expect("nudge"));
        assert!((editor.document.molecule.atoms[0].position[0] - before[0] - 0.1).abs() < 1.0e-9);
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.atoms[0].position, before);
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
