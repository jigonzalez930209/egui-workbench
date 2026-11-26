//! Unit tests.

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

fn benzene(scene: &mut Mol3dScene, z: f64) -> Vec<AtomId> {
    let ids: Vec<AtomId> = (0..6)
        .map(|index| {
            let angle = std::f64::consts::FRAC_PI_3 * f64::from(index);
            atom(scene, "C", [1.4 * angle.cos(), 1.4 * angle.sin(), z])
        })
        .collect();
    for index in 0..6 {
        scene.bonds.push(crate::scene::Bond3D {
            id: egui_workbench_chem_core::BondId::new(scene.bonds.len() as u32 + 1, 1),
            a: ids[index],
            b: ids[(index + 1) % 6],
            order: egui_workbench_chem_core::BondOrder::Aromatic,
        });
    }
    ids
}

#[test]
fn two_stacked_benzene_rings_report_pi_stacking() {
    let mut scene = Mol3dScene::default();
    benzene(&mut scene, 0.0);
    benzene(&mut scene, 3.8);
    let interactions = detect_interactions(&scene);
    assert!(
        interactions
            .iter()
            .any(|entry| entry.kind == InteractionKind::PiStacking),
        "{interactions:?}"
    );
}

#[test]
fn atoms_within_three_bonds_are_not_reported_as_contacts() {
    // A zig-zag C-C-C-C chain: 1-3 is 2.5 A, 1-4 is 3.8 A, both closer than the old
    // van der Waals cut-off minus 0.55.
    let mut scene = Mol3dScene::default();
    let ids: Vec<AtomId> = [[0.0, 0.0, 0.0], [1.25, 0.72, 0.0], [2.5, 0.0, 0.0], [3.75, 0.72, 0.0]]
        .iter()
        .map(|position| atom(&mut scene, "C", *position))
        .collect();
    for pair in ids.windows(2) {
        scene.bonds.push(crate::scene::Bond3D {
            id: egui_workbench_chem_core::BondId::new(scene.bonds.len() as u32 + 1, 1),
            a: pair[0],
            b: pair[1],
            order: egui_workbench_chem_core::BondOrder::Single,
        });
    }
    assert!(detect_interactions(&scene).is_empty());
}

#[test]
fn a_large_scene_is_scanned_without_quadratic_cost() {
    // 20,000 well-spaced atoms: the all-pairs loop needs 2e8 distance checks.
    let mut scene = Mol3dScene::default();
    for index in 0..20_000 {
        let (x, y, z) = (index % 30, (index / 30) % 30, index / 900);
        atom(
            &mut scene,
            "C",
            [f64::from(x) * 6.0, f64::from(y) * 6.0, f64::from(z) * 6.0],
        );
    }
    let started = std::time::Instant::now();
    assert!(detect_interactions(&scene).is_empty());
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
}
