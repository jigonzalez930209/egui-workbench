//! Deterministic editing commands and undo/redo.

use egui_workbench_chem_core::{AtomId, BondId, BondOrder, BondStereo, ChemSelection, Element};

use crate::Chem2dError;
use crate::annotations::Annotation;
use crate::document::ChemDocument;
use crate::superatom::SuperatomGroup;
use crate::templates::RingTemplate;

/// One atomic edit.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Command {
    /// Add an atom at a 2D position and select it.
    AddAtom {
        /// Element.
        element: Element,
        /// Position in Angstrom.
        position: [f64; 2],
    },
    /// Add a bond and select it.
    AddBond {
        /// First atom.
        a: AtomId,
        /// Second atom.
        b: AtomId,
        /// Bond order.
        order: BondOrder,
    },
    /// Remove every selected atom and bond.
    DeleteSelection,
    /// Remove every atom, bond, superatom, and annotation from the document.
    ClearDocument,
    /// Change a bond order.
    SetBondOrder {
        /// Bond to edit.
        bond: BondId,
        /// New order.
        order: BondOrder,
    },
    /// Change several bond orders in one undo step.
    SetBondOrders {
        /// Bonds to edit.
        bonds: Vec<BondId>,
        /// New order.
        order: BondOrder,
    },
    /// Change bond stereochemistry.
    SetStereo {
        /// Bond to edit.
        bond: BondId,
        /// New stereo.
        stereo: BondStereo,
    },
    /// Change several bond stereochemistries in one undo step.
    SetBondStereos {
        /// Bonds to edit.
        bonds: Vec<BondId>,
        /// New stereo.
        stereo: BondStereo,
    },
    /// Draw a bond between two atoms, creating either endpoint when it is not supplied.
    ///
    /// This is the drag-to-draw command: one gesture creates the atoms it needs and the
    /// bond between them as a single undo step.
    DrawBond {
        /// Existing atom at the start, when the gesture began on one.
        from: Option<AtomId>,
        /// Start position when `from` is `None`.
        from_position: [f64; 2],
        /// Existing atom at the end, when the gesture ended on one.
        to: Option<AtomId>,
        /// End position when `to` is `None`.
        to_position: [f64; 2],
        /// Bond order.
        order: BondOrder,
        /// Element for a newly created endpoint.
        element: Element,
        /// Bond stereochemistry.
        stereo: BondStereo,
    },
    /// Change the element of atoms and clear their custom labels.
    SetAtomElements {
        /// Atoms to edit.
        atoms: Vec<AtomId>,
        /// New element.
        element: Element,
    },
    /// Add `delta` to the formal charge of atoms.
    AdjustAtomCharges {
        /// Atoms to edit.
        atoms: Vec<AtomId>,
        /// Change in formal charge.
        delta: i8,
    },
    /// Set an atom's element, formal charge, and display label in one undo step.
    SetAtomProperties {
        /// Atom to edit.
        atom: AtomId,
        /// New element.
        element: Element,
        /// New formal charge.
        charge: i8,
        /// New display label; `None` clears it.
        label: Option<String>,
    },
    /// Set or clear a custom display label on atoms.
    SetAtomLabels {
        /// Atoms to edit.
        atoms: Vec<AtomId>,
        /// New display label; `None` clears it.
        label: Option<String>,
    },
    /// Move a fragment.
    TranslateFragment {
        /// Atoms to move.
        atoms: Vec<AtomId>,
        /// xy displacement in Angstrom.
        delta: [f64; 2],
    },
    /// Stamp a ring template.
    AddRing {
        /// Template to add.
        template: RingTemplate,
        /// Center in Angstrom.
        center: [f64; 2],
        /// Rotation in radians.
        rotation: f64,
    },
    /// Attach a ring template to an existing atom with one new single bond.
    AttachRing {
        /// Template to add.
        template: RingTemplate,
        /// Atom the ring grows from.
        atom: AtomId,
    },
    /// Add a zig-zag carbon chain.
    AddChain {
        /// First atom position.
        start: [f64; 2],
        /// Number of carbons.
        count: usize,
        /// Main direction in radians.
        angle: f64,
    },
    /// Rotate a fragment around `center`.
    RotateFragment {
        /// Atoms to rotate.
        atoms: Vec<AtomId>,
        /// Rotation center.
        center: [f64; 2],
        /// Angle in radians.
        radians: f64,
    },
    /// Mirror a fragment across its centroid.
    MirrorFragment {
        /// Atoms to mirror.
        atoms: Vec<AtomId>,
        /// Mirror x (vertical axis) when true, y when false.
        vertical: bool,
    },
    /// Expand a labeled pseudo atom into its superatom fragment.
    ExpandSuperatom {
        /// Pseudo atom.
        pseudo: AtomId,
    },
    /// Collapse an expanded superatom.
    ContractSuperatom {
        /// Group to remove.
        group: SuperatomGroup,
    },
    /// Add a reaction arrow, conditions box, or polymer bracket.
    AddAnnotation(Annotation),
    /// Insert a parsed SMILES graph.
    InsertSmiles {
        /// SMILES string.
        text: String,
        /// Center position.
        center: [f64; 2],
    },
    /// Insert a parsed SMILES graph and connect its first atom to an existing atom.
    AttachSmiles {
        /// SMILES string.
        text: String,
        /// Atom the fragment grows from.
        anchor: AtomId,
    },
    /// Optimize bond lengths, bond angles, ring shapes, and steric overlap.
    CleanLayout,
}

