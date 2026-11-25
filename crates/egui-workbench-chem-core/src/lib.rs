//! Shared, egui-free chemistry model for egui-workbench.
//!
//! The crate defines the contract between a 2D editor (`egui-workbench-chem2d`),
//! a 3D viewer (`egui-workbench-mol3d`), and the external `sci-form` and
//! `xtalforge-core` engines:
//!
//! - [`id`] gives documents, atoms, and bonds stable generational identifiers.
//! - [`element`] exposes the periodic table, covalent radii, and CPK colors.
//! - [`model`] stores the molecular graph and its stereochemistry.
//! - [`selection`] is the selection shared by every panel.
//! - [`events`] carries typed changes through [`events::EventQueue`].
//! - [`bridge`] is the boundary to the external chemistry/xtal engines.
//! - [`jobs`] runs expensive work off the UI thread with cancellation and progress.
//!
//! No module in this crate depends on egui or a GPU.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod bridge;
pub mod element;
pub mod events;
pub mod id;
pub mod jobs;
pub mod model;
pub mod selection;

pub use bridge::{BridgeError, SciFormBridge, XtalBridge};
pub use element::Element;
pub use events::{ChemEvent, EventQueue};
pub use id::{AtomId, BondId, DocumentId, IdPool};
pub use jobs::{CancellationToken, Job, JobContext, JobError, JobHandle, JobQueue, Progress};
pub use model::{Atom, Bond, BondOrder, BondStereo, ChemDocument, Molecule, UnitCell};
pub use selection::{ChemSelection, SelectionMode};

/// Why a chemistry operation failed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChemError {
    /// The atom id is not in the molecule.
    UnknownAtom(AtomId),
    /// The bond id is not in the molecule.
    UnknownBond(BondId),
    /// A bond between the same two atoms already exists.
    DuplicateBond {
        /// First atom.
        a: AtomId,
        /// Second atom.
        b: AtomId,
    },
    /// A bond cannot join an atom to itself.
    SelfBond(AtomId),
    /// A coordinate or cell parameter is not finite.
    InvalidGeometry(String),
    /// The element symbol is unknown.
    UnknownElement(String),
}

impl std::fmt::Display for ChemError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAtom(id) => write!(formatter, "atom {} is not in the molecule", id.slot()),
            Self::UnknownBond(id) => write!(formatter, "bond {} is not in the molecule", id.slot()),
            Self::DuplicateBond { a, b } => write!(
                formatter,
                "a bond between atoms {} and {} already exists",
                a.slot(),
                b.slot()
            ),
            Self::SelfBond(id) => write!(formatter, "atom {} cannot bond to itself", id.slot()),
            Self::InvalidGeometry(message) => formatter.write_str(message),
            Self::UnknownElement(symbol) => write!(formatter, "unknown element `{symbol}`"),
        }
    }
}

impl std::error::Error for ChemError {}
