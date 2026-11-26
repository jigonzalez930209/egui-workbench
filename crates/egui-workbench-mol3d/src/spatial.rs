//! Spatial blocks for frustum culling and level-of-detail decisions.

use std::collections::BTreeMap;

use egui::Rect;

use crate::camera::Camera3D;
use crate::math::Vec3;
use crate::scene::Mol3dScene;

/// A uniform grid of atom indices.
#[derive(Clone, Debug, Default)]
pub struct SpatialGrid {
    /// Cell edge length in Angstrom.
    pub cell_size: f64,
    /// World position of the `(0, 0, 0)` cell corner.
    pub origin: Vec3,
    /// Atom indices per cell.
    pub cells: BTreeMap<(i32, i32, i32), Vec<usize>>,
}

impl SpatialGrid {
    /// Build a grid with approximately `cell_size` edges.
    pub fn build(scene: &Mol3dScene, cell_size: f64) -> Self {
        let cell_size = cell_size.max(0.5);
        let mut grid = Self {
            cell_size,
            origin: Vec3::ZERO,
            cells: BTreeMap::new(),
        };
        for (index, atom) in scene.atoms.iter().enumerate() {
            grid.cells.entry(grid.key(atom.position)).or_default().push(index);
        }
        grid
    }

    /// The cell key for a world position.
    pub fn key(&self, position: Vec3) -> (i32, i32, i32) {
        let coordinate = |value: f64| ((value - self.origin.x) / self.cell_size).floor() as i32;
        (coordinate(position.x), coordinate(position.y), coordinate(position.z))
    }

    /// Atom indices whose cells intersect the camera frustum and viewport.
    pub fn visible(&self, scene: &Mol3dScene, camera: Camera3D, viewport: Rect) -> Vec<usize> {
        let mut indices = Vec::new();
        let expanded = viewport.expand(24.0);
        for cell in self.cells.values() {
            let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
            let mut max = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
            for index in cell {
                let position = scene.atoms[*index].position;
                min = Vec3::new(min.x.min(position.x), min.y.min(position.y), min.z.min(position.z));
                max = Vec3::new(max.x.max(position.x), max.y.max(position.y), max.z.max(position.z));
            }
            let center = (min + max) * 0.5;
            let radius = (max - min).length() * 0.5;
            let Some((screen, depth)) = camera.project(center, viewport) else {
                continue;
            };
            if depth <= 0.0 {
                continue;
            }
            let projected = camera.project_radius(center, radius, viewport);
            let closest = egui::pos2(
                screen.x.clamp(expanded.left(), expanded.right()),
                screen.y.clamp(expanded.top(), expanded.bottom()),
            );
            if expanded.contains(screen) || (screen - closest).length() <= projected {
                indices.extend(cell.iter().copied());
            }
        }
        indices
    }
}

/// Points at or below this projected radius use the cheap LOD path.
pub const LOD_POINT_RADIUS: f64 = 1.2;

/// The number of atoms above which culling and LOD activate.
pub const CULLING_THRESHOLD: usize = 2_000;

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;
    use egui_workbench_chem_core::Element;

    #[test]
    fn culling_keeps_front_atoms_and_drops_atoms_behind_the_camera() {
        let mut scene = Mol3dScene::default();
        scene.push_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        scene.push_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 100.0]);
        let camera = Camera3D::default();
        let viewport = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        let grid = SpatialGrid::build(&scene, 4.0);
        let visible = grid.visible(&scene, camera, viewport);
        assert!(visible.contains(&0));
        assert!(!visible.contains(&1));
    }
}
