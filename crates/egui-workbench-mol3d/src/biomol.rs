//! Protein backbone traces, cartoon ribbons, and non-covalent interactions.

use std::collections::{BTreeMap, BTreeSet};

use egui_workbench_chem_core::AtomId;

use crate::math::Vec3;
use crate::measure::{centroid, mean_plane_normal};
use crate::representation::vdw_radius;
use crate::scene::{Atom3D, Mol3dScene};

/// Secondary-structure assignment of one residue.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SecondaryStructure {
    /// Alpha helix.
    Helix,
    /// Beta strand.
    Sheet,
    /// Unstructured loop.
    #[default]
    Loop,
}

/// One CA trace, ordered by residue number.
#[derive(Clone, Debug, PartialEq)]
pub struct BackboneTrace {
    /// Chain identifier.
    pub chain: String,
    /// CA positions.
    pub points: Vec<Vec3>,
    /// Residue identifiers matching `points`.
    pub residues: Vec<(String, i32)>,
    /// Per-residue secondary structure.
    pub secondary: Vec<SecondaryStructure>,
}

/// A triangle mesh for a cartoon ribbon.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RibbonMesh {
    /// Vertices.
    pub vertices: Vec<Vec3>,
    /// Triangles as vertex indices.
    pub triangles: Vec<[usize; 3]>,
    /// Per-vertex secondary structure, for coloring.
    pub secondary: Vec<SecondaryStructure>,
}

/// A non-covalent interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum InteractionKind {
    /// N/O/F donor to N/O/F acceptor.
    HydrogenBond,
    /// Parallel aromatic ring stacking.
    PiStacking,
    /// Oppositely charged side chains.
    SaltBridge,
    /// Non-bonded atoms closer than their van der Waals contact.
    ShortContact,
}

impl InteractionKind {
    /// A short label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::HydrogenBond => "H-bond",
            Self::PiStacking => "π–π",
            Self::SaltBridge => "salt bridge",
            Self::ShortContact => "short contact",
        }
    }
}

/// One detected interaction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interaction {
    /// Interaction class.
    pub kind: InteractionKind,
    /// First atom.
    pub a: AtomId,
    /// Second atom.
    pub b: AtomId,
    /// Distance in Angstrom.
    pub distance: f64,
}

/// Extract CA traces from a biomolecular scene.
pub fn backbone_traces(scene: &Mol3dScene) -> Vec<BackboneTrace> {
    let mut chains = BTreeMap::<String, Vec<&Atom3D>>::new();
    for atom in &scene.atoms {
        let is_alpha = atom
            .label
            .as_deref()
            .is_some_and(|label| label.eq_ignore_ascii_case("CA") || label.eq_ignore_ascii_case("Cα"));
        if !is_alpha || atom.residue.is_none() {
            continue;
        }
        let chain = atom
            .residue
            .as_ref()
            .map_or_else(String::new, |residue| residue.chain.clone());
        chains.entry(chain).or_default().push(atom);
    }
    let mut traces = Vec::new();
    for (chain, mut atoms) in chains {
        atoms.sort_by_key(|atom| atom.residue.as_ref().map_or(0, |residue| residue.number));
        atoms.dedup_by_key(|atom| atom.residue.as_ref().map_or(0, |residue| residue.number));
        if atoms.len() < 3 {
            continue;
        }
        let points = atoms.iter().map(|atom| atom.position).collect::<Vec<_>>();
        let residues = atoms
            .iter()
            .map(|atom| {
                atom.residue.as_ref().map_or_else(
                    || ("UNK".to_owned(), 0),
                    |residue| (residue.name.clone(), residue.number),
                )
            })
            .collect::<Vec<_>>();
        let secondary = classify(&points);
        traces.push(BackboneTrace {
            chain,
            points,
            residues,
            secondary,
        });
    }
    traces
}

