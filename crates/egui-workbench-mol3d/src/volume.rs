//! Scalar volumes, marching-tetrahedra isosurfaces, orbitals, and `.cube` files.

use std::collections::HashMap;

use egui_workbench_chem_core::Element;

use crate::math::Vec3;
use crate::scene::Mol3dScene;

/// A regular scalar grid in Angstrom.
#[derive(Clone, Debug, PartialEq)]
pub struct VolumeGrid {
    /// World position of sample `(0, 0, 0)`.
    pub origin: Vec3,
    /// Grid step along each axis in Angstrom.
    pub step: Vec3,
    /// Sample counts along x, y, and z.
    pub dimensions: [usize; 3],
    /// Row-major values, x fastest.
    pub values: Vec<f64>,
}

impl VolumeGrid {
    /// A zero-filled grid.
    pub fn new(origin: Vec3, step: Vec3, dimensions: [usize; 3]) -> Self {
        Self {
            origin,
            step,
            dimensions,
            values: vec![0.0; dimensions[0] * dimensions[1] * dimensions[2]],
        }
    }

    /// The value at integer coordinates.
    pub fn at(&self, x: usize, y: usize, z: usize) -> f64 {
        let index = x + self.dimensions[0] * (y + self.dimensions[1] * z);
        self.values.get(index).copied().unwrap_or(0.0)
    }

    /// Set the value at integer coordinates.
    pub fn set(&mut self, x: usize, y: usize, z: usize, value: f64) {
        let index = x + self.dimensions[0] * (y + self.dimensions[1] * z);
        if let Some(slot) = self.values.get_mut(index) {
            *slot = value;
        }
    }

    /// The world position of integer coordinates.
    pub fn position(&self, x: usize, y: usize, z: usize) -> Vec3 {
        Vec3::new(
            self.origin.x + x as f64 * self.step.x,
            self.origin.y + y as f64 * self.step.y,
            self.origin.z + z as f64 * self.step.z,
        )
    }

    /// Trilinear interpolation at a world position.
    pub fn sample(&self, point: Vec3) -> f64 {
        let coordinate = |value: f64, origin: f64, step: f64, count: usize| {
            let raw = (value - origin) / step;
            raw.clamp(0.0, (count - 1) as f64)
        };
        let x = coordinate(point.x, self.origin.x, self.step.x, self.dimensions[0]);
        let y = coordinate(point.y, self.origin.y, self.step.y, self.dimensions[1]);
        let z = coordinate(point.z, self.origin.z, self.step.z, self.dimensions[2]);
        let (x0, y0, z0) = (x.floor() as usize, y.floor() as usize, z.floor() as usize);
        let x1 = (x0 + 1).min(self.dimensions[0] - 1);
        let y1 = (y0 + 1).min(self.dimensions[1] - 1);
        let z1 = (z0 + 1).min(self.dimensions[2] - 1);
        let (fx, fy, fz) = (x - x0 as f64, y - y0 as f64, z - z0 as f64);
        let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
        let c00 = lerp(self.at(x0, y0, z0), self.at(x1, y0, z0), fx);
        let c10 = lerp(self.at(x0, y1, z0), self.at(x1, y1, z0), fx);
        let c01 = lerp(self.at(x0, y0, z1), self.at(x1, y0, z1), fx);
        let c11 = lerp(self.at(x0, y1, z1), self.at(x1, y1, z1), fx);
        lerp(lerp(c00, c10, fy), lerp(c01, c11, fy), fz)
    }

    /// The gradient at a world position, in value per Angstrom.
    pub fn gradient(&self, point: Vec3) -> Vec3 {
        let h = 0.25;
        Vec3::new(
            (self.sample(point + Vec3::new(h, 0.0, 0.0)) - self.sample(point - Vec3::new(h, 0.0, 0.0))) / (2.0 * h),
            (self.sample(point + Vec3::new(0.0, h, 0.0)) - self.sample(point - Vec3::new(0.0, h, 0.0))) / (2.0 * h),
            (self.sample(point + Vec3::new(0.0, 0.0, h)) - self.sample(point - Vec3::new(0.0, 0.0, h))) / (2.0 * h),
        )
    }
}

