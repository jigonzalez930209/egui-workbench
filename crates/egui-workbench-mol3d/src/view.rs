//! The interactive 3D viewport widget.

use egui::{Color32, Key, PointerButton, Rect, Response, Sense, Ui, Vec2, vec2};
use egui_workbench_chem_core::{AtomId, ChemSelection, SelectionMode};

use crate::biomol::{Interaction, RibbonMesh};
use crate::camera::{Camera3D, Projection3D};
use crate::color::ColorScheme;
use crate::crystal::Polyhedron;
use crate::math::Vec3;
use crate::measure::Measurement;
use crate::representation::RepresentationSet;
use crate::scene::Mol3dScene;
use crate::spatial::{CULLING_THRESHOLD, SpatialGrid};
use crate::volume::SurfaceMesh;

mod paint;

/// Which renderer draws the atoms.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Renderer3D {
    /// The CPU painter: the reference and the fallback.
    #[default]
    Cpu,
    /// GPU sphere impostors through `egui_wgpu`; requires the `gpu` feature.
    Gpu,
}

/// Appearance options for the 3D viewport.
#[derive(Clone, Debug, PartialEq)]
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
    /// Draw the crystallographic `a`, `b`, and `c` axes.
    pub show_axes: bool,
    /// Polyhedron fill opacity in `[0, 1]`.
    pub polyhedron_opacity: f32,
    /// Atom renderer.
    pub renderer: Renderer3D,
}

