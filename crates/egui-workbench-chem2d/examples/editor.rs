//! Interactive 2D chemistry editor demo.
//!
//! Run with `cargo run -p egui-workbench-chem2d --example editor`.

use eframe::egui;
use egui_workbench_chem2d::annotations::{Annotation, ArrowKind, ReactionArrow};
use egui_workbench_chem2d::{Chem2dEditor, ChemDocument, DocumentId, EditorTool, Element, ExportOptions, RingTemplate};

fn main() -> eframe::Result {
    eframe::run_native(
        "egui-workbench chem2d",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(EditorApp::new()))),
    )
}

struct EditorApp {
    editor: Chem2dEditor,
    smiles: String,
    chain: usize,
    ring: RingTemplate,
    status: String,
}

impl EditorApp {
    fn new() -> Self {
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(1), "demo"));
        editor.add_ring(RingTemplate::Benzene, [0.0, 0.0], 0.0).expect("ring");
        editor.insert_smiles("CCO", [4.0, 0.0]).expect("smiles");
        Self {
            editor,
            smiles: "c1ccncc1".to_owned(),
            chain: 4,
            ring: RingTemplate::Benzene,
            status: "Select an atom or draw a structure.".to_owned(),
        }
    }

    fn tools(&mut self, ui: &mut egui::Ui) {
        ui.heading("Tools");
        for (tool, label) in [
            (EditorTool::Select, "Select"),
            (EditorTool::Pan, "Pan"),
            (EditorTool::AddAtom(Element::from_symbol("C").expect("C")), "Carbon"),
            (EditorTool::AddAtom(Element::from_symbol("N").expect("N")), "Nitrogen"),
            (EditorTool::AddAtom(Element::from_symbol("O").expect("O")), "Oxygen"),
            (EditorTool::AddBond, "Bond"),
            (EditorTool::Erase, "Erase"),
            (EditorTool::Lasso, "Lasso"),
        ] {
            if ui.selectable_label(self.editor.tool == tool, label).clicked() {
                self.editor.tool = tool;
            }
        }
        ui.separator();
        ui.heading("Bond order");
        for (order, label) in [
            (egui_workbench_chem_core::BondOrder::Single, "Single"),
            (egui_workbench_chem_core::BondOrder::Double, "Double"),
            (egui_workbench_chem_core::BondOrder::Triple, "Triple"),
        ] {
            if ui.selectable_label(self.editor.bond_order == order, label).clicked() {
                self.editor.bond_order = order;
            }
        }
        ui.separator();
        ui.heading("Templates");
        egui::ComboBox::from_id_salt("ring")
            .selected_text(self.ring.label())
            .show_ui(ui, |ui| {
                for template in RingTemplate::ALL {
                    ui.selectable_value(&mut self.ring, template, template.label());
                }
            });
        if ui.button(format!("Stamp {}", self.ring.label())).clicked() {
            self.editor.tool = EditorTool::AddRing(self.ring);
        }
        ui.add(egui::Slider::new(&mut self.chain, 2..=12).text("Chain length"));
        if ui.button("Chain tool").clicked() {
            self.editor.tool = EditorTool::AddChain { count: self.chain };
        }
        ui.separator();
        ui.heading("Reaction");
        if ui.button("Add reaction arrow").clicked() {
            let start = [self.editor.camera.center[0] - 3.0, self.editor.camera.center[1] + 3.0];
            let end = [start[0] + 5.0, start[1]];
            let _ = self
                .editor
                .add_reaction_step(ReactionArrow::new(ArrowKind::Reaction, start, end));
        }
        if ui.button("Add conditions box").clicked() {
            let _ = self
                .editor
                .add_annotation(Annotation::Conditions(egui_workbench_chem2d::ConditionBox::line(
                    [self.editor.camera.center[0] - 2.0, self.editor.camera.center[1] + 4.0],
                    "H_2O, 25 ^oC",
                )));
        }
        ui.separator();
        ui.heading("Superatom");
        if ui.button("Insert and expand COOH").clicked() {
            let pseudo = self.editor.document.molecule.add_atom(
                Element::UNKNOWN,
                [self.editor.camera.center[0], self.editor.camera.center[1], 0.0],
            );
            if let Some(atom) = self.editor.document.molecule.atom_mut(pseudo) {
                atom.label = Some("COOH".to_owned());
            }
            match self.editor.expand_superatom(pseudo) {
                Ok(_) => self.status = "Expanded COOH.".to_owned(),
                Err(error) => self.status = error.to_string(),
            }
        }
        if ui.button("Contract last superatom").clicked()
            && let Some(group) = self.editor.document.superatoms.last().cloned()
        {
            match self.editor.contract_superatom(group) {
                Ok(()) => self.status = "Contracted.".to_owned(),
                Err(error) => self.status = error.to_string(),
            }
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("Document");
        ui.label(format!("{} atoms", self.editor.document.molecule.atom_count()));
        let properties = self.editor.properties();
        ui.separator();
        ui.heading("Properties");
        ui.label(format!("Formula: {}", properties.formula));
        ui.label(format!("MW: {:.2}", properties.molecular_weight));
        ui.label(format!("Exact: {:.4}", properties.exact_mass));
        ui.label(format!("DBE: {:.1}", properties.double_bond_equivalents));
        let issues = self.editor.validate_valences();
        if issues.is_empty() {
            ui.label("Valences: OK");
        } else {
            for (atom, excess) in issues {
                ui.label(format!("Atom {}: +{excess} valence", atom.slot()));
            }
        }
        ui.separator();
        ui.heading("SMILES");
        ui.text_edit_singleline(&mut self.smiles);
        if ui.button("Insert SMILES").clicked() {
            let center = [self.editor.camera.center[0] - 3.0, self.editor.camera.center[1] - 3.0];
            match self.editor.insert_smiles(&self.smiles, center) {
                Ok(()) => self.status = "SMILES inserted.".to_owned(),
                Err(error) => self.status = error.to_string(),
            }
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Undo").clicked() {
                self.editor.undo();
            }
            if ui.button("Redo").clicked() {
                self.editor.redo();
            }
            if ui.button("Clean").clicked() {
                let _ = self.editor.clean_layout();
            }
        });
        if ui.button("Export SVG").clicked() {
            let svg = self.editor.export_svg(ExportOptions::default());
            let path = std::env::temp_dir().join("egui-workbench-chem2d.svg");
            match std::fs::write(&path, svg) {
                Ok(()) => self.status = format!("SVG: {}", path.display()),
                Err(error) => self.status = error.to_string(),
            }
        }
        if ui.button("Export PNG 300 dpi").clicked() {
            match egui_workbench_chem2d::to_png(&self.editor.document, ExportOptions::default(), 300) {
                Ok(bytes) => {
                    let path = std::env::temp_dir().join("egui-workbench-chem2d.png");
                    match std::fs::write(&path, bytes) {
                        Ok(()) => self.status = format!("PNG: {}", path.display()),
                        Err(error) => self.status = error.to_string(),
                    }
                }
                Err(error) => self.status = error,
            }
        }
        ui.separator();
        ui.label(&self.status);
    }
}

impl eframe::App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("chem2d_tools")
            .resizable(true)
            .default_size(220.0)
            .show(ui, |ui| self.tools(ui));
        egui::Panel::right("chem2d_properties")
            .resizable(true)
            .default_size(300.0)
            .show(ui, |ui| self.sidebar(ui));
        egui::CentralPanel::default().show(ui, |ui| {
            self.editor.show(ui);
        });
    }
}
