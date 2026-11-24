//! The orbit camera and its data → screen projection.

use egui::{Pos2, Rect, pos2};

use super::config::Plot3DConfig;
use super::types::Bounds3D;

/// How the camera projects depth.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Projection {
    /// Nearer points are larger.
    #[default]
    Perspective,
    /// A parallel projection with no foreshortening.
    Orthographic,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An orbit camera: angles, distance, target, field of view, and projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera3D {
    /// Rotation around the vertical axis, in radians.
    pub yaw: f32,
    /// Elevation above the horizon, in radians; clamped to ±85° by interaction.
    pub pitch: f32,
    /// Distance from the target to the eye.
    pub distance: f64,
    /// The point the camera looks at, in scaled data units.
    pub target: [f64; 3],
    /// Vertical field of view in degrees.
    pub fov: f32,
    /// Perspective or orthographic.
    pub projection: Projection,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            yaw: -0.72,
            pitch: 0.42,
            distance: 3.2,
            target: [0.0; 3],
            fov: 45.0,
            projection: Projection::Perspective,
        }
    }
}

impl Camera3D {
    /// A camera that fits `bounds` in `frame` at the configured angles.
    pub(super) fn auto(bounds: Bounds3D, config: &Plot3DConfig, frame: Rect) -> Self {
        let center = bounds.center();
        let radius = corners(bounds)
            .iter()
            .map(|corner| {
                let delta = [corner[0] - center[0], corner[1] - center[1], corner[2] - center[2]];
                delta.iter().map(|value| value * value).sum::<f64>().sqrt()
            })
            .fold(0.0, f64::max)
            .max(0.001);
        let fov_degrees = if config.fov.is_finite() {
            config.fov.clamp(15.0, 120.0)
        } else {
            45.0
        };
        let fov = f64::from(fov_degrees).to_radians();
        // Fit the bounding sphere in the shorter side of the frame, with a little air.
        let aspect = (frame.width() / frame.height().max(1.0)).max(0.05) as f64;
        let wider = if aspect < 1.0 { radius / aspect } else { radius };
        let distance = (wider / (fov * 0.5).tan() * 1.45).max(radius * 1.2);
        Self {
            yaw: if config.yaw.is_finite() {
                config.yaw
            } else {
                Self::default().yaw
            },
            pitch: if config.pitch.is_finite() {
                config.pitch.clamp(-1.5, 1.5)
            } else {
                Self::default().pitch
            },
            distance,
            target: center,
            fov: fov_degrees,
            projection: config.projection,
        }
    }

    /// Turn the camera by a pointer delta in points.
    pub fn orbit(self, delta: egui::Vec2) -> Self {
        Self {
            yaw: self.yaw - delta.x * 0.01,
            pitch: (self.pitch + delta.y * 0.008).clamp(-1.5, 1.5),
            ..self
        }
    }

    /// Zoom around the target: `factor > 1` moves closer.
    pub fn zoom(self, factor: f64) -> Self {
        Self {
            distance: (self.distance / factor.clamp(0.05, 20.0)).clamp(1e-5, 1e12),
            ..self
        }
    }

    /// Move the target so the data follows a drag of `delta` points.
    pub fn pan(self, delta: egui::Vec2, frame: Rect) -> Self {
        let (right, up) = basis(self.yaw, self.pitch);
        let units =
            2.0 * self.distance * (f64::from(self.fov).to_radians() * 0.5).tan() / f64::from(frame.height().max(1.0));
        let target = [
            self.target[0] - right[0] * f64::from(delta.x) * units + up[0] * f64::from(delta.y) * units,
            self.target[1] - right[1] * f64::from(delta.x) * units + up[1] * f64::from(delta.y) * units,
            self.target[2] - right[2] * f64::from(delta.x) * units + up[2] * f64::from(delta.y) * units,
        ];
        Self { target, ..self }
    }
}

/// A camera prepared for one frame.
#[derive(Clone, Copy, Debug)]
pub(super) struct Projector {
    frame: Rect,
    eye: [f64; 3],
    right: [f64; 3],
    up: [f64; 3],
    forward: [f64; 3],
    focal: f64,
    distance: f64,
    projection: Projection,
}

