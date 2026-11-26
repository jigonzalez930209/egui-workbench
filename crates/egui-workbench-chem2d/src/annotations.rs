//! Reaction arrows, condition boxes, polymer brackets, and chemical text.

use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Ui, pos2, vec2};
use serde::{Deserialize, Serialize};

use crate::camera::Camera2D;
use crate::document::ChemDocument;

/// The kind of reaction or mechanism arrow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ArrowKind {
    /// A forward reaction.
    Reaction,
    /// An equilibrium.
    Equilibrium,
    /// A resonance arrow.
    Resonance,
    /// A retrosynthesis arrow.
    Retrosynthesis,
    /// A failed reaction.
    Failed,
    /// A one-electron fishhook.
    FishhookOne,
    /// A two-electron fishhook.
    FishhookTwo,
}

/// A reaction or mechanism arrow in data coordinates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReactionArrow {
    /// Arrow kind.
    pub kind: ArrowKind,
    /// Start point.
    pub start: [f64; 2],
    /// End point.
    pub end: [f64; 2],
    /// Curvature offset for fishhook arrows.
    pub bend: f64,
}

impl ReactionArrow {
    /// A straight arrow.
    pub fn new(kind: ArrowKind, start: [f64; 2], end: [f64; 2]) -> Self {
        Self {
            kind,
            start,
            end,
            bend: 0.0,
        }
    }

    /// A curved fishhook arrow.
    pub fn fishhook(kind: ArrowKind, start: [f64; 2], end: [f64; 2], bend: f64) -> Self {
        Self { kind, start, end, bend }
    }
}

/// A reaction conditions box.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionBox {
    /// Top-left corner in data coordinates.
    pub position: [f64; 2],
    /// Lines; `_2` and `^2` render as sub/superscripts.
    pub lines: Vec<String>,
    /// Width in Angstrom.
    pub width: f64,
}

impl ConditionBox {
    /// A box with one line.
    pub fn line(position: [f64; 2], text: impl Into<String>) -> Self {
        Self {
            position,
            lines: vec![text.into()],
            width: 4.0,
        }
    }
}

/// A polymer repeat bracket.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolymerBracket {
    /// Left end in data coordinates.
    pub start: [f64; 2],
    /// Right end in data coordinates.
    pub end: [f64; 2],
    /// Repeat count label, usually `n`.
    pub subscript: String,
}

/// One annotation drawn over the document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "annotation", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Annotation {
    /// A reaction or mechanism arrow.
    Arrow(ReactionArrow),
    /// A conditions box.
    Conditions(ConditionBox),
    /// A polymer bracket.
    Polymer(PolymerBracket),
}

/// One semantic reaction scheme.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReactionScheme {
    /// Reactant atom ids.
    pub reactants: Vec<egui_workbench_chem_core::AtomId>,
    /// Conditions box.
    pub conditions: ConditionBox,
    /// Product atom ids.
    pub products: Vec<egui_workbench_chem_core::AtomId>,
}

/// Convert `H_2O`, `SO_4^2-`, and `C^+` to Unicode sub/superscripts.
pub fn format_chemical_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        let superscript = character == '^';
        // A subscript takes digits only. Letting it swallow a following `-` or `+` turned
        // the bond in `CH_3-CH_3` into a subscript minus; signs belong to superscript
        // charges (`SO_4^2-`).
        let accepts = |next: char| next.is_ascii_digit() || (superscript && (next == '+' || next == '-'));
        if (character == '_' || superscript) && chars.peek().is_some_and(|next| accepts(*next)) {
            let mut number = String::new();
            while chars.peek().is_some_and(|next| accepts(*next)) {
                number.push(chars.next().unwrap_or_default());
            }
            output.push_str(&map_script(&number, superscript));
        } else {
            output.push(character);
        }
    }
    output
}