fn classify(points: &[Vec3]) -> Vec<SecondaryStructure> {
    let mut assignment = vec![SecondaryStructure::Loop; points.len()];
    for index in 1..points.len().saturating_sub(1) {
        let span = (points[index + 1] - points[index - 1]).length();
        assignment[index] = if span < 6.2 {
            SecondaryStructure::Helix
        } else if span > 6.6 {
            SecondaryStructure::Sheet
        } else {
            SecondaryStructure::Loop
        };
    }
    assignment
}

/// Build smooth ribbon strips for every CA trace.
pub fn ribbon_meshes(scene: &Mol3dScene) -> Vec<RibbonMesh> {
    backbone_traces(scene).iter().map(ribbon_mesh).collect()
}

/// Build one ribbon strip with Catmull-Rom smoothing.
pub fn ribbon_mesh(trace: &BackboneTrace) -> RibbonMesh {
    let smoothed = catmull_rom(&trace.points, 6);
    if smoothed.len() < 2 {
        return RibbonMesh::default();
    }
    let mut vertices = Vec::with_capacity(smoothed.len() * 2);
    let mut secondary = Vec::with_capacity(smoothed.len() * 2);
    let mut triangles = Vec::new();
    for (index, point) in smoothed.iter().enumerate() {
        let tangent = if index == 0 {
            smoothed[1] - smoothed[0]
        } else if index + 1 == smoothed.len() {
            smoothed[index] - smoothed[index - 1]
        } else {
            smoothed[index + 1] - smoothed[index - 1]
        }
        .normalized();
        let reference = if tangent.y.abs() < 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let side = tangent.cross(reference).normalized();
        let residue = ((index as f64 / 6.0).round() as usize).min(trace.points.len() - 1);
        let kind = trace.secondary[residue];
        let width = match kind {
            SecondaryStructure::Helix => 0.75,
            SecondaryStructure::Sheet => 1.05,
            SecondaryStructure::Loop => 0.35,
        };
        vertices.push(*point - side * width);
        vertices.push(*point + side * width);
        secondary.extend([kind, kind]);
        if index > 0 {
            let base = index * 2;
            triangles.push([base - 2, base - 1, base]);
            triangles.push([base - 1, base + 1, base]);
        }
    }
    RibbonMesh {
        vertices,
        triangles,
        secondary,
    }
}

fn catmull_rom(points: &[Vec3], subdivisions: usize) -> Vec<Vec3> {
    if points.len() < 2 {
        return points.to_vec();
    }
    let mut result = Vec::new();
    for index in 0..points.len().saturating_sub(1) {
        let p0 = points[index.saturating_sub(1)];
        let p1 = points[index];
        let p2 = points[index + 1];
        let p3 = points[(index + 2).min(points.len() - 1)];
        for step in 0..subdivisions {
            let t = step as f64 / subdivisions as f64;
            let t2 = t * t;
            let t3 = t2 * t;
            let point = (p1 * 2.0
                + (p2 - p0) * t
                + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t2
                + (p1 * 3.0 - p0 - p2 * 3.0 + p3) * t3)
                * 0.5;
            result.push(point);
        }
    }
    result.push(*points.last().expect("non-empty"));
    result
}

