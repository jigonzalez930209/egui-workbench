//! Public 3D bounds, state, and response types.

use egui::{Pos2, Rect, Response};

use super::camera::{Camera3D, Projector};

/// An axis-aligned range in 3D data space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds3D {
    /// Smallest x.
    pub min_x: f64,
    /// Largest x.
    pub max_x: f64,
    /// Smallest y.
    pub min_y: f64,
    /// Largest y.
    pub max_y: f64,
    /// Smallest z.
    pub min_z: f64,
    /// Largest z.
    pub max_z: f64,
}

impl Bounds3D {
    /// Bounds from two opposite corners in any order.
    pub fn from_corners(a: [f64; 3], b: [f64; 3]) -> Self {
        Self {
            min_x: a[0].min(b[0]),
            max_x: a[0].max(b[0]),
            min_y: a[1].min(b[1]),
            max_y: a[1].max(b[1]),
            min_z: a[2].min(b[2]),
            max_z: a[2].max(b[2]),
        }
    }

    pub(super) const NOTHING: Self = Self {
        min_x: f64::INFINITY,
        max_x: f64::NEG_INFINITY,
        min_y: f64::INFINITY,
        max_y: f64::NEG_INFINITY,
        min_z: f64::INFINITY,
        max_z: f64::NEG_INFINITY,
    };

    pub(super) fn is_valid(&self) -> bool {
        self.min_x.is_finite()
            && self.max_x.is_finite()
            && self.min_y.is_finite()
            && self.max_y.is_finite()
            && self.min_z.is_finite()
            && self.max_z.is_finite()
            && self.min_x <= self.max_x
            && self.min_y <= self.max_y
            && self.min_z <= self.max_z
    }

    pub(super) fn extend(&mut self, point: [f64; 3]) {
        if point.iter().all(|value| value.is_finite()) {
            self.min_x = self.min_x.min(point[0]);
            self.max_x = self.max_x.max(point[0]);
            self.min_y = self.min_y.min(point[1]);
            self.max_y = self.max_y.max(point[1]);
            self.min_z = self.min_z.min(point[2]);
            self.max_z = self.max_z.max(point[2]);
        }
    }

    pub(super) fn union(&mut self, other: &Self) {
        if other.is_valid() {
            self.extend([other.min_x, other.min_y, other.min_z]);
            self.extend([other.max_x, other.max_y, other.max_z]);
        }
    }

    /// Width in data units.
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Depth in data units.
    pub fn depth(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Height in data units.
    pub fn height(&self) -> f64 {
        self.max_z - self.min_z
    }

    /// The center of the box.
    pub fn center(&self) -> [f64; 3] {
        [
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
            (self.min_z + self.max_z) * 0.5,
        ]
    }

    pub(super) fn corner(&self, x: bool, y: bool, z: bool) -> [f64; 3] {
        [
            if x { self.max_x } else { self.min_x },
            if y { self.max_y } else { self.min_y },
            if z { self.max_z } else { self.min_z },
        ]
    }
}

/// What a primary-button drag does in a 3D plot.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Plot3DTool {
    /// Drag turns the camera around the data.
    #[default]
    Orbit,
    /// Drag moves the camera target across the view plane.
    Pan,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Drag {
    Orbit,
    Pan,
}

/// The interactive state of one 3D plot. Keep it in the application model.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct Plot3DState {
    camera: Option<Camera3D>,
    /// The tool the primary button uses.
    pub tool: Plot3DTool,
    hidden: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) drag: Option<Drag>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) export_menu: bool,
    /// Where the dragged legend sits: 0–1 across the room left in the frame.
    legend_pos: Option<[f32; 2]>,
    /// Series extents by fingerprint, so the automatic view does not walk every point.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) extents: Vec<(u64, Bounds3D)>,
    #[cfg_attr(feature = "serde", serde(skip))]
    legend_hover: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    legend_hover_start: f64,
}

impl Plot3DState {
    /// The camera the user moved to, or `None` while the view is automatic.
    pub fn camera(&self) -> Option<Camera3D> {
        self.camera
    }

    /// Place the camera explicitly.
    pub fn set_camera(&mut self, camera: Camera3D) {
        self.camera = Some(camera);
    }

    /// Fit the data again and return to the configured camera angles.
    pub fn reset_view(&mut self) {
        self.camera = None;
        self.drag = None;
    }

    /// Whether the camera follows the data (no orbit, pan, or zoom since the last reset).
    pub fn is_auto(&self) -> bool {
        self.camera.is_none()
    }

    /// Where the legend was dragged to, as fractions of the free room.
    pub fn legend_position(&self) -> Option<[f32; 2]> {
        self.legend_pos
    }

    /// Place the legend; `None` returns it to its configured corner.
    pub fn set_legend_position(&mut self, position: Option<[f32; 2]>) {
        self.legend_pos = position.map(|[x, y]| [x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)]);
    }

    /// Whether the legend has hidden the series called `name`.
    pub fn is_hidden(&self, name: &str) -> bool {
        self.hidden.iter().any(|hidden| hidden == name)
    }

    /// Show or hide a series by name.
    pub fn set_hidden(&mut self, name: &str, hidden: bool) {
        self.hidden.retain(|entry| entry != name);
        if hidden {
            self.hidden.push(name.to_owned());
        }
    }

    pub(super) fn legend_hover(&self) -> Option<usize> {
        self.legend_hover
    }

    pub(super) fn legend_hover_start(&self) -> f64 {
        self.legend_hover_start
    }

    pub(super) fn set_legend_hover(&mut self, series: Option<usize>, now: f64) {
        if series != self.legend_hover {
            self.legend_hover = series;
            self.legend_hover_start = now;
        }
    }

    /// The series whose legend row is under the pointer, if any.
    pub fn hovered_series(&self) -> Option<usize> {
        self.legend_hover
    }
}

/// The nearest 3D data point under the pointer.
#[derive(Clone, Debug, PartialEq)]
pub struct Plot3DHover {
    /// Series name.
    pub series: String,
    /// Index into the series data (a grid vertex for surfaces).
    pub index: usize,
    /// The point, in data coordinates.
    pub point: [f64; 3],
}

/// What happened in the 3D plot this frame.
pub struct Plot3DResponse {
    /// The response of the whole plot area.
    pub response: Response,
    /// The nearest point within 12 pt of the pointer.
    pub hovered: Option<Plot3DHover>,
    /// A point that was clicked.
    pub clicked: Option<Plot3DHover>,
    /// The visible data bounds (the same box the axes draw).
    pub view: Bounds3D,
    /// The camera used this frame.
    pub camera: Camera3D,
    /// The data area on screen.
    pub frame: Rect,
    /// The toolbar's settings button was clicked.
    pub settings_requested: bool,
    /// The toolbar's export menu chose a format.
    pub export_requested: Option<super::super::ExportFormat>,
}

impl Plot3DResponse {
    /// Where a 3D data point lands on screen this frame.
    pub fn screen_pos(&self, point: [f64; 3]) -> Option<Pos2> {
        Projector::new(self.camera, self.frame)
            .project(point)
            .map(|(pos, _)| pos)
    }
}
