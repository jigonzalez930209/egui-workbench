//! Regression tests for hostile or unusual input to the readers.

use egui_workbench_mol3d::{parse_cif, parse_cube, parse_mmcif, parse_mol, parse_pdb};

#[test]
fn a_multibyte_character_in_a_mol_column_is_an_error() {
    let text = "t\n  x\n\n  1  0  0  0  0  0  0  0  0  0999 V2000\n   0.0000    0.0000é   0.0000 C   0  0\nM  END\n";
    assert!(parse_mol(text).is_err());
}

#[test]
fn mmcif_uses_the_atom_site_loop_not_the_first_loop() {
    let text = "data_x\nloop_\n_entity.id\n_entity.type\n1 polymer\n#\nloop_\n_atom_site.Cartn_x\n_atom_site.Cartn_y\n_atom_site.Cartn_z\n_atom_site.type_symbol\n0 0 0 O\n0.96 0 0 H\n#\n";
    assert_eq!(parse_mmcif(text).expect("mmcif").atoms.len(), 2);
}

#[test]
fn a_short_mmcif_row_is_skipped() {
    let text =
        "loop_\n_atom_site.type_symbol\n_atom_site.Cartn_x\n_atom_site.Cartn_y\n_atom_site.Cartn_z\nO 0 0 0\nH 1\n#\n";
    assert_eq!(parse_mmcif(text).expect("mmcif").atoms.len(), 1);
}

#[test]
fn cif_coordinates_may_carry_an_uncertainty() {
    let text = "_cell_length_a 10.0\n_cell_length_b 10.0\n_cell_length_c 10.0\nloop_\n_atom_site_label\n_atom_site_type_symbol\n_atom_site_fract_x\n_atom_site_fract_y\n_atom_site_fract_z\nNa1 Na 0.1234(5) 0.5 0.25(1)\n";
    assert_eq!(parse_cif(text).expect("cif").atoms.len(), 1);
}

#[test]
fn a_cif_loop_with_only_fract_x_is_an_error_not_a_panic() {
    let text = "_cell_length_a 10.0\n_cell_length_b 10.0\n_cell_length_c 10.0\nloop_\n_atom_site_label\n_atom_site_fract_x\nNa1 0.1\n";
    assert!(parse_cif(text).is_err());
}

#[test]
fn a_cube_header_cannot_request_more_than_the_file_holds() {
    let text = "cube\nbomb\n 0 0.0 0.0 0.0\n 100000 1.0 0.0 0.0\n 100000 0.0 1.0 0.0\n 100000 0.0 0.0 1.0\n1.0\n";
    assert!(parse_cube(text).is_err());
}

#[test]
fn a_zero_sized_cube_is_an_error() {
    let text = "cube\nzero\n 0 0.0 0.0 0.0\n 0 1.0 0.0 0.0\n 2 0.0 1.0 0.0\n 2 0.0 0.0 1.0\n1.0\n";
    assert!(parse_cube(text).is_err());
}

#[test]
fn bond_inference_finds_the_same_bonds_in_a_large_chain() {
    // 3000 carbons 1.5 A apart: only neighbours bond, and the grid must agree with that.
    let mut text = String::new();
    for index in 0..3000 {
        text.push_str(&format!(
            "ATOM  {:>5}  C   ALA A{:>4}    {:>8.3}{:>8.3}{:>8.3}  1.00  0.00           C\n",
            index + 1,
            index + 1,
            index as f64 * 1.5,
            0.0,
            0.0
        ));
    }
    let scene = parse_pdb(&text).expect("pdb");
    assert_eq!(scene.atoms.len(), 3000);
    assert_eq!(scene.bonds.len(), 2999);
}
