//! Serializable chart annotations and draggable regions of interest.

use egui::{Color32, Pos2, Rect, Vec2};

use super::axis::Transform;
use super::math_text::to_unicode_math;
use super::scene::Prim;

/// A text note, arrow, span, or region drawn in data coordinates.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "snake_case"))]
#[non_exhaustive]
pub enum PlotAnnotation {
    /// Text at a data point.
    Text {
        /// Anchor.
        at: [f64; 2],
        /// LaTeX-subset text.
        text: String,
        /// Point size.
        size: f32,
        /// Optional color.
        color: Option<Color32>,
    },
    /// An arrow from `from` to `to`.
    Arrow {
        /// Tail.
        from: [f64; 2],
        /// Head.
        to: [f64; 2],
        /// Optional color.
        color: Option<Color32>,
    },
    /// A vertical span between two x values.
    SpanX {
        /// First x.
        x0: f64,
        /// Second x.
        x1: f64,
        /// Optional fill.
        color: Option<Color32>,
    },
    /// A horizontal span between two y values.
    SpanY {
        /// First y.
        y0: f64,
        /// Second y.
        y1: f64,
        /// Optional fill.
        color: Option<Color32>,
    },
    /// A rectangle region; `draggable` is honoured by the host editor.
    Rect {
        /// First corner.
        min: [f64; 2],
        /// Second corner.
        max: [f64; 2],
        /// Stroke color.
        color: Option<Color32>,
        /// Fill color.
        fill: Option<Color32>,
        /// Whether the region can be edited by the host.
        draggable: bool,
    },
    /// An ellipse region.
    Ellipse {
        /// Center.
        center: [f64; 2],
        /// Horizontal/vertical radius.
        radius: [f64; 2],
        /// Stroke color.
        color: Option<Color32>,
        /// Fill color.
        fill: Option<Color32>,
        /// Whether the region can be edited by the host.
        draggable: bool,
    },
    /// A polygon region.
    Polygon {
        /// Vertices in data coordinates.
        points: Vec<[f64; 2]>,
        /// Stroke color.
        color: Option<Color32>,
        /// Fill color.
        fill: Option<Color32>,
        /// Whether the region can be edited by the host.
        draggable: bool,
    },
}

impl PlotAnnotation {
    /// A text note.
    pub fn text(at: [f64; 2], text: impl Into<String>) -> Self {
        Self::Text {
            at,
            text: text.into(),
            size: 12.0,
            color: None,
        }
    }

    /// An arrow.
    pub fn arrow(from: [f64; 2], to: [f64; 2]) -> Self {
        Self::Arrow { from, to, color: None }
    }

    /// A rectangle.
    pub fn rect(min: [f64; 2], max: [f64; 2]) -> Self {
        Self::Rect {
            min,
            max,
            color: None,
            fill: Some(Color32::from_white_alpha(20)),
            draggable: true,
        }
    }

    /// An ellipse.
    pub fn ellipse(center: [f64; 2], radius: [f64; 2]) -> Self {
        Self::Ellipse {
            center,
            radius,
            color: None,
            fill: Some(Color32::from_white_alpha(20)),
            draggable: true,
        }
    }

    /// A polygon.
    pub fn polygon(points: impl IntoIterator<Item = [f64; 2]>) -> Self {
        Self::Polygon {
            points: points.into_iter().collect(),
            color: None,
            fill: Some(Color32::from_white_alpha(20)),
            draggable: true,
        }
    }

    /// The color, falling back to the theme accent.
    fn color(&self, theme: &crate::theme::Theme) -> Color32 {
        match self {
            Self::Text { color, .. }
            | Self::Arrow { color, .. }
            | Self::Rect { color, .. }
            | Self::Ellipse { color, .. }
            | Self::Polygon { color, .. } => color.unwrap_or(theme.link),
            Self::SpanX { color, .. } | Self::SpanY { color, .. } => color.unwrap_or(theme.warning),
        }
    }
}

