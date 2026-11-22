//! The data ↔ screen transform, with the right y axis.

use egui::{Pos2, Rect, pos2};

use super::axis::{Bounds, Scale};

/// Maps data to screen and back. Works in "scaled" space (log10 already applied), so
/// zoom and pan are linear in what the user sees.
#[derive(Clone, Copy, Debug)]
pub(super) struct Transform {
    pub frame: Rect,
    /// Visible range in scaled units.
    pub view: Bounds,
    pub x_scale: Scale,
    pub y_scale: Scale,
    /// The right axis: its visible range in scaled units, and its scale.
    pub y2: Option<SecondAxis>,
}

/// The visible range of the right y axis, in scaled units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SecondAxis {
    pub min: f64,
    pub max: f64,
    pub scale: Scale,
}

impl Transform {
    /// The transform for series on `axis`: the right axis swaps in its range and scale.
    pub fn for_axis(self, axis: super::series::YAxis) -> Self {
        match (axis, self.y2) {
            (super::series::YAxis::Right, Some(y2)) => Self {
                view: Bounds {
                    min_y: y2.min,
                    max_y: y2.max,
                    ..self.view
                },
                y_scale: y2.scale,
                ..self
            },
            _ => self,
        }
    }

    pub fn to_screen(self, point: [f64; 2]) -> Pos2 {
        let x = self.x_scale.forward(point[0]);
        let y = self.y_scale.forward(point[1]);
        self.scaled_to_screen(x, y)
    }

    pub fn scaled_to_screen(&self, x: f64, y: f64) -> Pos2 {
        let tx = (x - self.view.min_x) / self.view.width();
        let ty = (y - self.view.min_y) / self.view.height();
        pos2(
            self.frame.left() + (tx * self.frame.width() as f64) as f32,
            self.frame.bottom() - (ty * self.frame.height() as f64) as f32,
        )
    }

    /// Screen position → scaled coordinates.
    pub fn to_scaled(self, pos: Pos2) -> [f64; 2] {
        let tx = f64::from((pos.x - self.frame.left()) / self.frame.width());
        let ty = f64::from((self.frame.bottom() - pos.y) / self.frame.height());
        [
            self.view.min_x + tx * self.view.width(),
            self.view.min_y + ty * self.view.height(),
        ]
    }

    /// Screen position → data coordinates.
    pub fn to_data(self, pos: Pos2) -> [f64; 2] {
        let [x, y] = self.to_scaled(pos);
        [self.x_scale.inverse(x), self.y_scale.inverse(y)]
    }

    /// Data units per screen point, in scaled space.
    pub fn units_per_point(&self) -> [f64; 2] {
        [
            self.view.width() / f64::from(self.frame.width().max(1.0)),
            self.view.height() / f64::from(self.frame.height().max(1.0)),
        ]
    }
}
