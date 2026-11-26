//! Instance construction from the scene.

use egui::Rect;

use egui_workbench_chem_core::{AtomId, BondId};

use super::{BOND_BIT, BondInstance, Instance};
use crate::camera::Camera3D;
use crate::scene::Mol3dScene;

/// Build sphere and bond instances plus their pick-id tables.
#[allow(clippy::too_many_arguments)]
pub(super) fn build(
    scene: &Mol3dScene,
    camera: Camera3D,
    viewport: Rect,
    scale: f64,
    representations: &crate::representation::RepresentationSet,
    colors: crate::color::ColorScheme,
    style: &crate::view::Mol3dStyle,
) -> (Vec<Instance>, Vec<BondInstance>, Vec<AtomId>, Vec<BondId>) {
    let mut spheres = Vec::new();
    let mut atom_ids = Vec::new();
    for atom in &scene.atoms {
        let representation = representations.for_atom(atom.id);
        if !representation.shows_atoms() {
            continue;
        }
        let Some((_, depth)) = camera.project(atom.position, viewport) else {
            continue;
        };
        let [r, g, b] = colors.color(atom);
        let pick = atom_ids.len() as u32 + 1;
        atom_ids.push(atom.id);
        spheres.push((
            depth,
            Instance {
                center: [atom.position.x as f32, atom.position.y as f32, atom.position.z as f32],
                radius: (representation.atom_radius(atom) * scale) as f32,
                color: [f32::from(r) / 255.0, f32::from(g) / 255.0, f32::from(b) / 255.0, 1.0],
                pick,
                _pad: [0; 3],
            },
        ));
    }
    spheres.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut bonds = Vec::new();
    let mut bond_ids = Vec::new();
    for bond in &scene.bonds {
        let (Some(a), Some(b)) = (scene.atom(bond.a), scene.atom(bond.b)) else {
            continue;
        };
        let representation = representations.for_atom(a.id);
        if !representation.shows_bonds() {
            continue;
        }
        let pick = BOND_BIT | (bond_ids.len() as u32 + 1);
        bond_ids.push(bond.id);
        let radius = (representation.bond_radius() * f64::from(style.bond_width) * 0.5).max(0.02);
        let [ar, ag, ab] = colors.color(a);
        let [br, bg, bb] = colors.color(b);
        let axis = b.position - a.position;
        let perpendicular = axis.cross(camera.forward()).normalized();
        let strokes = f32::from(bond.order.strokes().max(1));
        for stroke in 0..bond.order.strokes().max(1) {
            let offset = (f32::from(stroke) - (strokes - 1.0) * 0.5) * radius as f32 * 2.0;
            let shift = perpendicular * f64::from(offset);
            let start = a.position + shift;
            let end = b.position + shift;
            let middle = (start + end) * 0.5;
            bonds.push(BondInstance {
                a: [start.x as f32, start.y as f32, start.z as f32],
                _pad0: 0.0,
                b: [middle.x as f32, middle.y as f32, middle.z as f32],
                radius: radius as f32,
                color: [f32::from(ar) / 255.0, f32::from(ag) / 255.0, f32::from(ab) / 255.0, 1.0],
                pick,
                _pad1: [0; 3],
            });
            bonds.push(BondInstance {
                a: [middle.x as f32, middle.y as f32, middle.z as f32],
                _pad0: 0.0,
                b: [end.x as f32, end.y as f32, end.z as f32],
                radius: radius as f32,
                color: [f32::from(br) / 255.0, f32::from(bg) / 255.0, f32::from(bb) / 255.0, 1.0],
                pick,
                _pad1: [0; 3],
            });
        }
    }
    (
        spheres.into_iter().map(|(_, instance)| instance).collect(),
        bonds,
        atom_ids,
        bond_ids,
    )
}
