//! Headless figure renderer.
//!
//! Run with `cargo run -p egui-workbench-mol3d --example render -- out.png [input file]`.

use egui_workbench_chem_core::Element;
use egui_workbench_mol3d::{
    Camera3D, Mol3dScene, RenderOptions, parse_cif, parse_mmcif, parse_mol, parse_pdb, parse_poscar, parse_xyz,
    render_png,
};

fn main() -> Result<(), String> {
    let mut arguments = std::env::args().skip(1);
    let output = arguments.next().unwrap_or_else(|| "mol3d.png".to_owned());
    let scene = match arguments.next().as_deref() {
        Some("demo") | None => demo(),
        Some(path) => read(path)?,
    };
    let width = arguments
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1600);
    let height = arguments
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1200);
    let mut camera = Camera3D::default();
    if let Some(bounds) = scene.bounds() {
        camera.fit(
            bounds,
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(width as f32, height as f32)),
        );
    }
    let options = RenderOptions {
        width,
        height,
        transparent: true,
        silhouette: true,
        ..RenderOptions::default()
    };
    let png = render_png(&scene, camera, &options)?;
    std::fs::write(&output, png).map_err(|error| error.to_string())?;
    Ok(())
}

fn read(path: &str) -> Result<Mol3dScene, String> {
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "xyz" => parse_xyz(&text)
            .map_err(|error| error.to_string())?
            .into_iter()
            .next()
            .ok_or_else(|| "empty XYZ".to_owned()),
        "pdb" => parse_pdb(&text).map_err(|error| error.to_string()),
        "cif" => parse_cif(&text).map_err(|error| error.to_string()),
        "mmcif" => parse_mmcif(&text).map_err(|error| error.to_string()),
        "mol" | "sdf" => parse_mol(&text).map_err(|error| error.to_string()),
        "poscar" | "contcar" => parse_poscar(&text).map_err(|error| error.to_string()),
        _ => Err(format!("unsupported file: {path}")),
    }
}

fn demo() -> Mol3dScene {
    let mut scene = Mol3dScene::default();
    scene.push_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
    scene.push_atom(Element::from_symbol("H").expect("H"), [0.96, 0.0, 0.0]);
    scene.push_atom(Element::from_symbol("H").expect("H"), [-0.24, 0.93, 0.0]);
    scene.infer_bonds();
    scene
}
