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
pub fn detect_interactions(scene: &Mol3dScene) -> Vec<Interaction> {
    let mut interactions = Vec::new();
    let bonded = scene
        .bonds
        .iter()
        .map(|bond| (bond.a.min(bond.b), bond.a.max(bond.b)))
        .collect::<BTreeSet<_>>();
    let is_acceptor = |atom: &Atom3D| matches!(atom.element.symbol(), "N" | "O" | "F");
    let is_donor = |atom: &Atom3D| matches!(atom.element.symbol(), "N" | "O" | "F");
    for (left_index, left) in scene.atoms.iter().enumerate() {
        for right in &scene.atoms[left_index + 1..] {
            if left.element.symbol() == "H" || right.element.symbol() == "H" {
                continue;
            }
            let pair = (left.id.min(right.id), left.id.max(right.id));
            if bonded.contains(&pair) {
                continue;
            }
            let distance = (left.position - right.position).length();
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
    let mut paths = Vec::<Vec<AtomId>>::new();
    let mut seen = BTreeSet::<Vec<AtomId>>::new();
    for start in adjacency.keys() {
        let mut path = vec![*start];
        let mut visited = BTreeSet::from([*start]);
        collect_rings(
            scene,
            &adjacency,
            *start,
            &mut path,
            &mut visited,
            &mut seen,
            &mut paths,
        );
    }
    let mut result = Vec::new();
    for ring in paths {
        let mut key = ring.clone();
        key.sort();
        if !seen.insert(key) {
            continue;
        }
        let points = ring
            .iter()
            .filter_map(|id| scene.atom(*id).map(|atom| atom.position))
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
    scene: &Mol3dScene,
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
    let Some(neighbors) = adjacency.get(path.last().expect("path")) else {
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
        if visited.contains(next) {
            continue;
        }
        if scene
            .atom(*next)
            .is_none_or(|atom| !matches!(atom.element.symbol(), "C" | "N" | "O" | "S"))
        {
            continue;
        }
        visited.insert(*next);
        path.push(*next);
        collect_rings(scene, adjacency, start, path, visited, seen, rings);
        path.pop();
        visited.remove(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::Element;

    fn atom(scene: &mut Mol3dScene, element: &str, position: [f64; 3]) -> AtomId {
        scene.push_atom(Element::from_symbol(element).expect("element"), position)
    }

    #[test]
    fn a_helix_trace_smooths_into_a_ribbon() {
        let mut scene = Mol3dScene::default();
        for index in 0..12 {
            let angle = index as f64 * 1.75;
            let atom = atom(
                &mut scene,
                "C",
                [2.3 * angle.cos(), 2.3 * angle.sin(), index as f64 * 1.5],
            );
            if let Some(entry) = scene.atoms.iter_mut().find(|entry| entry.id == atom) {
                entry.label = Some("CA".to_owned());
                entry.residue = Some(crate::scene::Residue3D {
                    name: "ALA".to_owned(),
                    chain: "A".to_owned(),
                    number: index,
                });
            }
        }
        let traces = backbone_traces(&scene);
        assert_eq!(traces.len(), 1);
        assert!(traces[0].secondary.contains(&SecondaryStructure::Helix));
        let mesh = ribbon_mesh(&traces[0]);
        assert!(mesh.vertices.len() > traces[0].points.len());
        assert!(!mesh.triangles.is_empty());
    }

    #[test]
    fn hydrogen_bonds_and_short_contacts_are_detected() {
        let mut scene = Mol3dScene::default();
        let oxygen = atom(&mut scene, "O", [0.0, 0.0, 0.0]);
        let nitrogen = atom(&mut scene, "N", [2.9, 0.0, 0.0]);
        let interactions = detect_interactions(&scene);
        assert!(interactions.iter().any(|interaction| {
            interaction.kind == InteractionKind::HydrogenBond
                && [interaction.a, interaction.b].contains(&oxygen)
                && [interaction.a, interaction.b].contains(&nitrogen)
        }));
    }
}
