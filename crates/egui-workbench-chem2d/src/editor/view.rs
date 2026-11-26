//! The editor widget interaction and overlays.

use egui::{CursorIcon, Event, Key, PointerButton, Rect, Sense, Stroke, Ui, Vec2, pos2, vec2};
use egui_workbench_chem_core::{AtomId, BondOrder, BondStereo, Element, SelectionMode};

use super::{AtomEditorState, Chem2dEditor, Chem2dResponse, Drag, EditorTool, normalized_label, selection_mode};
use crate::history::Command;
use crate::interact::{Hit, hit_test, select_lasso, select_rect};
use crate::paint;
use crate::templates::snap_angle;

/// Bond length used by the drag-to-draw gesture, in Angstrom.
const DRAW_BOND_LENGTH: f64 = 1.5;
/// Pointer travel in Angstrom before a press becomes a drawn bond.
const DRAW_THRESHOLD: f64 = 0.35;
/// How long two typed letters still form one element symbol.
const ELEMENT_TYPING_WINDOW: f64 = 0.9;
/// Padding around a selection, in screen points.
const SELECTION_PADDING: f32 = 8.0;
/// Distance from the top of the selection box to the rotation handle.
const ROTATION_HANDLE_OFFSET: f32 = 24.0;
/// Screen-space radius for hitting the rotation handle.
const ROTATION_HANDLE_HIT: f32 = 11.0;
/// Screen-space radius drawn for the rotation handle.
const ROTATION_HANDLE_RADIUS: f32 = 5.0;