fn map_script(number: &str, superscript: bool) -> String {
    number
        .chars()
        .map(|character| match (character, superscript) {
            ('0', false) => '₀',
            ('1', false) => '₁',
            ('2', false) => '₂',
            ('3', false) => '₃',
            ('4', false) => '₄',
            ('5', false) => '₅',
            ('6', false) => '₆',
            ('7', false) => '₇',
            ('8', false) => '₈',
            ('9', false) => '₉',
            ('+', false) => '₊',
            ('-', false) => '₋',
            ('0', true) => '⁰',
            ('1', true) => '¹',
            ('2', true) => '²',
            ('3', true) => '³',
            ('4', true) => '⁴',
            ('5', true) => '⁵',
            ('6', true) => '⁶',
            ('7', true) => '⁷',
            ('8', true) => '⁸',
            ('9', true) => '⁹',
            ('+', true) => '⁺',
            ('-', true) => '⁻',
            _ => character,
        })
        .collect()
}

/// Draw every annotation.
pub(super) fn paint(ui: &Ui, viewport: Rect, document: &ChemDocument, camera: Camera2D) {
    let painter = ui.painter_at(viewport);
    let color = ui.visuals().text_color();
    for annotation in &document.annotations {
        match annotation {
            Annotation::Arrow(arrow) => paint_arrow(&painter, camera, viewport, arrow, color),
            Annotation::Conditions(conditions) => {
                let origin = camera.world_to_screen(conditions.position, viewport);
                let width = (conditions.width * camera.zoom) as f32;
                let height = 16.0 * conditions.lines.len() as f32 + 8.0;
                let rect = Rect::from_min_size(origin, vec2(width, height));
                painter.rect(
                    rect,
                    3.0,
                    ui.visuals().window_fill.gamma_multiply(0.9),
                    Stroke::new(1.0, color.gamma_multiply(0.5)),
                    egui::StrokeKind::Inside,
                );
                for (index, line) in conditions.lines.iter().enumerate() {
                    painter.text(
                        pos2(rect.left() + 4.0, rect.top() + 4.0 + index as f32 * 16.0),
                        Align2::LEFT_TOP,
                        format_chemical_text(line),
                        FontId::proportional(11.0),
                        color,
                    );
                }
            }
            Annotation::Polymer(bracket) => {
                let start = camera.world_to_screen(bracket.start, viewport);
                let end = camera.world_to_screen(bracket.end, viewport);
                let stroke = Stroke::new(1.2, color);
                let tick = 5.0;
                painter.line_segment([pos2(start.x, start.y - 12.0), pos2(start.x, start.y + 12.0)], stroke);
                painter.line_segment(
                    [
                        pos2(start.x - tick, start.y - 12.0),
                        pos2(start.x + tick, start.y - 12.0),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        pos2(start.x - tick, start.y + 12.0),
                        pos2(start.x + tick, start.y + 12.0),
                    ],
                    stroke,
                );
                painter.line_segment([pos2(end.x, end.y - 12.0), pos2(end.x, end.y + 12.0)], stroke);
                painter.line_segment(
                    [pos2(end.x - tick, end.y - 12.0), pos2(end.x + tick, end.y - 12.0)],
                    stroke,
                );
                painter.line_segment(
                    [pos2(end.x - tick, end.y + 12.0), pos2(end.x + tick, end.y + 12.0)],
                    stroke,
                );
                painter.text(
                    pos2(end.x + 6.0, end.y + 12.0),
                    Align2::LEFT_TOP,
                    &bracket.subscript,
                    FontId::proportional(11.0),
                    color,
                );
            }
        }
    }
}

