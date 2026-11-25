//! The DPI-independent 2D camera.

use egui::{Pos2, Rect, Vec2, pos2};

/// A camera that maps Angstrom to points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera2D {
    /// World point shown at the centre of the viewport.
    pub center: [f64; 2],
    /// Points per Angstrom.
    pub zoom: f64,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            center: [0.0; 2],
            zoom: 40.0,
        }
    }
}

impl Camera2D {
    /// A camera with an explicit centre and zoom.
    pub fn new(center: [f64; 2], zoom: f64) -> Self {
        Self {
            center,
            zoom: zoom.clamp(1.0, 1000.0),
        }
    }

    /// World to screen points. The y axis grows upward in world space.
    pub fn world_to_screen(self, world: [f64; 2], viewport: Rect) -> Pos2 {
        let center = viewport.center();
        pos2(
            center.x + ((world[0] - self.center[0]) * self.zoom) as f32,
            center.y - ((world[1] - self.center[1]) * self.zoom) as f32,
        )
    }

    /// Screen points to world.
    pub fn screen_to_world(self, position: Pos2, viewport: Rect) -> [f64; 2] {
        let center = viewport.center();
        [
            self.center[0] + f64::from(position.x - center.x) / self.zoom,
            self.center[1] - f64::from(position.y - center.y) / self.zoom,
        ]
    }

    /// Pan by a screen delta.
    pub fn pan(&mut self, delta: Vec2) {
        self.center[0] -= f64::from(delta.x) / self.zoom;
        self.center[1] += f64::from(delta.y) / self.zoom;
    }

    /// Zoom by `factor`, keeping the world point under `anchor` fixed.
    pub fn zoom_at(&mut self, factor: f64, anchor: Pos2, viewport: Rect) {
        let world = self.screen_to_world(anchor, viewport);
        self.zoom = (self.zoom * factor).clamp(1.0, 1000.0);
        let center = viewport.center();
        self.center[0] = world[0] - f64::from(anchor.x - center.x) / self.zoom;
        self.center[1] = world[1] + f64::from(anchor.y - center.y) / self.zoom;
    }

    /// Fit `[min_x, min_y, max_x, max_y]` with a margin in points.
    pub fn fit(&mut self, bounds: [f64; 4], viewport: Rect, margin: f32) {
        let [min_x, min_y, max_x, max_y] = bounds;
        let width = (max_x - min_x).abs();
        let height = (max_y - min_y).abs();
        let available = viewport.size() - Vec2::splat(2.0 * margin);
        if available.x <= 1.0 || available.y <= 1.0 {
            return;
        }
        let zoom_x = f64::from(available.x) / width.max(0.5);
        let zoom_y = f64::from(available.y) / height.max(0.5);
        self.zoom = zoom_x.min(zoom_y).clamp(1.0, 1000.0);
        self.center = [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5];
    }

    /// A 1-2-5 grid step that is roughly 40 points wide.
    pub fn grid_step(self) -> f64 {
        let target = 40.0 / self.zoom;
        let exponent = target.log10().floor();
        let base = 10.0_f64.powf(exponent);
        let normalized = target / base;
        let step = if normalized <= 1.0 {
            1.0
        } else if normalized <= 2.0 {
            2.0
        } else if normalized <= 5.0 {
            5.0
        } else {
            10.0
        };
        step * base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Rect, pos2, vec2};

    fn viewport() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 420.0))
    }

    #[test]
    fn world_and_screen_round_trip() {
        let camera = Camera2D::new([2.0, 3.0], 50.0);
        let world = [4.5, -1.0];
        let screen = camera.world_to_screen(world, viewport());
        let back = camera.screen_to_world(screen, viewport());
        assert!((back[0] - world[0]).abs() < 1e-9);
        assert!((back[1] - world[1]).abs() < 1e-9);
    }

    #[test]
    fn zoom_keeps_the_anchor_fixed() {
        let mut camera = Camera2D::default();
        let anchor = pos2(500.0, 120.0);
        let before = camera.screen_to_world(anchor, viewport());
        camera.zoom_at(2.5, anchor, viewport());
        let after = camera.screen_to_world(anchor, viewport());
        assert!((before[0] - after[0]).abs() < 1e-9);
        assert!((before[1] - after[1]).abs() < 1e-9);
    }

    #[test]
    fn fit_centres_the_bounds_and_uses_a_positive_zoom() {
        let mut camera = Camera2D::default();
        camera.fit([-2.0, -1.0, 6.0, 3.0], viewport(), 20.0);
        assert_eq!(camera.center, [2.0, 1.0]);
        assert!(camera.zoom > 1.0);
    }

    #[test]
    fn the_grid_step_is_one_two_or_five() {
        for zoom in [8.0, 40.0, 120.0] {
            let step = Camera2D::new([0.0; 2], zoom).grid_step();
            let normalized = step / 10.0_f64.powf(step.log10().floor());
            assert!([1.0, 2.0, 5.0].contains(&normalized.round()), "{step}");
        }
    }
}
