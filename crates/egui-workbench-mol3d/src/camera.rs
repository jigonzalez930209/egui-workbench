//! The arcball orbit camera and its projections.

use egui::{Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::ops::Mul;

use crate::math::{Quat, Vec3};

/// The projection used by the viewport.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Projection3D {
    /// A perspective projection with a vertical field of view in degrees.
    Perspective {
        /// Vertical field of view.
        fov_degrees: f64,
    },
    /// An orthographic projection with a vertical half-height in Angstrom.
    Orthographic {
        /// Half of the visible height.
        half_height: f64,
    },
}

impl Default for Projection3D {
    fn default() -> Self {
        Self::Perspective { fov_degrees: 45.0 }
    }
}

/// A ray in world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    /// Ray origin.
    pub origin: Vec3,
    /// Unit direction.
    pub direction: Vec3,
}

/// An arcball orbit camera.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera3D {
    /// Orbits around this point.
    pub target: Vec3,
    /// Distance from the target in Angstrom.
    pub distance: f64,
    /// Rotation from camera-local axes to world axes.
    pub rotation: Quat,
    /// Perspective or orthographic projection.
    pub projection: Projection3D,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 8.0,
            rotation: Quat::IDENTITY,
            projection: Projection3D::default(),
        }
    }
}

impl Camera3D {
    /// The camera position in world space.
    pub fn eye(self) -> Vec3 {
        self.target - self.forward() * self.distance
    }

    /// The direction the camera looks in world space.
    pub fn forward(self) -> Vec3 {
        self.rotation.rotate(Vec3::new(0.0, 0.0, -1.0))
    }

    /// The camera right axis in world space.
    pub fn right(self) -> Vec3 {
        self.rotation.rotate(Vec3::new(1.0, 0.0, 0.0))
    }

    /// The camera up axis in world space.
    pub fn up(self) -> Vec3 {
        self.rotation.rotate(Vec3::new(0.0, 1.0, 0.0))
    }

    /// World to camera-space coordinates (looking down `-z`).
    pub fn to_camera(self, world: Vec3) -> Vec3 {
        let delta = world - self.eye();
        Vec3::new(
            delta.dot(self.right()),
            delta.dot(self.up()),
            delta.dot(-self.forward()),
        )
    }

    /// Project `world` to a screen point and a positive depth.
    pub fn project(self, world: Vec3, viewport: Rect) -> Option<(Pos2, f64)> {
        let camera = self.to_camera(world);
        let depth = -camera.z;
        if !camera.is_finite() || depth <= 1.0e-4 {
            return None;
        }
        let aspect = f64::from(viewport.width().max(1.0)) / f64::from(viewport.height().max(1.0));
        let (ndc_x, ndc_y) = match self.projection {
            Projection3D::Perspective { fov_degrees } => {
                let tan_half = (fov_degrees.to_radians() * 0.5).tan();
                (camera.x / (depth * tan_half * aspect), camera.y / (depth * tan_half))
            }
            Projection3D::Orthographic { half_height } => (camera.x / (half_height * aspect), camera.y / half_height),
        };
        if !ndc_x.is_finite() || !ndc_y.is_finite() {
            return None;
        }
        let center = viewport.center();
        Some((
            egui::pos2(
                center.x + (ndc_x * f64::from(viewport.width()) * 0.5) as f32,
                center.y - (ndc_y * f64::from(viewport.height()) * 0.5) as f32,
            ),
            depth,
        ))
    }

    /// The screen radius of a sphere of `radius` at `world`.
    pub fn project_radius(self, world: Vec3, radius: f64, viewport: Rect) -> f32 {
        let half_screen = f64::from(viewport.height().max(1.0)) * 0.5;
        match self.projection {
            Projection3D::Perspective { fov_degrees } => {
                let depth = -self.to_camera(world).z;
                if depth <= 1.0e-4 {
                    return 0.0;
                }
                let tan_half = (fov_degrees.to_radians() * 0.5).tan();
                (radius / (depth * tan_half) * half_screen) as f32
            }
            Projection3D::Orthographic { half_height } => (radius / half_height * half_screen) as f32,
        }
    }

    /// The world-space ray through a screen point.
    pub fn ray(self, pointer: Pos2, viewport: Rect) -> Ray {
        let ndc_x = f64::from(pointer.x - viewport.center().x) / f64::from(viewport.width().max(1.0)) * 2.0;
        let ndc_y = -f64::from(pointer.y - viewport.center().y) / f64::from(viewport.height().max(1.0)) * 2.0;
        let aspect = f64::from(viewport.width().max(1.0)) / f64::from(viewport.height().max(1.0));
        let (origin, direction) = match self.projection {
            Projection3D::Perspective { fov_degrees } => {
                let tan_half = (fov_degrees.to_radians() * 0.5).tan();
                let local = Vec3::new(ndc_x * tan_half * aspect, ndc_y * tan_half, -1.0);
                (self.eye(), self.rotation.rotate(local).normalized())
            }
            Projection3D::Orthographic { half_height } => {
                let offset = self.right() * (ndc_x * half_height * aspect) + self.up() * (ndc_y * half_height);
                (self.eye() + offset, self.forward())
            }
        };
        Ray { origin, direction }
    }

