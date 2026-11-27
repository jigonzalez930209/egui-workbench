//! Interactive 2D chemistry editor demo with a ChemDraw/MolView-style layout.
//!
//! Run with `cargo run -p egui-workbench-chem2d --example editor`.

mod icons;

use eframe::egui;
use egui_workbench_chem2d::annotations::{Annotation, ArrowKind, ReactionArrow};
use egui_workbench_chem2d::{
    BondOrder, BondStereo, Chem2dEditor, ChemDocument, Command, ConditionBox, DocumentId, EditorTool, Element,
    ExportOptions, LIBRARY, LibraryKind, RingTemplate,
};
use icons::{Icon, icon_button, icon_button_sized, paint_icon};

fn main() -> eframe::Result {
    eframe::run_native(
        "egui-workbench chem2d",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1120.0, 760.0]),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(EditorApp::new()))),
    )
}

/// Common elements shown beside the canvas, as MolView does.
const COMMON_ELEMENTS: [&str; 10] = ["H", "C", "N", "O", "P", "S", "F", "Cl", "Br", "I"];

/// Main periodic table rows, 18 columns. Zero is an empty cell.
const PERIODIC_ROWS: [[u8; 18]; 7] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
    [3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 6, 7, 8, 9, 10],
    [11, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 14, 15, 16, 17, 18],
    [19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36],
    [37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54],
    [55, 56, 0, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86],
    [
        87, 88, 0, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118,
    ],
];
const LANTHANIDES: [u8; 15] = [57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71];
const ACTINIDES: [u8; 15] = [89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103];

struct EditorApp {
    editor: Chem2dEditor,
    smiles: String,
    label: String,
    chain: usize,
    periodic_table_open: bool,
    library_filter: String,
    status: String,
}

