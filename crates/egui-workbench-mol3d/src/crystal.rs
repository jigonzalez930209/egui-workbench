//! Periodic cells, supercells, Miller planes, and coordination polyhedra.

use egui_workbench_chem_core::{AtomId, BondId, UnitCell};

use crate::math::Vec3;
use crate::representation::vdw_radius;
use crate::scene::{Bond3D, Mol3dScene};

/// Direct and reciprocal cell vectors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellAxes {
    /// The `a` vector.
    pub a: Vec3,
    /// The `b` vector.
    pub b: Vec3,
    /// The `c` vector.
    pub c: Vec3,
    /// The reciprocal vectors without the `2π` factor.
    pub reciprocal: [Vec3; 3],
}

impl CellAxes {
    /// Build the direct and reciprocal vectors of `cell`.
    pub fn new(cell: UnitCell) -> Self {
        let vector = |fractional: [f64; 3]| {
            let point = cell.fractional_to_cartesian(fractional);
            Vec3::new(point[0], point[1], point[2])
        };
        let a = vector([1.0, 0.0, 0.0]);
        let b = vector([0.0, 1.0, 0.0]);
        let c = vector([0.0, 0.0, 1.0]);
        let volume = a.dot(b.cross(c));
        let reciprocal = if volume.abs() <= 1.0e-9 {
            [Vec3::ZERO; 3]
        } else {
            [
                b.cross(c) * (1.0 / volume),
                c.cross(a) * (1.0 / volume),
                a.cross(b) * (1.0 / volume),
            ]
        };
        Self { a, b, c, reciprocal }
    }

    /// The reciprocal `a*`, `b*`, and `c*` vectors.
    pub fn reciprocal_vectors(&self) -> [Vec3; 3] {
        self.reciprocal
    }
}

/// A coordination polyhedron around a metal center.
#[derive(Clone, Debug, PartialEq)]
pub struct Polyhedron {
    /// The central atom.
    pub center: AtomId,
    /// World position of the center.
    pub center_position: Vec3,
    /// Neighbor atom ids, one per vertex.
    pub vertices: Vec<AtomId>,
    /// World positions of the vertices.
    pub vertex_positions: Vec<Vec3>,
    /// Edges as vertex-index pairs.
    pub edges: Vec<(usize, usize)>,
    /// Triangular faces as vertex indices, counter-clockwise when seen from outside.
    pub faces: Vec<[usize; 3]>,
}

impl Polyhedron {
    /// A descriptive name from the vertex count.
    pub fn shape_name(&self) -> &'static str {
        match self.vertices.len() {
            3 => "trigonal",
            4 => "tetrahedral",
            5 => "trigonal bipyramidal",
            6 => "octahedral",
            7 => "pentagonal bipyramidal",
            8 => "square antiprismatic",
            12 => "icosahedral",
            _ => "coordination",
        }
    }
}

/// Most copies of the cell along one axis in a supercell.
const MAX_REPEAT: usize = 32;

/// Most atoms a supercell may hold; a larger request returns the scene unchanged.
const MAX_SUPERCELL_ATOMS: usize = 2_000_000;

/// Most grid points sampled by `pore_points`.
const MAX_PORE_SAMPLES: f64 = 1_000_000.0;

/// Most neighbours of one centre that enter a coordination polyhedron. The hull search is
/// quartic in the neighbour count, and no real centre has more than twelve to sixteen.
const MAX_COORDINATION: usize = 16;

