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

pub mod camera;
pub mod document;
pub mod editor;
pub mod history;
pub mod interact;
pub mod paint;

pub use camera::Camera2D;
pub use document::{Atom2D, Bond2D, ChemDocument, Molecule2D};
pub use editor::{Chem2dEditor, Chem2dResponse, EditorTool};
pub use egui_workbench_chem_core::{
    AtomId, BondId, BondOrder, BondStereo, ChemSelection, DocumentId, Element, SelectionMode,
};
pub use history::{Command, History};
pub use interact::{Hit, implicit_hydrogens};

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