    /// Orbit by a pointer delta in points. The arcball has no pitch clamp, so a drag
    /// can cross the poles without gimbal lock.
    pub fn orbit(&mut self, delta: Vec2) {
        let (right, up, _) = self.rotation.basis();
        let yaw = Quat::from_axis_angle(up, -f64::from(delta.x) * 0.008);
        let pitch = Quat::from_axis_angle(right, -f64::from(delta.y) * 0.008);
        self.rotation = yaw.mul(pitch).mul(self.rotation).normalized();
        if !self.rotation.is_finite() {
            self.rotation = Quat::IDENTITY;
        }
    }

    /// Pan the target by a pointer delta in points.
    pub fn pan(&mut self, delta: Vec2, viewport: Rect) {
        let height = f64::from(viewport.height().max(1.0));
        let world_per_point = match self.projection {
            Projection3D::Perspective { fov_degrees } => {
                2.0 * self.distance * (fov_degrees.to_radians() * 0.5).tan() / height
            }
            Projection3D::Orthographic { half_height } => 2.0 * half_height / height,
        };
        self.target = self.target
            + self.right() * (-f64::from(delta.x) * world_per_point)
            + self.up() * (f64::from(delta.y) * world_per_point);
    }

    /// Zoom by `factor` (greater than one zooms in).
    pub fn zoom(&mut self, factor: f64) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        match &mut self.projection {
            Projection3D::Perspective { .. } => {
                self.distance = (self.distance / factor).clamp(0.5, 500.0);
            }
            Projection3D::Orthographic { half_height } => {
                *half_height = (*half_height / factor).clamp(0.1, 500.0);
            }
        }
    }

    /// Frame `[min_x, min_y, min_z, max_x, max_y, max_z]`.
    pub fn fit(&mut self, bounds: [f64; 6], viewport: Rect) {
        let [min_x, min_y, min_z, max_x, max_y, max_z] = bounds;
        self.target = Vec3::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5, (min_z + max_z) * 0.5);
        let radius = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2) + (max_z - min_z).powi(2))
            .sqrt()
            .max(1.0)
            * 0.5;
        let aspect = f64::from(viewport.width().max(1.0)) / f64::from(viewport.height().max(1.0));
        match &mut self.projection {
            Projection3D::Perspective { fov_degrees } => {
                let tan_half = (fov_degrees.to_radians() * 0.5).tan();
                self.distance = (radius / tan_half * 1.2 / aspect.min(1.0)).clamp(0.5, 500.0);
            }
            Projection3D::Orthographic { half_height } => {
                *half_height = (radius * 1.2 / aspect.min(1.0)).clamp(0.1, 500.0);
            }
        }
    }

    /// Switch between perspective and orthographic, preserving the apparent size.
    pub fn set_projection(&mut self, projection: Projection3D) {
        self.projection = projection;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Rect, pos2, vec2};

    fn viewport() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))
    }

    #[test]
    fn the_target_projects_to_the_centre() {
        let camera = Camera3D::default();
        let (screen, depth) = camera.project(Vec3::ZERO, viewport()).expect("project");
        assert!((screen.x - 400.0).abs() < 1e-3);
        assert!((screen.y - 300.0).abs() < 1e-3);
        assert!((depth - 8.0).abs() < 1e-9);
    }

    #[test]
    fn a_ray_through_the_centre_points_at_the_target() {
        let camera = Camera3D::default();
        let ray = camera.ray(viewport().center(), viewport());
        assert!((ray.direction - camera.forward()).length() < 1e-9);
    }

    #[test]
    fn orbiting_crosses_the_poles_without_losing_the_basis() {
        let mut camera = Camera3D::default();
        for _ in 0..40 {
            camera.orbit(vec2(0.0, 100.0));
        }
        let (right, up, back) = camera.rotation.basis();
        assert!((right.length() - 1.0).abs() < 1e-9);
        assert!((up.length() - 1.0).abs() < 1e-9);
        assert!((back.length() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn orthographic_projection_and_zoom_keep_the_centre() {
        let mut camera = Camera3D::default();
        camera.set_projection(Projection3D::Orthographic { half_height: 5.0 });
        let (screen, _) = camera.project(Vec3::ZERO, viewport()).expect("project");
        assert!((screen.x - 400.0).abs() < 1e-3);
        camera.zoom(2.0);
        assert_eq!(camera.projection, Projection3D::Orthographic { half_height: 2.5 });
    }

    #[test]
    fn fit_centres_the_bounds() {
        let mut camera = Camera3D::default();
        camera.fit([-2.0, -1.0, -3.0, 4.0, 5.0, 1.0], viewport());
        assert_eq!(camera.target, Vec3::new(1.0, 2.0, -1.0));
        assert!(camera.distance > 0.0);
    }
}
