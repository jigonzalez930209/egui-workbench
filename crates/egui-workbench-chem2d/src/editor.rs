//! The ready-to-use 2D editor widget.

use egui::{Key, PointerButton, Rect, Response, Sense, Ui, Vec2, pos2, vec2};
use egui_workbench_chem_core::{AtomId, BondOrder, ChemSelection, Element, SelectionMode};

use crate::camera::Camera2D;
use crate::document::ChemDocument;
use crate::history::{Command, History};
use crate::interact::{Hit, hit_test, select_lasso, select_rect};
use crate::{Chem2dError, paint};

/// The active editing tool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditorTool {
    /// Click to select; drag empty space for a rectangle; drag an atom to move it.
    #[default]
    Select,
    /// Drag to pan.
    Pan,
    /// Click to add an atom of this element.
    AddAtom(Element),
    /// Click two atoms to bond them.
    AddBond,
    /// Click an atom or bond to delete it.
    Erase,
    /// Drag a free-form lasso.
    Lasso,
}

/// What the editor did this frame.
pub struct Chem2dResponse {
    /// The viewport response.
    pub response: Response,
    /// The document changed (including undo/redo).
    pub changed: bool,
    /// The selection changed.
    pub selection_changed: bool,
    /// The atom or bond under the pointer.
    pub hovered: Option<Hit>,
}

enum Drag {
    Pan,
    Move {
        atoms: Vec<(AtomId, [f64; 2])>,
        start: [f64; 2],
        delta: [f64; 2],
    },
    Rect {
        start: [f64; 2],
        current: [f64; 2],
    },
    Lasso {
        points: Vec<[f64; 2]>,
    },
}

/// A self-contained 2D editor: document, camera, selection, history, and tools.
pub struct Chem2dEditor {
    /// The document being edited.
    pub document: ChemDocument,
    /// The camera.
    pub camera: Camera2D,
    /// The shared selection.
    pub selection: ChemSelection,
    /// Undo/redo history.
    pub history: History,
    /// The active tool.
    pub tool: EditorTool,
    /// Whether the 1-2-5 grid draws.
    pub grid: bool,
    drag: Option<Drag>,
    pending_bond: Option<AtomId>,
}

impl Chem2dEditor {
    /// An editor for `document`, fitted to its atoms.
    pub fn new(document: ChemDocument) -> Self {
        let mut editor = Self {
            document,
            camera: Camera2D::default(),
            selection: ChemSelection::new(),
            history: History::new(),
            tool: EditorTool::Select,
            grid: true,
            drag: None,
            pending_bond: None,
        };
        editor.fit();
        editor
    }