/// Detect hydrogen bonds, π–π stacking, salt bridges, and short contacts.
///
/// Atoms within three bonds of each other are never paired: the ends of a bond angle
/// (1-3) and of a torsion (1-4) sit closer than a van der Waals contact by construction,
/// so counting them reported every angle of every residue as a clash.
pub fn detect_interactions(scene: &Mol3dScene) -> Vec<Interaction> {
    let mut interactions = Vec::new();
    let mut adjacency = BTreeMap::<AtomId, Vec<AtomId>>::new();
    for bond in &scene.bonds {
        adjacency.entry(bond.a).or_default().push(bond.b);
        adjacency.entry(bond.b).or_default().push(bond.a);
    }
    let is_acceptor = |atom: &Atom3D| matches!(atom.element.symbol(), "N" | "O" | "F");
    let is_donor = |atom: &Atom3D| matches!(atom.element.symbol(), "N" | "O" | "F");
    // The longest interaction reaches 4.6 A (a salt bridge is 4.0, a contact under the sum
    // of two van der Waals radii), so cells of 5 A put every candidate in a neighbouring
    // cell. The all-pairs loop was quadratic and froze on a protein.
    const CELL: f64 = 5.0;
    let cell_of = |position: Vec3| {
        (
            (position.x / CELL).floor() as i64,
            (position.y / CELL).floor() as i64,
            (position.z / CELL).floor() as i64,
        )
    };
    let mut cells = std::collections::HashMap::<(i64, i64, i64), Vec<usize>>::new();
    for (index, atom) in scene.atoms.iter().enumerate() {
        let finite = atom.position.x.is_finite() && atom.position.y.is_finite() && atom.position.z.is_finite();
        if finite && atom.element.symbol() != "H" {
            cells.entry(cell_of(atom.position)).or_default().push(index);
        }
    }
    let mut pairs = Vec::<(usize, usize)>::new();
    for (index, left) in scene.atoms.iter().enumerate() {
        if left.element.symbol() == "H" || !left.position.x.is_finite() {
            continue;
        }
        let (cx, cy, cz) = cell_of(left.position);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(bucket) = cells.get(&(cx + dx, cy + dy, cz + dz)) {
                        pairs.extend(
                            bucket
                                .iter()
                                .filter(|other| **other > index)
                                .map(|other| (index, *other)),
                        );
                    }
                }
            }
        }
    }
    pairs.sort_unstable();
    for (left_index, right_index) in pairs {
        let (left, right) = (&scene.atoms[left_index], &scene.atoms[right_index]);
        let distance = (left.position - right.position).length();
        if distance > 4.6 || within_bonds(&adjacency, left.id, right.id, 3) {
            continue;
        }
        if is_donor(left) && is_acceptor(right) && (2.4..=3.5).contains(&distance) {
            interactions.push(Interaction {
                kind: InteractionKind::HydrogenBond,
                a: left.id,
                b: right.id,
                distance,
            });
            continue;
        }
        if is_salt_bridge(left, right) && distance <= 4.0 {
            interactions.push(Interaction {
                kind: InteractionKind::SaltBridge,
                a: left.id,
                b: right.id,
                distance,
            });
            continue;
        }
        let contact = vdw_radius(left.element) + vdw_radius(right.element) - 0.55;
        if distance < contact && left.element != egui_workbench_chem_core::Element::UNKNOWN {
            interactions.push(Interaction {
                kind: InteractionKind::ShortContact,
                a: left.id,
                b: right.id,
                distance,
            });
        }
    }
    for (first, second) in aromatic_pairs(scene) {
        interactions.push(Interaction {
            kind: InteractionKind::PiStacking,
            a: first.0,
            b: second.0,
            distance: (first.1 - second.1).length(),
        });
    }
    interactions
}

/// Whether `from` reaches `to` through at most `depth` bonds.
fn within_bonds(adjacency: &BTreeMap<AtomId, Vec<AtomId>>, from: AtomId, to: AtomId, depth: usize) -> bool {
    let mut frontier = vec![from];
    let mut seen = BTreeSet::from([from]);
    for _ in 0..depth {
        let mut next = Vec::new();
        for atom in &frontier {
            for neighbor in adjacency.get(atom).into_iter().flatten() {
                if *neighbor == to {
                    return true;
                }
                if seen.insert(*neighbor) {
                    next.push(*neighbor);
                }
            }
        }
        if next.is_empty() {
            return false;
        }
        frontier = next;
    }
    false
}

fn is_salt_bridge(left: &Atom3D, right: &Atom3D) -> bool {
    let acidic = |atom: &Atom3D| {
        atom.residue.as_ref().is_some_and(|residue| {
            matches!(residue.name.as_str(), "ASP" | "GLU")
                && matches!(atom.label.as_deref(), Some("OD1" | "OD2" | "OE1" | "OE2"))
        })
    };
    let basic = |atom: &Atom3D| {
        atom.residue.as_ref().is_some_and(|residue| {
            matches!(residue.name.as_str(), "LYS" | "ARG" | "HIS")
                && matches!(atom.label.as_deref(), Some("NZ" | "NH1" | "NH2" | "ND1" | "NE2"))
        })
    };
    (acidic(left) && basic(right)) || (basic(left) && acidic(right))
}