impl Mol3dScene {
    /// Expand the scene into an `na × nb × nc` supercell.
    #[must_use]
    pub fn supercell(&self, na: usize, nb: usize, nc: usize) -> Self {
        let (na, nb, nc) = (
            na.clamp(1, MAX_REPEAT),
            nb.clamp(1, MAX_REPEAT),
            nc.clamp(1, MAX_REPEAT),
        );
        let Some(cell) = self.cell else {
            return self.clone();
        };
        // 32 x 32 x 32 copies of a large cell would be tens of millions of atoms.
        if na
            .saturating_mul(nb)
            .saturating_mul(nc)
            .saturating_mul(self.atoms.len())
            > MAX_SUPERCELL_ATOMS
        {
            return self.clone();
        }
        let axes = CellAxes::new(cell);
        let shift = |i: usize, j: usize, k: usize| axes.a * i as f64 + axes.b * j as f64 + axes.c * k as f64;
        let mut scene = Mol3dScene::default().with_cell(UnitCell {
            a: cell.a * na as f64,
            b: cell.b * nb as f64,
            c: cell.c * nc as f64,
            ..cell
        });
        let mut slot = 1u32;
        for i in 0..na {
            for j in 0..nb {
                for k in 0..nc {
                    let offset = shift(i, j, k);
                    let mut map = std::collections::BTreeMap::new();
                    for atom in &self.atoms {
                        let id = AtomId::new(slot, 1);
                        slot += 1;
                        let mut copy = atom.clone();
                        copy.id = id;
                        copy.position = atom.position + offset;
                        scene.atoms.push(copy);
                        map.insert(atom.id, id);
                    }
                    for bond in &self.bonds {
                        let (Some(a), Some(b)) = (map.get(&bond.a), map.get(&bond.b)) else {
                            continue;
                        };
                        scene.bonds.push(Bond3D {
                            id: BondId::new(scene.bonds.len() as u32 + 1, 1),
                            a: *a,
                            b: *b,
                            order: bond.order,
                        });
                    }
                }
            }
        }
        scene.infer_bonds();
        scene
    }

    /// Keep only atoms within `thickness / 2` of the `(hkl)` plane through the cell center.
    #[must_use]
    pub fn slab(&self, h: i32, k: i32, l: i32, thickness: f64) -> Self {
        let Some(cell) = self.cell else {
            return self.clone();
        };
        let axes = CellAxes::new(cell);
        let normal =
            axes.reciprocal[0] * f64::from(h) + axes.reciprocal[1] * f64::from(k) + axes.reciprocal[2] * f64::from(l);
        if normal.length() <= 1.0e-9 {
            return self.clone();
        }
        let normal = normal.normalized();
        let center = (axes.a + axes.b + axes.c) * 0.5;
        let keep = |position: Vec3| (position - center).dot(normal).abs() <= thickness * 0.5;
        let atoms = self
            .atoms
            .iter()
            .filter(|atom| keep(atom.position))
            .cloned()
            .collect::<Vec<_>>();
        let ids = atoms
            .iter()
            .map(|atom| atom.id)
            .collect::<std::collections::BTreeSet<_>>();
        let bonds = self
            .bonds
            .iter()
            .filter(|bond| ids.contains(&bond.a) && ids.contains(&bond.b))
            .cloned()
            .collect();
        Self {
            atoms,
            bonds,
            cell: self.cell,
        }
    }

    /// Mesh-grid points that are at least `probe_radius` away from every atom,
    /// highlighting accessible pores and channels.
    pub fn pore_points(&self, probe_radius: f64, spacing: f64) -> Vec<Vec3> {
        let Some(bounds) = self.bounds() else {
            return Vec::new();
        };
        let mut spacing = spacing.max(0.2);
        // A 100 A box at 0.2 A is 125 million samples, each tested against every atom. Widen
        // the spacing until the grid fits the budget.
        let extent = |axis: usize| (bounds[axis + 3] - bounds[axis]).max(0.0);
        let samples =
            |spacing: f64| (extent(0) / spacing + 1.0) * (extent(1) / spacing + 1.0) * (extent(2) / spacing + 1.0);
        while samples(spacing) > MAX_PORE_SAMPLES && spacing.is_finite() {
            spacing *= 1.25;
        }
        if !spacing.is_finite() || !samples(spacing).is_finite() {
            return Vec::new();
        }
        let mut points = Vec::new();
        let mut x = bounds[0];
        while x <= bounds[3] {
            let mut y = bounds[1];
            while y <= bounds[4] {
                let mut z = bounds[2];
                while z <= bounds[5] {
                    let candidate = Vec3::new(x, y, z);
                    let free = self
                        .atoms
                        .iter()
                        .all(|atom| (candidate - atom.position).length() > vdw_radius(atom.element) + probe_radius);
                    if free {
                        points.push(candidate);
                    }
                    z += spacing;
                }
                y += spacing;
            }
            x += spacing;
        }
        points
    }

