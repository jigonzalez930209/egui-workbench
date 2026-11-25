//! A 3D molecular and crystal viewport for egui-workbench.
//!
//! The viewport uses the shared [`egui_workbench_chem_core`] molecule, ids, and
//! selection. This milestone provides:
//!
//! - [`scene`]: atoms, bonds, radii, and cell edges.
//! - [`math`]: a small quaternion/vector layer used by the camera.
//! - [`camera`]: an arcball orbit camera with perspective and orthographic projection.
//! - [`view`]: a depth-sorted `egui::Painter` viewport with picking and selection.
//!
//! The analytic wgpu impostor renderer is a later milestone; [`view`] is the reference
//! renderer and the fallback.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod camera;
pub mod math;
pub mod scene;
pub mod view;

pub use camera::{Camera3D, Projection3D};
pub use egui_workbench_chem_core::{AtomId, BondId, ChemSelection, Element, SelectionMode};
pub use math::{Quat, Vec3};
pub use scene::{Atom3D, Bond3D, Mol3dScene};
pub use view::{Mol3dResponse, Mol3dStyle, Mol3dView};
