//! The snippets of the chemistry guide, compiled and run.

use egui_workbench_mol3d::{Camera3D, Measurement, Mol3dScene, Mol3dView, parse_xyz};

const BUTANE: &str = "4\nanti butane skeleton\nC 0.0 1.0 0.0\nC 0.0 0.0 0.0\nC 1.5 0.0 0.0\nC 1.5 -1.0 0.0\n";

#[test]
fn an_xyz_file_is_measured_and_rendered() {
    let scene: Mol3dScene = parse_xyz(BUTANE).expect("xyz").remove(0);
    let ids: Vec<_> = scene.atoms.iter().map(|atom| atom.id).collect();
    let dihedral = Measurement::Dihedral {
        a: ids[0],
        b: ids[1],
        c: ids[2],
        d: ids[3],
    }
    .evaluate(&scene)
    .expect("four atoms");
    // The two end carbons lie on opposite sides of the central bond: anti.
    assert!((dihedral.value.abs() - 180.0).abs() < 1.0e-9, "{}", dihedral.value);

    let mut view = Mol3dView::new(scene);
    view.camera = Camera3D::default();
    let png = view
        .capture_png(320, 240, false)
        .expect("a software render needs no GPU");
    assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
}