impl Command {
    /// Apply the command to `document` and update the selection.
    ///
    /// # Errors
    /// When an atom or bond is unknown or the document rejects the change.
    pub fn apply(&self, document: &mut ChemDocument, selection: &mut ChemSelection) -> Result<(), Chem2dError> {
        match self {
            Self::AddAtom { element, position } => {
                let id = document.molecule.add_atom(*element, [position[0], position[1], 0.0]);
                selection.select_atom(id, egui_workbench_chem_core::SelectionMode::Replace);
            }
            Self::AddBond { a, b, order } => {
                let id = document.molecule.add_bond(*a, *b, *order)?;
                selection.select_bond(id, egui_workbench_chem_core::SelectionMode::Replace);
            }
            Self::DeleteSelection => {
                let atoms = selection.atoms().iter().copied().collect::<Vec<_>>();
                let bonds = selection.bonds().iter().copied().collect::<Vec<_>>();
                for bond in bonds {
                    document.molecule.remove_bond(bond);
                }
                for atom in atoms {
                    document.molecule.remove_atom(atom);
                }
                document.prune_superatoms();
                selection.clear();
            }
            Self::ClearDocument => {
                document.molecule = crate::document::Molecule2D::new();
                document.superatoms.clear();
                document.annotations.clear();
                selection.clear();
            }
            Self::SetBondOrder { bond, order } => {
                let Some(entry) = document.molecule.bonds.iter_mut().find(|entry| entry.id == *bond) else {
                    return Err(Chem2dError::UnknownBond(*bond));
                };
                entry.order = *order;
            }
            Self::SetBondOrders { bonds, order } => {
                for bond in bonds {
                    if !document.molecule.bonds.iter().any(|entry| entry.id == *bond) {
                        return Err(Chem2dError::UnknownBond(*bond));
                    }
                }
                for bond in bonds {
                    if let Some(entry) = document.molecule.bonds.iter_mut().find(|entry| entry.id == *bond) {
                        entry.order = *order;
                    }
                }
            }
            Self::SetStereo { bond, stereo } => {
                let Some(entry) = document.molecule.bonds.iter_mut().find(|entry| entry.id == *bond) else {
                    return Err(Chem2dError::UnknownBond(*bond));
                };
                entry.stereo = *stereo;
            }
            Self::SetBondStereos { bonds, stereo } => {
                for bond in bonds {
                    if !document.molecule.bonds.iter().any(|entry| entry.id == *bond) {
                        return Err(Chem2dError::UnknownBond(*bond));
                    }
                }
                for bond in bonds {
                    if let Some(entry) = document.molecule.bonds.iter_mut().find(|entry| entry.id == *bond) {
                        entry.stereo = *stereo;
                    }
                }
            }
            Self::DrawBond {
                from,
                from_position,
                to,
                to_position,
                order,
                element,
                stereo,
            } => {
                if let Some(atom) = from
                    && document.molecule.atom(*atom).is_none()
                {
                    return Err(Chem2dError::UnknownAtom(*atom));
                }
                if let Some(atom) = to
                    && document.molecule.atom(*atom).is_none()
                {
                    return Err(Chem2dError::UnknownAtom(*atom));
                }
                if from.is_some() && from == to {
                    return Err(Chem2dError::Chemistry("a bond needs two different atoms".to_owned()));
                }
                let a = (*from).unwrap_or_else(|| {
                    document
                        .molecule
                        .add_atom(*element, [from_position[0], from_position[1], 0.0])
                });
                let b = (*to).unwrap_or_else(|| {
                    document
                        .molecule
                        .add_atom(*element, [to_position[0], to_position[1], 0.0])
                });
                if let Some(existing) = document.molecule.bond_between(a, b) {
                    let id = existing.id;
                    if let Some(entry) = document.molecule.bonds.iter_mut().find(|entry| entry.id == id) {
                        entry.order = *order;
                        entry.stereo = *stereo;
                    }
                    selection.select_bond(id, egui_workbench_chem_core::SelectionMode::Replace);
                } else {
                    let id = document.molecule.add_bond(a, b, *order)?;
                    if let Some(entry) = document.molecule.bonds.iter_mut().find(|entry| entry.id == id) {
                        entry.stereo = *stereo;
                    }
                    selection.select_bond(id, egui_workbench_chem_core::SelectionMode::Replace);
                }
            }
            Self::SetAtomElements { atoms, element } => {
                for atom in atoms {
                    if document.molecule.atom(*atom).is_none() {
                        return Err(Chem2dError::UnknownAtom(*atom));
                    }
                }
                for atom in atoms {
                    if let Some(entry) = document.molecule.atom_mut(*atom) {
                        entry.element = *element;
                        entry.label = None;
                    }
                }
            }
            Self::AdjustAtomCharges { atoms, delta } => {
                for atom in atoms {
                    if document.molecule.atom(*atom).is_none() {
                        return Err(Chem2dError::UnknownAtom(*atom));
                    }
                }
                for atom in atoms {
                    if let Some(entry) = document.molecule.atom_mut(*atom) {
                        entry.charge = entry.charge.saturating_add(*delta);
                    }
                }
            }
            Self::SetAtomProperties {
                atom,
                element,
                charge,
                label,
            } => {
                let Some(entry) = document.molecule.atom_mut(*atom) else {
                    return Err(Chem2dError::UnknownAtom(*atom));
                };
                entry.element = *element;
                entry.charge = *charge;
                entry.label.clone_from(label);
            }
            Self::SetAtomLabels { atoms, label } => {
                for atom in atoms {
                    if document.molecule.atom(*atom).is_none() {
                        return Err(Chem2dError::UnknownAtom(*atom));
                    }
                }
                for atom in atoms {
                    if let Some(entry) = document.molecule.atom_mut(*atom) {
                        entry.label.clone_from(label);
                    }
                }
            }
            Self::TranslateFragment { atoms, delta } => {
                for atom in atoms {
                    if document.molecule.atom(*atom).is_none() {
                        return Err(Chem2dError::UnknownAtom(*atom));
                    }
                }
                document.molecule.translate(atoms, *delta);
            }
            Self::AddRing {
                template,
                center,
                rotation,
            } => {
                let ids = crate::templates::add_ring(&mut document.molecule, *template, *center, *rotation);
                selection.clear();
                for id in ids {
                    selection.select_atom(id, egui_workbench_chem_core::SelectionMode::Add);
                }
            }
            Self::AttachRing { template, atom } => {
                if document.molecule.atom(*atom).is_none() {
                    return Err(Chem2dError::UnknownAtom(*atom));
                }
                let ids = crate::templates::attach_ring(&mut document.molecule, *template, *atom);
                if ids.is_empty() {
                    return Err(Chem2dError::Chemistry("the ring could not be attached".to_owned()));
                }
                selection.clear();
                for id in ids {
                    selection.select_atom(id, egui_workbench_chem_core::SelectionMode::Add);
                }
            }
            Self::AddChain { start, count, angle } => {
                let ids = crate::templates::add_chain(&mut document.molecule, *start, *count, *angle);
                selection.clear();
                for id in ids {
                    selection.select_atom(id, egui_workbench_chem_core::SelectionMode::Add);
                }
            }
            Self::RotateFragment { atoms, center, radians } => {
                let (sine, cosine) = radians.sin_cos();
                for id in atoms {
                    let Some(atom) = document.molecule.atom_mut(*id) else {
                        return Err(Chem2dError::UnknownAtom(*id));
                    };
                    let x = atom.position[0] - center[0];
                    let y = atom.position[1] - center[1];
                    atom.position[0] = center[0] + x * cosine - y * sine;
                    atom.position[1] = center[1] + x * sine + y * cosine;
                }
            }
            Self::MirrorFragment { atoms, vertical } => {
                let mut count = 0.0;
                let mut sum = [0.0; 2];
                for id in atoms {
                    let Some(atom) = document.molecule.atom(*id) else {
                        return Err(Chem2dError::UnknownAtom(*id));
                    };
                    sum[0] += atom.position[0];
                    sum[1] += atom.position[1];
                    count += 1.0;
                }
                if count == 0.0 {
                    return Ok(());
                }
                let center = [sum[0] / count, sum[1] / count];
                for id in atoms {
                    let Some(atom) = document.molecule.atom_mut(*id) else {
                        return Err(Chem2dError::UnknownAtom(*id));
                    };
                    if *vertical {
                        atom.position[0] = 2.0 * center[0] - atom.position[0];
                    } else {
                        atom.position[1] = 2.0 * center[1] - atom.position[1];
                    }
                }
            }
            Self::ExpandSuperatom { pseudo } => {
                let group = document.expand_superatom(*pseudo)?;
                selection.clear();
                for atom in group.atoms {
                    selection.select_atom(atom, egui_workbench_chem_core::SelectionMode::Add);
                }
            }
            Self::ContractSuperatom { group } => {
                document.contract_superatom(group)?;
                selection.clear();
                selection.select_atom(group.pseudo, egui_workbench_chem_core::SelectionMode::Replace);
            }
            Self::AddAnnotation(annotation) => document.annotations.push(annotation.clone()),
            Self::InsertSmiles { text, center } => {
                let graph =
                    crate::smiles::parse_smiles(text).map_err(|error| Chem2dError::Chemistry(error.to_string()))?;
                let ids = graph.insert(document, *center);
                selection.clear();
                for atom in ids {
                    selection.select_atom(atom, egui_workbench_chem_core::SelectionMode::Add);
                }
            }
            Self::AttachSmiles { text, anchor } => {
                let Some(anchor_atom) = document.molecule.atom(*anchor) else {
                    return Err(Chem2dError::UnknownAtom(*anchor));
                };
                let anchor_position = [anchor_atom.position[0], anchor_atom.position[1]];
                let graph =
                    crate::smiles::parse_smiles(text).map_err(|error| Chem2dError::Chemistry(error.to_string()))?;
                let mut layout = graph.to_molecule();
                let Some(first_position) = layout.atoms.first().map(|atom| atom.position) else {
                    return Err(Chem2dError::Chemistry("the SMILES is empty".to_owned()));
                };
                let mut sum = [0.0; 2];
                for atom in &layout.atoms {
                    sum[0] += atom.position[0];
                    sum[1] += atom.position[1];
                }
                let count = layout.atoms.len() as f64;
                let centroid = [sum[0] / count, sum[1] / count];
                let to_centroid = [centroid[0] - first_position[0], centroid[1] - first_position[1]];
                let radius = layout
                    .atoms
                    .iter()
                    .map(|atom| (atom.position[0] - first_position[0]).hypot(atom.position[1] - first_position[1]))
                    .fold(0.0, f64::max);
                let angle = crate::templates::sprout_angle(&document.molecule, *anchor, 1.5 + radius)
                    .ok_or(Chem2dError::UnknownAtom(*anchor))?;
                let rotation = if to_centroid[0].hypot(to_centroid[1]) <= f64::EPSILON {
                    0.0
                } else {
                    angle - to_centroid[1].atan2(to_centroid[0])
                };
                let (sine, cosine) = rotation.sin_cos();
                for atom in &mut layout.atoms {
                    let x = atom.position[0] - first_position[0];
                    let y = atom.position[1] - first_position[1];
                    atom.position[0] = first_position[0] + x * cosine - y * sine;
                    atom.position[1] = first_position[1] + x * sine + y * cosine;
                }
                let attach = [
                    anchor_position[0] + 1.5 * angle.cos(),
                    anchor_position[1] + 1.5 * angle.sin(),
                ];
                let offset = [attach[0] - first_position[0], attach[1] - first_position[1]];
                let ids = crate::smiles::insert_molecule(document, &layout, offset);
                let Some(first_id) = ids.first().copied() else {
                    return Err(Chem2dError::Chemistry("the SMILES is empty".to_owned()));
                };
                let bond = document.molecule.add_bond(*anchor, first_id, BondOrder::Single)?;
                selection.clear();
                for atom in ids {
                    selection.select_atom(atom, egui_workbench_chem_core::SelectionMode::Add);
                }
                selection.select_bond(bond, egui_workbench_chem_core::SelectionMode::Add);
            }
            Self::CleanLayout => {
                crate::layout::optimize_layout(&mut document.molecule);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Snapshot {
    document: ChemDocument,
    selection: ChemSelection,
}

/// Undo/redo history of whole-document snapshots.
///
/// Snapshots are simple and deterministic; a 2D document is small enough that this is
/// cheap. Use [`History::with_limit`] to cap memory in long editing sessions.
#[derive(Clone, Debug, Default)]
pub struct History {
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    limit: Option<usize>,
}

impl History {
    /// An unlimited history.
    pub fn new() -> Self {
        Self::default()
    }

    /// A history that keeps at most `limit` undo steps.
    pub fn with_limit(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: Some(limit.max(1)),
        }
    }

    /// Apply `command`, recording the previous state.
    ///
    /// # Errors
    /// When the command fails; the document is not changed in that case.
    pub fn execute(
        &mut self,
        document: &mut ChemDocument,
        selection: &mut ChemSelection,
        command: &Command,
    ) -> Result<(), Chem2dError> {
        let before = Snapshot {
            document: document.clone(),
            selection: selection.clone(),
        };
        if let Err(error) = command.apply(document, selection) {
            // A command can fail after changing some atoms (a rotation of a list with one
            // unknown id); put everything back instead of leaving a half-applied edit that
            // has no undo entry.
            *document = before.document;
            *selection = before.selection;
            return Err(error);
        }
        self.undo.push(before);
        if let Some(limit) = self.limit
            && self.undo.len() > limit
        {
            self.undo.remove(0);
        }
        self.redo.clear();
        Ok(())
    }

    /// Undo the last command. Returns whether anything changed.
    pub fn undo(&mut self, document: &mut ChemDocument, selection: &mut ChemSelection) -> bool {
        let Some(snapshot) = self.undo.pop() else {
            return false;
        };
        self.redo.push(Snapshot {
            document: document.clone(),
            selection: selection.clone(),
        });
        *document = snapshot.document;
        *selection = snapshot.selection;
        true
    }

    /// Redo the last undone command. Returns whether anything changed.
    pub fn redo(&mut self, document: &mut ChemDocument, selection: &mut ChemSelection) -> bool {
        let Some(snapshot) = self.redo.pop() else {
            return false;
        };
        self.undo.push(Snapshot {
            document: document.clone(),
            selection: selection.clone(),
        });
        *document = snapshot.document;
        *selection = snapshot.selection;
        true
    }

    /// Whether an undo step exists.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether a redo step exists.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Number of undo steps.
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Drop both stacks.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ChemDocument;
    use egui_workbench_chem_core::DocumentId;

    fn document() -> ChemDocument {
        ChemDocument::new(DocumentId::new(1), "test")
    }

    #[test]
    fn add_atom_undo_and_redo_restore_the_graph() {
        let mut document = document();
        let mut selection = ChemSelection::new();
        let mut history = History::new();
        history
            .execute(
                &mut document,
                &mut selection,
                &Command::AddAtom {
                    element: Element::UNKNOWN,
                    position: [1.0, 2.0],
                },
            )
            .expect("add");
        assert_eq!(document.molecule.atom_count(), 1);
        assert!(history.can_undo());
        assert!(history.undo(&mut document, &mut selection));
        assert_eq!(document.molecule.atom_count(), 0);
        assert!(history.can_redo());
        assert!(history.redo(&mut document, &mut selection));
        assert_eq!(document.molecule.atom_count(), 1);
    }

    #[test]
    fn a_failed_command_does_not_change_the_document() {
        let mut document = document();
        let mut selection = ChemSelection::new();
        let mut history = History::new();
        let result = history.execute(
            &mut document,
            &mut selection,
            &Command::TranslateFragment {
                atoms: vec![AtomId::new(9, 9)],
                delta: [1.0, 1.0],
            },
        );
        assert!(result.is_err());
        assert!(!history.can_undo());
    }

    #[test]
    fn clearing_the_document_is_one_undo_step() {
        let mut document = document();
        let a = document.molecule.add_atom(Element::UNKNOWN, [0.0, 0.0, 0.0]);
        let b = document.molecule.add_atom(Element::UNKNOWN, [1.5, 0.0, 0.0]);
        document.molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        document
            .annotations
            .push(Annotation::Conditions(crate::annotations::ConditionBox::line(
                [0.0, 0.0],
                "x",
            )));
        let mut selection = ChemSelection::new();
        selection.select_atom(a, egui_workbench_chem_core::SelectionMode::Replace);
        let mut history = History::new();
        history
            .execute(&mut document, &mut selection, &Command::ClearDocument)
            .expect("clear");
        assert_eq!(document.molecule.atom_count(), 0);
        assert!(document.annotations.is_empty());
        assert!(selection.is_empty());
        assert!(history.undo(&mut document, &mut selection));
        assert_eq!(document.molecule.atom_count(), 2);
        assert_eq!(document.annotations.len(), 1);
    }

    #[test]
    fn draw_bond_creates_its_endpoints_as_one_undo_step() {
        let mut document = document();
        let mut selection = ChemSelection::new();
        let mut history = History::new();
        history
            .execute(
                &mut document,
                &mut selection,
                &Command::DrawBond {
                    from: None,
                    from_position: [0.0, 0.0],
                    to: None,
                    to_position: [1.5, 0.0],
                    order: BondOrder::Double,
                    element: Element::from_symbol("N").expect("N"),
                    stereo: BondStereo::Wedge,
                },
            )
            .expect("draw");
        assert_eq!(document.molecule.atom_count(), 2);
        assert_eq!(document.molecule.bond_count(), 1);
        assert!(
            document
                .molecule
                .atoms
                .iter()
                .all(|atom| atom.element.atomic_number() == 7)
        );
        let bond = &document.molecule.bonds[0];
        assert_eq!(bond.order, BondOrder::Double);
        assert_eq!(bond.stereo, BondStereo::Wedge);
        assert!(selection.bonds().contains(&bond.id));
        assert!(history.undo(&mut document, &mut selection));
        assert_eq!(document.molecule.atom_count(), 0);
    }

    #[test]
    fn drawing_over_an_existing_bond_updates_it_instead_of_duplicating() {
        let mut document = document();
        let a = document.molecule.add_atom(Element::UNKNOWN, [0.0, 0.0, 0.0]);
        let b = document.molecule.add_atom(Element::UNKNOWN, [1.5, 0.0, 0.0]);
        document.molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        let mut selection = ChemSelection::new();
        let mut history = History::new();
        history
            .execute(
                &mut document,
                &mut selection,
                &Command::DrawBond {
                    from: Some(a),
                    from_position: [0.0, 0.0],
                    to: Some(b),
                    to_position: [1.5, 0.0],
                    order: BondOrder::Triple,
                    element: Element::UNKNOWN,
                    stereo: BondStereo::None,
                },
            )
            .expect("draw");
        assert_eq!(document.molecule.bond_count(), 1);
        assert_eq!(document.molecule.bonds[0].order, BondOrder::Triple);
    }

    #[test]
    fn a_ring_and_a_smiles_fragment_attach_to_an_existing_atom() {
        let mut document = document();
        let anchor = document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let mut selection = ChemSelection::new();
        let mut history = History::new();
        history
            .execute(
                &mut document,
                &mut selection,
                &Command::AttachRing {
                    template: RingTemplate::Benzene,
                    atom: anchor,
                },
            )
            .expect("ring");
        assert_eq!(document.molecule.atom_count(), 7);
        assert_eq!(document.molecule.rings().len(), 1);
        assert!(history.undo(&mut document, &mut selection));
        assert_eq!(document.molecule.atom_count(), 1);

        history
            .execute(
                &mut document,
                &mut selection,
                &Command::AttachSmiles {
                    text: "c1ccncc1".to_owned(),
                    anchor,
                },
            )
            .expect("smiles");
        assert_eq!(document.molecule.atom_count(), 7);
        assert_eq!(document.molecule.bond_count(), 7);
        let attached = document
            .molecule
            .neighbors(anchor)
            .into_iter()
            .find_map(|id| document.molecule.position_2d(id));
        let attached = attached.expect("attachment bond");
        assert!((attached[0].hypot(attached[1]) - 1.5).abs() < 1.0e-9);
        assert!(history.undo(&mut document, &mut selection));
        assert_eq!(document.molecule.atom_count(), 1);
    }

    #[test]
    fn bulk_atom_and_bond_edits_are_validated_and_undone_together() {
        let mut document = document();
        let a = document.molecule.add_atom(Element::UNKNOWN, [0.0, 0.0, 0.0]);
        let b = document.molecule.add_atom(Element::UNKNOWN, [1.5, 0.0, 0.0]);
        let bond = document.molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        let mut selection = ChemSelection::new();
        let mut history = History::new();
        history
            .execute(
                &mut document,
                &mut selection,
                &Command::SetAtomElements {
                    atoms: vec![a, b],
                    element: Element::from_symbol("O").expect("O"),
                },
            )
            .expect("elements");
        history
            .execute(
                &mut document,
                &mut selection,
                &Command::AdjustAtomCharges {
                    atoms: vec![a, b],
                    delta: -1,
                },
            )
            .expect("charges");
        history
            .execute(
                &mut document,
                &mut selection,
                &Command::SetBondOrders {
                    bonds: vec![bond],
                    order: BondOrder::Aromatic,
                },
            )
            .expect("orders");
        assert!(document.molecule.atoms.iter().all(|atom| atom.charge == -1));
        assert_eq!(document.molecule.bonds[0].order, BondOrder::Aromatic);
        assert!(history.undo(&mut document, &mut selection));
        assert_eq!(document.molecule.bonds[0].order, BondOrder::Single);
        assert!(history.undo(&mut document, &mut selection));
        assert!(document.molecule.atoms.iter().all(|atom| atom.charge == 0));
    }

    #[test]
    fn delete_selection_removes_atoms_and_bonds() {
        let mut document = document();
        let a = document.molecule.add_atom(Element::UNKNOWN, [0.0, 0.0, 0.0]);
        let b = document.molecule.add_atom(Element::UNKNOWN, [1.0, 0.0, 0.0]);
        document.molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        let mut selection = ChemSelection::new();
        selection.select_atom(a, egui_workbench_chem_core::SelectionMode::Add);
        let mut history = History::new();
        history
            .execute(&mut document, &mut selection, &Command::DeleteSelection)
            .expect("delete");
        assert_eq!(document.molecule.atom_count(), 1);
        assert_eq!(document.molecule.bond_count(), 0);
    }
}