    /// Detect the coordination polyhedra around every atom whose symbol is in `metals`.
    pub fn coordination_polyhedra(&self, metals: &[&str], cutoff: f64) -> Vec<Polyhedron> {
        let mut polyhedra = Vec::new();
        for center in &self.atoms {
            if !metals.is_empty()
                && !metals
                    .iter()
                    .any(|symbol| symbol.eq_ignore_ascii_case(center.element.symbol()))
            {
                continue;
            }
            let mut neighbors = self
                .atoms
                .iter()
                .filter(|atom| atom.id != center.id)
                .filter(|atom| (atom.position - center.position).length() <= cutoff)
                .collect::<Vec<_>>();
            neighbors.sort_by(|left, right| {
                (left.position - center.position)
                    .length()
                    .total_cmp(&(right.position - center.position).length())
            });
            neighbors.truncate(MAX_COORDINATION);
            if neighbors.len() < 3 {
                continue;
            }
            let vertices = neighbors.iter().map(|atom| atom.id).collect::<Vec<_>>();
            let points = neighbors.iter().map(|atom| atom.position).collect::<Vec<_>>();
            let faces = convex_hull_faces(&points, center.position);
            if faces.is_empty() {
                continue;
            }
            let mut edges = std::collections::BTreeSet::new();
            for face in &faces {
                for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
                    edges.insert((a.min(b), a.max(b)));
                }
            }
            polyhedra.push(Polyhedron {
                center: center.id,
                center_position: center.position,
                vertices,
                vertex_positions: points,
                edges: edges.into_iter().collect(),
                faces,
            });
        }
        polyhedra
    }
}

/// The four corners of a `(hkl)` plane visualisation, centred in the cell.
pub fn miller_plane(cell: UnitCell, h: i32, k: i32, l: i32) -> Option<[Vec3; 4]> {
    let axes = CellAxes::new(cell);
    let normal =
        axes.reciprocal[0] * f64::from(h) + axes.reciprocal[1] * f64::from(k) + axes.reciprocal[2] * f64::from(l);
    if normal.length() <= 1.0e-9 {
        return None;
    }
    let normal = normal.normalized();
    let center = (axes.a + axes.b + axes.c) * 0.5;
    let center = center - normal * center.dot(normal);
    let reference = if normal.dot(axes.a).abs() < 0.9 { axes.a } else { axes.b };
    let u = (reference - normal * reference.dot(normal)).normalized();
    let v = normal.cross(u);
    let size = axes.a.length().max(axes.b.length()).max(axes.c.length()) * 0.75;
    Some([
        center - u * size - v * size,
        center + u * size - v * size,
        center + u * size + v * size,
        center - u * size + v * size,
    ])
}