impl Chem2dEditor {
    /// Draw and interact with the editor.
    pub fn show(&mut self, ui: &mut Ui) -> Chem2dResponse {
        if let EditorTool::AddAtom(element) = self.tool {
            self.draw_element = element;
        }

        let available = ui.available_size();
        let size = vec2(available.x.max(240.0), available.y.max(200.0));
        let (viewport, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        self.viewport = viewport;
        if response.clicked() || response.drag_started() {
            response.request_focus();
        }
        let mut changed = false;
        let mut selection_changed = false;
        let pointer = response.hover_pos();
        let world = pointer.map(|pointer| self.camera.screen_to_world(pointer, viewport));
        let tolerance = 10.0 / self.camera.zoom;
        let hovered = world.and_then(|point| hit_test(&self.document.molecule, point, tolerance));
        if self.tool == EditorTool::Select {
            if let Some(pointer) = pointer
                && let Some(rect) = self.selection_screen_bounds(viewport)
            {
                if pointer.distance(rotation_handle(rect)) <= ROTATION_HANDLE_HIT {
                    ui.ctx()
                        .set_cursor_icon(if matches!(self.drag.as_ref(), Some(Drag::Rotate { .. })) {
                            CursorIcon::Grabbing
                        } else {
                            CursorIcon::Grab
                        });
                } else if rect.contains(pointer) {
                    ui.ctx().set_cursor_icon(CursorIcon::Move);
                }
            }
            if matches!(self.drag.as_ref(), Some(Drag::Move { .. } | Drag::Rotate { .. })) {
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
        }

        if let Some(pointer) = pointer {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.camera
                    .zoom_at((f64::from(scroll) * 0.002).exp(), pointer, viewport);
                ui.ctx().input_mut(|input| input.smooth_scroll_delta = Vec2::ZERO);
            }
        }

        if response.drag_started() {
            let press = ui
                .input(|input| input.pointer.press_origin())
                .or_else(|| response.interact_pointer_pos());
            let press_world = press.map(|press| self.camera.screen_to_world(press, viewport));
            let selection_drag = if self.tool == EditorTool::Select {
                self.selection_drag_start(press, press_world.or(world), viewport)
            } else {
                None
            };
            self.drag = Some(if let Some(drag) = selection_drag {
                drag
            } else {
                match self.tool {
                    EditorTool::Pan => Drag::Pan,
                    _ if response.drag_started_by(PointerButton::Middle) => Drag::Pan,
                    EditorTool::Lasso => Drag::Lasso { points: Vec::new() },
                    EditorTool::AddBond => {
                        let from = match hovered {
                            Some(Hit::Atom(atom)) => Some(atom),
                            _ => None,
                        };
                        let from_position = from
                            .and_then(|atom| self.document.molecule.position_2d(atom))
                            .or(world)
                            .unwrap_or([0.0; 2]);
                        Drag::DrawBond {
                            from,
                            from_position,
                            to: None,
                            current: from_position,
                            moved: false,
                        }
                    }
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
                                start: press_world.or(world).unwrap_or([0.0; 2]),
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
                }
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
                Drag::Rotate {
                    atoms,
                    center,
                    last_angle,
                    angle,
                } => {
                    if let Some(current) = world {
                        let current_angle = (current[1] - center[1]).atan2(current[0] - center[0]);
                        *angle += wrap_signed(current_angle - *last_angle);
                        *last_angle = current_angle;
                        let (sine, cosine) = angle.sin_cos();
                        for (id, origin) in atoms {
                            let x = origin[0] - center[0];
                            let y = origin[1] - center[1];
                            self.document.molecule.set_position_2d(
                                *id,
                                [center[0] + x * cosine - y * sine, center[1] + x * sine + y * cosine],
                            );
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
                Drag::DrawBond {
                    from,
                    from_position,
                    to,
                    current,
                    moved,
                } => {
                    if let Some(pointer) = world {
                        let target = match hit_test(&self.document.molecule, pointer, tolerance) {
                            Some(Hit::Atom(atom)) if Some(atom) != *from => Some(atom),
                            _ => None,
                        };
                        *moved |= (pointer[0] - from_position[0]).hypot(pointer[1] - from_position[1]) > DRAW_THRESHOLD;
                        *to = target;
                        *current = target
                            .and_then(|atom| self.document.molecule.position_2d(atom))
                            .unwrap_or_else(|| snapped_endpoint(*from_position, pointer));
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
                Drag::Rotate {
                    atoms, center, angle, ..
                } => {
                    for (id, origin) in &atoms {
                        self.document.molecule.set_position_2d(*id, *origin);
                    }
                    if angle.abs() > 1.0e-6 {
                        let ids = atoms.iter().map(|(id, _)| *id).collect::<Vec<_>>();
                        if self
                            .execute(&Command::RotateFragment {
                                atoms: ids,
                                center,
                                radians: angle,
                            })
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
                Drag::DrawBond {
                    from,
                    from_position,
                    to,
                    current,
                    moved,
                } => {
                    if moved
                        && self
                            .execute(&Command::DrawBond {
                                from,
                                from_position,
                                to,
                                to_position: current,
                                order: self.bond_order,
                                element: self.draw_element,
                                stereo: self.bond_stereo,
                            })
                            .is_ok()
                    {
                        changed = true;
                    }
                }
            }
        }

        if response.clicked()
            && let Some(point) = world
        {
            let hit = hit_test(&self.document.molecule, point, tolerance);
            match self.tool {
                EditorTool::Select => {
                    let before = self.selection.clone();
                    match hit {
                        Some(Hit::Atom(atom)) => self.selection.select_atom(atom, selection_mode(ui)),
                        Some(Hit::Bond(bond)) => self.selection.select_bond(bond, selection_mode(ui)),
                        None => {
                            let keep = pointer.is_some_and(|pointer| {
                                self.selection_screen_bounds(viewport).is_some_and(|rect| {
                                    rect.contains(pointer)
                                        || pointer.distance(rotation_handle(rect)) <= ROTATION_HANDLE_HIT
                                })
                            });
                            if !keep {
                                self.selection.clear();
                            }
                        }
                    }
                    selection_changed = self.selection != before;
                }
                EditorTool::AddAtom(element) => {
                    let before = self.selection.clone();
                    match hit {
                        Some(Hit::Atom(atom)) => {
                            self.selection.select_atom(atom, SelectionMode::Replace);
                            if self.set_selected_element(element).unwrap_or(false) {
                                changed = true;
                            }
                        }
                        Some(Hit::Bond(bond)) => {
                            self.selection.select_bond(bond, SelectionMode::Replace);
                        }
                        None => {
                            if self.add_atom(element, point).is_ok() {
                                changed = true;
                            }
                        }
                    }
                    selection_changed |= self.selection != before;
                }
                EditorTool::AddBond => match hit {
                    Some(Hit::Atom(atom)) => {
                        let before = self.selection.clone();
                        let from_position = self.document.molecule.position_2d(atom).unwrap_or([0.0; 2]);
                        if let Some(to_position) = sprout_endpoint(&self.document.molecule, atom)
                            && self
                                .execute(&Command::DrawBond {
                                    from: Some(atom),
                                    from_position,
                                    to: None,
                                    to_position,
                                    order: self.bond_order,
                                    element: self.draw_element,
                                    stereo: self.bond_stereo,
                                })
                                .is_ok()
                        {
                            changed = true;
                        }
                        selection_changed |= self.selection != before;
                    }
                    Some(Hit::Bond(bond)) => {
                        let before = self.selection.clone();
                        self.selection.select_bond(bond, SelectionMode::Replace);
                        let order = match self.document.molecule.bonds.iter().find(|entry| entry.id == bond) {
                            Some(entry) if entry.order == self.bond_order => {
                                if self.bond_order == BondOrder::Single {
                                    BondOrder::Double
                                } else if self.bond_order == BondOrder::Double {
                                    BondOrder::Single
                                } else {
                                    self.bond_order
                                }
                            }
                            _ => self.bond_order,
                        };
                        if self.set_selected_bond_order(order).unwrap_or(false) {
                            changed = true;
                        }
                        if self.set_selected_bond_stereo(self.bond_stereo).unwrap_or(false) {
                            changed = true;
                        }
                        selection_changed |= self.selection != before;
                    }
                    None => {
                        let before = self.selection.clone();
                        if self
                            .execute(&Command::DrawBond {
                                from: None,
                                from_position: point,
                                to: None,
                                to_position: [point[0] + DRAW_BOND_LENGTH, point[1]],
                                order: self.bond_order,
                                element: self.draw_element,
                                stereo: self.bond_stereo,
                            })
                            .is_ok()
                        {
                            changed = true;
                        }
                        selection_changed |= self.selection != before;
                    }
                },
                EditorTool::Charge { delta } => {
                    let before = self.selection.clone();
                    if let Some(Hit::Atom(atom)) = hit {
                        self.selection.select_atom(atom, SelectionMode::Replace);
                        if self.adjust_selected_charge(delta).unwrap_or(false) {
                            changed = true;
                        }
                    }
                    selection_changed |= self.selection != before;
                }
                EditorTool::AttachFragment => {
                    let smiles = self.fragment_smiles.clone();
                    if !smiles.trim().is_empty() {
                        let result = if let Some(Hit::Atom(atom)) = hit {
                            self.attach_smiles(&smiles, atom)
                        } else {
                            self.insert_smiles(&smiles, point)
                        };
                        if result.is_ok() {
                            changed = true;
                        }
                    }
                }
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
                            selection_changed = true;
                        }
                    }
                }
                EditorTool::AddRing(template) => {
                    let result = if let Some(Hit::Atom(atom)) = hit {
                        self.attach_ring(template, atom)
                    } else {
                        self.add_ring(template, point, 0.0)
                    };
                    if result.is_ok() {
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

        if response.double_clicked()
            && self.tool == EditorTool::Select
            && let Some(Hit::Atom(atom)) = world.and_then(|point| hit_test(&self.document.molecule, point, tolerance))
            && let Some(entry) = self.document.molecule.atom(atom)
        {
            self.atom_editor = Some(AtomEditorState {
                atom,
                symbol: entry.element.symbol().to_owned(),
                charge: i32::from(entry.charge),
                label: entry.label.clone().unwrap_or_default(),
            });
            let before = self.selection.clone();
            self.selection.select_atom(atom, SelectionMode::Replace);
            selection_changed |= self.selection != before;
        }

        let other_widget_focused = ui.memory(|memory| memory.focused().is_some_and(|id| id != response.id));
        if !ui.ctx().text_edit_focused() && !other_widget_focused && (response.has_focus() || response.hovered()) {
            let (edited, selection_delta) = self.handle_keyboard(ui, hovered, world, viewport);
            changed |= edited;
            selection_changed |= selection_delta;
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
        self.paint_overlay(ui, viewport, hovered);
        if self.show_atom_editor(ui, viewport) {
            changed = true;
        }
        Chem2dResponse {
            response,
            changed,
            selection_changed,
            hovered,
        }
    }

    fn selected_geometry_atoms(&self) -> Vec<AtomId> {
        let mut atoms = std::collections::BTreeSet::new();
        for atom in self.selection.atoms() {
            if self.document.molecule.atom(*atom).is_some() {
                atoms.insert(*atom);
            }
        }
        for bond in self.selection.bonds() {
            if let Some(entry) = self.document.molecule.bonds.iter().find(|entry| entry.id == *bond) {
                atoms.insert(entry.a);
                atoms.insert(entry.b);
            }
        }
        atoms.into_iter().collect()
    }

    fn selected_center(&self, atoms: &[AtomId]) -> Option<[f64; 2]> {
        let mut sum = [0.0; 2];
        let mut count = 0.0;
        for atom in atoms {
            if let Some(position) = self.document.molecule.position_2d(*atom) {
                sum[0] += position[0];
                sum[1] += position[1];
                count += 1.0;
            }
        }
        (count > 0.0).then(|| [sum[0] / count, sum[1] / count])
    }

    fn selection_screen_bounds(&self, viewport: Rect) -> Option<Rect> {
        let mut bounds: Option<Rect> = None;
        for atom in self.selected_geometry_atoms() {
            let Some(position) = self.document.molecule.position_2d(atom) else {
                continue;
            };
            let point = self.camera.world_to_screen(position, viewport);
            let point_rect = Rect::from_min_size(point, Vec2::ZERO);
            bounds = Some(bounds.map_or(point_rect, |current| current.union(point_rect)));
        }
        bounds.map(|rect| rect.expand(SELECTION_PADDING))
    }

    fn selection_drag_start(
        &self,
        pointer: Option<egui::Pos2>,
        world: Option<[f64; 2]>,
        viewport: Rect,
    ) -> Option<Drag> {
        let pointer = pointer?;
        let rect = self.selection_screen_bounds(viewport)?;
        let atoms = self.selected_geometry_atoms();
        let origins = atoms
            .iter()
            .filter_map(|id| self.document.molecule.position_2d(*id).map(|position| (*id, position)))
            .collect::<Vec<_>>();
        if origins.is_empty() {
            return None;
        }
        if pointer.distance(rotation_handle(rect)) <= ROTATION_HANDLE_HIT {
            let center = self.selected_center(&atoms)?;
            let start_world = world.unwrap_or_else(|| self.camera.screen_to_world(pointer, viewport));
            let start_angle = (start_world[1] - center[1]).atan2(start_world[0] - center[0]);
            return Some(Drag::Rotate {
                atoms: origins,
                center,
                last_angle: start_angle,
                angle: 0.0,
            });
        }
        if rect.contains(pointer) {
            return Some(Drag::Move {
                atoms: origins,
                start: world.unwrap_or([0.0; 2]),
                delta: [0.0; 2],
            });
        }
        None
    }

    fn handle_keyboard(
        &mut self,
        ui: &Ui,
        hovered: Option<Hit>,
        world: Option<[f64; 2]>,
        viewport: Rect,
    ) -> (bool, bool) {
        let mut changed = false;
        let mut selection_changed = false;
        let hovered_atom = match hovered {
            Some(Hit::Atom(atom)) => Some(atom),
            _ => None,
        };
        let (undo, redo, delete, select_all, escape, clean) = ui.input(|input| {
            let command = input.modifiers.command;
            (
                command && input.key_pressed(Key::Z) && !input.modifiers.shift,
                command && ((input.key_pressed(Key::Z) && input.modifiers.shift) || input.key_pressed(Key::Y)),
                input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace),
                command && input.key_pressed(Key::A),
                input.key_pressed(Key::Escape),
                command && input.modifiers.shift && input.key_pressed(Key::K),
            )
        });
        if escape {
            let had_editor = self.atom_editor.take().is_some();
            if !self.selection.is_empty() || had_editor {
                self.selection.clear();
                selection_changed = true;
            }
        }
        if undo && self.undo() {
            changed = true;
            selection_changed = true;
        }
        if redo && self.redo() {
            changed = true;
            selection_changed = true;
        }
        if select_all && self.select_all() {
            selection_changed = true;
        }
        if delete && !self.selection.is_empty() && self.delete_selection().is_ok() {
            changed = true;
            selection_changed = true;
        }
        if clean && self.clean_layout().is_ok() {
            changed = true;
        }

        let (command, alt, shift) =
            ui.input(|input| (input.modifiers.command, input.modifiers.alt, input.modifiers.shift));
        if !command && !alt {
            let (order, rotate, nudge, stereo, events) = ui.input(|input| {
                let order = if input.key_pressed(Key::Num1) {
                    Some(BondOrder::Single)
                } else if input.key_pressed(Key::Num2) {
                    Some(BondOrder::Double)
                } else if input.key_pressed(Key::Num3) {
                    Some(BondOrder::Triple)
                } else if input.key_pressed(Key::Num4) {
                    Some(BondOrder::Aromatic)
                } else {
                    None
                };
                let nudge = if input.key_pressed(Key::ArrowLeft) {
                    Some([-1.0, 0.0])
                } else if input.key_pressed(Key::ArrowRight) {
                    Some([1.0, 0.0])
                } else if input.key_pressed(Key::ArrowUp) {
                    Some([0.0, 1.0])
                } else if input.key_pressed(Key::ArrowDown) {
                    Some([0.0, -1.0])
                } else {
                    None
                };
                let stereo = if input.key_pressed(Key::W) {
                    Some(BondStereo::Wedge)
                } else if input.key_pressed(Key::H) {
                    Some(BondStereo::Hash)
                } else {
                    None
                };
                (
                    order,
                    input.key_pressed(Key::Space),
                    nudge,
                    stereo,
                    input.events.clone(),
                )
            });
            if let Some(order) = order {
                self.bond_order = order;
                if !self.selection.bonds().is_empty() {
                    if self.set_selected_bond_order(order).unwrap_or(false) {
                        changed = true;
                    }
                } else if let Some(Hit::Bond(bond)) = hovered
                    && self
                        .document
                        .molecule
                        .bonds
                        .iter()
                        .any(|entry| entry.id == bond && entry.order != order)
                    && self
                        .execute(&Command::SetBondOrders {
                            bonds: vec![bond],
                            order,
                        })
                        .is_ok()
                {
                    changed = true;
                }
            }
            if rotate && self.rotate_selection(std::f64::consts::FRAC_PI_6).is_ok() {
                changed = true;
            }
            if let Some(direction) = nudge {
                let step = if shift { 1.0 } else { 0.1 };
                if self
                    .nudge_selection([direction[0] * step, direction[1] * step])
                    .unwrap_or(false)
                {
                    changed = true;
                }
            }
            let mut stereo_applied = false;
            if let Some(stereo) = stereo {
                self.bond_stereo = stereo;
                if !self.selection.bonds().is_empty() {
                    if self.set_selected_bond_stereo(stereo).unwrap_or(false) {
                        changed = true;
                    }
                    stereo_applied = true;
                } else if let Some(Hit::Bond(bond)) = hovered {
                    stereo_applied = true;
                    if self
                        .document
                        .molecule
                        .bonds
                        .iter()
                        .any(|entry| entry.id == bond && entry.stereo != stereo)
                        && self
                            .execute(&Command::SetBondStereos {
                                bonds: vec![bond],
                                stereo,
                            })
                            .is_ok()
                    {
                        changed = true;
                    }
                }
            }
            let now = ui.input(|input| input.time);
            for event in events {
                match event {
                    Event::Text(text) if !stereo_applied => {
                        if text == "+" {
                            changed |= self.adjust_charge_around(hovered_atom, 1);
                        } else if text == "-" || text == "−" {
                            changed |= self.adjust_charge_around(hovered_atom, -1);
                        } else if text.chars().all(|character| character.is_ascii_alphabetic()) {
                            changed |= self.type_element(&text, hovered_atom, now);
                        }
                    }
                    Event::Paste(text) => {
                        changed |= self.paste_smiles(&text, hovered_atom, world, viewport);
                    }
                    _ => {}
                }
            }
        } else {
            // Command shortcuts still accept a SMILES paste.
            let events = ui.input(|input| input.events.clone());
            for event in events {
                if let Event::Paste(text) = event {
                    changed |= self.paste_smiles(&text, hovered_atom, world, viewport);
                }
            }
        }
        (changed, selection_changed)
    }

    fn paste_smiles(
        &mut self,
        text: &str,
        hovered_atom: Option<AtomId>,
        world: Option<[f64; 2]>,
        viewport: Rect,
    ) -> bool {
        let text = text.trim();
        if text.is_empty() || crate::smiles::parse_smiles(text).is_err() {
            return false;
        }
        // Hovering an atom means "attach this fragment here"; with no hovered atom the
        // fragment is inserted at the pointer. A single selected atom is the fallback.
        let anchor = hovered_atom.or_else(|| {
            let mut atoms = self.selection.atoms().iter();
            match (atoms.next(), atoms.next()) {
                (Some(atom), None) => Some(*atom),
                _ => None,
            }
        });
        if let Some(anchor) = anchor {
            return self.attach_smiles(text, anchor).is_ok();
        }
        let center = world.unwrap_or_else(|| self.camera.screen_to_world(viewport.center(), viewport));
        self.execute(&Command::InsertSmiles {
            text: text.to_owned(),
            center,
        })
        .is_ok()
    }

    fn type_element(&mut self, text: &str, hovered: Option<AtomId>, now: f64) -> bool {
        if now - self.element_buffer_time > ELEMENT_TYPING_WINDOW {
            self.element_buffer.clear();
        }
        self.element_buffer_time = now;
        self.element_buffer.push_str(text);
        let chars = self.element_buffer.chars().collect::<Vec<_>>();
        if chars.len() > 2 {
            self.element_buffer = chars[chars.len() - 2..].iter().collect();
        }
        let buffer = self.element_buffer.clone();
        let element = match Element::from_symbol(&buffer) {
            Some(element) => element,
            None => {
                let Some(last) = buffer.chars().last().map(|character| character.to_string()) else {
                    return false;
                };
                self.element_buffer = last.clone();
                let Some(element) = Element::from_symbol(&last) else {
                    self.element_buffer.clear();
                    return false;
                };
                element
            }
        };
        if !self.selection.atoms().is_empty() {
            return self.set_selected_element(element).unwrap_or(false);
        }
        if let Some(atom) = hovered
            && self.document.molecule.atom(atom).is_some()
            && self
                .execute(&Command::SetAtomElements {
                    atoms: vec![atom],
                    element,
                })
                .is_ok()
        {
            return true;
        }
        self.draw_element = element;
        if !matches!(self.tool, EditorTool::AddBond) {
            self.tool = EditorTool::AddAtom(element);
        }
        false
    }

    fn adjust_charge_around(&mut self, hovered: Option<AtomId>, delta: i8) -> bool {
        if !self.selection.atoms().is_empty() {
            return self.adjust_selected_charge(delta).unwrap_or(false);
        }
        let Some(atom) = hovered else {
            return false;
        };
        if self.document.molecule.atom(atom).is_none() {
            return false;
        }
        self.execute(&Command::AdjustAtomCharges {
            atoms: vec![atom],
            delta,
        })
        .is_ok()
    }

    fn paint_overlay(&self, ui: &Ui, viewport: Rect, hovered: Option<Hit>) {
        let painter = ui.painter_at(viewport);
        let selection = ui.visuals().selection.stroke.color;
        if self.drag.is_none()
            && self.tool == EditorTool::AddBond
            && let Some(Hit::Atom(atom)) = hovered
            && let (Some(from), Some(to)) = (
                self.document.molecule.position_2d(atom),
                sprout_endpoint(&self.document.molecule, atom),
            )
        {
            self.paint_draft_bond(ui, &painter, viewport, from, to, true);
        }
        match &self.drag {
            Some(Drag::Rect { start, current }) => {
                let a = self.camera.world_to_screen(*start, viewport);
                let b = self.camera.world_to_screen(*current, viewport);
                painter.rect(
                    Rect::from_two_pos(a, b),
                    0.0,
                    selection.gamma_multiply(0.06),
                    Stroke::new(1.0, selection),
                    egui::StrokeKind::Inside,
                );
            }
            Some(Drag::Lasso { points }) if points.len() > 1 => {
                let points = points
                    .iter()
                    .map(|point| self.camera.world_to_screen(*point, viewport))
                    .collect::<Vec<_>>();
                painter.add(egui::Shape::line(points, Stroke::new(1.0, selection)));
            }
            Some(Drag::DrawBond {
                from_position,
                to,
                current,
                ..
            }) => {
                self.paint_draft_bond(ui, &painter, viewport, *from_position, *current, to.is_none());
            }
            _ => {}
        }
        if !matches!(self.drag.as_ref(), Some(Drag::Rect { .. } | Drag::Lasso { .. })) {
            self.paint_selection_box(
                &painter,
                viewport,
                ui.visuals().selection.stroke.color,
                ui.visuals().panel_fill,
            );
        }
    }

    fn paint_selection_box(&self, painter: &egui::Painter, viewport: Rect, color: egui::Color32, fill: egui::Color32) {
        let Some(rect) = self.selection_screen_bounds(viewport) else {
            return;
        };
        let stroke = Stroke::new(1.2, color);
        let outline = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
            rect.left_top(),
        ];
        painter.extend(egui::Shape::dashed_line(&outline, stroke, 5.0, 3.0));
        let handle = rotation_handle(rect);
        painter.line_segment([pos2(rect.center().x, rect.top()), handle], Stroke::new(1.0, color));
        painter.circle(handle, ROTATION_HANDLE_RADIUS, fill, Stroke::new(1.5, color));
    }

    fn paint_draft_bond(
        &self,
        ui: &Ui,
        painter: &egui::Painter,
        viewport: Rect,
        from: [f64; 2],
        to: [f64; 2],
        ghost: bool,
    ) {
        let color = ui.visuals().selection.stroke.color;
        let start = self.camera.world_to_screen(from, viewport);
        let end = self.camera.world_to_screen(to, viewport);
        if !start.is_finite() || !end.is_finite() {
            return;
        }
        if start.distance(end) > 1.0 {
            painter.extend(egui::Shape::dashed_line(
                &[start, end],
                Stroke::new(1.5, color),
                5.0,
                3.0,
            ));
        }
        if ghost {
            let [r, g, b] = self.draw_element.cpk_color();
            let fill = egui::Color32::from_rgb(r, g, b).gamma_multiply(0.45);
            painter.circle(end, 8.0, fill, Stroke::new(1.5, color));
            painter.text(
                end,
                egui::Align2::CENTER_CENTER,
                self.draw_element.symbol(),
                egui::FontId::proportional(10.0),
                ui.visuals().text_color(),
            );
        }
    }

    fn show_atom_editor(&mut self, ui: &Ui, viewport: Rect) -> bool {
        let Some(mut state) = self.atom_editor.clone() else {
            return false;
        };
        let Some(atom) = self.document.molecule.atom(state.atom) else {
            self.atom_editor = None;
            return false;
        };
        let anchor = self
            .camera
            .world_to_screen([atom.position[0], atom.position[1]], viewport);
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        egui::Window::new("Atom properties")
            .id(ui.scope_id().with("chem2d_atom_editor"))
            .collapsible(false)
            .resizable(false)
            .default_pos(anchor + vec2(14.0, 14.0))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                egui::Grid::new("chem2d_atom_fields")
                    .num_columns(2)
                    .spacing([8.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Element");
                        ui.text_edit_singleline(&mut state.symbol);
                        ui.end_row();
                        ui.label("Charge");
                        ui.add(egui::DragValue::new(&mut state.charge).range(-15..=15));
                        ui.end_row();
                        ui.label("Label");
                        ui.text_edit_singleline(&mut state.label);
                        ui.end_row();
                    });
                if Element::from_symbol(state.symbol.trim()).is_none() {
                    ui.colored_label(ui.visuals().error_fg_color, "Unknown element symbol");
                }
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        apply = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if !open || cancel {
            self.atom_editor = None;
            return false;
        }
        if apply && let Some(element) = Element::from_symbol(state.symbol.trim()) {
            let label = normalized_label(Some(state.label.as_str()));
            let charge = state.charge.clamp(i32::from(i8::MIN), i32::from(i8::MAX)) as i8;
            let result = self.execute(&Command::SetAtomProperties {
                atom: state.atom,
                element,
                charge,
                label,
            });
            self.atom_editor = None;
            return result.is_ok();
        }
        self.atom_editor = Some(state);
        false
    }
}

/// Where a click on `atom` sprouts its next bond.
///
/// The direction is 120 degrees from an existing bond, which keeps a chain zigzagging
/// the way the bond tools in ChemDraw and MolView do. With no neighbours the bond grows
/// to the right, and a new direction is chosen to leave the largest gap between bonds.
fn sprout_endpoint(molecule: &crate::document::Molecule2D, atom: AtomId) -> Option<[f64; 2]> {
    let entry = molecule.atom(atom)?;
    let angle = crate::templates::sprout_angle(molecule, atom, DRAW_BOND_LENGTH)?;
    Some([
        entry.position[0] + DRAW_BOND_LENGTH * angle.cos(),
        entry.position[1] + DRAW_BOND_LENGTH * angle.sin(),
    ])
}

fn rotation_handle(rect: Rect) -> egui::Pos2 {
    pos2(rect.center().x, rect.top() - ROTATION_HANDLE_OFFSET)
}

fn wrap_signed(angle: f64) -> f64 {
    let mut angle = angle.rem_euclid(std::f64::consts::TAU);
    if angle > std::f64::consts::PI {
        angle -= std::f64::consts::TAU;
    }
    angle
}

fn snapped_endpoint(start: [f64; 2], pointer: [f64; 2]) -> [f64; 2] {
    let delta = [pointer[0] - start[0], pointer[1] - start[1]];
    if delta[0].hypot(delta[1]) <= f64::EPSILON {
        return start;
    }
    let angle = snap_angle(delta[1].atan2(delta[0]), 30.0);
    [
        start[0] + DRAW_BOND_LENGTH * angle.cos(),
        start[1] + DRAW_BOND_LENGTH * angle.sin(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, pos2, vec2};
    use egui_workbench_chem_core::{ChemSelection, DocumentId};

    use crate::ChemDocument;

    fn pointer_button(position: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        }
    }

    fn run_editor(context: &egui::Context, editor: &mut Chem2dEditor, events: Vec<Event>) -> Chem2dResponse {
        let mut input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };
        input.events = events;
        let mut response = None;
        let mut output = context.run_ui(input, |ui| {
            response = Some(editor.show(ui));
        });
        output.textures_delta.clear();
        response.expect("a response")
    }

    fn click_editor(context: &egui::Context, editor: &mut Chem2dEditor, position: Pos2) {
        run_editor(context, editor, vec![Event::PointerMoved(position)]);
        run_editor(context, editor, vec![pointer_button(position, true)]);
        run_editor(context, editor, vec![pointer_button(position, false)]);
    }

    #[test]
    fn the_attach_tool_grows_a_complex_fragment_from_a_clicked_atom() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(7), "attach"));
        editor
            .document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        editor.fragment_smiles = "c1ccncc1".to_owned();
        editor.tool = EditorTool::AttachFragment;
        click_editor(&context, &mut editor, pos2(400.0, 300.0));
        assert_eq!(editor.document.molecule.atom_count(), 7);
        assert_eq!(editor.document.molecule.bond_count(), 7);
    }

    #[test]
    fn the_attach_tool_places_a_fragment_on_empty_canvas() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(9), "place"));
        editor.fragment_smiles = "c1ccccc1".to_owned();
        editor.tool = EditorTool::AttachFragment;
        click_editor(&context, &mut editor, pos2(400.0, 300.0));
        assert_eq!(editor.document.molecule.atom_count(), 6);
        assert_eq!(editor.document.molecule.bond_count(), 6);
    }

    #[test]
    fn the_charge_tool_repeats_on_click() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(6), "charge"));
        let atom = editor
            .document
            .molecule
            .add_atom(Element::from_symbol("N").expect("N"), [0.0, 0.0, 0.0]);
        editor.tool = EditorTool::Charge { delta: 1 };
        click_editor(&context, &mut editor, pos2(400.0, 300.0));
        click_editor(&context, &mut editor, pos2(400.0, 300.0));
        assert_eq!(editor.document.molecule.atom(atom).expect("atom").charge, 2);
    }

    fn drag_editor(context: &egui::Context, editor: &mut Chem2dEditor, from: Pos2, to: Pos2) {
        run_editor(context, editor, vec![Event::PointerMoved(from)]);
        run_editor(context, editor, vec![pointer_button(from, true)]);
        run_editor(context, editor, vec![Event::PointerMoved(from + (to - from) * 0.5)]);
        run_editor(context, editor, vec![Event::PointerMoved(to)]);
        run_editor(context, editor, vec![pointer_button(to, false)]);
    }

    #[test]
    fn the_selection_box_moves_the_whole_selected_part() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(10), "box move"));
        let a = editor
            .document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let b = editor
            .document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [1.5, 0.0, 0.0]);
        editor.selection.select_atom(a, SelectionMode::Add);
        editor.selection.select_atom(b, SelectionMode::Add);
        drag_editor(&context, &mut editor, pos2(430.0, 300.0), pos2(470.0, 300.0));
        assert!((editor.document.molecule.atom(a).expect("a").position[0] - 1.0).abs() < 1.0e-9);
        assert!((editor.document.molecule.atom(b).expect("b").position[0] - 2.5).abs() < 1.0e-9);
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.atom(a).expect("a").position[0], 0.0);
    }

    #[test]
    fn the_rotation_handle_rotates_the_selected_part() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(11), "box rotate"));
        let a = editor
            .document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let b = editor
            .document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [1.5, 0.0, 0.0]);
        editor.selection.select_atom(a, SelectionMode::Add);
        editor.selection.select_atom(b, SelectionMode::Add);
        // The box is 76x76 screen points; its top-centre rotation handle is at (430, 268).
        drag_editor(&context, &mut editor, pos2(430.0, 268.0), pos2(460.0, 300.0));
        let position = editor.document.molecule.atom(a).expect("a").position;
        assert!(
            (position[0] - 0.75).abs() < 0.2 && (position[1] - 0.75).abs() < 0.2,
            "{position:?}"
        );
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.atom(a).expect("a").position[0], 0.0);
    }

    #[test]
    fn a_click_draws_a_bond_and_clicking_its_end_grows_the_chain() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(4), "sprout"));
        editor.tool = EditorTool::AddBond;
        click_editor(&context, &mut editor, pos2(400.0, 300.0));
        assert_eq!(editor.document.molecule.atom_count(), 2);
        assert_eq!(editor.document.molecule.bond_count(), 1);
        // The first click is world (0, 0); at 40 points/Angstrom the new atom sits at
        // screen x 460 and the next click sprouts from it.
        click_editor(&context, &mut editor, pos2(460.0, 300.0));
        assert_eq!(editor.document.molecule.atom_count(), 3);
        assert_eq!(editor.document.molecule.bond_count(), 2);
    }

    #[test]
    fn clicking_a_single_bond_with_the_single_tool_makes_it_double() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(5), "bond order"));
        editor.tool = EditorTool::AddBond;
        click_editor(&context, &mut editor, pos2(400.0, 300.0));
        let midpoint = pos2(430.0, 300.0);
        click_editor(&context, &mut editor, midpoint);
        assert_eq!(editor.document.molecule.bonds[0].order, BondOrder::Double);
        click_editor(&context, &mut editor, midpoint);
        assert_eq!(editor.document.molecule.bonds[0].order, BondOrder::Single);
    }

    #[test]
    fn dragging_with_the_bond_tool_creates_a_bond_and_its_endpoint() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(1), "drag"));
        editor.tool = EditorTool::AddBond;
        let start = pos2(200.0, 200.0);
        run_editor(&context, &mut editor, vec![Event::PointerMoved(start)]);
        run_editor(&context, &mut editor, vec![pointer_button(start, true)]);
        run_editor(&context, &mut editor, vec![Event::PointerMoved(pos2(230.0, 200.0))]);
        run_editor(&context, &mut editor, vec![Event::PointerMoved(pos2(260.0, 200.0))]);
        run_editor(&context, &mut editor, vec![pointer_button(pos2(260.0, 200.0), false)]);
        assert_eq!(editor.document.molecule.atom_count(), 2);
        assert_eq!(editor.document.molecule.bond_count(), 1);
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.atom_count(), 0);
    }

    #[test]
    fn typing_changes_the_selected_atom_and_plus_sets_its_charge() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(2), "type"));
        let atom = editor
            .document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        editor.selection = ChemSelection::new();
        editor.selection.select_atom(atom, SelectionMode::Replace);
        run_editor(&context, &mut editor, vec![Event::PointerMoved(pos2(400.0, 300.0))]);
        run_editor(&context, &mut editor, vec![Event::Text("O".to_owned())]);
        assert_eq!(
            editor.document.molecule.atom(atom).expect("atom").element,
            Element::from_symbol("O").expect("O")
        );
        run_editor(&context, &mut editor, vec![Event::Text("+".to_owned())]);
        assert_eq!(editor.document.molecule.atom(atom).expect("atom").charge, 1);
        run_editor(&context, &mut editor, vec![Event::Text("C".to_owned())]);
        run_editor(&context, &mut editor, vec![Event::Text("l".to_owned())]);
        assert_eq!(
            editor.document.molecule.atom(atom).expect("atom").element,
            Element::from_symbol("Cl").expect("Cl")
        );
    }

    #[test]
    fn pasted_smiles_is_trimmed_and_inserted() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(3), "paste"));
        run_editor(&context, &mut editor, vec![Event::PointerMoved(pos2(400.0, 300.0))]);
        run_editor(&context, &mut editor, vec![Event::Paste("  CCO\n".to_owned())]);
        assert_eq!(editor.document.molecule.atom_count(), 3);
        assert_eq!(editor.document.molecule.bond_count(), 2);
        assert!(editor.undo());
        assert_eq!(editor.document.molecule.atom_count(), 0);
    }

    #[test]
    fn a_click_sprouts_a_bond_at_the_next_chain_angle() {
        let mut molecule = crate::document::Molecule2D::new();
        let a = molecule.add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let b = molecule.add_atom(Element::from_symbol("C").expect("C"), [1.5, 0.0, 0.0]);
        molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        let endpoint = sprout_endpoint(&molecule, b).expect("endpoint");
        let angle = (endpoint[1] - 0.0).atan2(endpoint[0] - 1.5).to_degrees();
        assert!((angle - 60.0).abs() < 1.0e-9, "{angle}");
        let next = sprout_endpoint(&molecule, a).expect("endpoint");
        let angle = next[1].atan2(next[0]).to_degrees();
        assert!((angle - 120.0).abs() < 1.0e-9, "{angle}");
    }

    #[test]
    fn pasting_smiles_over_an_atom_attaches_it_instead_of_inserting_a_copy() {
        let context = egui::Context::default();
        let mut editor = Chem2dEditor::new(ChemDocument::new(DocumentId::new(8), "paste attach"));
        let anchor = editor
            .document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        run_editor(&context, &mut editor, vec![Event::PointerMoved(pos2(400.0, 300.0))]);
        run_editor(&context, &mut editor, vec![Event::Paste("c1ccncc1".to_owned())]);
        assert_eq!(editor.document.molecule.atom_count(), 7);
        assert_eq!(editor.document.molecule.neighbors(anchor).len(), 1);
    }

    #[test]
    fn drawing_snaps_the_endpoint_to_a_thirty_degree_step() {
        let start = [0.0, 0.0];
        let endpoint = snapped_endpoint(start, [1.49, 0.1]);
        assert!((endpoint[0] - DRAW_BOND_LENGTH).abs() < 1.0e-9);
        assert!(endpoint[1].abs() < 1.0e-9);
        let diagonal = snapped_endpoint(start, [1.0, 1.1]);
        let angle = diagonal[1].atan2(diagonal[0]).to_degrees();
        assert!(((angle / 30.0).round() * 30.0 - angle).abs() < 1.0e-9);
    }
}