impl EditorApp {
    fn new() -> Self {
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(1), "untitled"));
        editor.add_ring(RingTemplate::Benzene, [0.0, 0.0], 0.0).expect("ring");
        editor.insert_smiles("CCO", [4.0, 0.0]).expect("smiles");
        editor.selection.clear();
        editor.tool = EditorTool::AddBond;
        Self {
            editor,
            smiles: "c1ccncc1".to_owned(),
            label: String::new(),
            chain: 4,
            periodic_table_open: false,
            library_filter: String::new(),
            status: "Bond tool selected: click empty space for a bond, click an atom to grow it.".to_owned(),
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            if icon_button(
                ui,
                Icon::Select,
                self.editor.tool == EditorTool::Select,
                "Select and move (V)",
            )
            .clicked()
            {
                self.editor.tool = EditorTool::Select;
                self.status = "Select: drag a box, drag inside it to move, drag the top handle to rotate.".to_owned();
            }
            if icon_button(ui, Icon::Pan, self.editor.tool == EditorTool::Pan, "Pan the canvas").clicked() {
                self.editor.tool = EditorTool::Pan;
            }
            if icon_button(
                ui,
                Icon::Lasso,
                self.editor.tool == EditorTool::Lasso,
                "Free-form selection",
            )
            .clicked()
            {
                self.editor.tool = EditorTool::Lasso;
                self.status = "Lasso: drag a free-form selection.".to_owned();
            }
            if icon_button(
                ui,
                Icon::Erase,
                self.editor.tool == EditorTool::Erase,
                "Erase an atom or bond",
            )
            .clicked()
            {
                self.editor.tool = EditorTool::Erase;
                self.status = "Erase: click an atom or bond.".to_owned();
            }
            if icon_button(
                ui,
                Icon::Attach,
                self.editor.tool == EditorTool::AttachFragment,
                "Attach SMILES: click an atom to attach, or empty canvas to place",
            )
            .clicked()
            {
                self.editor.fragment_smiles = self.smiles.clone();
                self.editor.tool = EditorTool::AttachFragment;
                self.status = "Attach tool: click an atom to attach the SMILES fragment.".to_owned();
            }
            ui.separator();
            if icon_button(ui, Icon::Undo, false, "Undo (Ctrl+Z)").clicked() {
                self.editor.undo();
            }
            if icon_button(ui, Icon::Redo, false, "Redo (Ctrl+Shift+Z)").clicked() {
                self.editor.redo();
            }
            if icon_button(ui, Icon::Trash, false, "Clear the document").clicked() {
                self.editor.clear_document().ok();
                self.status = "Canvas cleared.".to_owned();
            }
            if icon_button(
                ui,
                Icon::Clean,
                false,
                "Optimize 2D geometry: bond lengths, angles, rings (Ctrl+Shift+K)",
            )
            .clicked()
            {
                match self.editor.clean_layout() {
                    Ok(()) => self.status = "Layout cleaned.".to_owned(),
                    Err(error) => self.status = error.to_string(),
                }
            }
            if icon_button(ui, Icon::Fit, false, "Fit the structure in the canvas").clicked() {
                self.editor.fit();
            }
        });
    }

    fn bond_palette(&mut self, ui: &mut egui::Ui) {
        ui.heading("Bonds");
        ui.horizontal_wrapped(|ui| {
            for (order, icon, hover) in [
                (BondOrder::Single, Icon::BondSingle, "Single bond (1)"),
                (BondOrder::Double, Icon::BondDouble, "Double bond (2)"),
                (BondOrder::Triple, Icon::BondTriple, "Triple bond (3)"),
                (BondOrder::Aromatic, Icon::BondAromatic, "Aromatic bond (4)"),
            ] {
                let selected = self.editor.bond_order == order && self.editor.tool == EditorTool::AddBond;
                if icon_button_sized(ui, icon, selected, egui::vec2(28.0, 28.0), hover).clicked() {
                    self.editor.bond_order = order;
                    self.editor.tool = EditorTool::AddBond;
                    self.status = "Bond: click empty space, click an atom to grow, or drag.".to_owned();
                }
            }
        });
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            for (stereo, icon, hover) in [
                (BondStereo::None, Icon::BondSingle, "Plain bond"),
                (BondStereo::Wedge, Icon::BondWedge, "Wedge bond"),
                (BondStereo::Hash, Icon::BondHash, "Hashed bond"),
                (BondStereo::Wavy, Icon::BondWavy, "Unknown stereochemistry"),
            ] {
                let selected = self.editor.bond_stereo == stereo && self.editor.tool == EditorTool::AddBond;
                if icon_button_sized(ui, icon, selected, egui::vec2(28.0, 28.0), hover).clicked() {
                    self.editor.bond_stereo = stereo;
                    self.editor.tool = EditorTool::AddBond;
                    self.status = "Bond style selected.".to_owned();
                }
            }
        });

        ui.separator();
        ui.heading("Rings");
        ui.horizontal_wrapped(|ui| {
            for template in RingTemplate::ALL {
                let selected = self.editor.tool == EditorTool::AddRing(template);
                let hover = format!(
                    "{} — click empty space to stamp, or an atom to attach",
                    template.label()
                );
                if icon_button_sized(ui, ring_icon(template), selected, egui::vec2(28.0, 28.0), &hover).clicked() {
                    self.editor.tool = EditorTool::AddRing(template);
                }
            }
        });

        ui.separator();
        ui.heading("Chain");
        ui.horizontal(|ui| {
            if icon_button_sized(
                ui,
                Icon::Chain,
                matches!(self.editor.tool, EditorTool::AddChain { .. }),
                egui::vec2(28.0, 28.0),
                "Zig-zag carbon chain",
            )
            .clicked()
            {
                self.editor.tool = EditorTool::AddChain { count: self.chain };
                self.status = "Chain: click to start a zig-zag carbon chain.".to_owned();
            }
            ui.add(egui::Slider::new(&mut self.chain, 2..=12).text("length"));
        });
    }

    fn atom_palette(&mut self, ui: &mut egui::Ui) {
        ui.heading("Atoms");
        egui::Grid::new("common_atoms")
            .num_columns(2)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for (index, symbol) in COMMON_ELEMENTS.iter().enumerate() {
                    let Some(element) = Element::from_symbol(symbol) else {
                        continue;
                    };
                    let active = self.editor.draw_element == element;
                    if ui
                        .add(
                            egui::Button::new(*symbol)
                                .min_size(egui::vec2(38.0, 26.0))
                                .selected(active),
                        )
                        .on_hover_text(format!("{symbol} atom"))
                        .clicked()
                    {
                        self.select_element(element);
                    }
                    if index % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
        ui.horizontal(|ui| {
            if icon_button(ui, Icon::PeriodicTable, false, "Open the periodic table").clicked() {
                self.periodic_table_open = true;
            }
            ui.label("Periodic table");
        });

        ui.separator();
        ui.heading("Charge");
        ui.horizontal(|ui| {
            if icon_button(
                ui,
                Icon::ChargePositive,
                self.editor.tool == EditorTool::Charge { delta: 1 },
                "Click an atom to add a positive charge",
            )
            .clicked()
            {
                self.editor.tool = EditorTool::Charge { delta: 1 };
                self.status = "Charge +: click an atom.".to_owned();
            }
            if icon_button(
                ui,
                Icon::ChargeNegative,
                self.editor.tool == EditorTool::Charge { delta: -1 },
                "Click an atom to add a negative charge",
            )
            .clicked()
            {
                self.editor.tool = EditorTool::Charge { delta: -1 };
                self.status = "Charge -: click an atom.".to_owned();
            }
            ui.label("selection");
            if icon_button_sized(
                ui,
                Icon::ChargePositive,
                false,
                egui::vec2(24.0, 22.0),
                "Add +1 to the selected atoms",
            )
            .clicked()
            {
                self.editor.adjust_selected_charge(1).ok();
            }
            if icon_button_sized(
                ui,
                Icon::ChargeNegative,
                false,
                egui::vec2(24.0, 22.0),
                "Add -1 to the selected atoms",
            )
            .clicked()
            {
                self.editor.adjust_selected_charge(-1).ok();
            }
        });

        ui.separator();
        self.libraries(ui);
        ui.separator();
        ui.heading("Label");
        ui.text_edit_singleline(&mut self.label);
        ui.horizontal(|ui| {
            if ui.button("Set").clicked() {
                match self.editor.set_selected_label(Some(self.label.as_str())) {
                    Ok(true) => self.status = format!("Label set to {}.", self.label),
                    Ok(false) => self.status = "Select an atom first.".to_owned(),
                    Err(error) => self.status = error.to_string(),
                }
            }
            if ui.button("Clear").clicked() {
                self.editor.set_selected_label(None).ok();
            }
        });

        ui.separator();
        self.properties(ui);
        ui.separator();
        self.smiles_panel(ui);
        ui.separator();
        self.advanced(ui);
    }

    fn libraries(&mut self, ui: &mut egui::Ui) {
        ui.heading("Libraries");
        ui.add(egui::TextEdit::singleline(&mut self.library_filter).hint_text("Search amino acids, sugars, groups…"));
        let filter = self.library_filter.trim().to_ascii_lowercase();
        let mut selected = None;
        for group in LIBRARY {
            let entries = group
                .entries
                .iter()
                .filter(|entry| filter.is_empty() || entry.name.to_ascii_lowercase().contains(&filter))
                .collect::<Vec<_>>();
            if entries.is_empty() {
                continue;
            }
            let id = ui.make_persistent_id(("chem2d_library", group.name));
            let mut state =
                egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, !filter.is_empty());
            if !filter.is_empty() {
                state.set_open(true);
            }
            let _ = state
                .show_header(ui, |ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                    paint_icon(ui.painter(), rect, library_icon(group.name), ui.visuals().text_color());
                    ui.label(egui::RichText::new(group.name).strong());
                })
                .body(|ui| {
                    for entry in entries {
                        let active = self.editor.tool == EditorTool::AttachFragment
                            && self.editor.fragment_smiles == entry.smiles;
                        if ui
                            .add(
                                egui::Button::new(entry.name)
                                    .min_size(egui::vec2(ui.available_width(), 20.0))
                                    .selected(active),
                            )
                            .on_hover_text(entry.smiles)
                            .clicked()
                        {
                            selected = Some(entry);
                        }
                    }
                });
        }
        if let Some(entry) = selected {
            self.editor.fragment_smiles = entry.smiles.to_owned();
            self.editor.tool = EditorTool::AttachFragment;
            self.status = match entry.kind {
                LibraryKind::Molecule => {
                    format!(
                        "{}: click empty canvas to place it, or an atom to attach it.",
                        entry.name
                    )
                }
                LibraryKind::Group => {
                    format!(
                        "{}: click an atom to attach it, or empty canvas to place it.",
                        entry.name
                    )
                }
            };
        }
    }

    fn properties(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Properties")
            .default_open(true)
            .show(ui, |ui| {
                let properties = self.editor.properties();
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
                ui.label(format!(
                    "{} atoms · {} bonds",
                    self.editor.document.molecule.atom_count(),
                    self.editor.document.molecule.bond_count()
                ));
            });
    }

    fn smiles_panel(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("SMILES").default_open(true).show(ui, |ui| {
            ui.text_edit_singleline(&mut self.smiles);
            if ui.button("Insert SMILES").clicked() {
                let center = [self.editor.camera.center[0] - 3.0, self.editor.camera.center[1] - 3.0];
                match self.editor.insert_smiles(&self.smiles, center) {
                    Ok(()) => self.status = "SMILES inserted.".to_owned(),
                    Err(error) => self.status = error.to_string(),
                }
            }
            if icon_button(
                ui,
                Icon::Attach,
                self.editor.tool == EditorTool::AttachFragment,
                "Attach SMILES: click an atom to attach, or empty canvas to place",
            )
            .clicked()
            {
                self.editor.fragment_smiles = self.smiles.clone();
                self.editor.tool = EditorTool::AttachFragment;
                self.status = "Click an atom to attach the SMILES fragment.".to_owned();
            }
            ui.small("Ctrl+V over an atom attaches SMILES there.");
        });
    }

    fn advanced(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Reaction and groups")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if icon_button(ui, Icon::Reaction, false, "Add a reaction arrow").clicked() {
                        let start = [self.editor.camera.center[0] - 3.0, self.editor.camera.center[1] + 3.0];
                        let end = [start[0] + 5.0, start[1]];
                        let _ = self
                            .editor
                            .add_reaction_step(ReactionArrow::new(ArrowKind::Reaction, start, end));
                    }
                    if icon_button(ui, Icon::Conditions, false, "Add a conditions box").clicked() {
                        let _ = self.editor.add_annotation(Annotation::Conditions(ConditionBox::line(
                            [self.editor.camera.center[0] - 2.0, self.editor.camera.center[1] + 4.0],
                            "H_2O, 25 ^oC",
                        )));
                    }
                });
                if ui.button("Pseudo COOH group").clicked() {
                    let position = [self.editor.camera.center[0], self.editor.camera.center[1]];
                    if self
                        .editor
                        .execute(&Command::AddAtom {
                            element: Element::UNKNOWN,
                            position,
                        })
                        .is_ok()
                    {
                        self.editor.set_selected_label(Some("COOH")).ok();
                        self.status = "Pseudo COOH inserted; double-click it to edit the label.".to_owned();
                    }
                }
                if ui.button("Expand selected group").clicked() {
                    let atoms = self.editor.selection.atoms().iter().copied().collect::<Vec<_>>();
                    for atom in atoms {
                        let _ = self.editor.expand_superatom(atom);
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button("Export SVG").clicked() {
                        let svg = self.editor.export_svg(ExportOptions::default());
                        let path = std::env::temp_dir().join("egui-workbench-chem2d.svg");
                        match std::fs::write(&path, svg) {
                            Ok(()) => self.status = format!("SVG: {}", path.display()),
                            Err(error) => self.status = error.to_string(),
                        }
                    }
                    if ui.button("Export PNG").clicked() {
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
                });
            });
    }

    fn select_element(&mut self, element: Element) {
        self.editor.draw_element = element;
        self.editor.tool = EditorTool::AddAtom(element);
        self.status = format!(
            "{} selected: click an atom to relabel it or empty space to add it.",
            element.symbol()
        );
    }

    fn periodic_table(&mut self, context: &egui::Context) {
        if !self.periodic_table_open {
            return;
        }
        let mut open = true;
        let mut chosen = None;
        let current = self.editor.draw_element;
        egui::Window::new("Periodic table")
            .id(egui::Id::unique("chem2d_periodic_table"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(context, |ui| {
                for row in PERIODIC_ROWS {
                    ui.horizontal(|ui| element_row(ui, &row, current, &mut chosen));
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Ln");
                    element_row(ui, &LANTHANIDES, current, &mut chosen);
                });
                ui.horizontal(|ui| {
                    ui.label("An");
                    element_row(ui, &ACTINIDES, current, &mut chosen);
                });
            });
        self.periodic_table_open = open && chosen.is_none();
        if let Some(element) = chosen {
            self.select_element(element);
        }
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(&self.status);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "{} atoms · {} bonds · zoom {:.0}%",
                    self.editor.document.molecule.atom_count(),
                    self.editor.document.molecule.bond_count(),
                    self.editor.camera.zoom / 40.0 * 100.0
                ));
            });
        });
    }
}

