//! Painting for the 3D viewport.

use egui::{Align2, Color32, FontId, Rect, Stroke, Ui, vec2};
use egui_workbench_chem_core::AtomId;

use super::Mol3dView;
#[cfg(feature = "gpu")]
use super::Renderer3D;
use crate::biomol::{InteractionKind, SecondaryStructure};
use crate::crystal::{CellAxes, miller_plane};
use crate::math::Vec3;

impl Mol3dView {
    pub(super) fn paint(&self, ui: &Ui, viewport: Rect, hovered: Option<AtomId>, visible: Option<&[usize]>) {
        let painter = ui.painter_at(viewport);
        let visuals = ui.visuals();
        painter.rect_filled(viewport, 0.0, self.style.background.unwrap_or(visuals.extreme_bg_color));
        #[cfg(feature = "gpu")]
        let mut gpu_atoms = false;
        #[cfg(not(feature = "gpu"))]
        let gpu_atoms = false;
        #[cfg(feature = "gpu")]
        if self.style.renderer == Renderer3D::Gpu {
            gpu_atoms = crate::gpu::paint_atoms(
                ui,
                viewport,
                self.camera,
                &self.scene,
                self.style.atom_scale,
                &self.representations,
                self.color_scheme,
            );
        }
        if self.style.show_cell {
            for [a, b] in self.scene.cell_edges() {
                if let (Some((a, _)), Some((b, _))) =
                    (self.camera.project(a, viewport), self.camera.project(b, viewport))
                {
                    painter.line_segment([a, b], Stroke::new(1.0, visuals.weak_text_color()));
                }
            }
        }
        if self.style.show_axes
            && let Some(cell) = self.scene.cell
        {
            let axes = CellAxes::new(cell);
            for (vector, label) in [(axes.a, "a"), (axes.b, "b"), (axes.c, "c")] {
                let Some((start, _)) = self.camera.project(Vec3::ZERO, viewport) else {
                    continue;
                };
                let Some((end, _)) = self.camera.project(vector, viewport) else {
                    continue;
                };
                let color = visuals.strong_text_color();
                painter.arrow(start, end - start, Stroke::new(1.5, color));
                painter.text(
                    end + vec2(4.0, -4.0),
                    Align2::LEFT_BOTTOM,
                    label,
                    FontId::proportional(12.0),
                    color,
                );
            }
        }
        self.paint_polyhedra(viewport, &painter);
        self.paint_ribbons(viewport, &painter);
        self.paint_interactions(viewport, &painter, visuals);
        self.paint_surfaces(viewport, &painter);
        self.paint_miller_planes(viewport, &painter, visuals);
        for point in &self.pore_points {
            if let Some((screen, _)) = self.camera.project(*point, viewport) {
                painter.circle_filled(screen, 1.6, Color32::from_rgba_unmultiplied(80, 200, 255, 120));
            }
        }

        let pixels_per_angstrom = self.camera.project_radius(Vec3::ZERO, 1.0, viewport).max(0.01);
        for bond in &self.scene.bonds {
            let (Some(a), Some(b)) = (self.scene.atom(bond.a), self.scene.atom(bond.b)) else {
                continue;
            };
            let representation = self.representations.for_atom(a.id);
            if !representation.shows_bonds() {
                continue;
            }
            let (Some((start, _)), Some((end, _))) = (
                self.camera.project(a.position, viewport),
                self.camera.project(b.position, viewport),
            ) else {
                continue;
            };
            let selected = self.selection.bonds().contains(&bond.id);
            let width = (representation.bond_radius() as f32 * pixels_per_angstrom).clamp(0.5, 32.0);
            let [ar, ag, ab] = self.color_scheme.color(a);
            let [br, bg, bb] = self.color_scheme.color(b);
            let color_a = Color32::from_rgb(ar, ag, ab);
            let color_b = Color32::from_rgb(br, bg, bb);
            if selected {
                painter.line_segment(
                    [start, end],
                    Stroke::new(width.max(2.0), visuals.selection.stroke.color),
                );
                continue;
            }
            let direction = end - start;
            let length = direction.length().max(0.001);
            let perpendicular = vec2(-direction.y, direction.x) / length;
            let strokes = f32::from(bond.order.strokes().max(1));
            let spacing = width * 1.1;
            for stroke in 0..bond.order.strokes().max(1) {
                let offset = (f32::from(stroke) - (strokes - 1.0) * 0.5) * spacing;
                let shift = perpendicular * offset;
                let (from, to) = (start + shift, end + shift);
                match bond.order {
                    egui_workbench_chem_core::BondOrder::Dative => {
                        let segments = 8;
                        for segment in 0..segments {
                            if segment % 2 == 1 {
                                continue;
                            }
                            let t0 = segment as f32 / segments as f32;
                            let t1 = (segment + 1) as f32 / segments as f32;
                            painter.line_segment(
                                [from.lerp(to, t0), from.lerp(to, t1)],
                                Stroke::new(width * 0.6, color_a),
                            );
                        }
                    }
                    _ => {
                        let middle = from.lerp(to, 0.5);
                        painter.line_segment([from, middle], Stroke::new(width, color_a));
                        painter.line_segment([middle, to], Stroke::new(width, color_b));
                    }
                }
            }
        }

        if !gpu_atoms {
            self.paint_atoms(viewport, hovered, visuals, &painter, visible);
        }
        self.paint_measurements(viewport, &painter, visuals);
    }