    /// Fit the camera to the document.
    pub fn fit(&mut self) {
        if let Some(bounds) = self.document.bounds() {
            let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 420.0));
            self.camera.fit(bounds, viewport, 40.0);
        } else {
            self.camera = Camera2D::default();
        }
    }

    /// Add an atom through the history.
    ///
    /// # Errors
    /// When the document rejects the atom.
    pub fn add_atom(&mut self, element: Element, position: [f64; 2]) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::AddAtom { element, position },
        )
    }

    /// Add a bond through the history.
    ///
    /// # Errors
    /// When either atom is unknown or the bond already exists.
    pub fn add_bond(&mut self, a: AtomId, b: AtomId, order: BondOrder) -> Result<(), Chem2dError> {
        self.history.execute(
            &mut self.document,
            &mut self.selection,
            &Command::AddBond { a, b, order },
        )
    }

    /// Delete the selection through the history.
    ///
    /// # Errors
    /// When the document rejects the change.
    pub fn delete_selection(&mut self) -> Result<(), Chem2dError> {
        self.history
            .execute(&mut self.document, &mut self.selection, &Command::DeleteSelection)
    }

    /// Undo one command.
    pub fn undo(&mut self) -> bool {
        self.history.undo(&mut self.document, &mut self.selection)
    }

    /// Redo one command.
    pub fn redo(&mut self) -> bool {
        self.history.redo(&mut self.document, &mut self.selection)
    }

    /// Draw and interact with the editor.
    pub fn show(&mut self, ui: &mut Ui) -> Chem2dResponse {
        let available = ui.available_size();
        let size = vec2(available.x.max(240.0), available.y.max(200.0));
        let (viewport, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let mut changed = false;
        let mut selection_changed = false;
        let pointer = response.hover_pos();
        let world = pointer.map(|pointer| self.camera.screen_to_world(pointer, viewport));
        let hovered = world.and_then(|point| hit_test(&self.document.molecule, point, 10.0 / self.camera.zoom));

        if let Some(pointer) = pointer {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.camera
                    .zoom_at((f64::from(scroll) * 0.002).exp(), pointer, viewport);
                ui.ctx().input_mut(|input| input.smooth_scroll_delta = Vec2::ZERO);
            }
        }

        if response.drag_started() {
            self.drag = Some(match self.tool {
                EditorTool::Pan => Drag::Pan,
                _ if response.drag_started_by(PointerButton::Middle) => Drag::Pan,
                EditorTool::Lasso => Drag::Lasso { points: Vec::new() },
                EditorTool::Select => match hovered {
                    Some(Hit::Atom(atom)) => {
                        let ids = if self.selection.atoms().contains(&atom) {
                            self.selection.atoms().iter().copied().collect::<Vec<_>>()
                        } else {
                            vec![atom]
                        };
                        let atoms = ids
                            .into_iter()
                            .filter_map(|id| self.document.molecule.position_2d(id).map(|position| (id, position)))
                            .collect();
                        Drag::Move {
                            atoms,
                            start: world.unwrap_or([0.0; 2]),
                            delta: [0.0; 2],
                        }
                    }
                    _ => Drag::Rect {
                        start: world.unwrap_or([0.0; 2]),
                        current: world.unwrap_or([0.0; 2]),
                    },
                },
                _ => Drag::Rect {
                    start: world.unwrap_or([0.0; 2]),
                    current: world.unwrap_or([0.0; 2]),
                },
            });
        }

        if response.dragged()
            && let Some(drag) = &mut self.drag
        {
            match drag {
                Drag::Pan => self.camera.pan(response.drag_delta()),
                Drag::Move { atoms, start, delta } => {
                    if let Some(current) = world {
                        *delta = [current[0] - start[0], current[1] - start[1]];
                        for (id, origin) in atoms {
                            self.document
                                .molecule
                                .set_position_2d(*id, [origin[0] + delta[0], origin[1] + delta[1]]);
                        }
                    }
                }
                Drag::Rect { current, .. } => {
                    if let Some(point) = world {
                        *current = point;
                    }
                }
                Drag::Lasso { points } => {
                    if let Some(current) = world
                        && points
                            .last()
                            .is_none_or(|last| (last[0] - current[0]).abs() + (last[1] - current[1]).abs() > 0.02)
                    {
                        points.push(current);
                    }
                }
            }
        }

        if response.drag_stopped()
            && let Some(drag) = self.drag.take()
        {
            match drag {
                Drag::Pan => {}
                Drag::Move { atoms, delta, .. } => {
                    if delta[0].abs() + delta[1].abs() > 1.0e-6 {
                        let ids = atoms.iter().map(|(id, _)| *id).collect::<Vec<_>>();
                        for (id, origin) in &atoms {
                            self.document.molecule.set_position_2d(*id, *origin);
                        }
                        if self
                            .history
                            .execute(
                                &mut self.document,
                                &mut self.selection,
                                &Command::TranslateFragment { atoms: ids, delta },
                            )
                            .is_ok()
                        {
                            changed = true;
                        }
                    }
                }
                Drag::Rect { start, current } => {
                    let mode = selection_mode(ui);
                    let before = self.selection.clone();
                    select_rect(&self.document.molecule, &mut self.selection, start, current, mode);
                    selection_changed = self.selection != before;
                }
                Drag::Lasso { points } => {
                    let mode = selection_mode(ui);
                    let before = self.selection.clone();
                    select_lasso(&self.document.molecule, &mut self.selection, &points, mode);
                    selection_changed = self.selection != before;
                }
            }
        }

        if response.clicked()
            && let Some(point) = world
        {
            let hit = hit_test(&self.document.molecule, point, 10.0 / self.camera.zoom);
            match self.tool {
                EditorTool::Select => {
                    let before = self.selection.clone();
                    match hit {
                        Some(Hit::Atom(atom)) => self.selection.select_atom(atom, selection_mode(ui)),
                        Some(Hit::Bond(bond)) => self.selection.select_bond(bond, selection_mode(ui)),
                        None => self.selection.clear(),
                    }
                    selection_changed = self.selection != before;
                }
                EditorTool::AddAtom(element) => {
                    if self.add_atom(element, point).is_ok() {
                        changed = true;
                    }
                }
                EditorTool::AddBond => match hit {
                    Some(Hit::Atom(atom)) => match self.pending_bond {
                        Some(first) if first != atom => {
                            if self.add_bond(first, atom, BondOrder::Single).is_ok() {
                                changed = true;
                            }
                            self.pending_bond = None;
                        }
                        Some(_) => self.pending_bond = None,
                        None => self.pending_bond = Some(atom),
                    },
                    _ => self.pending_bond = None,
                },
                EditorTool::Erase => {
                    if let Some(hit) = hit {
                        self.selection.clear();
                        match hit {
                            Hit::Atom(atom) => {
                                self.selection.select_atom(atom, SelectionMode::Replace);
                            }
                            Hit::Bond(bond) => {
                                self.selection.select_bond(bond, SelectionMode::Replace);
                            }
                        }
                        if self.delete_selection().is_ok() {
                            changed = true;
                        }
                    }
                }
                EditorTool::Pan | EditorTool::Lasso => {}
            }
        }

        let (undo, redo, delete) = ui.input(|input| {
            let command = input.modifiers.command;
            (
                command && input.key_pressed(Key::Z) && !input.modifiers.shift,
                command && ((input.key_pressed(Key::Z) && input.modifiers.shift) || input.key_pressed(Key::Y)),
                input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace),
            )
        });
        if undo && self.undo() {
            changed = true;
        }
        if redo && self.redo() {
            changed = true;
        }
        if delete && !self.selection.is_empty() && self.delete_selection().is_ok() {
            changed = true;
            selection_changed = true;
        }

        paint::paint(
            ui,
            viewport,
            &self.document,
            self.camera,
            &self.selection,
            self.grid,
            hovered,
        );
        self.paint_overlay(ui, viewport);
        Chem2dResponse {
            response,
            changed,
            selection_changed,
            hovered,
        }
    }

    fn paint_overlay(&self, ui: &Ui, viewport: Rect) {
        let painter = ui.painter_at(viewport);
        let selection = ui.visuals().selection.stroke.color;
        match &self.drag {
            Some(Drag::Rect { start, current }) => {
                let a = self.camera.world_to_screen(*start, viewport);
                let b = self.camera.world_to_screen(*current, viewport);
                painter.rect(
                    Rect::from_two_pos(a, b),
                    0.0,
                    selection.gamma_multiply(0.06),
                    egui::Stroke::new(1.0, selection),
                    egui::StrokeKind::Inside,
                );
            }
            Some(Drag::Lasso { points }) if points.len() > 1 => {
                let points = points
                    .iter()
                    .map(|point| self.camera.world_to_screen(*point, viewport))
                    .collect::<Vec<_>>();
                painter.add(egui::Shape::line(points, egui::Stroke::new(1.0, selection)));
            }
            _ => {}
        }
        if let Some(atom) = self.pending_bond
            && let Some(position) = self.document.molecule.position_2d(atom)
        {
            let center = self.camera.world_to_screen(position, viewport);
            painter.circle_stroke(center, 11.0, egui::Stroke::new(1.5, selection));
        }
    }
}

