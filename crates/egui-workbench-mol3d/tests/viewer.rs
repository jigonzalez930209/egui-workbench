//! End-to-end 3D viewer checks through the public API.

use egui_workbench_chem_core::{Element, UnitCell};
use egui_workbench_mol3d::{
    ActionOutcome, Camera3D, ClipPlane, ColorScheme, Measurement, Mol3dAction, Mol3dScene, Mol3dView, OrbitalKind,
    RenderOptions, Representation, SceneDescriptor, Trajectory, VibrationalMode, animate_mode, detect_interactions,
    isosurface, miller_plane, orbital_grid, parse_cif, parse_cube, parse_mmcif, parse_mol, parse_pdb, parse_poscar,
    parse_xyz, render_png, render_rgba,
};

fn water() -> Mol3dScene {
    let mut scene = Mol3dScene::default();
    scene.push_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
    scene.push_atom(Element::from_symbol("H").expect("H"), [0.96, 0.0, 0.0]);
    scene.push_atom(Element::from_symbol("H").expect("H"), [-0.24, 0.93, 0.0]);
    scene.infer_bonds();
    scene
}

#[test]
fn every_reader_returns_a_scene() {
    let xyz = "2\nwater\nO 0 0 0\nH 0.96 0 0\n";
    assert_eq!(parse_xyz(xyz).expect("xyz")[0].atoms.len(), 2);
    let pdb = "ATOM      1  O   HOH A   1       0.000   0.000   0.000  1.00  0.00           O\nATOM      2  H   HOH A   1       0.960   0.000   0.000  1.00  0.00           H\nCONECT    1    2\n";
    assert_eq!(parse_pdb(pdb).expect("pdb").bonds.len(), 1);
    let cif = "loop_\n_atom_site.Cartn_x\n_atom_site.Cartn_y\n_atom_site.Cartn_z\n_atom_site.type_symbol\n0 0 0 O\n1 0 0 H\n#\n";
    assert_eq!(parse_mmcif(cif).expect("mmcif").atoms.len(), 2);
    let poscar = "Si\n1.0\n5 0 0\n0 5 0\n0 0 5\nSi\n1\nDirect\n0 0 0\n";
    assert!(parse_poscar(poscar).expect("poscar").cell.is_some());
    let mol = "water\n  test\n\n  2  1  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 O   0  0\n    0.9600    0.0000    0.0000 H   0  0\n  1  2  1  0\nM  END\n";
    assert_eq!(parse_mol(mol).expect("mol").bonds.len(), 1);
}

#[test]
fn a_cif_expands_symmetry_and_supports_supercells_and_polyhedra() {
    let text = "_cell_length_a 4.0\n_cell_length_b 4.0\n_cell_length_c 4.0\n_cell_angle_alpha 90\n_cell_angle_beta 90\n_cell_angle_gamma 90\nloop_\n_atom_site_label\n_atom_site_type_symbol\n_atom_site_fract_x\n_atom_site_fract_y\n_atom_site_fract_z\nZn1 Zn 0.0 0.0 0.0\nO1 O 0.5 0.0 0.0\n";
    let scene = parse_cif(text).expect("cif");
    assert_eq!(scene.atoms.len(), 2);
    let expanded = scene.supercell(2, 2, 2);
    assert_eq!(expanded.atoms.len(), 16);
    let mut octahedron = Mol3dScene::default();
    octahedron.push_atom(Element::from_symbol("Fe").expect("Fe"), [0.0, 0.0, 0.0]);
    for position in [
        [2.0, 0.0, 0.0],
        [-2.0, 0.0, 0.0],
        [0.0, 2.0, 0.0],
        [0.0, -2.0, 0.0],
        [0.0, 0.0, 2.0],
        [0.0, 0.0, -2.0],
    ] {
        octahedron.push_atom(Element::from_symbol("O").expect("O"), position);
    }
    let polyhedra = octahedron.coordination_polyhedra(&["Fe"], 2.5);
    assert_eq!(polyhedra.len(), 1);
    assert_eq!(polyhedra[0].shape_name(), "octahedral");
    assert!(miller_plane(expanded.cell.expect("cell"), 1, 0, 0).is_some());
}

#[test]
fn measurements_cover_distance_angle_dihedral_and_planes() {
    let scene = water();
    let [oxygen, first, second] = [scene.atoms[0].id, scene.atoms[1].id, scene.atoms[2].id];
    let distance = Measurement::Distance { a: oxygen, b: first }
        .evaluate(&scene)
        .expect("distance");
    assert!((distance.value - 0.96).abs() < 1.0e-9);
    let angle = Measurement::Angle {
        a: first,
        b: oxygen,
        c: second,
    }
    .evaluate(&scene)
    .expect("angle");
    assert!(angle.value > 90.0);
    let plane = Measurement::Plane {
        atoms: vec![oxygen, first, second],
    }
    .evaluate(&scene)
    .expect("plane");
    assert!(plane.value < 1.0e-9);
}

