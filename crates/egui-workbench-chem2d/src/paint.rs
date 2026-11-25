//! The `egui::Painter` renderer for a 2D document.

use egui::epaint::{PathShape, PathStroke};
use egui::{Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, Ui, Vec2, pos2, vec2};
use egui_workbench_chem_core::{Bond, BondOrder, BondStereo, ChemSelection, Element};

use crate::camera::Camera2D;
use crate::document::{Atom2D, ChemDocument};
use crate::interact::{Hit, atom_label};

/// Draw the grid, bonds, atoms, and labels.
pub fn paint(
    ui: &Ui,
    viewport: Rect,
    document: &ChemDocument,
    camera: Camera2D,
    selection: &ChemSelection,
    grid: bool,
    hovered: Option<Hit>,
) {
    let painter = ui.painter_at(viewport);
    let visuals = ui.visuals();
    if grid {
        paint_grid(&painter, viewport, camera, visuals.weak_text_color());
    }
    let bond_color = visuals.text_color().gamma_multiply(0.8);
    for bond in &document.molecule.bonds {
        let (Some(a), Some(b)) = (document.molecule.atom(bond.a), document.molecule.atom(bond.b)) else {
            continue;
        };
        let start = camera.world_to_screen(position(a), viewport);
        let end = camera.world_to_screen(position(b), viewport);
        if !start.is_finite() || !end.is_finite() {
            continue;
        }
        let selected = selection.bonds().contains(&bond.id);
        let highlighted = selected || hovered == Some(Hit::Bond(bond.id));
        let color = if highlighted {
            visuals.selection.stroke.color
        } else {
            bond_color
        };
        let width = if highlighted { 3.0 } else { 1.5 };
        draw_bond(&painter, start, end, bond, color, width);
    }
    for atom in &document.molecule.atoms {
        let center = camera.world_to_screen(position(atom), viewport);
        if !center.is_finite() || !viewport.expand(20.0).contains(center) {
            continue;
        }
        let selected = selection.atoms().contains(&atom.id);
        let radius = atom_radius(atom);
        let fill = Color32::from_rgb(
            atom.element.cpk_color()[0],
            atom.element.cpk_color()[1],
            atom.element.cpk_color()[2],
        );
        let stroke = if selected || hovered == Some(Hit::Atom(atom.id)) {
            Stroke::new(2.0, visuals.selection.stroke.color)
        } else {
            Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color)
        };
        painter.circle(center, radius, fill, stroke);
        let label = atom_label(&document.molecule, atom.id);
        if !label.is_empty() {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
                label_color(fill),
            );
        }
    }
}

fn paint_grid(painter: &egui::Painter, viewport: Rect, camera: Camera2D, color: Color32) {
    let step = camera.grid_step();
    if !step.is_finite() || step <= 0.0 {
        return;
    }
    let top_left = camera.screen_to_world(viewport.left_top(), viewport);
    let bottom_right = camera.screen_to_world(viewport.right_bottom(), viewport);
    let (min_x, max_x) = (top_left[0].min(bottom_right[0]), top_left[0].max(bottom_right[0]));
    let (min_y, max_y) = (top_left[1].min(bottom_right[1]), top_left[1].max(bottom_right[1]));
    let stroke = Stroke::new(1.0, color.gamma_multiply(0.25));
    let mut x = (min_x / step).floor() * step;
    while x <= max_x && x / step < 500.0 {
        let screen_x = camera.world_to_screen([x, 0.0], viewport).x;
        painter.line_segment(
            [pos2(screen_x, viewport.top()), pos2(screen_x, viewport.bottom())],
            stroke,
        );
        x += step;
    }
    let mut y = (min_y / step).floor() * step;
    while y <= max_y && y / step < 500.0 {
        let screen_y = camera.world_to_screen([0.0, y], viewport).y;
        painter.line_segment(
            [pos2(viewport.left(), screen_y), pos2(viewport.right(), screen_y)],
            stroke,
        );
        y += step;
    }
}

