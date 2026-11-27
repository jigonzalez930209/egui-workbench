//! Vector icons for the chemistry editor. They are drawn with `egui::Painter`, so the
//! example never depends on an external icon font or asset registration.

use egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, pos2, vec2,
};

/// Every tool or chemistry category shown in the editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Select,
    Pan,
    Lasso,
    Erase,
    Undo,
    Redo,
    Trash,
    Clean,
    Fit,
    BondSingle,
    BondDouble,
    BondTriple,
    BondAromatic,
    BondWedge,
    BondHash,
    BondWavy,
    Chain,
    RingBenzene,
    RingCyclohexane,
    RingCyclopentadiene,
    RingPyridine,
    RingPyrrole,
    RingChair,
    RingBoat,
    ChargePositive,
    ChargeNegative,
    PeriodicTable,
    Attach,
    AminoAcid,
    Sugar,
    Nucleobase,
    Heterocycle,
    FunctionalGroup,
    Solvent,
    Ligand,
    Lipid,
    Reaction,
    Conditions,
}

/// A compact icon button for the toolbars.
pub fn icon_button(ui: &mut Ui, icon: Icon, selected: bool, tooltip: &str) -> Response {
    icon_button_sized(ui, icon, selected, vec2(28.0, 26.0), tooltip)
}

/// An icon button with an explicit size.
pub fn icon_button_sized(ui: &mut Ui, icon: Icon, selected: bool, size: Vec2, tooltip: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let visuals = ui.visuals();
    let (fill, stroke) = if selected {
        (visuals.selection.bg_fill, visuals.selection.stroke)
    } else if response.hovered() {
        (visuals.widgets.hovered.bg_fill, visuals.widgets.hovered.bg_stroke)
    } else {
        (visuals.widgets.inactive.bg_fill, visuals.widgets.inactive.bg_stroke)
    };
    ui.painter().rect(rect, 4.0, fill, stroke, StrokeKind::Inside);
    let color = if selected {
        visuals.selection.stroke.color
    } else {
        visuals.text_color()
    };
    paint_icon(ui.painter(), rect.shrink(6.0), icon, color);
    response.on_hover_text(tooltip)
}