fn paint_arrow(painter: &egui::Painter, camera: Camera2D, viewport: Rect, arrow: &ReactionArrow, color: Color32) {
    let start = camera.world_to_screen(arrow.start, viewport);
    let end = camera.world_to_screen(arrow.end, viewport);
    let delta = end - start;
    let length = delta.length();
    if length <= 1.0 {
        return;
    }
    let direction = delta / length;
    let normal = vec2(-direction.y, direction.x);
    let stroke = Stroke::new(1.5, color);
    let points = if arrow.bend.abs() > f64::EPSILON {
        let control = start + delta * 0.5 + normal * arrow.bend as f32;
        (0..=24)
            .map(|index| {
                let t = index as f32 / 24.0;
                let one = 1.0 - t;
                pos2(
                    start.x * (one * one) + control.x * (2.0 * one * t) + end.x * (t * t),
                    start.y * (one * one) + control.y * (2.0 * one * t) + end.y * (t * t),
                )
            })
            .collect::<Vec<_>>()
    } else {
        vec![start, end]
    };
    match arrow.kind {
        ArrowKind::Equilibrium => {
            let offset = normal * 2.0;
            painter.line_segment([start + offset, end + offset], stroke);
            painter.line_segment([start - offset, end - offset], stroke);
            arrowhead(painter, end + offset, direction, color, 6.0);
            arrowhead(painter, start - offset, -direction, color, 6.0);
        }
        ArrowKind::Resonance => {
            painter.add(egui::Shape::line(points.clone(), stroke));
            painter.add(egui::Shape::line(
                points.iter().map(|point| *point + normal * 3.0).collect(),
                stroke,
            ));
            arrowhead(painter, end, direction, color, 7.0);
        }
        ArrowKind::Retrosynthesis => {
            painter.add(egui::Shape::line(points.clone(), stroke));
            painter.line_segment([end - direction * 7.0 + normal * 4.0, end], Stroke::new(1.0, color));
            painter.line_segment([end - direction * 7.0 - normal * 4.0, end], Stroke::new(1.0, color));
        }
        ArrowKind::Failed => {
            painter.add(egui::Shape::line(points, stroke));
            let middle = start + delta * 0.85;
            painter.line_segment([middle - normal * 5.0, middle + normal * 5.0], Stroke::new(2.0, color));
        }
        ArrowKind::FishhookOne | ArrowKind::FishhookTwo => {
            painter.add(egui::Shape::line(points, stroke));
            arrowhead(painter, end, direction, color, 7.0);
            let hooks = if arrow.kind == ArrowKind::FishhookOne { 1 } else { 2 };
            for index in 0..hooks {
                let point = start + normal * (index as f32 * 4.0);
                painter.circle_stroke(point, 3.0, Stroke::new(1.0, color));
            }
        }
        _ => {
            painter.add(egui::Shape::line(points, stroke));
            arrowhead(painter, end, direction, color, 8.0);
        }
    }
}

fn arrowhead(painter: &egui::Painter, tip: Pos2, direction: egui::Vec2, color: Color32, size: f32) {
    let normal = vec2(-direction.y, direction.x);
    let base = tip - direction * size;
    painter.add(egui::Shape::convex_polygon(
        vec![tip, base + normal * size * 0.45, base - normal * size * 0.45],
        color,
        Stroke::NONE,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chemical_text_formats_subscripts_and_superscripts() {
        assert_eq!(format_chemical_text("H_2O"), "H₂O");
        assert_eq!(format_chemical_text("SO_4^2-"), "SO₄²⁻");
        assert_eq!(format_chemical_text("C^+"), "C⁺");
    }

    #[test]
    fn arrows_and_conditions_round_trip_through_serde() {
        let annotation = Annotation::Arrow(ReactionArrow::new(ArrowKind::Equilibrium, [0.0, 0.0], [3.0, 0.0]));
        let json = serde_json::to_string(&annotation).expect("serialize");
        let restored: Annotation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, annotation);

        let conditions = Annotation::Conditions(ConditionBox::line([0.0, 1.0], "H_2O, 25 ^oC"));
        let json = serde_json::to_string(&conditions).expect("serialize");
        let restored: Annotation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, conditions);
    }

    #[test]
    fn a_subscript_does_not_swallow_the_bond_after_it() {
        assert_eq!(format_chemical_text("CH_3-CH_3"), "CH₃-CH₃");
        assert_eq!(format_chemical_text("SO_4^2-"), "SO₄²⁻");
        assert_eq!(format_chemical_text("Fe^3+"), "Fe³⁺");
        assert_eq!(format_chemical_text("H_2O"), "H₂O");
    }
}
