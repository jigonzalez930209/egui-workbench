//! The interactive 3D viewport widget.

use egui::{Align2, Color32, FontId, Key, PointerButton, Rect, Response, Sense, Stroke, Ui, Vec2, vec2};
use egui_workbench_chem_core::{AtomId, ChemSelection, SelectionMode};

use crate::camera::{Camera3D, Projection3D};
use crate::math::Vec3;
use crate::scene::Mol3dScene;

/// Appearance options for the 3D viewport.
#[derive(Clone, Debug)]
pub struct Mol3dStyle {
    /// Background color; `None` uses the egui canvas color.
    pub background: Option<Color32>,
    /// Bond line width in points.
    pub bond_width: f32,
    /// Multiplier for atom display radius.
    pub atom_scale: f64,
    /// Dim distant atoms.
    pub depth_cue: bool,
    /// Draw the crystal cell edges.
    pub show_cell: bool,
}

impl Default for Mol3dStyle {
    fn default() -> Self {
        Self {
            background: None,
            bond_width: 2.0,
            atom_scale: 1.0,
            depth_cue: true,
            show_cell: true,
        }
    }
}

/// What the 3D viewport did this frame.
pub struct Mol3dResponse {
    /// The viewport response.
    pub response: Response,
    /// The atom under the pointer, if any.
    pub hovered: Option<AtomId>,
    /// The selection changed.
    pub selection_changed: bool,
}

/// An interactive 3D molecular viewport.
pub struct Mol3dView {
    /// The scene.
    pub scene: Mol3dScene,
    /// The orbit camera.
    pub camera: Camera3D,
    /// The shared selection.
    pub selection: ChemSelection,
    /// Appearance.
    pub style: Mol3dStyle,
}