/// Triangular faces of the convex hull of `points`, oriented away from `inside`.
fn convex_hull_faces(points: &[Vec3], inside: Vec3) -> Vec<[usize; 3]> {
    let mut faces = Vec::new();
    let count = points.len();
    for i in 0..count {
        for j in (i + 1)..count {
            for k in (j + 1)..count {
                let normal = (points[j] - points[i]).cross(points[k] - points[i]);
                if normal.length() <= 1.0e-8 {
                    continue;
                }
                let oriented = points[i] - inside;
                let sign = normal.dot(oriented).signum();
                if sign == 0.0 {
                    continue;
                }
                let mut coplanar = 0;
                let mut outside = false;
                for (index, point) in points.iter().enumerate() {
                    if index == i || index == j || index == k {
                        continue;
                    }
                    let side = normal.dot(*point - points[i]) * sign;
                    if side > 1.0e-6 {
                        outside = true;
                        break;
                    }
                    if side.abs() <= 1.0e-6 {
                        coplanar += 1;
                    }
                }
                if outside {
                    continue;
                }
                let face = if sign > 0.0 { [i, j, k] } else { [i, k, j] };
                if !faces.contains(&face) && coplanar < 2 {
                    faces.push(face);
                }
            }
        }
    }
    faces
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::Element;

    fn cubic() -> UnitCell {
        UnitCell::new(5.0, 5.0, 5.0, 90.0, 90.0, 90.0).expect("cell")
    }

    #[test]
    fn cell_axes_and_reciprocal_vectors_are_orthogonal_for_a_cube() {
        let axes = CellAxes::new(cubic());
        assert!((axes.a.length() - 5.0).abs() < 1.0e-9);
        assert!(axes.reciprocal[0].dot(axes.a) - 1.0 < 1.0e-9);
        assert!(axes.reciprocal[0].dot(axes.b).abs() < 1.0e-9);
    }

    #[test]
    fn a_supercell_multiplies_the_atom_count() {
        let mut scene = Mol3dScene::default().with_cell(cubic());
        scene.push_atom(Element::from_symbol("Na").expect("Na"), [0.0, 0.0, 0.0]);
        scene.push_atom(Element::from_symbol("Cl").expect("Cl"), [2.5, 2.5, 2.5]);
        let expanded = scene.supercell(2, 2, 2);
        assert_eq!(expanded.atoms.len(), 16);
        assert_eq!(expanded.cell.expect("cell").a, 10.0);
    }

    #[test]
    fn an_octahedron_is_detected_around_a_metal() {
        let mut scene = Mol3dScene::default();
        let center = scene.push_atom(Element::from_symbol("Fe").expect("Fe"), [0.0, 0.0, 0.0]);
        for position in [
            [2.0, 0.0, 0.0],
            [-2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, -2.0, 0.0],
            [0.0, 0.0, 2.0],
            [0.0, 0.0, -2.0],
        ] {
            scene.push_atom(Element::from_symbol("O").expect("O"), position);
        }
        let polyhedra = scene.coordination_polyhedra(&["Fe"], 2.5);
        assert_eq!(polyhedra.len(), 1);
        assert_eq!(polyhedra[0].center, center);
        assert_eq!(polyhedra[0].vertices.len(), 6);
        assert_eq!(polyhedra[0].edges.len(), 12);
        assert_eq!(polyhedra[0].shape_name(), "octahedral");
    }

    #[test]
    fn a_miller_plane_has_four_finite_corners() {
        let plane = miller_plane(cubic(), 1, 0, 0).expect("plane");
        assert!(plane.iter().all(|point| point.is_finite()));
        assert!((plane[1] - plane[0]).length() > 1.0);
    }
}

#[cfg(test)]
mod limit_tests {
    use super::*;
    use egui_workbench_chem_core::Element;

    fn cubic_scene(atoms: usize) -> Mol3dScene {
        let cell = UnitCell::new(5.0, 5.0, 5.0, 90.0, 90.0, 90.0).expect("cell");
        let mut scene = Mol3dScene::default().with_cell(cell);
        for index in 0..atoms {
            scene.push_atom(Element::from_symbol("Na").expect("Na"), [index as f64 * 1.7, 0.0, 0.0]);
        }
        scene
    }

    #[test]
    fn a_huge_supercell_request_is_refused_instead_of_allocating() {
        let scene = cubic_scene(4);
        assert_eq!(
            scene.supercell(usize::MAX, usize::MAX, usize::MAX).atoms.len(),
            4 * 32 * 32 * 32
        );
        let big = cubic_scene(200_000);
        assert_eq!(big.supercell(32, 32, 32).atoms.len(), 200_000, "over budget: unchanged");
    }

    #[test]
    fn pore_sampling_stays_inside_its_budget() {
        let mut scene = Mol3dScene::default();
        scene.push_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        scene.push_atom(Element::from_symbol("C").expect("C"), [100.0, 100.0, 100.0]);
        let started = std::time::Instant::now();
        let points = scene.pore_points(1.2, 0.2);
        assert!(points.len() <= 1_500_000);
        assert!(
            started.elapsed() < std::time::Duration::from_secs(20),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn coordination_hulls_use_at_most_sixteen_neighbours() {
        let mut scene = Mol3dScene::default();
        scene.push_atom(Element::from_symbol("Fe").expect("Fe"), [0.0, 0.0, 0.0]);
        for index in 0..60 {
            let angle = f64::from(index) * 0.7;
            scene.push_atom(
                Element::from_symbol("O").expect("O"),
                [2.0 * angle.cos(), 2.0 * angle.sin(), f64::from(index % 5) - 2.0],
            );
        }
        let polyhedra = scene.coordination_polyhedra(&["Fe"], 4.0);
        assert!(polyhedra.iter().all(|polyhedron| polyhedron.vertices.len() <= 16));
    }
}