/// A triangle surface with per-vertex normals.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SurfaceMesh {
    /// Vertex positions.
    pub vertices: Vec<Vec3>,
    /// Vertex normals.
    pub normals: Vec<Vec3>,
    /// Counter-clockwise triangles.
    pub triangles: Vec<[u32; 3]>,
}

/// Extract the isosurface at `isovalue` with marching tetrahedra.
///
/// Each cube is split into six tetrahedra; every tetrahedron whose vertices straddle
/// the isovalue contributes one or two triangles. This is the tetrahedral variant of
/// marching cubes: it has no ambiguous cases and produces a watertight surface.
pub fn isosurface(grid: &VolumeGrid, isovalue: f64) -> SurfaceMesh {
    let mut mesh = SurfaceMesh::default();
    let mut lookup = HashMap::<(i64, i64, i64), u32>::new();
    let [nx, ny, nz] = grid.dimensions;
    if nx < 2 || ny < 2 || nz < 2 {
        return mesh;
    }
    let corners = [
        (0, 0, 0),
        (1, 0, 0),
        (0, 1, 0),
        (1, 1, 0),
        (0, 0, 1),
        (1, 0, 1),
        (0, 1, 1),
        (1, 1, 1),
    ];
    let tetrahedra = [
        [0, 1, 3, 7],
        [0, 3, 2, 7],
        [0, 2, 6, 7],
        [0, 6, 4, 7],
        [0, 4, 5, 7],
        [0, 5, 1, 7],
    ];
    for z in 0..nz - 1 {
        for y in 0..ny - 1 {
            for x in 0..nx - 1 {
                let positions = corners.map(|(dx, dy, dz)| grid.position(x + dx, y + dy, z + dz));
                let values = corners.map(|(dx, dy, dz)| grid.at(x + dx, y + dy, z + dz));
                for tetrahedron in &tetrahedra {
                    let mut inside = Vec::new();
                    let mut outside = Vec::new();
                    for index in tetrahedron {
                        if values[*index] < isovalue {
                            inside.push(*index);
                        } else {
                            outside.push(*index);
                        }
                    }
                    if inside.is_empty() || outside.is_empty() {
                        continue;
                    }
                    let mut crossings = Vec::new();
                    for a in &inside {
                        for b in &outside {
                            let (pa, pb) = (positions[*a], positions[*b]);
                            let (va, vb) = (values[*a], values[*b]);
                            let t = ((isovalue - va) / (vb - va)).clamp(0.0, 1.0);
                            crossings.push(pa.lerp(pb, t));
                        }
                    }
                    if crossings.len() < 3 {
                        continue;
                    }
                    let center =
                        crossings.iter().fold(Vec3::ZERO, |sum, point| sum + *point) * (1.0 / crossings.len() as f64);
                    let normal = grid.gradient(center).normalized();
                    let mut ordered = crossings.clone();
                    ordered.sort_by(|left, right| {
                        let a = (*left - center).normalized();
                        let b = (*right - center).normalized();
                        a.cross(b).dot(normal).total_cmp(&0.0).reverse()
                    });
                    let indices = ordered
                        .iter()
                        .map(|point| vertex_index(&mut mesh, &mut lookup, *point, grid))
                        .collect::<Vec<_>>();
                    for index in 1..indices.len() - 1 {
                        mesh.triangles.push([indices[0], indices[index], indices[index + 1]]);
                    }
                }
            }
        }
    }
    mesh
}

fn vertex_index(
    mesh: &mut SurfaceMesh,
    lookup: &mut HashMap<(i64, i64, i64), u32>,
    point: Vec3,
    grid: &VolumeGrid,
) -> u32 {
    let key = (
        (point.x * 1000.0).round() as i64,
        (point.y * 1000.0).round() as i64,
        (point.z * 1000.0).round() as i64,
    );
    if let Some(index) = lookup.get(&key) {
        return *index;
    }
    let index = mesh.vertices.len() as u32;
    mesh.vertices.push(point);
    let normal = grid.gradient(point).normalized();
    mesh.normals.push(if normal.length() == 0.0 {
        Vec3::new(0.0, 0.0, 1.0)
    } else {
        normal
    });
    lookup.insert(key, index);
    index
}

