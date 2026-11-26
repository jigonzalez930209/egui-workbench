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

pub mod actions;
pub mod biomol;
pub mod camera;
pub mod cif;
pub mod color;
pub mod crystal;
pub mod descriptor;
pub mod formats;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod math;
pub mod measure;
pub mod render;
pub mod representation;
pub mod scene;
pub mod spatial;
pub mod surface;
pub mod trajectory;
pub mod view;
pub mod volume;

pub use actions::{ActionOutcome, Mol3dAction};
pub use biomol::{BackboneTrace, Interaction, InteractionKind, SecondaryStructure, detect_interactions, ribbon_meshes};
pub use camera::{Camera3D, Projection3D};
pub use cif::{SymmetryOperation, parse_cif, parse_operation};
pub use color::ColorScheme;
pub use crystal::{CellAxes, Polyhedron, miller_plane};
pub use descriptor::SceneDescriptor;
pub use egui_workbench_chem_core::{AtomId, BondId, ChemSelection, Element, SelectionMode};
pub use formats::{FormatError, parse_mmcif, parse_mol, parse_pdb, parse_poscar, parse_xyz};
#[cfg(feature = "gpu")]
pub use gpu::set_output_format;
pub use math::{Quat, Vec3};
pub use measure::{Measurement, MeasurementValue};
pub use render::{ClipPlane, RenderOptions, render_png, render_rgba};
pub use representation::{Representation, RepresentationSet};
pub use scene::{Atom3D, Bond3D, Mol3dScene, Residue3D};
pub use spatial::SpatialGrid;
pub use surface::{cell_from_vectors, smoothed_surface_grid, surface_grid};
pub use trajectory::{Trajectory, TrajectoryPlayer, VibrationalMode, animate_mode, parse_extended_xyz};
pub use view::{Mol3dResponse, Mol3dStyle, Mol3dView};
pub use volume::{OrbitalKind, SurfaceMesh, VolumeGrid, isosurface, orbital_grid, parse_cube};