impl Mol3dView {
    /// A viewport fitted to `scene`.
    pub fn new(scene: Mol3dScene) -> Self {
        let mut view = Self {
            scene,
            camera: Camera3D::default(),
            selection: ChemSelection::new(),
            style: Mol3dStyle::default(),
        };
        view.fit(Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(800.0, 600.0)));
        view
    }

    /// Frame the whole scene.
    pub fn fit(&mut self, viewport: Rect) {
        if let Some(bounds) = self.scene.bounds() {
            self.camera.fit(bounds, viewport);
        }
    }

    /// Switch the projection while preserving the target.
    pub fn set_projection(&mut self, projection: Projection3D) {
        self.camera.set_projection(projection);
    }

    /// Draw and interact with the viewport.
    pub fn show(&mut self, ui: &mut Ui) -> Mol3dResponse {
        let available = ui.available_size();
        let (viewport, response) = ui.allocate_exact_size(
            vec2(available.x.max(240.0), available.y.max(200.0)),
            Sense::click_and_drag(),
        );
        let pointer = response.hover_pos();
        let hovered = pointer.and_then(|pointer| self.scene.pick(self.camera, viewport, pointer));

        if response.dragged() {
            let delta = response.drag_delta();
            let pan = response.dragged_by(PointerButton::Middle) || ui.input(|input| input.modifiers.shift);
            if pan {
                self.camera.pan(delta, viewport);
            } else {
                self.camera.orbit(delta);
            }
        }
        if response.hovered() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.camera.zoom((f64::from(scroll) * 0.002).exp());
                ui.ctx().input_mut(|input| input.smooth_scroll_delta = Vec2::ZERO);
            }
        }
        if response.double_clicked() {
            self.fit(viewport);
        }

        let mut selection_changed = false;
        if response.clicked()
            && let Some(pointer) = pointer
            && let Some(atom) = self.scene.pick(self.camera, viewport, pointer)
        {
            let before = self.selection.clone();
            let (shift, command) = ui.input(|input| (input.modifiers.shift, input.modifiers.command));
            let mode = if command {
                SelectionMode::Toggle
            } else if shift {
                SelectionMode::Add
            } else {
                SelectionMode::Replace
            };
            self.selection.select_atom(atom, mode);
            selection_changed = self.selection != before;
        }
        if ui.input(|input| input.key_pressed(Key::Escape)) {
            self.selection.clear();
            selection_changed = true;
        }

        self.paint(ui, viewport, hovered);
        Mol3dResponse {
            response,
            hovered,
            selection_changed,
        }
    }

    fn paint(&self, ui: &Ui, viewport: Rect, hovered: Option<AtomId>) {
        let painter = ui.painter_at(viewport);
        let visuals = ui.visuals();
        painter.rect_filled(viewport, 0.0, self.style.background.unwrap_or(visuals.extreme_bg_color));
        if self.style.show_cell {
            for [a, b] in self.scene.cell_edges() {
                if let (Some((a, _)), Some((b, _))) =
                    (self.camera.project(a, viewport), self.camera.project(b, viewport))
                {
                    painter.line_segment([a, b], Stroke::new(1.0, visuals.weak_text_color()));
                }
            }
        }

        let bond_color = visuals.text_color().gamma_multiply(0.7);
        for bond in &self.scene.bonds {
            let (Some(a), Some(b)) = (self.scene.atom(bond.a), self.scene.atom(bond.b)) else {
                continue;
            };
            let (Some((start, _)), Some((end, _))) = (
                self.camera.project(a.position, viewport),
                self.camera.project(b.position, viewport),
            ) else {
                continue;
            };
            let selected = self.selection.bonds().contains(&bond.id);
            let color = if selected {
                visuals.selection.stroke.color
            } else {
                bond_color
            };
            let width = self.style.bond_width * f32::from(bond.order.strokes().max(1));
            painter.line_segment([start, end], Stroke::new(width, color));
        }

        let mut atoms = self
            .scene
            .atoms
            .iter()
            .filter_map(|atom| {
                self.camera
                    .project(atom.position, viewport)
                    .map(|(screen, depth)| (atom, screen, depth))
            })
            .collect::<Vec<_>>();
        atoms.sort_by(|a, b| b.2.total_cmp(&a.2));
        let min_depth = atoms.iter().map(|(_, _, depth)| *depth).fold(f64::INFINITY, f64::min);
        let max_depth = atoms
            .iter()
            .map(|(_, _, depth)| *depth)
            .fold(f64::NEG_INFINITY, f64::max);
        let span = (max_depth - min_depth).max(1.0e-6);
        for (atom, screen, depth) in atoms {
            let radius = self
                .camera
                .project_radius(atom.position, atom.radius * self.style.atom_scale, viewport);
            if radius <= 0.2 {
                continue;
            }
            let [r, g, b] = atom.element.cpk_color();
            let mut color = Color32::from_rgb(r, g, b);
            if self.style.depth_cue {
                let normalized = ((depth - min_depth) / span) as f32;
                color = color.gamma_multiply(1.0 - 0.3 * normalized);
            }
            let selected = self.selection.atoms().contains(&atom.id);
            let stroke = if selected {
                Stroke::new(2.5, visuals.selection.stroke.color)
            } else if hovered == Some(atom.id) {
                Stroke::new(2.0, visuals.strong_text_color())
            } else {
                Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color)
            };
            painter.circle(screen, radius, color, stroke);
            if selected || hovered == Some(atom.id) {
                painter.text(
                    screen,
                    Align2::CENTER_CENTER,
                    atom.element.symbol(),
                    FontId::proportional((radius * 0.9).clamp(8.0, 16.0)),
                    Color32::WHITE,
                );
            }
        }
    }

    /// The world-space position of `id`.
    pub fn atom_position(&self, id: AtomId) -> Option<Vec3> {
        self.scene.atom(id).map(|atom| atom.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Context;
    use egui_workbench_chem_core::{BondOrder, DocumentId, Element};

    fn scene() -> Mol3dScene {
        let mut document = egui_workbench_chem_core::ChemDocument::new(DocumentId::new(1), "water");
        let oxygen = document
            .molecule
            .add_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
        let hydrogen = document
            .molecule
            .add_atom(Element::from_symbol("H").expect("H"), [0.96, 0.0, 0.0]);
        document
            .molecule
            .add_bond(oxygen, hydrogen, BondOrder::Single)
            .expect("bond");
        Mol3dScene::from_molecule(&document.molecule)
    }

    #[test]
    fn the_viewport_paints_without_a_panic() {
        let mut view = Mol3dView::new(scene());
        let context = Context::default();
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            view.show(ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn a_click_selects_the_picked_atom() {
        let view = Mol3dView::new(scene());
        let viewport = Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(800.0, 600.0));
        let (screen, _) = view
            .camera
            .project(view.scene.atoms[0].position, viewport)
            .expect("project");
        let atom = view.scene.pick(view.camera, viewport, screen);
        assert_eq!(atom, Some(view.scene.atoms[0].id));
    }
}
