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
    let scale = (camera.zoom / 40.0).clamp(0.75, 2.2) as f32;
    let font_size = (camera.zoom * 0.34).clamp(9.0, 28.0) as f32;
    let bond_color = visuals.text_color().gamma_multiply(0.8);
    let index = document
        .molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.id, index))
        .collect::<std::collections::HashMap<_, _>>();
    let centers = document
        .molecule
        .atoms
        .iter()
        .map(|atom| camera.world_to_screen(position(atom), viewport))
        .collect::<Vec<_>>();
    let mut labels = vec![None; document.molecule.atoms.len()];
    for (index, atom) in document.molecule.atoms.iter().enumerate() {
        let text = atom_label(&document.molecule, atom.id);
        if text.is_empty() {
            continue;
        }
        let color = element_label_color(atom.element.cpk_color(), visuals.dark_mode);
        let galley = painter.layout_no_wrap(text.clone(), FontId::proportional(font_size), color);
        let rect = Align2::CENTER_CENTER.anchor_size(centers[index], galley.size());
        labels[index] = Some(AtomLabelPaint { text, rect, color });
    }

    let rings = document.molecule.rings();
    let mut ring_centroids = std::collections::BTreeMap::new();
    let mut circle_bonds = std::collections::BTreeSet::new();
    for ring in &rings {
        if let Some(centroid) = ring.centroid(&document.molecule) {
            for bond in &ring.bonds {
                ring_centroids.insert(*bond, camera.world_to_screen(centroid, viewport));
                if ring.aromatic {
                    circle_bonds.insert(*bond);
                }
            }
        }
    }
    for bond in &document.molecule.bonds {
        let (Some(a), Some(b)) = (index.get(&bond.a), index.get(&bond.b)) else {
            continue;
        };
        let start = trim_to_label(centers[*a], centers[*b], labels[*a].as_ref());
        let end = trim_to_label(centers[*b], centers[*a], labels[*b].as_ref());
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
        let width = if highlighted { 3.0 * scale } else { 1.5 * scale };
        let display_order = if circle_bonds.contains(&bond.id) {
            // A ring with an aromatic circle uses the circle as its delocalization mark;
            // drawing Kekulé double bonds as well would show the same information twice.
            BondOrder::Aromatic
        } else {
            bond.order
        };
        draw_bond(
            &painter,
            start,
            end,
            bond,
            display_order,
            BondStyle { color, width, scale },
            ring_centroids.get(&bond.id).copied(),
        );
    }
    for ring in &rings {
        if !ring.aromatic {
            continue;
        }
        let Some(centroid) = ring.centroid(&document.molecule) else {
            continue;
        };
        let center = camera.world_to_screen(centroid, viewport);
        let radius = ring
            .atoms
            .iter()
            .filter_map(|id| document.molecule.atom(*id))
            .map(|atom| camera.world_to_screen(position(atom), viewport).distance(center))
            .fold(f32::INFINITY, f32::min)
            * 0.55;
        if radius > 2.0 {
            let highlighted = ring
                .bonds
                .iter()
                .any(|bond| selection.bonds().contains(bond) || hovered == Some(Hit::Bond(*bond)))
                || ring
                    .atoms
                    .iter()
                    .any(|atom| selection.atoms().contains(atom) || hovered == Some(Hit::Atom(*atom)));
            let color = if highlighted {
                visuals.selection.stroke.color
            } else {
                bond_color
            };
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(if highlighted { 1.6 * scale } else { 1.0 * scale }, color),
            );
        }
    }
    for (index, atom) in document.molecule.atoms.iter().enumerate() {
        let center = centers[index];
        if !center.is_finite() || !viewport.expand(20.0).contains(center) {
            continue;
        }
        let highlighted = selection.atoms().contains(&atom.id) || hovered == Some(Hit::Atom(atom.id));
        if let Some(label) = &labels[index] {
            if highlighted {
                painter.rect_stroke(
                    label.rect.expand(2.0 * scale),
                    2.0,
                    Stroke::new(1.5 * scale, visuals.selection.stroke.color),
                    egui::StrokeKind::Outside,
                );
            }
            painter.text(
                center,
                Align2::CENTER_CENTER,
                &label.text,
                FontId::proportional(font_size),
                label.color,
            );
        } else if highlighted {
            painter.circle_stroke(
                center,
                5.0 * scale,
                Stroke::new(2.0 * scale, visuals.selection.stroke.color),
            );
        }
    }
    crate::annotations::paint(ui, viewport, document, camera);
}

/// The point where a bond leaves an atom label, so the line does not run under a
/// heteroatom symbol.
fn trim_to_label(center: Pos2, other: Pos2, label: Option<&AtomLabelPaint>) -> Pos2 {
    let Some(label) = label else {
        return center;
    };
    let direction = other - center;
    if direction.length_sq() <= 1.0e-6 {
        return center;
    }
    label
        .rect
        .expand(1.0)
        .intersects_ray_from_center(direction.normalized())
}