    fn paint_atoms(
        &self,
        viewport: Rect,
        hovered: Option<AtomId>,
        visuals: &egui::Visuals,
        painter: &egui::Painter,
        visible: Option<&[usize]>,
    ) {
        let allowed = visible.map(|indices| {
            indices
                .iter()
                .filter_map(|index| self.scene.atoms.get(*index).map(|atom| atom.id))
                .collect::<std::collections::BTreeSet<_>>()
        });
        let mut atoms = self
            .scene
            .atoms
            .iter()
            .filter(|atom| allowed.as_ref().is_none_or(|allowed| allowed.contains(&atom.id)))
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
            let representation = self.representations.for_atom(atom.id);
            if !representation.shows_atoms() {
                continue;
            }
            let radius = self.camera.project_radius(
                atom.position,
                representation.atom_radius(atom) * self.style.atom_scale,
                viewport,
            );
            if radius <= 0.2 {
                continue;
            }
            if allowed.is_some() && radius <= crate::spatial::LOD_POINT_RADIUS as f32 {
                let [r, g, b] = self.color_scheme.color(atom);
                painter.circle_filled(screen, radius.max(0.8), Color32::from_rgb(r, g, b));
                continue;
            }
            let [r, g, b] = self.color_scheme.color(atom);
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

    fn paint_measurements(&self, viewport: Rect, painter: &egui::Painter, visuals: &egui::Visuals) {
        let color = visuals.warn_fg_color;
        for measurement in &self.measurements {
            let Some(value) = measurement.evaluate(&self.scene) else {
                continue;
            };
            let points = value
                .points
                .iter()
                .filter_map(|point| self.camera.project(*point, viewport).map(|(screen, _)| screen))
                .collect::<Vec<_>>();
            if points.len() >= 2 {
                painter.add(egui::Shape::line(points.clone(), Stroke::new(1.5, color)));
            }
            for point in &points {
                painter.circle_filled(*point, 3.0, color);
            }
            if let Some(anchor) = value.anchor
                && let Some((screen, _)) = self.camera.project(anchor, viewport)
            {
                painter.text(
                    screen - vec2(0.0, 10.0),
                    Align2::CENTER_BOTTOM,
                    &value.label,
                    FontId::proportional(12.0),
                    color,
                );
            }
            if let Some(normal) = value.normal
                && let Some(anchor) = value.anchor
            {
                let tip = anchor + normal * 1.2;
                if let (Some((start, _)), Some((end, _))) = (
                    self.camera.project(anchor, viewport),
                    self.camera.project(tip, viewport),
                ) {
                    painter.arrow(start, end - start, Stroke::new(1.0, color));
                }
            }
        }
    }

    fn paint_ribbons(&self, viewport: Rect, painter: &egui::Painter) {
        for ribbon in &self.ribbons {
            for triangle in &ribbon.triangles {
                let points = triangle
                    .iter()
                    .filter_map(|index| {
                        self.camera
                            .project(ribbon.vertices[*index], viewport)
                            .map(|(screen, _)| screen)
                    })
                    .collect::<Vec<_>>();
                if points.len() != 3 {
                    continue;
                }
                let kind = ribbon.secondary[triangle[0]];
                let color = match kind {
                    SecondaryStructure::Helix => Color32::from_rgb(230, 90, 90),
                    SecondaryStructure::Sheet => Color32::from_rgb(230, 190, 70),
                    SecondaryStructure::Loop => Color32::from_rgb(160, 170, 180),
                };
                painter.add(egui::Shape::convex_polygon(
                    points,
                    color.gamma_multiply(0.85),
                    Stroke::NONE,
                ));
            }
        }
    }

    fn paint_interactions(&self, viewport: Rect, painter: &egui::Painter, visuals: &egui::Visuals) {
        for interaction in &self.interactions {
            let (Some(a), Some(b)) = (self.scene.atom(interaction.a), self.scene.atom(interaction.b)) else {
                continue;
            };
            let (Some((start, _)), Some((end, _))) = (
                self.camera.project(a.position, viewport),
                self.camera.project(b.position, viewport),
            ) else {
                continue;
            };
            let color = match interaction.kind {
                InteractionKind::HydrogenBond => Color32::from_rgb(80, 170, 255),
                InteractionKind::PiStacking => Color32::from_rgb(200, 120, 255),
                InteractionKind::SaltBridge => Color32::from_rgb(255, 140, 60),
                InteractionKind::ShortContact => visuals.warn_fg_color,
            };
            let direction = end - start;
            let length = direction.length().max(0.001);
            let segments = (length / 6.0).max(2.0) as usize;
            for segment in (0..segments).step_by(2) {
                let t0 = segment as f32 / segments as f32;
                let t1 = (segment + 1) as f32 / segments as f32;
                painter.line_segment(
                    [start + direction * t0, start + direction * t1],
                    Stroke::new(1.5, color),
                );
            }
        }
    }

    fn paint_surfaces(&self, viewport: Rect, painter: &egui::Painter) {
        for layer in &self.surfaces {
            let fill = Color32::from_rgba_unmultiplied(layer.color[0], layer.color[1], layer.color[2], layer.color[3]);
            for triangle in &layer.mesh.triangles {
                let points = triangle
                    .iter()
                    .filter_map(|index| {
                        let vertex = *layer.mesh.vertices.get(*index as usize)?;
                        self.camera.project(vertex, viewport).map(|(screen, _)| screen)
                    })
                    .collect::<Vec<_>>();
                if points.len() == 3 {
                    painter.add(egui::Shape::convex_polygon(points, fill, Stroke::NONE));
                }
            }
        }
    }

    fn paint_polyhedra(&self, viewport: Rect, painter: &egui::Painter) {
        for polyhedron in &self.polyhedra {
            let color = Color32::from_rgba_unmultiplied(
                110,
                190,
                255,
                (self.style.polyhedron_opacity.clamp(0.0, 1.0) * 255.0) as u8,
            );
            for face in &polyhedron.faces {
                let points = face
                    .iter()
                    .filter_map(|index| {
                        self.camera
                            .project(polyhedron.vertex_positions[*index], viewport)
                            .map(|(screen, _)| screen)
                    })
                    .collect::<Vec<_>>();
                if points.len() == 3 {
                    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
                }
            }
            for (a, b) in &polyhedron.edges {
                let (Some((start, _)), Some((end, _))) = (
                    self.camera.project(polyhedron.vertex_positions[*a], viewport),
                    self.camera.project(polyhedron.vertex_positions[*b], viewport),
                ) else {
                    continue;
                };
                painter.line_segment([start, end], Stroke::new(1.5, Color32::from_rgb(80, 150, 230)));
            }
        }
    }

    fn paint_miller_planes(&self, viewport: Rect, painter: &egui::Painter, visuals: &egui::Visuals) {
        let Some(cell) = self.scene.cell else {
            return;
        };
        for (h, k, l) in &self.miller_planes {
            let Some(corners) = miller_plane(cell, *h, *k, *l) else {
                continue;
            };
            let points = corners
                .iter()
                .filter_map(|corner| self.camera.project(*corner, viewport).map(|(p, _)| p))
                .collect::<Vec<_>>();
            if points.len() == 4 {
                painter.add(egui::Shape::convex_polygon(
                    points,
                    Color32::from_rgba_unmultiplied(255, 200, 60, 35),
                    Stroke::new(1.5, visuals.warn_fg_color),
                ));
                if let Some(center) = crate::measure::centroid(corners.as_ref())
                    && let Some((screen, _)) = self.camera.project(center, viewport)
                {
                    painter.text(
                        screen,
                        Align2::CENTER_CENTER,
                        format!("({h}{k}{l})"),
                        FontId::proportional(11.0),
                        visuals.warn_fg_color,
                    );
                }
            }
        }
    }
}