/// Append the annotations to a scene in the data layer.
pub(super) fn paint(
    out: &mut Vec<Prim>,
    transform: &Transform,
    annotations: &[PlotAnnotation],
    theme: &crate::theme::Theme,
) {
    let frame = transform.frame;
    for annotation in annotations {
        let color = annotation.color(theme);
        match annotation {
            PlotAnnotation::Text { at, text, size, .. } => {
                let position = transform.to_screen(*at);
                if position.is_finite() {
                    out.push(Prim::Text {
                        pos: position,
                        text: to_unicode_math(text),
                        size: *size,
                        color,
                        baseline: size * 0.8,
                        angle: 0.0,
                    });
                }
            }
            PlotAnnotation::Arrow { from, to, .. } => {
                let start = transform.to_screen(*from);
                let end = transform.to_screen(*to);
                if start.is_finite() && end.is_finite() {
                    out.push(Prim::Line {
                        points: vec![start, end],
                        width: 1.5,
                        color,
                        dash: super::scene::Dash::Solid,
                    });
                    let direction = (end - start).normalized();
                    let normal = Vec2::new(-direction.y, direction.x);
                    let tip = end;
                    let left = end - direction * 9.0 + normal * 4.0;
                    let right = end - direction * 9.0 - normal * 4.0;
                    out.push(Prim::Polygon {
                        points: vec![tip, left, right],
                        color,
                    });
                }
            }
            PlotAnnotation::SpanX { x0, x1, .. } => {
                let a = transform.to_screen([*x0, transform.view.min_y]);
                let b = transform.to_screen([*x1, transform.view.max_y]);
                let rect = Rect::from_two_pos(a, b).intersect(frame);
                if rect.is_positive() {
                    out.push(Prim::Rect {
                        rect,
                        radius: 0.0,
                        fill: color.gamma_multiply(0.12),
                        stroke: Some((1.0, color.gamma_multiply(0.7))),
                    });
                }
            }
            PlotAnnotation::SpanY { y0, y1, .. } => {
                let a = transform.to_screen([transform.view.min_x, *y0]);
                let b = transform.to_screen([transform.view.max_x, *y1]);
                let rect = Rect::from_two_pos(a, b).intersect(frame);
                if rect.is_positive() {
                    out.push(Prim::Rect {
                        rect,
                        radius: 0.0,
                        fill: color.gamma_multiply(0.12),
                        stroke: Some((1.0, color.gamma_multiply(0.7))),
                    });
                }
            }
            PlotAnnotation::Rect { min, max, fill, .. } => {
                let a = transform.to_screen(*min);
                let b = transform.to_screen(*max);
                let rect = Rect::from_two_pos(a, b).intersect(frame);
                if rect.is_positive() {
                    out.push(Prim::Rect {
                        rect,
                        radius: 0.0,
                        fill: fill.unwrap_or(Color32::TRANSPARENT),
                        stroke: Some((1.5, color)),
                    });
                }
            }
            PlotAnnotation::Ellipse {
                center, radius, fill, ..
            } => {
                let center = transform.to_screen(*center);
                let rx = (transform
                    .to_screen([f64::from(center.x) + radius[0], f64::from(center.y)])
                    .x
                    - center.x)
                    .abs();
                let ry = (transform
                    .to_screen([f64::from(center.x), f64::from(center.y) + radius[1]])
                    .y
                    - center.y)
                    .abs();
                if center.is_finite() && rx > 0.5 && ry > 0.5 {
                    let points: Vec<Pos2> = (0..48)
                        .map(|index| {
                            let angle = index as f32 / 48.0 * std::f32::consts::TAU;
                            Pos2::new(center.x + rx * angle.cos(), center.y + ry * angle.sin())
                        })
                        .collect();
                    out.push(Prim::Polygon {
                        points: points.clone(),
                        color: fill.unwrap_or(Color32::TRANSPARENT),
                    });
                    for pair in points.windows(2) {
                        out.push(Prim::Line {
                            points: pair.to_vec(),
                            width: 1.5,
                            color,
                            dash: super::scene::Dash::Solid,
                        });
                    }
                }
            }
            PlotAnnotation::Polygon { points, fill, .. } => {
                let points: Vec<Pos2> = points
                    .iter()
                    .map(|point| transform.to_screen(*point))
                    .filter(|point| point.is_finite())
                    .collect();
                if points.len() >= 3 {
                    out.push(Prim::Polygon {
                        points: points.clone(),
                        color: fill.unwrap_or(Color32::TRANSPARENT),
                    });
                    for index in 0..points.len() {
                        out.push(Prim::Line {
                            points: vec![points[index], points[(index + 1) % points.len()]],
                            width: 1.5,
                            color,
                            dash: super::scene::Dash::Solid,
                        });
                    }
                }
            }
        }
    }
}

/// A rectangle's data-space width/height, useful for an editor.
pub fn region_size(min: [f64; 2], max: [f64; 2]) -> [f64; 2] {
    [(max[0] - min[0]).abs(), (max[1] - min[1]).abs()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::plot::{Bounds, Scale};
    use egui::{Rect, pos2};

    fn transform() -> Transform {
        Transform {
            frame: Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 100.0)),
            view: Bounds::from_corners([0.0, 0.0], [10.0, 10.0]),
            x_scale: Scale::Linear,
            y_scale: Scale::Linear,
            y2: None,
        }
    }

    #[test]
    fn annotations_become_prims() {
        let theme = crate::theme::Theme::dark();
        let mut prims = Vec::new();
        let annotations = vec![
            PlotAnnotation::text([1.0, 1.0], r"$\alpha$"),
            PlotAnnotation::arrow([1.0, 1.0], [5.0, 5.0]),
            PlotAnnotation::rect([2.0, 2.0], [4.0, 4.0]),
            PlotAnnotation::ellipse([5.0, 5.0], [1.0, 2.0]),
            PlotAnnotation::polygon([[1.0, 2.0], [3.0, 2.0], [2.0, 4.0]]),
        ];
        paint(&mut prims, &transform(), &annotations, &theme);
        assert!(prims.len() > 5);
        assert!(prims.iter().any(|prim| matches!(prim, Prim::Text { .. })));
    }

    #[test]
    fn regions_have_serializable_geometry() {
        assert_eq!(region_size([0.0, 0.0], [2.0, 3.0]), [2.0, 3.0]);
    }
}