fn aromatic_pairs(scene: &Mol3dScene) -> Vec<((AtomId, Vec3), (AtomId, Vec3))> {
    let rings = aromatic_rings(scene);
    let mut pairs = Vec::new();
    for (index, first) in rings.iter().enumerate() {
        for second in &rings[index + 1..] {
            let distance = (first.1 - second.1).length();
            if !(3.2..=4.6).contains(&distance) {
                continue;
            }
            let parallel = first.2.dot(second.2).abs();
            if parallel < 0.75 {
                continue;
            }
            pairs.push(((first.0, first.1), (second.0, second.1)));
        }
    }
    pairs
}

fn aromatic_rings(scene: &Mol3dScene) -> Vec<(AtomId, Vec3, Vec3)> {
    let mut adjacency = BTreeMap::<AtomId, Vec<AtomId>>::new();
    for bond in &scene.bonds {
        adjacency.entry(bond.a).or_default().push(bond.b);
        adjacency.entry(bond.b).or_default().push(bond.a);
    }
    // Look atoms up once. `scene.atom(id)` scans the whole list, and the ring search asked
    // for it at every step from every atom: cubic on a protein.
    let positions: BTreeMap<AtomId, Vec3> = scene.atoms.iter().map(|atom| (atom.id, atom.position)).collect();
    let eligible: BTreeSet<AtomId> = scene
        .atoms
        .iter()
        .filter(|atom| matches!(atom.element.symbol(), "C" | "N" | "O" | "S"))
        .map(|atom| atom.id)
        .collect();
    let mut rings = Vec::<Vec<AtomId>>::new();
    let mut seen = BTreeSet::<Vec<AtomId>>::new();
    for start in adjacency.keys().filter(|id| eligible.contains(id)) {
        let mut path = vec![*start];
        let mut visited = BTreeSet::from([*start]);
        collect_rings(
            &eligible,
            &adjacency,
            *start,
            &mut path,
            &mut visited,
            &mut seen,
            &mut rings,
        );
    }
    let mut result = Vec::new();
    // `collect_rings` already keeps one entry per atom set; checking `seen` again here
    // discarded every ring, so π-stacking was never reported.
    for ring in rings {
        let points = ring
            .iter()
            .filter_map(|id| positions.get(id).copied())
            .collect::<Vec<_>>();
        if points.len() != ring.len() || points.len() < 5 {
            continue;
        }
        let Some(center) = centroid(&points) else {
            continue;
        };
        let Some(plane) = mean_plane_normal(&points) else {
            continue;
        };
        let rms = (points
            .iter()
            .map(|point| (*point - center).dot(plane).powi(2))
            .sum::<f64>()
            / points.len() as f64)
            .sqrt();
        if rms < 0.15 {
            result.push((ring[0], center, plane));
        }
    }
    result
}

fn collect_rings(
    eligible: &BTreeSet<AtomId>,
    adjacency: &BTreeMap<AtomId, Vec<AtomId>>,
    start: AtomId,
    path: &mut Vec<AtomId>,
    visited: &mut BTreeSet<AtomId>,
    seen: &mut BTreeSet<Vec<AtomId>>,
    rings: &mut Vec<Vec<AtomId>>,
) {
    if path.len() > 6 {
        return;
    }
    let Some(neighbors) = path.last().and_then(|last| adjacency.get(last)) else {
        return;
    };
    for next in neighbors {
        if *next == start && path.len() >= 5 {
            let mut key = path.clone();
            key.sort();
            if seen.insert(key) {
                rings.push(path.clone());
            }
            continue;
        }
        if visited.contains(next) || !eligible.contains(next) {
            continue;
        }
        visited.insert(*next);
        path.push(*next);
        collect_rings(eligible, adjacency, start, path, visited, seen, rings);
        path.pop();
        visited.remove(next);
    }
}

#[cfg(test)]
mod tests;