impl eframe::App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("chem2d_toolbar")
            .exact_size(38.0)
            .show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("chem2d_status")
            .exact_size(24.0)
            .show(ui, |ui| self.status_bar(ui));
        egui::Panel::left("chem2d_bonds")
            .resizable(false)
            .default_size(132.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.bond_palette(ui));
            });
        egui::Panel::right("chem2d_atoms")
            .resizable(true)
            .default_size(210.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.atom_palette(ui));
            });
        egui::CentralPanel::default().show(ui, |ui| {
            self.editor.show(ui);
        });
        let context = ui.ctx().clone();
        self.periodic_table(&context);
    }
}

fn ring_icon(template: RingTemplate) -> Icon {
    match template {
        RingTemplate::Benzene => Icon::RingBenzene,
        RingTemplate::Cyclohexane => Icon::RingCyclohexane,
        RingTemplate::Cyclopentadiene => Icon::RingCyclopentadiene,
        RingTemplate::Pyridine => Icon::RingPyridine,
        RingTemplate::Pyrrole => Icon::RingPyrrole,
        RingTemplate::Chair => Icon::RingChair,
        RingTemplate::Boat => Icon::RingBoat,
        _ => Icon::Heterocycle,
    }
}

fn library_icon(name: &str) -> Icon {
    match name {
        "Amino acids" | "Amino acid side chains" => Icon::AminoAcid,
        "Sugars" => Icon::Sugar,
        "Nucleobases" => Icon::Nucleobase,
        "Rings and heterocycles" => Icon::Heterocycle,
        "Functional groups" => Icon::FunctionalGroup,
        "Solvents" => Icon::Solvent,
        "Ligands and chelators" => Icon::Ligand,
        "Lipids and building blocks" => Icon::Lipid,
        _ => Icon::Heterocycle,
    }
}

fn element_row(ui: &mut egui::Ui, row: &[u8], current: Element, chosen: &mut Option<Element>) {
    for number in row {
        if *number == 0 {
            ui.add_space(30.0);
            continue;
        }
        let Some(element) = Element::new(*number) else {
            continue;
        };
        if ui
            .add(
                egui::Button::new(element.symbol())
                    .min_size(egui::vec2(28.0, 24.0))
                    .selected(current == element),
            )
            .on_hover_text(format!("{} ({})", element.symbol(), element.atomic_number()))
            .clicked()
        {
            *chosen = Some(element);
        }
    }
}