impl Projector {
    pub fn new(camera: Camera3D, frame: Rect) -> Self {
        let (right, up) = basis(camera.yaw, camera.pitch);
        let cp = f64::from(camera.pitch).cos();
        let eye = [
            camera.target[0] + camera.distance * cp * f64::from(camera.yaw).cos(),
            camera.target[1] + camera.distance * cp * f64::from(camera.yaw).sin(),
            camera.target[2] + camera.distance * f64::from(camera.pitch).sin(),
        ];
        let forward = normalize([
            camera.target[0] - eye[0],
            camera.target[1] - eye[1],
            camera.target[2] - eye[2],
        ]);
        Self {
            frame,
            eye,
            right,
            up,
            forward,
            focal: 0.5 * f64::from(frame.height().max(1.0))
                / (f64::from(if camera.fov.is_finite() {
                    camera.fov.clamp(15.0, 120.0)
                } else {
                    45.0
                })
                .to_radians()
                    * 0.5)
                    .tan(),
            distance: camera.distance.max(1e-6),
            projection: camera.projection,
        }
    }

    pub fn project(&self, point: [f64; 3]) -> Option<(Pos2, f64)> {
        let rel = [point[0] - self.eye[0], point[1] - self.eye[1], point[2] - self.eye[2]];
        let x = dot(rel, self.right);
        let y = dot(rel, self.up);
        let z = dot(rel, self.forward);
        let scale = match self.projection {
            Projection::Perspective => {
                if z <= self.distance * 0.01 {
                    return None;
                }
                self.focal / z
            }
            Projection::Orthographic => self.focal / self.distance,
        };
        let pos = pos2(
            self.frame.center().x + (x * scale) as f32,
            self.frame.center().y - (y * scale) as f32,
        );
        if !pos.is_finite() {
            return None;
        }
        // A point just in front of the near plane can project thousands of pixels
        // away. Treat that as behind the camera instead of painting a line to the
        // frame edge.
        let limit = f64::from(self.frame.width().max(self.frame.height()).max(1.0)) * 8.0;
        let (left, right) = (
            f64::from(self.frame.left()) - limit,
            f64::from(self.frame.right()) + limit,
        );
        let (top, bottom) = (
            f64::from(self.frame.top()) - limit,
            f64::from(self.frame.bottom()) + limit,
        );
        let (px, py) = (f64::from(pos.x), f64::from(pos.y));
        (px >= left && px <= right && py >= top && py <= bottom).then_some((pos, z))
    }

    pub fn eye(&self) -> [f64; 3] {
        self.eye
    }

    pub fn frame(&self) -> Rect {
        self.frame
    }

    /// Camera-space forward position, for depth sorting.
    pub fn depth(&self, point: [f64; 3]) -> f64 {
        dot(
            [point[0] - self.eye[0], point[1] - self.eye[1], point[2] - self.eye[2]],
            self.forward,
        )
    }
}

/// The eight corners of a bounded box.
pub(super) fn corners(bounds: Bounds3D) -> [[f64; 3]; 8] {
    [
        bounds.corner(false, false, false),
        bounds.corner(true, false, false),
        bounds.corner(false, true, false),
        bounds.corner(true, true, false),
        bounds.corner(false, false, true),
        bounds.corner(true, false, true),
        bounds.corner(false, true, true),
        bounds.corner(true, true, true),
    ]
}

fn basis(yaw: f32, pitch: f32) -> ([f64; 3], [f64; 3]) {
    let (yaw, pitch) = (f64::from(yaw), f64::from(pitch));
    let cp = pitch.cos();
    let forward = [-cp * yaw.cos(), -cp * yaw.sin(), -pitch.sin()];
    let right = normalize(cross(forward, [0.0, 0.0, 1.0]));
    let right = if right == [0.0, 0.0, 0.0] {
        [1.0, 0.0, 0.0]
    } else {
        right
    };
    let up = cross(right, forward);
    (right, up)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    let length = dot(vector, vector).sqrt();
    if length <= 1e-12 {
        [0.0; 3]
    } else {
        [vector[0] / length, vector[1] / length, vector[2] / length]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_far_outside_the_frame_are_rejected() {
        let frame = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0));
        let projector = Projector::new(Camera3D::default(), frame);
        assert!(projector.project([0.0, 0.0, 0.0]).is_some());
        assert!(projector.project([1e12, 0.0, 0.0]).is_none());
    }
}