/// Draw one icon into `rect`.
pub fn paint_icon(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let line = Stroke::new(1.5, color);
    let thin = Stroke::new(1.0, color);
    let at = |x: f32, y: f32| pos2(rect.left() + x * rect.width(), rect.top() + y * rect.height());
    match icon {
        Icon::Select => {
            let points = vec![
                at(0.25, 0.12),
                at(0.25, 0.78),
                at(0.43, 0.62),
                at(0.54, 0.9),
                at(0.66, 0.84),
                at(0.55, 0.57),
                at(0.78, 0.57),
            ];
            painter.add(Shape::convex_polygon(points.clone(), color.gamma_multiply(0.15), line));
            painter.add(Shape::line(points, line));
        }
        Icon::Pan => {
            painter.rect_stroke(
                Rect::from_min_max(at(0.3, 0.46), at(0.7, 0.86)),
                4.0,
                line,
                StrokeKind::Inside,
            );
            for x in [0.4, 0.5, 0.6] {
                painter.line_segment([at(x, 0.46), at(x, 0.2)], line);
            }
            painter.line_segment([at(0.3, 0.62), at(0.15, 0.48)], line);
        }
        Icon::Lasso => {
            painter.add(Shape::line(
                vec![
                    at(0.62, 0.22),
                    at(0.38, 0.2),
                    at(0.2, 0.4),
                    at(0.26, 0.62),
                    at(0.48, 0.68),
                    at(0.64, 0.56),
                    at(0.62, 0.38),
                ],
                line,
            ));
            painter.add(Shape::line(vec![at(0.48, 0.68), at(0.6, 0.84), at(0.5, 0.9)], line));
        }
        Icon::Erase => {
            let points = vec![at(0.2, 0.62), at(0.5, 0.2), at(0.82, 0.45), at(0.5, 0.84)];
            painter.add(Shape::convex_polygon(points.clone(), color.gamma_multiply(0.12), line));
            painter.add(Shape::line(points, line));
            painter.line_segment([at(0.36, 0.47), at(0.65, 0.7)], thin);
            painter.line_segment([at(0.25, 0.87), at(0.85, 0.87)], thin);
        }
        Icon::Undo | Icon::Redo => {
            let flip = icon == Icon::Redo;
            let point = |x: f32, y: f32| {
                if flip { at(1.0 - x, y) } else { at(x, y) }
            };
            let points = (0..=12)
                .map(|index| {
                    let t = index as f32 / 12.0;
                    let angle = std::f32::consts::PI * (0.15 + 1.25 * t);
                    point(0.5 + 0.3 * angle.cos(), 0.55 - 0.3 * angle.sin())
                })
                .collect::<Vec<_>>();
            painter.add(Shape::line(points.clone(), line));
            let tip = *points.last().unwrap_or(&point(0.5, 0.5));
            let dir = if flip { vec2(1.0, 0.4) } else { vec2(-1.0, 0.4) };
            arrow_head(painter, tip, dir, color);
        }
        Icon::Trash => {
            painter.line_segment([at(0.2, 0.24), at(0.8, 0.24)], line);
            painter.line_segment([at(0.4, 0.14), at(0.6, 0.14)], thin);
            painter.add(Shape::line(
                vec![at(0.27, 0.3), at(0.35, 0.88), at(0.65, 0.88), at(0.73, 0.3)],
                line,
            ));
            for x in [0.44, 0.56] {
                painter.line_segment([at(x, 0.4), at(x, 0.76)], thin);
            }
        }
        Icon::Clean => {
            painter.line_segment([at(0.72, 0.12), at(0.44, 0.5)], line);
            painter.add(Shape::convex_polygon(
                vec![at(0.28, 0.48), at(0.62, 0.4), at(0.56, 0.88), at(0.34, 0.8)],
                color.gamma_multiply(0.16),
                line,
            ));
            painter.line_segment([at(0.18, 0.2), at(0.3, 0.2)], thin);
            painter.line_segment([at(0.24, 0.14), at(0.24, 0.26)], thin);
        }
        Icon::Fit => {
            for (x, y, dx, dy) in [
                (0.16, 0.16, 1.0, 1.0),
                (0.84, 0.16, -1.0, 1.0),
                (0.16, 0.84, 1.0, -1.0),
                (0.84, 0.84, -1.0, -1.0),
            ] {
                painter.line_segment([at(x, y), at(x + 0.16 * dx, y)], line);
                painter.line_segment([at(x, y), at(x, y + 0.16 * dy)], line);
            }
        }
        Icon::BondSingle => {
            painter.line_segment([at(0.12, 0.5), at(0.88, 0.5)], line);
        }
        Icon::BondDouble => {
            painter.line_segment([at(0.12, 0.38), at(0.88, 0.38)], line);
            painter.line_segment([at(0.12, 0.62), at(0.88, 0.62)], line);
        }
        Icon::BondTriple => {
            for y in [0.26, 0.5, 0.74] {
                painter.line_segment([at(0.12, y), at(0.88, y)], line);
            }
        }
        Icon::BondAromatic => {
            ring(painter, &at, 6, 0.32, 0.5, 0.5, line);
            painter.circle_stroke(at(0.5, 0.5), rect.width() * 0.14, thin);
        }
        Icon::BondWedge => {
            painter.add(Shape::convex_polygon(
                vec![at(0.12, 0.5), at(0.88, 0.28), at(0.88, 0.72)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::BondHash => {
            for index in 0..5 {
                let x = 0.18 + index as f32 * 0.14;
                let half = 0.06 + index as f32 * 0.035;
                painter.line_segment([at(x, 0.5 - half), at(x, 0.5 + half)], line);
            }
        }
        Icon::BondWavy => {
            let points = (0..=16)
                .map(|index| {
                    let x = index as f32 / 16.0;
                    at(0.12 + 0.76 * x, 0.5 + 0.1 * (x * std::f32::consts::TAU).sin())
                })
                .collect::<Vec<_>>();
            painter.add(Shape::line(points, line));
        }
        Icon::Chain => {
            painter.add(Shape::line(
                vec![
                    at(0.1, 0.62),
                    at(0.32, 0.32),
                    at(0.52, 0.62),
                    at(0.72, 0.32),
                    at(0.9, 0.62),
                ],
                line,
            ));
        }
        Icon::RingBenzene => {
            ring(painter, &at, 6, 0.34, 0.5, 0.5, line);
            painter.circle_stroke(at(0.5, 0.5), rect.width() * 0.14, thin);
        }
        Icon::RingCyclohexane => ring(painter, &at, 6, 0.34, 0.5, 0.5, line),
        Icon::RingCyclopentadiene => {
            ring(painter, &at, 5, 0.35, 0.5, 0.52, line);
            painter.line_segment([at(0.3, 0.58), at(0.45, 0.42)], thin);
            painter.line_segment([at(0.55, 0.4), at(0.7, 0.56)], thin);
        }
        Icon::RingPyridine => {
            ring(painter, &at, 6, 0.34, 0.5, 0.5, line);
            painter.circle_stroke(at(0.5, 0.5), rect.width() * 0.14, thin);
            small_text(painter, at(0.5, 0.18), "N", color, rect);
        }
        Icon::RingPyrrole => {
            ring(painter, &at, 5, 0.35, 0.5, 0.52, line);
            painter.circle_stroke(at(0.5, 0.52), rect.width() * 0.13, thin);
            small_text(painter, at(0.5, 0.2), "N", color, rect);
        }
        Icon::RingChair => {
            painter.add(Shape::line(
                vec![
                    at(0.18, 0.5),
                    at(0.3, 0.3),
                    at(0.62, 0.24),
                    at(0.82, 0.44),
                    at(0.7, 0.66),
                    at(0.34, 0.74),
                    at(0.18, 0.5),
                ],
                line,
            ));
        }
        Icon::RingBoat => {
            painter.add(Shape::line(
                vec![
                    at(0.16, 0.58),
                    at(0.28, 0.3),
                    at(0.5, 0.44),
                    at(0.72, 0.3),
                    at(0.84, 0.58),
                    at(0.5, 0.78),
                    at(0.16, 0.58),
                ],
                line,
            ));
        }
        Icon::ChargePositive | Icon::ChargeNegative => {
            painter.circle_stroke(at(0.5, 0.5), rect.width() * 0.35, line);
            painter.line_segment([at(0.3, 0.5), at(0.7, 0.5)], line);
            if icon == Icon::ChargePositive {
                painter.line_segment([at(0.5, 0.3), at(0.5, 0.7)], line);
            }
        }
        Icon::PeriodicTable => {
            for row in 0..3 {
                for column in 0..4 {
                    let x = 0.16 + column as f32 * 0.18;
                    let y = 0.2 + row as f32 * 0.22;
                    painter.rect_stroke(
                        Rect::from_min_size(at(x, y), vec2(rect.width() * 0.13, rect.height() * 0.15)),
                        1.0,
                        thin,
                        StrokeKind::Inside,
                    );
                }
            }
        }
        Icon::Attach => {
            painter.circle_stroke(at(0.38, 0.5), rect.width() * 0.17, line);
            painter.circle_stroke(at(0.62, 0.5), rect.width() * 0.17, line);
            painter.line_segment([at(0.2, 0.5), at(0.8, 0.5)], thin);
        }
        Icon::AminoAcid => {
            painter.line_segment([at(0.24, 0.5), at(0.48, 0.5)], line);
            painter.line_segment([at(0.48, 0.5), at(0.7, 0.3)], line);
            painter.line_segment([at(0.48, 0.5), at(0.7, 0.7)], line);
            small_text(painter, at(0.16, 0.5), "N", color, rect);
            small_text(painter, at(0.8, 0.2), "O", color, rect);
        }
        Icon::Sugar => {
            ring(painter, &at, 6, 0.32, 0.5, 0.48, line);
            small_text(painter, at(0.5, 0.18), "O", color, rect);
            painter.line_segment([at(0.72, 0.3), at(0.82, 0.18)], thin);
        }
        Icon::Nucleobase => {
            ring(painter, &at, 6, 0.24, 0.42, 0.48, line);
            ring(painter, &at, 5, 0.24, 0.7, 0.5, line);
            small_text(painter, at(0.42, 0.26), "N", color, rect);
        }
        Icon::Heterocycle => {
            ring(painter, &at, 5, 0.32, 0.5, 0.5, line);
            painter.circle_filled(at(0.5, 0.18), 2.0, color);
        }
        Icon::FunctionalGroup => {
            small_text(painter, at(0.3, 0.5), "R", color, rect);
            painter.line_segment([at(0.45, 0.5), at(0.85, 0.5)], line);
        }
        Icon::Solvent => {
            painter.add(Shape::line(
                vec![
                    at(0.4, 0.14),
                    at(0.4, 0.42),
                    at(0.2, 0.84),
                    at(0.8, 0.84),
                    at(0.6, 0.42),
                    at(0.6, 0.14),
                ],
                line,
            ));
            painter.add(Shape::line(
                vec![at(0.28, 0.64), at(0.72, 0.64)],
                Stroke::new(2.0, color.gamma_multiply(0.5)),
            ));
        }
        Icon::Ligand => {
            ring(painter, &at, 6, 0.22, 0.38, 0.5, line);
            ring(painter, &at, 6, 0.22, 0.68, 0.5, line);
            painter.line_segment([at(0.56, 0.5), at(0.62, 0.5)], line);
        }
        Icon::Lipid => {
            painter.add(Shape::line(
                vec![
                    at(0.12, 0.5),
                    at(0.28, 0.28),
                    at(0.42, 0.5),
                    at(0.56, 0.28),
                    at(0.7, 0.5),
                    at(0.84, 0.28),
                ],
                line,
            ));
            painter.circle_filled(at(0.1, 0.5), 3.0, color);
        }
        Icon::Reaction => {
            painter.line_segment([at(0.1, 0.5), at(0.8, 0.5)], line);
            arrow_head(painter, at(0.86, 0.5), vec2(1.0, 0.0), color);
        }
        Icon::Conditions => {
            painter.rect_stroke(
                Rect::from_min_max(at(0.12, 0.24), at(0.88, 0.76)),
                2.0,
                line,
                StrokeKind::Inside,
            );
            for y in [0.38, 0.5, 0.62] {
                painter.line_segment([at(0.24, y), at(0.62, y)], thin);
            }
        }
    }
}

fn small_text(painter: &Painter, position: Pos2, text: &str, color: Color32, rect: Rect) {
    painter.text(
        position,
        Align2::CENTER_CENTER,
        text,
        FontId::proportional((rect.height() * 0.32).clamp(6.0, 10.0)),
        color,
    );
}

fn ring(
    painter: &Painter,
    at: &impl Fn(f32, f32) -> Pos2,
    sides: usize,
    radius: f32,
    center_x: f32,
    center_y: f32,
    stroke: Stroke,
) {
    let mut points = Vec::with_capacity(sides + 1);
    for index in 0..=sides {
        let angle = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * index as f32 / sides as f32;
        points.push(at(center_x + radius * angle.cos(), center_y + radius * angle.sin()));
    }
    painter.add(Shape::line(points, stroke));
}

fn arrow_head(painter: &Painter, tip: Pos2, direction: Vec2, color: Color32) {
    let direction = direction.normalized();
    let normal = vec2(-direction.y, direction.x);
    let base = tip - direction * 5.0;
    painter.add(Shape::convex_polygon(
        vec![tip, base + normal * 2.5, base - normal * 2.5],
        color,
        Stroke::NONE,
    ));
}