/// A real hydrogen-like orbital.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OrbitalKind {
    /// 1s.
    OneS,
    /// 2s.
    TwoS,
    /// 2p_x.
    TwoPx,
    /// 2p_y.
    TwoPy,
    /// 2p_z.
    TwoPz,
    /// 3d_z².
    ThreeDz2,
    /// 3d_xz.
    ThreeDxz,
    /// 3d_yz.
    ThreeDyz,
    /// 3d_xy.
    ThreeDxy,
    /// 3d_x²−y².
    ThreeDx2y2,
}

impl OrbitalKind {
    /// All orbitals, in menu order.
    pub const ALL: [Self; 10] = [
        Self::OneS,
        Self::TwoS,
        Self::TwoPx,
        Self::TwoPy,
        Self::TwoPz,
        Self::ThreeDz2,
        Self::ThreeDxz,
        Self::ThreeDyz,
        Self::ThreeDxy,
        Self::ThreeDx2y2,
    ];

    /// A short label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::OneS => "1s",
            Self::TwoS => "2s",
            Self::TwoPx => "2p_x",
            Self::TwoPy => "2p_y",
            Self::TwoPz => "2p_z",
            Self::ThreeDz2 => "3d_z²",
            Self::ThreeDxz => "3d_xz",
            Self::ThreeDyz => "3d_yz",
            Self::ThreeDxy => "3d_xy",
            Self::ThreeDx2y2 => "3d_x²−y²",
        }
    }

    /// The radial-angular value at `offset` from the nucleus.
    pub fn evaluate(self, offset: Vec3) -> f64 {
        let radius = offset.length().max(1.0e-6);
        let (x, y, z) = (offset.x / radius, offset.y / radius, offset.z / radius);
        match self {
            Self::OneS => (-radius).exp(),
            Self::TwoS => (2.0 - radius) * (-radius * 0.5).exp(),
            Self::TwoPx => radius * (-radius * 0.5).exp() * x,
            Self::TwoPy => radius * (-radius * 0.5).exp() * y,
            Self::TwoPz => radius * (-radius * 0.5).exp() * z,
            Self::ThreeDz2 => radius.powi(2) * (-radius / 3.0).exp() * (3.0 * z * z - 1.0),
            Self::ThreeDxz => radius.powi(2) * (-radius / 3.0).exp() * x * z,
            Self::ThreeDyz => radius.powi(2) * (-radius / 3.0).exp() * y * z,
            Self::ThreeDxy => radius.powi(2) * (-radius / 3.0).exp() * x * y,
            Self::ThreeDx2y2 => radius.powi(2) * (-radius / 3.0).exp() * (x * x - y * y),
        }
    }
}

/// Sample a hydrogen-like orbital into a cubic grid.
pub fn orbital_grid(kind: OrbitalKind, center: Vec3, extent: f64, resolution: usize) -> VolumeGrid {
    let resolution = resolution.max(8);
    let step = 2.0 * extent / (resolution - 1) as f64;
    let origin = center - Vec3::new(extent, extent, extent);
    let mut grid = VolumeGrid::new(origin, Vec3::new(step, step, step), [resolution; 3]);
    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                let point = grid.position(x, y, z);
                grid.set(x, y, z, kind.evaluate(point - center));
            }
        }
    }
    grid
}