#[derive(Clone, Debug)]
struct AtomLabelPaint {
    text: String,
    rect: Rect,
    color: Color32,
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
    let minor = Stroke::new(1.0, color.gamma_multiply(0.18));
    let major = Stroke::new(1.0, color.gamma_multiply(0.34));
    let axis = Stroke::new(1.0, color.gamma_multiply(0.55));
    let major_step = step * 5.0;
    let mut x = (min_x / step).floor() * step;
    while x <= max_x && x / step < 500.0 {
        let screen_x = camera.world_to_screen([x, 0.0], viewport).x;
        let distance = (x - (x / major_step).round() * major_step).abs();
        let stroke = if x.abs() < step * 0.01 {
            axis
        } else if distance < step * 0.01 {
            major
        } else {
            minor
        };
        painter.line_segment(
            [pos2(screen_x, viewport.top()), pos2(screen_x, viewport.bottom())],
            stroke,
        );
        x += step;
    }
    let mut y = (min_y / step).floor() * step;
    while y <= max_y && y / step < 500.0 {
        let screen_y = camera.world_to_screen([0.0, y], viewport).y;
        let distance = (y - (y / major_step).round() * major_step).abs();
        let stroke = if y.abs() < step * 0.01 {
            axis
        } else if distance < step * 0.01 {
            major
        } else {
            minor
        };
        painter.line_segment(
            [pos2(viewport.left(), screen_y), pos2(viewport.right(), screen_y)],
            stroke,
        );
        y += step;
    }
}

#[derive(Clone, Copy)]
struct BondStyle {
    color: Color32,
    width: f32,
    scale: f32,
}

fn draw_bond(
    painter: &egui::Painter,
    start: Pos2,
    end: Pos2,
    bond: &Bond,
    order: BondOrder,
    style: BondStyle,
    inward: Option<Pos2>,
) {
    let BondStyle { color, width, scale } = style;
    let delta = end - start;
    let length = delta.length();
    if length <= f32::EPSILON {
        return;
    }
    let direction = delta / length;
    let normal = vec2(-direction.y, direction.x);
    match bond.stereo {
        BondStereo::Wedge => {
            let half = 4.0 * scale;
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
                let half = (1.0 + 3.0 * t) * scale;
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
                let wave = (t * std::f32::consts::TAU * 2.0).sin() * 2.5 * scale;
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
            let cross = normal * 4.0 * scale;
            painter.line_segment([middle - cross, middle + cross], Stroke::new(width, color));
            return;
        }
        BondStereo::Dotted => {
            for index in 1..10 {
                let t = index as f32 / 10.0;
                painter.circle_filled(start + delta * t, width * 0.6, color);
            }
            return;
        }
        BondStereo::None => {}
        _ => {}
    }
    match order {
        BondOrder::Single => {
            painter.line_segment([start, end], Stroke::new(width, color));
        }
        BondOrder::Double => {
            // Two parallel lines with a clear gap. A ring bond keeps the first line on
            // the ring edge and puts the second one inside; a chain bond is symmetric.
            // The old code drew three lines on a chain and put the ring line only
            // 2.5 points from the edge, which made a double bond look like one thick
            // stroke.
            let offset = 4.0 * scale;
            if let Some(inward) = inward {
                painter.line_segment([start, end], Stroke::new(width, color));
                let midpoint = start + delta * 0.5;
                let side = if normal.dot(inward - midpoint) < 0.0 {
                    -normal
                } else {
                    normal
                };
                painter.line_segment([start + side * offset, end + side * offset], Stroke::new(width, color));
            } else {
                painter.line_segment(
                    [start + normal * (offset * 0.75), end + normal * (offset * 0.75)],
                    Stroke::new(width, color),
                );
                painter.line_segment(
                    [start - normal * (offset * 0.75), end - normal * (offset * 0.75)],
                    Stroke::new(width, color),
                );
            }
        }
        BondOrder::Triple => {
            for offset in [-4.0 * scale, 0.0, 4.0 * scale] {
                painter.line_segment(
                    [start + normal * offset, end + normal * offset],
                    Stroke::new(width, color),
                );
            }
        }
        BondOrder::Aromatic => {
            // The aromatic ring circle marks delocalization; a second dashed line would
            // draw the same information twice.
            painter.line_segment([start, end], Stroke::new(width, color));
        }
        BondOrder::Dative => {
            painter.extend(Shape::dashed_line(&[start, end], Stroke::new(width, color), 5.0, 3.0));
        }
        _ => {
            painter.line_segment([start, end], Stroke::new(width, color));
        }
    }
}

fn position(atom: &Atom2D) -> [f64; 2] {
    [atom.position[0], atom.position[1]]
}

/// The CPK colour, lifted toward white on a dark canvas so a blue N or a brown Fe
/// stays readable. Light themes keep the pure CPK hue.
fn element_label_color(cpk: [u8; 3], dark: bool) -> Color32 {
    if dark {
        let lift = |channel: u8| {
            let channel = u16::from(channel);
            u8::try_from(channel + (255 - channel) * 45 / 100).unwrap_or(255)
        };
        Color32::from_rgb(lift(cpk[0]), lift(cpk[1]), lift(cpk[2]))
    } else {
        Color32::from_rgb(cpk[0], cpk[1], cpk[2])
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
