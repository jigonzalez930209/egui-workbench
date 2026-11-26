//! The atoms, bonds, and crystal cell drawn by the 3D viewport.

use egui::{Pos2, Rect};
use egui_workbench_chem_core::{AtomId, BondId, BondOrder, Element, Molecule, UnitCell};

use crate::camera::{Camera3D, Ray};
use crate::math::Vec3;

/// Residue metadata read from a biomolecular file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Residue3D {
    /// Residue name, such as `ALA` or `HOH`.
    pub name: String,
    /// Chain identifier.
    pub chain: String,
    /// Residue sequence number.
    pub number: i32,
}

/// One atom in the 3D scene.
#[derive(Clone, Debug, PartialEq)]
pub struct Atom3D {
    /// Stable id from the shared molecule.
    pub id: AtomId,
    /// Element.
    pub element: Element,
    /// Position in Angstrom.
    pub position: Vec3,
    /// Display and picking radius in Angstrom.
    pub radius: f64,
    /// Optional atom label, such as `CA` or `O1`.
    pub label: Option<String>,
    /// Optional residue metadata.
    pub residue: Option<Residue3D>,
    /// Partial charge in elementary charge units.
    pub charge: f64,
    /// Temperature factor in square Angstrom.
    pub b_factor: f64,
    /// Occupancy in `[0, 1]`.
    pub occupancy: f64,
}

impl Atom3D {
    /// An atom with default metadata.
    pub fn new(id: AtomId, element: Element, position: Vec3, radius: f64) -> Self {
        Self {
            id,
            element,
            position,
            radius,
            label: None,
            residue: None,
            charge: 0.0,
            b_factor: 0.0,
            occupancy: 1.0,
        }
    }
}

/// One bond in the 3D scene.
#[derive(Clone, Debug, PartialEq)]
pub struct Bond3D {
    /// Stable id from the shared molecule.
    pub id: BondId,
    /// First atom.
    pub a: AtomId,
    /// Second atom.
    pub b: AtomId,
    /// Bond order.
    pub order: BondOrder,
}

/// A renderable molecular scene.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mol3dScene {
    /// Atoms in molecule order.
    pub atoms: Vec<Atom3D>,
    /// Bonds in molecule order.
    pub bonds: Vec<Bond3D>,
    /// Optional crystal cell.
    pub cell: Option<UnitCell>,
}

impl Mol3dScene {
    /// Build a scene from the shared molecule.
    pub fn from_molecule(molecule: &Molecule) -> Self {
        let atoms = molecule
            .atoms
            .iter()
            .map(|atom| {
                Atom3D::new(
                    atom.id,
                    atom.element,
                    Vec3::new(atom.position[0], atom.position[1], atom.position[2]),
                    (atom.element.covalent_radius() * 1.2).max(0.4),
                )
            })
            .collect();
        let bonds = molecule
            .bonds
            .iter()
            .map(|bond| Bond3D {
                id: bond.id,
                a: bond.a,
                b: bond.b,
                order: bond.order,
            })
            .collect();
        Self {
            atoms,
            bonds,
            cell: None,
        }
    }

    /// Attach a crystal cell to the scene.
    pub fn with_cell(mut self, cell: UnitCell) -> Self {
        self.cell = Some(cell);
        self
    }

    /// The atom with `id`.
    pub fn atom(&self, id: AtomId) -> Option<&Atom3D> {
        self.atoms.iter().find(|atom| atom.id == id)
    }

    /// The bounding box `[min_x, min_y, min_z, max_x, max_y, max_z]`.
    pub fn bounds(&self) -> Option<[f64; 6]> {
        let mut bounds: Option<[f64; 6]> = None;
        for atom in &self.atoms {
            let [x, y, z] = [atom.position.x, atom.position.y, atom.position.z];
            if !atom.position.is_finite() {
                continue;
            }
            bounds = Some(match bounds {
                Some([min_x, min_y, min_z, max_x, max_y, max_z]) => [
                    min_x.min(x),
                    min_y.min(y),
                    min_z.min(z),
                    max_x.max(x),
                    max_y.max(y),
                    max_z.max(z),
                ],
                None => [x, y, z, x, y, z],
            });
        }
        bounds
    }

    /// The crystal cell edges as world-space segments.
    pub fn cell_edges(&self) -> Vec<[Vec3; 2]> {
        let Some(cell) = self.cell else {
            return Vec::new();
        };
        let corners = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ];
        let points = corners
            .iter()
            .map(|corner| {
                let point = cell.fractional_to_cartesian(*corner);
                Vec3::new(point[0], point[1], point[2])
            })
            .collect::<Vec<_>>();
        let edges = [
            (0, 1),
            (0, 2),
            (1, 3),
            (2, 3),
            (4, 5),
            (4, 6),
            (5, 7),
            (6, 7),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        edges.iter().map(|(a, b)| [points[*a], points[*b]]).collect()
    }

    /// The atom under `pointer`, using a ray-sphere test.
    pub fn pick(&self, camera: Camera3D, viewport: Rect, pointer: Pos2) -> Option<AtomId> {
        let ray = camera.ray(pointer, viewport);
        let mut best: Option<(f64, AtomId)> = None;
        for atom in &self.atoms {
            if let Some(distance) = ray_sphere(ray, atom.position, atom.radius * 1.15)
                && best.is_none_or(|(current, _)| distance < current)
            {
                best = Some((distance, atom.id));
            }
        }
        best.map(|(_, id)| id)
    }

