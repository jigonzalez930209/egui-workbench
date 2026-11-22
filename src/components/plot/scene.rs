//! A resolution-independent list of drawing primitives.
//!
//! The plot is described once as a [`Scene`]; [`paint`] draws it with egui and
//! `export` writes the same primitives as SVG, so an export matches the screen.

use egui::epaint::{Mesh, PathShape, PathStroke, TextShape};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2};

/// How a line is stroked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Dash {
    Solid,
    /// Dash and gap lengths.
    Dashed(f32, f32),
    /// Dot spacing; dots have the stroke's width as diameter.
    Dotted(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Prim {
    /// An open polyline.
    Line {
        points: Vec<Pos2>,
        width: f32,
        color: Color32,
        dash: Dash,
    },
    /// The area between a polyline and a horizontal base.
    Area {
        points: Vec<Pos2>,
        base_y: f32,
        color: Color32,
    },
    /// A filled convex polygon.
    Polygon { points: Vec<Pos2>, color: Color32 },
    Rect {
        rect: Rect,
        radius: f32,
        fill: Color32,
        stroke: Option<(f32, Color32)>,
    },
    Circle {
        center: Pos2,
        radius: f32,
        fill: Color32,
        stroke: Option<(f32, Color32)>,
    },
    /// Text whose top-left corner is `pos` (before rotation). `baseline` is the
    /// distance from the top to the baseline; `angle` rotates around `pos`.
    Text {
        pos: Pos2,
        text: String,
        size: f32,
        color: Color32,
        baseline: f32,
        angle: f32,
    },
}

/// Primitives in three layers: chrome under the data, data clipped to the frame, and
/// chrome over the data (frame, legend).
#[derive(Clone, Debug)]
pub(super) struct Scene {
    pub size: egui::Vec2,
    pub frame: Rect,
    pub under: Vec<Prim>,
    pub data: Vec<Prim>,
    pub over: Vec<Prim>,
    /// Per series (by index), the points drawn when the series was thinned; `None` for
    /// hidden series and those drawn in full.
    pub drawn: Vec<Option<Vec<usize>>>,
}

/// Paint a scene: `clipped` is the same painter clipped to the data frame.
///
/// The scene is consumed: its point lists move into the shapes instead of being
/// copied, and each layer reaches egui in one batch (one lock, not one per marker).
pub(super) fn paint(painter: &Painter, clipped: &Painter, scene: Scene) {
    let layer = |painter: &Painter, prims: Vec<Prim>| {
        let mut shapes = Vec::with_capacity(prims.len());
        for prim in prims {
            push_shapes(painter, prim, &mut shapes);
        }
        painter.extend(shapes);
    };
    layer(painter, scene.under);
    layer(clipped, scene.data);
    layer(painter, scene.over);
}

fn push_shapes(painter: &Painter, prim: Prim, out: &mut Vec<Shape>) {
    match prim {
        Prim::Line {
            points,
            width,
            color,
            dash,
        } => {
            let stroke = Stroke::new(width, color);
            match dash {
                Dash::Solid if points.len() == 2 => out.push(Shape::line_segment([points[0], points[1]], stroke)),
                Dash::Solid => out.push(Shape::Path(PathShape::line(points, PathStroke::from(stroke)))),
                Dash::Dashed(dash, gap) => out.extend(Shape::dashed_line(&points, stroke, dash, gap)),
                Dash::Dotted(spacing) => out.extend(Shape::dotted_line(&points, color, spacing, width * 0.5)),
            }
        }
        Prim::Area { points, base_y, color } => out.push(Shape::mesh(area_mesh(&points, base_y, color))),
        Prim::Polygon { points, color } => out.push(Shape::convex_polygon(points, color, Stroke::NONE)),
        Prim::Rect {
            rect,
            radius,
            fill,
            stroke,
        } => {
            let stroke = stroke.map_or(Stroke::NONE, |(width, color)| Stroke::new(width, color));
            out.push(Shape::rect_filled(rect, radius, fill));
            if stroke.width > 0.0 {
                out.push(Shape::rect_stroke(rect, radius, stroke, StrokeKind::Inside));
            }
        }
        Prim::Circle {
            center,
            radius,
            fill,
            stroke,
        } => {
            let stroke = stroke.map_or(Stroke::NONE, |(width, color)| Stroke::new(width, color));
            out.push(Shape::circle_filled(center, radius, fill));
            if stroke.width > 0.0 {
                out.push(Shape::circle_stroke(center, radius, stroke));
            }
        }
        Prim::Text {
            pos,
            text,
            size,
            color,
            angle,
            ..
        } => {
            let galley = painter.layout_no_wrap(text, FontId::proportional(size), color);
            out.push(Shape::Text(TextShape::new(pos, galley, color).with_angle(angle)));
        }
    }
}

/// A triangle strip between the line and a horizontal base (works for any x order).
fn area_mesh(points: &[Pos2], base_y: f32, color: Color32) -> Mesh {
    let mut mesh = Mesh::default();
    for pair in points.windows(2) {
        let index = mesh.vertices.len() as u32;
        for position in [pair[0], pos2(pair[0].x, base_y), pair[1], pos2(pair[1].x, base_y)] {
            mesh.colored_vertex(position, color);
        }
        mesh.add_triangle(index, index + 1, index + 2);
        mesh.add_triangle(index + 2, index + 1, index + 3);
    }
    mesh
}

/// Measures text with egui's fonts, so the layout (and the export) uses the real glyph
/// widths, and snaps coordinates to the physical pixel grid.
pub(super) struct Measure<'c> {
    pub ctx: &'c egui::Context,
    pub pixels_per_point: f32,
}

impl Measure<'_> {
    /// Width, height, and baseline offset of one line of text.
    pub fn text(&self, text: &str, size: f32) -> (f32, f32, f32) {
        let galley = self
            .ctx
            .fonts_mut(|fonts| fonts.layout_no_wrap(text.to_owned(), FontId::proportional(size), Color32::WHITE));
        let baseline = galley
            .rows
            .first()
            .and_then(|row| row.row.glyphs.first().map(|glyph| row.pos.y + glyph.pos.y))
            .unwrap_or(size * 0.9);
        (galley.size().x, galley.size().y, baseline)
    }

    /// Snap to a physical pixel boundary.
    pub fn snap(&self, value: f32) -> f32 {
        (value * self.pixels_per_point).round() / self.pixels_per_point
    }

    /// Snap to a physical pixel center (for 1 px lines).
    pub fn center(&self, value: f32) -> f32 {
        ((value * self.pixels_per_point - 0.5).round() + 0.5) / self.pixels_per_point
    }

    /// A text primitive placed by `anchor` relative to `at`.
    pub fn label(&self, at: Pos2, anchor: Align2, text: &str, size: f32, color: Color32) -> Prim {
        let (width, height, baseline) = self.text(text, size);
        let rect = anchor.anchor_size(at, egui::vec2(width, height));
        Prim::Text {
            pos: pos2(self.snap(rect.left()), self.snap(rect.top())),
            text: text.to_owned(),
            size,
            color,
            baseline,
            angle: 0.0,
        }
    }
}
