//! The editor widget interaction and overlays.

use egui::{Key, PointerButton, Rect, Sense, Ui, Vec2, vec2};
use egui_workbench_chem_core::{BondOrder, Element, SelectionMode};

use super::{Chem2dEditor, Chem2dResponse, Drag, EditorTool, selection_mode};
use crate::history::Command;
use crate::interact::{Hit, hit_test, select_lasso, select_rect};
use crate::paint;

impl Chem2dEditor {
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
                            if self.add_bond(first, atom, self.bond_order).is_ok() {
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
                EditorTool::AddRing(template) => {
                    if self.add_ring(template, point, 0.0).is_ok() {
                        changed = true;
                    }
                }
                EditorTool::AddChain { count } => {
                    if self.add_chain(point, count, 0.0).is_ok() {
                        changed = true;
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
        if !ui.input(|input| input.modifiers.command || input.modifiers.alt) {
            let (element, order, rotate) = ui.input(|input| {
                let element = if input.key_pressed(Key::C) {
                    Some(Element::from_symbol("C"))
                } else if input.key_pressed(Key::N) {
                    Some(Element::from_symbol("N"))
                } else if input.key_pressed(Key::O) {
                    Some(Element::from_symbol("O"))
                } else if input.key_pressed(Key::S) {
                    Some(Element::from_symbol("S"))
                } else if input.key_pressed(Key::P) {
                    Some(Element::from_symbol("P"))
                } else if input.key_pressed(Key::F) {
                    Some(Element::from_symbol("F"))
                } else {
                    None
                };
                let order = if input.key_pressed(Key::Num1) {
                    Some(BondOrder::Single)
                } else if input.key_pressed(Key::Num2) {
                    Some(BondOrder::Double)
                } else if input.key_pressed(Key::Num3) {
                    Some(BondOrder::Triple)
                } else {
                    None
                };
                (element.flatten(), order, input.key_pressed(Key::Space))
            });
            if let Some(element) = element {
                self.tool = EditorTool::AddAtom(element);
            }
            if let Some(order) = order {
                self.bond_order = order;
            }
            if rotate && self.rotate_selection(std::f64::consts::FRAC_PI_6).is_ok() {
                changed = true;
            }
        }
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