fn draw_bond(painter: &egui::Painter, start: Pos2, end: Pos2, bond: &Bond, color: Color32, width: f32) {
    let delta = end - start;
    let length = delta.length();
    if length <= f32::EPSILON {
        return;
    }
    let direction = delta / length;
    let normal = vec2(-direction.y, direction.x);
    match bond.stereo {
        BondStereo::Wedge => {
            let half = 4.0;
            painter.add(Shape::convex_polygon(
                vec![start, end + normal * half, end - normal * half],
                color,
                Stroke::NONE,
            ));
            return;
        }
        BondStereo::Hash => {
            for index in 1..=6 {
                let t = index as f32 / 7.0;
                let center = start + delta * t;
                let half = 1.0 + 3.0 * t;
                painter.line_segment(
                    [center - normal * half, center + normal * half],
                    Stroke::new(width, color),
                );
            }
            return;
        }
        BondStereo::Wavy => {
            let mut points = Vec::with_capacity(12);
            for index in 0..=10 {
                let t = index as f32 / 10.0;
                let wave = (t * std::f32::consts::TAU * 2.0).sin() * 2.5;
                points.push(start + delta * t + normal * wave);
            }
            painter.add(Shape::Path(PathShape::line(
                points,
                PathStroke::from(Stroke::new(width, color)),
            )));
            return;
        }
        BondStereo::Either => {
            painter.line_segment([start, end], Stroke::new(width, color));
            let middle = start + delta * 0.5;
            let cross = normal * 4.0;
            painter.line_segment([middle - cross, middle + cross], Stroke::new(width, color));
            return;
        }
        BondStereo::None => {}
        _ => {}
    }
    match bond.order {
        BondOrder::Single => {
            painter.line_segment([start, end], Stroke::new(width, color));
        }
        BondOrder::Double => {
            let offset = normal * 2.5;
            painter.line_segment([start + offset, end + offset], Stroke::new(width, color));
            painter.line_segment([start - offset, end - offset], Stroke::new(width, color));
        }
        BondOrder::Triple => {
            for offset in [-4.0, 0.0, 4.0] {
                painter.line_segment(
                    [start + normal * offset, end + normal * offset],
                    Stroke::new(width, color),
                );
            }
        }
        BondOrder::Aromatic => {
            painter.line_segment([start, end], Stroke::new(width, color));
            let offset = normal * 3.0;
            painter.extend(Shape::dashed_line(
                &[start + offset, end + offset],
                Stroke::new(width * 0.8, color),
                5.0,
                3.0,
            ));
        }
        BondOrder::Dative => {
            painter.extend(Shape::dashed_line(&[start, end], Stroke::new(width, color), 5.0, 3.0));
        }
        _ => {
            painter.line_segment([start, end], Stroke::new(width, color));
        }
    }
}

fn atom_radius(atom: &Atom2D) -> f32 {
    (6.0 + atom.element.covalent_radius() as f32 * 5.0).clamp(6.0, 16.0)
}

fn position(atom: &Atom2D) -> [f64; 2] {
    [atom.position[0], atom.position[1]]
}

fn label_color(background: Color32) -> Color32 {
    let luminance =
        0.299 * f32::from(background.r()) + 0.587 * f32::from(background.g()) + 0.114 * f32::from(background.b());
    if luminance > 160.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

/// The bond midpoint in screen points, for labels and tests.
pub fn bond_midpoint(start: Pos2, end: Pos2) -> Pos2 {
    pos2((start.x + end.x) * 0.5, (start.y + end.y) * 0.5)
}

/// The screen extent of a bond, used to place wedges.
pub fn bond_extent(start: Pos2, end: Pos2) -> Vec2 {
    end - start
}

/// The CPK color for an element.
pub fn element_color(element: Element) -> Color32 {
    let [r, g, b] = element.cpk_color();
    Color32::from_rgb(r, g, b)
}

/// Keep the `Pos2` import used by downstream helpers.
pub type ScreenPoint = Pos2;