fn selection_mode(ui: &Ui) -> SelectionMode {
    let (shift, command) = ui.input(|input| (input.modifiers.shift, input.modifiers.command));
    if command {
        SelectionMode::Toggle
    } else if shift {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::{DocumentId, Element};

    fn editor() -> Chem2dEditor {
        Chem2dEditor::new(ChemDocument::new(DocumentId::new(1), "test"))
    }

    #[test]
    fn editor_commands_go_through_history() {
        let mut editor = editor();
        editor
            .add_atom(Element::from_symbol("C").expect("carbon"), [0.0, 0.0])
            .expect("add");
        editor
            .add_atom(Element::from_symbol("O").expect("oxygen"), [1.2, 0.0])
            .expect("add");
        let ids = editor
            .document
            .molecule
            .atoms
            .iter()
            .map(|atom| atom.id)
            .collect::<Vec<_>>();
        editor.add_bond(ids[0], ids[1], BondOrder::Single).expect("bond");
        assert_eq!(editor.document.molecule.bond_count(), 1);
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.bond_count(), 0);
        assert!(editor.redo());
        assert_eq!(editor.document.molecule.bond_count(), 1);
    }

    #[test]
    fn the_editor_paints_without_a_panic() {
        let mut editor = editor();
        editor
            .add_atom(Element::from_symbol("C").expect("carbon"), [0.0, 0.0])
            .expect("add");
        let context = egui::Context::default();
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            editor.show(ui);
        });
        output.textures_delta.clear();
    }
}
