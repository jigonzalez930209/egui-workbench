//! Solvent-accessible and solvent-excluded surface grids.

use egui_workbench_chem_core::UnitCell;

use crate::math::Vec3;
use crate::representation::vdw_radius;
use crate::scene::Mol3dScene;
use crate::volume::VolumeGrid;

/// Rebuild a cell from explicit lattice vectors.
#[must_use]
pub fn cell_from_vectors(vectors: [[f64; 3]; 3]) -> Option<UnitCell> {
    let length = |vector: [f64; 3]| (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    let angle = |a: [f64; 3], b: [f64; 3]| {
        let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        (dot / (length(a) * length(b)).max(1.0e-9))
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    };
    UnitCell::new(
        length(vectors[0]),
        length(vectors[1]),
        length(vectors[2]),
        angle(vectors[1], vectors[2]),
        angle(vectors[0], vectors[2]),
        angle(vectors[0], vectors[1]),
    )
    .ok()
}

/// Build a distance-field grid for molecular surfaces.
///
/// Each value is `distance to the nearest atomic surface minus probe_radius`, so the
/// zero isosurface is the solvent-accessible surface (SAS), and `probe_radius = 0`
/// gives the van der Waals surface.
pub fn surface_grid(scene: &Mol3dScene, probe_radius: f64, resolution: usize, padding: f64) -> VolumeGrid {
    let Some(bounds) = scene.bounds() else {
        return VolumeGrid::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0), [2; 3]);
    };
    let resolution = resolution.clamp(8, 128);
    let padding = padding.max(1.0);
    let min = Vec3::new(bounds[0] - padding, bounds[1] - padding, bounds[2] - padding);
    let max = Vec3::new(bounds[3] + padding, bounds[4] + padding, bounds[5] + padding);
    let size = max - min;
    let step = size.x.max(size.y).max(size.z) / (resolution - 1) as f64;
    let dims = [
        ((size.x / step).ceil() as usize + 1).clamp(2, resolution),
        ((size.y / step).ceil() as usize + 1).clamp(2, resolution),
        ((size.z / step).ceil() as usize + 1).clamp(2, resolution),
    ];
    let mut grid = VolumeGrid::new(min, Vec3::new(step, step, step), dims);
    for z in 0..dims[2] {
        for y in 0..dims[1] {
            for x in 0..dims[0] {
                let point = grid.position(x, y, z);
                let nearest = scene
                    .atoms
                    .iter()
                    .map(|atom| (point - atom.position).length() - vdw_radius(atom.element))
                    .fold(f64::INFINITY, f64::min);
                grid.set(x, y, z, nearest - probe_radius);
            }
        }
    }
    grid
}

/// A smoothed surface grid, approximating the solvent-excluded surface (SES).
pub fn smoothed_surface_grid(scene: &Mol3dScene, probe_radius: f64, resolution: usize, padding: f64) -> VolumeGrid {
    let grid = surface_grid(scene, probe_radius, resolution, padding);
    let [nx, ny, nz] = grid.dimensions;
    let mut smoothed = grid.clone();
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let mut sum = 0.0;
                let mut count = 0.0f64;
                for dz in -1i32..=1 {
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let sx = x as i32 + dx;
                            let sy = y as i32 + dy;
                            let sz = z as i32 + dz;
                            if sx < 0 || sy < 0 || sz < 0 || sx >= nx as i32 || sy >= ny as i32 || sz >= nz as i32 {
                                continue;
                            }
                            sum += grid.at(sx as usize, sy as usize, sz as usize);
                            count += 1.0;
                        }
                    }
                }
                smoothed.set(x, y, z, sum / count.max(1.0));
            }
        }
    }
    smoothed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Mol3dScene;
    use egui_workbench_chem_core::Element;

    #[test]
    fn a_surface_grid_is_negative_inside_and_positive_far_away() {
        let mut scene = Mol3dScene::default();
        scene.push_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let grid = surface_grid(&scene, 1.4, 24, 2.0);
        assert!(grid.sample(Vec3::ZERO) < 0.0);
        assert!(grid.sample(Vec3::new(4.0, 4.0, 4.0)) > 0.0);
        let smoothed = smoothed_surface_grid(&scene, 1.4, 24, 2.0);
        assert_eq!(smoothed.dimensions, grid.dimensions);
    }
}
