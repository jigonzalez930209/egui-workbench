//! A vector 2D chemistry editor for egui-workbench.
//!
//! The crate uses the shared [`egui_workbench_chem_core`] molecule, selection, and
//! identifiers, so a 2D editor and a future 3D viewer operate on the same graph.
//!
//! - [`document`] wraps the molecule with a document name and 2D helpers.
//! - [`camera`] maps Angstrom to points with pan, zoom, and fit.
//! - [`history`] stores deterministic commands and undo/redo snapshots.
//! - [`interact`] does hit testing, rectangle/lasso selection, implicit hydrogens,
//!   and contextual atom labels.
//! - [`editor`] is the ready-to-use widget.
//! - [`paint`] draws a document with `egui::Painter`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod annotations;
pub mod camera;
pub mod cdxml;
pub mod delta;
pub mod document;
pub mod editor;
pub mod export;
pub mod history;
pub mod interact;
pub mod layout;
pub mod molfile;
pub mod paint;
pub mod properties;
pub mod rings;
pub mod smiles;
pub mod superatom;
pub mod templates;

pub use annotations::{
    Annotation, ArrowKind, ConditionBox, PolymerBracket, ReactionArrow, ReactionScheme, format_chemical_text,
};
pub use camera::Camera2D;
pub use cdxml::{from_cdxml, to_cdxml};
pub use delta::{DeltaResult, DocumentDelta};
pub use document::{Atom2D, Bond2D, ChemDocument, Molecule2D};
pub use editor::{Chem2dEditor, Chem2dResponse, EditorTool};
pub use egui_workbench_chem_core::{
    AtomId, BondId, BondOrder, BondStereo, ChemSelection, DocumentId, Element, SelectionMode,
};
pub use export::{ExportOptions, ExportStyle, to_png, to_svg};
pub use history::{Command, History};
pub use interact::{Hit, implicit_hydrogens};
pub use layout::relax;
pub use molfile::{MolfileError, from_molfile, from_rxn, from_sdf, from_v3000, to_rxn, to_sdf, to_v2000, to_v3000};
pub use properties::{MolecularProperties, StereoDescriptor, properties, stereo_descriptors};
pub use rings::{Ring, aromatic_rings, cycle_basis, ring_for_bond};
pub use smiles::{SmilesError, SmilesGraph, parse_smiles};
pub use superatom::{Superatom, SuperatomGroup, library as superatom_library};
pub use templates::{RingTemplate, add_chain, add_ring, snap_angle};

/// Why a 2D editing or chemistry operation failed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Chem2dError {
    /// The atom is not in the document.
    UnknownAtom(AtomId),
    /// The bond is not in the document.
    UnknownBond(BondId),
    /// The document rejected the operation.
    Chemistry(String),
}

impl std::fmt::Display for Chem2dError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAtom(id) => write!(formatter, "atom {id} is not in the document"),
            Self::UnknownBond(id) => write!(formatter, "bond {id} is not in the document"),
            Self::Chemistry(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for Chem2dError {}

impl From<egui_workbench_chem_core::ChemError> for Chem2dError {
    fn from(error: egui_workbench_chem_core::ChemError) -> Self {
        Self::Chemistry(error.to_string())
    }
}
