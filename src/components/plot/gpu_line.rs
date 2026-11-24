//! Screen-space line runs shared by the CPU and GPU data layers.

use egui::{Color32, Pos2};

/// One line or marker run for the GPU data layer, in screen coordinates.
#[derive(Clone, Debug)]
pub(crate) struct GpuLine {
    pub points: Vec<Pos2>,
    pub color: Color32,
    pub width: f32,
    pub closed: bool,
}

impl GpuLine {
    pub fn new(points: Vec<Pos2>, color: Color32, width: f32) -> Self {
        Self {
            points,
            color,
            width,
            closed: false,
        }
    }

    pub fn closed(mut self) -> Self {
        self.closed = true;
        self
    }
}