impl Default for Mol3dStyle {
    fn default() -> Self {
        Self {
            background: None,
            bond_width: 2.0,
            atom_scale: 1.0,
            depth_cue: true,
            show_cell: true,
            show_axes: true,
            polyhedron_opacity: 0.22,
            renderer: Renderer3D::Cpu,
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
    /// Default representation and per-selection overrides.
    pub representations: RepresentationSet,
    /// Atom color scheme.
    pub color_scheme: ColorScheme,
    /// Measurements drawn over the scene.
    pub measurements: Vec<Measurement>,
    /// Coordination polyhedra drawn over the scene.
    pub polyhedra: Vec<Polyhedron>,
    /// Miller planes `(h, k, l)` drawn through the cell.
    pub miller_planes: Vec<(i32, i32, i32)>,
    /// Accessible probe points highlighting pores.
    pub pore_points: Vec<Vec3>,
    /// Cartoon ribbons for protein backbones.
    pub ribbons: Vec<RibbonMesh>,
    /// Non-covalent interactions drawn as dashed lines.
    pub interactions: Vec<Interaction>,
    /// Isosurfaces and other translucent triangle meshes.
    pub surfaces: Vec<SurfaceLayer>,
    /// Spatial index for frustum culling on large scenes.
    spatial: Option<SpatialGrid>,
    spatial_atoms: usize,
}

/// A translucent triangle surface with its own color.
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceLayer {
    /// Geometry.
    pub mesh: SurfaceMesh,
    /// RGBA fill color.
    pub color: [u8; 4],
}

impl Mol3dView {
    /// A viewport fitted to `scene`.
    pub fn new(scene: Mol3dScene) -> Self {
        let mut view = Self {
            scene,
            camera: Camera3D::default(),
            selection: ChemSelection::new(),
            style: Mol3dStyle::default(),
            representations: RepresentationSet::default(),
            color_scheme: ColorScheme::default(),
            measurements: Vec::new(),
            polyhedra: Vec::new(),
            miller_planes: Vec::new(),
            pore_points: Vec::new(),
            ribbons: Vec::new(),
            interactions: Vec::new(),
            surfaces: Vec::new(),
            spatial: None,
            spatial_atoms: 0,
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
            if let Some(atom) = self.scene.pick(self.camera, viewport, pointer) {
                self.selection.select_atom(atom, mode);
            } else if let Some(bond) = self.scene.pick_bond(self.camera, viewport, pointer, 7.0) {
                self.selection.select_bond(bond, mode);
            }
            selection_changed = self.selection != before;
        }
        if ui.input(|input| input.key_pressed(Key::Escape)) {
            self.selection.clear();
            selection_changed = true;
        }

        if self.scene.atoms.len() >= CULLING_THRESHOLD
            && (self.spatial.is_none() || self.spatial_atoms != self.scene.atoms.len())
        {
            let extent = self.scene.bounds().map_or(8.0, |bounds| {
                Vec3::new(bounds[3] - bounds[0], bounds[4] - bounds[1], bounds[5] - bounds[2]).length()
            });
            self.spatial = Some(SpatialGrid::build(&self.scene, (extent / 24.0).max(0.5)));
            self.spatial_atoms = self.scene.atoms.len();
        }
        let visible = self
            .spatial
            .as_ref()
            .map(|grid| grid.visible(&self.scene, self.camera, viewport));
        self.paint(ui, viewport, hovered, visible.as_deref());
        Mol3dResponse {
            response,
            hovered,
            selection_changed,
        }
    }

    /// Measure the distance between two atoms and draw it.
    pub fn measure_distance(&mut self, a: AtomId, b: AtomId) -> Option<f64> {
        let measurement = Measurement::Distance { a, b };
        let value = measurement.evaluate(&self.scene)?;
        self.measurements.push(measurement);
        Some(value.value)
    }

    /// Measure the bond angle at `b` and draw it.
    pub fn measure_angle(&mut self, a: AtomId, b: AtomId, c: AtomId) -> Option<f64> {
        let measurement = Measurement::Angle { a, b, c };
        let value = measurement.evaluate(&self.scene)?;
        self.measurements.push(measurement);
        Some(value.value)
    }

    /// Measure the dihedral `a-b-c-d` and draw it.
    pub fn measure_dihedral(&mut self, a: AtomId, b: AtomId, c: AtomId, d: AtomId) -> Option<f64> {
        let measurement = Measurement::Dihedral { a, b, c, d };
        let value = measurement.evaluate(&self.scene)?;
        self.measurements.push(measurement);
        Some(value.value)
    }

    /// The mean plane of the selected atoms, or `None` when they are missing.
    pub fn add_plane(&mut self, atoms: Vec<AtomId>) -> Option<()> {
        let value = Measurement::Plane { atoms: atoms.clone() }.evaluate(&self.scene)?;
        self.measurements.push(Measurement::Plane { atoms });
        let _ = value;
        Some(())
    }

    /// Expand the current scene into a supercell, replacing it.
    pub fn expand_supercell(&mut self, na: usize, nb: usize, nc: usize) {
        self.scene = self.scene.supercell(na, nb, nc);
        self.polyhedra.clear();
        self.measurements.clear();
    }

    /// Detect coordination polyhedra around the given metal symbols.
    pub fn toggle_polyhedra(&mut self, metals: &[&str], cutoff: f64) {
        self.polyhedra = self.scene.coordination_polyhedra(metals, cutoff);
    }

    /// Add a Miller plane, or remove it when it is already shown.
    pub fn toggle_miller_plane(&mut self, h: i32, k: i32, l: i32) {
        if let Some(index) = self.miller_planes.iter().position(|plane| *plane == (h, k, l)) {
            self.miller_planes.remove(index);
        } else {
            self.miller_planes.push((h, k, l));
        }
    }

    /// Sample accessible pore points with a probe radius and grid spacing.
    pub fn highlight_pores(&mut self, probe_radius: f64, spacing: f64) {
        self.pore_points = self.scene.pore_points(probe_radius, spacing);
    }

    /// Build cartoon ribbons from the current CA traces.
    pub fn show_cartoon(&mut self) {
        self.ribbons = crate::biomol::ribbon_meshes(&self.scene);
    }

    /// Detect and draw non-covalent interactions.
    pub fn show_interactions(&mut self) {
        self.interactions = crate::biomol::detect_interactions(&self.scene);
    }

    /// Add a translucent surface layer.
    pub fn add_surface(&mut self, mesh: SurfaceMesh, color: [u8; 4]) {
        self.surfaces.push(SurfaceLayer { mesh, color });
    }

    /// Sample and display a hydrogen-like orbital isosurface.
    pub fn show_orbital(&mut self, kind: crate::volume::OrbitalKind, center: Vec3, isovalue: f64, resolution: usize) {
        let grid = crate::volume::orbital_grid(kind, center, 7.0, resolution);
        let positive = crate::volume::isosurface(&grid, isovalue);
        let negative = crate::volume::isosurface(&grid, -isovalue);
        if !positive.triangles.is_empty() {
            self.add_surface(positive, [80, 140, 255, 70]);
        }
        if !negative.triangles.is_empty() {
            self.add_surface(negative, [255, 90, 80, 70]);
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