/// Read a Gaussian `.cube` volume and its molecule.
///
/// Cube coordinates are in Bohr and are converted to Angstrom.
///
/// # Errors
/// When the header or the value block is truncated.
pub fn parse_cube(text: &str) -> Result<(VolumeGrid, Mol3dScene), String> {
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() < 6 {
        return Err("short cube file".to_owned());
    }
    let header = lines[2].split_whitespace().collect::<Vec<_>>();
    let atom_count = header
        .first()
        .and_then(|value| value.parse::<i32>().ok())
        .ok_or_else(|| "bad cube atom count".to_owned())?;
    let origin = Vec3::new(
        header
            .get(1)
            .and_then(|value| value.parse::<f64>().ok())
            .ok_or_else(|| "bad cube origin".to_owned())?,
        header
            .get(2)
            .and_then(|value| value.parse::<f64>().ok())
            .ok_or_else(|| "bad cube origin".to_owned())?,
        header
            .get(3)
            .and_then(|value| value.parse::<f64>().ok())
            .ok_or_else(|| "bad cube origin".to_owned())?,
    );
    let bohr = 0.529_177_210_92;
    let mut step = [0.0; 3];
    let mut dimensions = [0usize; 3];
    for axis in 0..3 {
        let fields = lines[3 + axis].split_whitespace().collect::<Vec<_>>();
        dimensions[axis] = fields
            .first()
            .and_then(|value| value.parse::<usize>().ok())
            .ok_or_else(|| "bad cube dimension".to_owned())?;
        step[axis] = fields
            .get(1 + axis)
            .and_then(|value| value.parse::<f64>().ok())
            .ok_or_else(|| "bad cube step".to_owned())?
            * bohr;
    }
    let mut scene = Mol3dScene::default();
    for index in 0..atom_count.max(0) as usize {
        let line = lines.get(6 + index).ok_or_else(|| "truncated cube atoms".to_owned())?;
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let atomic_number = fields.first().and_then(|value| value.parse::<u8>().ok()).unwrap_or(0);
        let element = Element::new(atomic_number).unwrap_or(Element::UNKNOWN);
        let coordinate = |index: usize| {
            fields
                .get(2 + index)
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or(0.0)
                * bohr
        };
        scene.push_atom(element, [coordinate(0), coordinate(1), coordinate(2)]);
    }
    let mut grid = VolumeGrid::new(
        Vec3::new(origin.x * bohr, origin.y * bohr, origin.z * bohr),
        Vec3::new(step[0], step[1], step[2]),
        dimensions,
    );
    let mut values = lines
        .iter()
        .skip(6 + atom_count.max(0) as usize)
        .flat_map(|line| line.split_whitespace())
        .filter_map(|value| value.parse::<f64>().ok());
    for value in &mut grid.values {
        *value = values.next().ok_or_else(|| "truncated cube values".to_owned())?;
    }
    scene.infer_bonds();
    Ok((grid, scene))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sphere_field_produces_a_closed_surface() {
        let resolution = 24;
        let step = 4.0 / (resolution - 1) as f64;
        let mut grid = VolumeGrid::new(
            Vec3::new(-2.0, -2.0, -2.0),
            Vec3::new(step, step, step),
            [resolution; 3],
        );
        for z in 0..resolution {
            for y in 0..resolution {
                for x in 0..resolution {
                    let point = grid.position(x, y, z);
                    grid.set(x, y, z, 1.0 - point.length());
                }
            }
        }
        let mesh = isosurface(&grid, 0.0);
        assert!(!mesh.triangles.is_empty());
        assert!(mesh.vertices.iter().all(|vertex| (vertex.length() - 1.0).abs() < 0.25));
        assert!(mesh.normals.iter().all(|normal| normal.is_finite()));
    }

    #[test]
    fn orbitals_have_the_expected_nodal_structure() {
        let grid = orbital_grid(OrbitalKind::TwoPz, Vec3::ZERO, 8.0, 24);
        let top = grid.sample(Vec3::new(0.0, 0.0, 2.0));
        let bottom = grid.sample(Vec3::new(0.0, 0.0, -2.0));
        assert!(top > 0.0 && bottom < 0.0);
        let one_s = OrbitalKind::OneS.evaluate(Vec3::ZERO);
        assert!(one_s > 0.9);
    }

    #[test]
    fn a_minimal_cube_round_trips() {
        let text = "cube\norbital\n 1 0.0 0.0 0.0\n 2 1.0 0.0 0.0\n 2 0.0 1.0 0.0\n 2 0.0 0.0 1.0\n1 0.0 0.0 0.0 0.0\n1.0 1.0 1.0 1.0\n1.0 1.0 1.0 1.0\n";
        let (grid, scene) = parse_cube(text).expect("cube");
        assert_eq!(scene.atoms.len(), 1);
        assert_eq!(grid.dimensions, [2, 2, 2]);
        assert!(grid.values.iter().all(|value| *value == 1.0));
    }
}
