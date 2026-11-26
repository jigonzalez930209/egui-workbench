//! Interactive 3D molecular and crystal viewer.
//!
//! Run with `cargo run -p egui-workbench-mol3d --example viewer -- [file.xyz|.pdb|.cif|.mol|.poscar]`.

use eframe::egui;
use egui_workbench_chem_core::Element;
use egui_workbench_mol3d::{
    Camera3D, ColorScheme, Mol3dScene, Mol3dView, OrbitalKind, RenderOptions, Representation, parse_cif, parse_mmcif,
    parse_mol, parse_pdb, parse_poscar, parse_xyz, render_png,
};

fn main() -> eframe::Result {
    let mut app = ViewerApp::new();
    if let Some(path) = std::env::args().nth(1) {
        app.load(&path);
    }
    eframe::run_native(
        "egui-workbench mol3d",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(app))),
    )
}

struct ViewerApp {
    view: Mol3dView,
    representation: Representation,
    color: ColorScheme,
    orbital: OrbitalKind,
    isovalue: f64,
    supercell: [usize; 3],
    metals: String,
    cutoff: f64,
    miller: [i32; 3],
    status: String,
    path: String,
}

impl ViewerApp {
    fn new() -> Self {
        let mut view = Mol3dView::new(demo_scene());
        view.fit(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(800.0, 600.0),
        ));
        Self {
            view,
            representation: Representation::BallAndStick,
            color: ColorScheme::Cpk,
            orbital: OrbitalKind::TwoPz,
            isovalue: 0.05,
            supercell: [1, 1, 1],
            metals: "Fe,Zn,Mg,Cu".to_owned(),
            cutoff: 2.8,
            miller: [1, 0, 0],
            status: "Drag to orbit, Shift-drag to pan, scroll to zoom.".to_owned(),
            path: String::new(),
        }
    }

    fn load(&mut self, path: &str) {
        let Ok(text) = std::fs::read_to_string(path) else {
            self.status = format!("cannot read {path}");
            return;
        };
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let scene = match extension.as_str() {
            "xyz" => parse_xyz(&text).ok().and_then(|mut frames| frames.drain(..).next()),
            "pdb" => parse_pdb(&text).ok(),
            "cif" => parse_cif(&text).ok(),
            "mmcif" => parse_mmcif(&text).ok(),
            "mol" | "sdf" => parse_mol(&text).ok(),
            "poscar" | "contcar" => parse_poscar(&text).ok(),
            _ => parse_cif(&text).or_else(|_| parse_pdb(&text)).ok(),
        };
        match scene {
            Some(scene) => {
                self.view = Mol3dView::new(scene);
                self.view.fit(egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(800.0, 600.0),
                ));
                self.status = format!("loaded {path}: {} atoms", self.view.scene.atoms.len());
            }
            None => self.status = format!("unsupported or invalid file: {path}"),
        }
    }

    fn tools(&mut self, ui: &mut egui::Ui) {
        ui.heading("File");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.path);
            if ui.button("Load").clicked() {
                let path = self.path.clone();
                self.load(&path);
            }
        });
        ui.separator();
        ui.heading("Representation");
        for representation in Representation::ALL {
            if ui
                .selectable_label(self.representation == representation, representation.label())
                .clicked()
            {
                self.representation = representation;
                self.view.representations.default = representation;
            }
        }
        ui.heading("Color");
        egui::ComboBox::from_id_salt("color")
            .selected_text(self.color.label())
            .show_ui(ui, |ui| {
                for scheme in ColorScheme::ALL {
                    ui.selectable_value(&mut self.color, scheme, scheme.label());
                }
            });
        self.view.color_scheme = self.color;
        ui.separator();
        ui.heading("Crystal");
        ui.horizontal(|ui| {
            for (axis, value) in self.supercell.iter_mut().enumerate() {
                ui.label(["a", "b", "c"][axis]);
                ui.add(egui::DragValue::new(value).range(1..=4));
            }
            if ui.button("Supercell").clicked() {
                self.view
                    .expand_supercell(self.supercell[0], self.supercell[1], self.supercell[2]);
                self.status = format!("{} atoms", self.view.scene.atoms.len());
            }
        });
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.metals);
            ui.add(egui::DragValue::new(&mut self.cutoff).range(1.0..=5.0).speed(0.05));
            if ui.button("Polyhedra").clicked() {
                let metals = self
                    .metals
                    .split(',')
                    .map(str::trim)
                    .filter(|symbol| !symbol.is_empty())
                    .collect::<Vec<_>>();
                self.view.toggle_polyhedra(&metals, self.cutoff);
                self.status = format!("{} polyhedra", self.view.polyhedra.len());
            }
        });
        ui.horizontal(|ui| {
            for (axis, value) in self.miller.iter_mut().enumerate() {
                ui.label(["(h", "k", "l)"][axis]);
                ui.add(egui::DragValue::new(value).range(-4..=4));
            }
            if ui.button("Miller plane").clicked() {
                self.view
                    .toggle_miller_plane(self.miller[0], self.miller[1], self.miller[2]);
            }
        });
        if ui.button("Highlight pores").clicked() {
            self.view.highlight_pores(1.2, 1.0);
            self.status = format!("{} pore points", self.view.pore_points.len());
        }
        ui.separator();
        ui.heading("Biology");
        ui.horizontal(|ui| {
            if ui.button("Cartoon").clicked() {
                self.view.show_cartoon();
                self.status = format!("{} ribbons", self.view.ribbons.len());
            }
            if ui.button("Interactions").clicked() {
                self.view.show_interactions();
                self.status = format!("{} interactions", self.view.interactions.len());
            }
        });
        ui.separator();
        ui.heading("Orbital");
        egui::ComboBox::from_id_salt("orbital")
            .selected_text(self.orbital.label())
            .show_ui(ui, |ui| {
                for orbital in OrbitalKind::ALL {
                    ui.selectable_value(&mut self.orbital, orbital, orbital.label());
                }
            });
        ui.add(egui::Slider::new(&mut self.isovalue, 0.01..=0.4).text("isovalue"));
        ui.horizontal(|ui| {
            if ui.button("Render orbital").clicked() {
                let center = self.view.scene_center();
                self.view.show_orbital(self.orbital, center, self.isovalue, 32);
                self.status = format!("{} surfaces", self.view.surfaces.len());
            }
            if ui.button("Clear surfaces").clicked() {
                self.view.surfaces.clear();
            }
        });
    }

    fn info(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scene");
        ui.label(format!("{} atoms", self.view.scene.atoms.len()));
        ui.label(format!("{} bonds", self.view.scene.bonds.len()));
        if let Some(cell) = self.view.scene.cell {
            ui.label(format!("cell {:.2} × {:.2} × {:.2} Å", cell.a, cell.b, cell.c));
        }
        ui.separator();
        ui.heading("Measure");
        ui.label(format!("{} selected", self.view.selection.len()));
        ui.horizontal(|ui| {
            if ui.button("Distance").clicked() {
                let atoms = self.view.selection.atoms().iter().copied().collect::<Vec<_>>();
                if atoms.len() >= 2 {
                    match self.view.measure_distance(atoms[0], atoms[1]) {
                        Some(value) => self.status = format!("{value:.2} Å"),
                        None => self.status = "invalid distance".to_owned(),
                    }
                }
            }
            if ui.button("Angle").clicked() {
                let atoms = self.view.selection.atoms().iter().copied().collect::<Vec<_>>();
                if atoms.len() >= 3
                    && let Some(value) = self.view.measure_angle(atoms[0], atoms[1], atoms[2])
                {
                    self.status = format!("{value:.1}°");
                }
            }
            if ui.button("Dihedral").clicked() {
                let atoms = self.view.selection.atoms().iter().copied().collect::<Vec<_>>();
                if atoms.len() >= 4
                    && let Some(value) = self.view.measure_dihedral(atoms[0], atoms[1], atoms[2], atoms[3])
                {
                    self.status = format!("{value:.1}°");
                }
            }
        });
        if ui.button("Plane of selection").clicked() {
            let atoms = self.view.selection.atoms().iter().copied().collect::<Vec<_>>();
            if self.view.add_plane(atoms).is_some() {
                self.status = "mean plane added".to_owned();
            }
        }
        for measurement in &self.view.measurements {
            if let Some(value) = measurement.evaluate(&self.view.scene) {
                ui.label(&value.label);
            }
        }
        if ui.button("Clear measurements").clicked() {
            self.view.measurements.clear();
        }
        ui.separator();
        ui.heading("Export");
        if ui.button("PNG 4K").clicked() {
            let options = RenderOptions::figure_4k();
            match render_png(&self.view.scene, self.view.camera, &options) {
                Ok(bytes) => {
                    let path = std::env::temp_dir().join("egui-workbench-mol3d-4k.png");
                    match std::fs::write(&path, bytes) {
                        Ok(()) => self.status = format!("PNG: {}", path.display()),
                        Err(error) => self.status = error.to_string(),
                    }
                }
                Err(error) => self.status = error,
            }
        }
        if ui.button("Reset view").clicked() {
            self.view.camera = Camera3D::default();
            self.view.fit(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            ));
        }
        ui.separator();
        ui.label(&self.status);
    }
}

impl eframe::App for ViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("mol3d_tools")
            .resizable(true)
            .default_size(300.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.tools(ui));
            });
        egui::Panel::right("mol3d_info")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| self.info(ui));
        egui::CentralPanel::default().show(ui, |ui| {
            self.view.show(ui);
        });
    }
}

fn demo_scene() -> Mol3dScene {
    let mut scene = Mol3dScene::default();
    scene.push_atom(Element::from_symbol("Fe").expect("Fe"), [0.0, 0.0, 0.0]);
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
    scene.push_atom(Element::from_symbol("C").expect("C"), [3.4, 1.6, 0.0]);
    scene.push_atom(Element::from_symbol("C").expect("C"), [4.6, 1.6, 0.0]);
    scene.push_atom(Element::from_symbol("O").expect("O"), [5.4, 2.4, 0.0]);
    scene.infer_bonds();
    scene.cell = egui_workbench_chem_core::UnitCell::new(8.0, 8.0, 8.0, 90.0, 90.0, 90.0).ok();
    scene
}