    /// The bond nearest to `pointer` within `tolerance` points, using the projected
    /// endpoints and the nearer depth for ties.
    pub fn pick_bond(&self, camera: Camera3D, viewport: Rect, pointer: Pos2, tolerance: f32) -> Option<BondId> {
        let mut best: Option<(f32, f64, BondId)> = None;
        for bond in &self.bonds {
            let (Some(a), Some(b)) = (self.atom(bond.a), self.atom(bond.b)) else {
                continue;
            };
            let (Some((start, depth_a)), Some((end, depth_b))) = (
                camera.project(a.position, viewport),
                camera.project(b.position, viewport),
            ) else {
                continue;
            };
            let segment = end - start;
            let length_squared = segment.length_sq();
            let t = if length_squared <= f32::EPSILON {
                0.0
            } else {
                ((pointer - start).dot(segment) / length_squared).clamp(0.0, 1.0)
            };
            let closest = start + segment * t;
            let distance = (pointer - closest).length();
            let depth = depth_a + (depth_b - depth_a) * f64::from(t);
            if distance <= tolerance
                && best.is_none_or(|(current, current_depth, _)| {
                    distance < current || (distance == current && depth < current_depth)
                })
            {
                best = Some((distance, depth, bond.id));
            }
        }
        best.map(|(_, _, id)| id)
    }

    /// Add one atom and return its id.
    pub fn push_atom(&mut self, element: Element, position: [f64; 3]) -> AtomId {
        let slot = self.atoms.len() as u32 + 1;
        let id = AtomId::new(slot, 1);
        self.atoms.push(Atom3D::new(
            id,
            element,
            Vec3::new(position[0], position[1], position[2]),
            (element.covalent_radius() * 0.42).max(0.25),
        ));
        id
    }

    /// Infer single bonds from covalent radii when the file has no bond table.
    pub fn infer_bonds(&mut self) {
        let mut existing = self
            .bonds
            .iter()
            .map(|bond| (bond.a.min(bond.b), bond.a.max(bond.b)))
            .collect::<std::collections::BTreeSet<_>>();
        for left in 0..self.atoms.len() {
            for right in (left + 1)..self.atoms.len() {
                let (a, b) = (&self.atoms[left], &self.atoms[right]);
                let pair = (a.id.min(b.id), a.id.max(b.id));
                if existing.contains(&pair) {
                    continue;
                }
                let max = (a.element.covalent_radius() + b.element.covalent_radius()) * 1.25;
                if (a.position - b.position).length() <= max {
                    let id = BondId::new(self.bonds.len() as u32 + 1, 1);
                    self.bonds.push(Bond3D {
                        id,
                        a: a.id,
                        b: b.id,
                        order: BondOrder::Single,
                    });
                    existing.insert(pair);
                }
            }
        }
    }
}

fn ray_sphere(ray: Ray, center: Vec3, radius: f64) -> Option<f64> {
    let offset = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * offset.dot(ray.direction);
    let c = offset.dot(offset) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }
    let root = discriminant.sqrt();
    let near = (-b - root) / (2.0 * a);
    let far = (-b + root) / (2.0 * a);
    if near > 0.0 {
        Some(near)
    } else if far > 0.0 {
        Some(far)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Rect, pos2, vec2};
    use egui_workbench_chem_core::DocumentId;

    fn water() -> Molecule {
        let mut document = egui_workbench_chem_core::ChemDocument::new(DocumentId::new(1), "water");
        let oxygen = document
            .molecule
            .add_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
        let h1 = document
            .molecule
            .add_atom(Element::from_symbol("H").expect("H"), [0.96, 0.0, 0.0]);
        let h2 = document
            .molecule
            .add_atom(Element::from_symbol("H").expect("H"), [-0.24, 0.93, 0.0]);
        document.molecule.add_bond(oxygen, h1, BondOrder::Single).expect("bond");
        document.molecule.add_bond(oxygen, h2, BondOrder::Single).expect("bond");
        document.molecule
    }

    #[test]
    fn a_molecule_becomes_atoms_and_bonds() {
        let scene = Mol3dScene::from_molecule(&water());
        assert_eq!(scene.atoms.len(), 3);
        assert_eq!(scene.bonds.len(), 2);
        let bounds = scene.bounds().expect("bounds");
        assert!(bounds[3] >= bounds[0]);
        assert!(scene.atoms[0].radius >= 0.4);
    }

    #[test]
    fn picking_hits_the_atom_under_the_pointer() {
        let scene = Mol3dScene::from_molecule(&water());
        let camera = Camera3D::default();
        let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
        let (screen, _) = camera.project(scene.atoms[0].position, viewport).expect("project");
        assert_eq!(scene.pick(camera, viewport, screen), Some(scene.atoms[0].id));
    }

    #[test]
    fn a_cell_produces_twelve_edges() {
        let cell = UnitCell::new(10.0, 10.0, 10.0, 90.0, 90.0, 90.0).expect("cell");
        let scene = Mol3dScene::from_molecule(&water()).with_cell(cell);
        assert_eq!(scene.cell_edges().len(), 12);
    }
    #[test]
    fn a_bond_can_be_picked_between_its_endpoints() {
        let mut molecule = Molecule::default();
        let a = molecule.add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let b = molecule.add_atom(Element::from_symbol("C").expect("C"), [2.0, 0.0, 0.0]);
        let bond = molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        let scene = Mol3dScene::from_molecule(&molecule);
        let camera = Camera3D::default();
        let viewport = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        let (start, _) = camera.project(Vec3::ZERO, viewport).expect("start");
        let (end, _) = camera.project(Vec3::new(2.0, 0.0, 0.0), viewport).expect("end");
        let middle = Pos2::new((start.x + end.x) * 0.5, (start.y + end.y) * 0.5);
        assert_eq!(scene.pick_bond(camera, viewport, middle, 6.0), Some(bond));
    }
}