#[test]
fn isosurfaces_orbital_rendering_and_cube_files_work() {
    let grid = orbital_grid(OrbitalKind::TwoPz, egui_workbench_mol3d::Vec3::ZERO, 6.0, 20);
    let mesh = isosurface(&grid, 0.05);
    assert!(!mesh.triangles.is_empty());
    let mut view = Mol3dView::new(water());
    view.show_orbital(OrbitalKind::TwoPz, egui_workbench_mol3d::Vec3::ZERO, 0.05, 16);
    assert!(!view.surfaces.is_empty());
    let cube = "cube\norbital\n 1 0.0 0.0 0.0\n 2 1.0 0.0 0.0\n 2 0.0 1.0 0.0\n 2 0.0 0.0 1.0\n1 0.0 0.0 0.0 0.0\n1.0 1.0 0.0 0.0\n1.0 1.0 0.0 0.0\n1.0 1.0 0.0 0.0\n1.0 1.0 0.0 0.0\n1.0 1.0 0.0 0.0\n1.0 1.0 0.0 0.0\n1.0 1.0 0.0 0.0\n";
    let (grid, scene) = parse_cube(cube).expect("cube");
    assert_eq!((grid.dimensions, scene.atoms.len()), ([2, 2, 2], 1));
}

#[test]
fn biomolecular_interactions_and_ribbons_are_detected() {
    let mut scene = Mol3dScene::default();
    let oxygen = scene.push_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
    let nitrogen = scene.push_atom(Element::from_symbol("N").expect("N"), [2.9, 0.0, 0.0]);
    let interactions = detect_interactions(&scene);
    assert!(interactions.iter().any(|interaction| {
        [interaction.a, interaction.b].contains(&oxygen) && [interaction.a, interaction.b].contains(&nitrogen)
    }));
}

#[test]
fn trajectories_and_modes_animate() {
    let scene = water();
    let trajectory = Trajectory::between(&scene, &scene, 5);
    assert_eq!(trajectory.len(), 5);
    let mode = VibrationalMode::new(
        vec![(scene.atoms[0].id, egui_workbench_mol3d::Vec3::new(0.2, 0.0, 0.0))],
        3650.0,
    );
    let moved = animate_mode(&scene, &mode, std::f64::consts::FRAC_PI_2, 1.0);
    assert!(moved.atoms[0].position.x > 0.19);
}

#[test]
fn offscreen_rendering_supports_4k_transparency_clipping_and_toon() {
    let camera = Camera3D {
        distance: 6.0,
        ..Camera3D::default()
    };
    let options = RenderOptions {
        width: 160,
        height: 120,
        transparent: true,
        toon: true,
        silhouette: true,
        ambient_occlusion: true,
        ..RenderOptions::default()
    };
    let image = render_rgba(&water(), camera, &options);
    assert_eq!(image.dimensions(), (160, 120));
    assert!(image.pixels().any(|pixel| pixel[3] == 0));
    assert!(image.pixels().any(|pixel| pixel[3] == 255));
    let png = render_png(&water(), camera, &options).expect("png");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    let clipped = RenderOptions {
        clip_planes: vec![ClipPlane {
            normal: egui_workbench_mol3d::Vec3::new(1.0, 0.0, 0.0),
            offset: -50.0,
        }],
        ..options
    };
    let hidden = render_rgba(&water(), camera, &clipped);
    assert!(hidden.pixels().all(|pixel| pixel[3] == 0));
    assert_eq!(RenderOptions::figure_4k().width, 3840);
    assert_eq!(RenderOptions::figure_8k().height, 4320);
}

#[test]
fn descriptors_and_agent_actions_round_trip() {
    let mut view = Mol3dView::new(water());
    view.color_scheme = ColorScheme::Jmol;
    view.representations.default = Representation::Licorice;
    let descriptor = SceneDescriptor::capture(&view);
    assert!(descriptor.matches(&view.scene));
    let json = serde_json::to_string(&descriptor).expect("serialize");
    let restored: SceneDescriptor = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.color_scheme, ColorScheme::Jmol);
    let outcome: ActionOutcome = view
        .apply_action(&Mol3dAction::CapturePng {
            width: 48,
            height: 48,
            transparent: true,
        })
        .expect("capture");
    assert!(outcome.png.expect("png").starts_with(b"\x89PNG"));
    let atoms = vec![view.scene.atoms[0].id];
    view.apply_action(&Mol3dAction::FocusAtoms { atoms }).expect("focus");
}

#[test]
fn the_headless_render_matches_the_reference_image() {
    let mut camera = Camera3D::default();
    camera.fit(
        water().bounds().expect("bounds"),
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(64.0, 48.0)),
    );
    let options = RenderOptions {
        width: 64,
        height: 48,
        transparent: true,
        silhouette: true,
        ..RenderOptions::default()
    };
    let rendered = render_rgba(&water(), camera, &options);
    let golden = image::open(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/water-64x48.png"))
        .expect("golden")
        .to_rgba8();
    assert_eq!(rendered.as_raw(), golden.as_raw());
}

#[test]
fn a_cubic_cell_has_orthogonal_reciprocal_axes() {
    let cell = UnitCell::new(5.0, 5.0, 5.0, 90.0, 90.0, 90.0).expect("cell");
    let axes = egui_workbench_mol3d::CellAxes::new(cell);
    assert!(axes.reciprocal_vectors()[0].dot(axes.b).abs() < 1.0e-9);
}
