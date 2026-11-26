//! A compact, serializable scene descriptor for remote synchronization.

use egui_workbench_chem_core::{AtomId, BondId, UnitCell};
use serde::{Deserialize, Serialize};

use crate::camera::Camera3D;
use crate::color::ColorScheme;
use crate::measure::Measurement;
use crate::representation::Representation;
use crate::scene::Mol3dScene;
use crate::view::Mol3dView;

/// Everything a remote client needs to reproduce a 3D view without streaming video.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneDescriptor {
    /// Number of atoms in the local scene.
    pub atom_count: usize,
    /// Number of bonds in the local scene.
    pub bond_count: usize,
    /// Stable hash of atom elements and quantized coordinates.
    pub geometric_hash: u64,
    /// Crystal cell, when the scene is periodic.
    pub cell: Option<UnitCell>,
    /// Orbit camera.
    pub camera: Camera3D,
    /// Default representation.
    pub representation: Representation,
    /// Color scheme.
    pub color_scheme: ColorScheme,
    /// Selected atom ids.
    pub selection_atoms: Vec<AtomId>,
    /// Selected bond ids.
    pub selection_bonds: Vec<BondId>,
    /// Camera-target measurements already attached to the view.
    pub measurements: Vec<Measurement>,
    /// Number of displayed coordination polyhedra.
    pub polyhedra: usize,
    /// Displayed Miller planes.
    pub miller_planes: Vec<(i32, i32, i32)>,
}

impl SceneDescriptor {
    /// Capture the current state of `view`.
    pub fn capture(view: &Mol3dView) -> Self {
        Self {
            atom_count: view.scene.atoms.len(),
            bond_count: view.scene.bonds.len(),
            geometric_hash: geometric_hash(&view.scene),
            cell: view.scene.cell,
            camera: view.camera,
            representation: view.representations.default,
            color_scheme: view.color_scheme,
            selection_atoms: view.selection.atoms().iter().copied().collect(),
            selection_bonds: view.selection.bonds().iter().copied().collect(),
            measurements: view.measurements.clone(),
            polyhedra: view.polyhedra.len(),
            miller_planes: view.miller_planes.clone(),
        }
    }

    /// Whether this descriptor still matches `scene` exactly.
    pub fn matches(&self, scene: &Mol3dScene) -> bool {
        self.atom_count == scene.atoms.len()
            && self.bond_count == scene.bonds.len()
            && self.cell == scene.cell
            && self.geometric_hash == geometric_hash(scene)
    }
}

/// Hash atoms and bonds after quantizing coordinates to milliangstrom.
pub fn geometric_hash(scene: &Mol3dScene) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut mix = |value: u64| {
        hash ^= value;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    };
    for atom in &scene.atoms {
        mix(u64::from(atom.element.atomic_number()));
        for axis in [atom.position.x, atom.position.y, atom.position.z] {
            mix((axis * 1000.0).round() as i64 as u64);
        }
    }
    for bond in &scene.bonds {
        mix(u64::from(bond.a.slot()));
        mix(u64::from(bond.b.slot()));
        mix(u64::from(bond.order.strokes()));
    }
    mix(scene.atoms.len() as u64);
    mix(scene.bonds.len() as u64);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::Element;

    #[test]
    fn the_hash_is_stable_and_sensitive_to_geometry() {
        let mut scene = Mol3dScene::default();
        scene.push_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let first = geometric_hash(&scene);
        assert_eq!(first, geometric_hash(&scene));
        scene.atoms[0].position.x = 0.01;
        assert_ne!(first, geometric_hash(&scene));
    }

    #[test]
    fn a_descriptor_round_trips_and_matches() {
        let mut scene = Mol3dScene::default();
        scene.push_atom(Element::from_symbol("O").expect("O"), [1.0, 2.0, 3.0]);
        let view = Mol3dView::new(scene.clone());
        let descriptor = SceneDescriptor::capture(&view);
        let json = serde_json::to_string(&descriptor).expect("serialize");
        let restored: SceneDescriptor = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.matches(&scene));
    }
}
