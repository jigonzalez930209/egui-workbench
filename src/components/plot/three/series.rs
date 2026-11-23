//! 3D series: lines, scatter clouds, vertical bars, and regular surfaces.

use std::borrow::Cow;

use egui::Color32;

use super::super::axis::Scale;
use super::super::series::{LineStyle, Marker};
use super::types::Bounds3D;

/// A regular surface: `x` columns, `y` rows, and row-major `z` values.
#[derive(Clone, Debug, PartialEq)]
pub struct Surface<'a> {
    /// Column coordinates, left to right.
    pub x: Cow<'a, [f64]>,
    /// Row coordinates, back to front.
    pub y: Cow<'a, [f64]>,
    /// Height at `y * x.len() + x`, row-major.
    pub z: Cow<'a, [f64]>,
}

impl Surface<'_> {
    pub(super) fn columns(&self) -> usize {
        self.x.len()
    }

    pub(super) fn rows(&self) -> usize {
        self.y.len()
    }

    pub(super) fn point(&self, index: usize) -> Option<[f64; 3]> {
        let columns = self.x.len();
        if columns == 0 {
            return None;
        }
        let column = index % columns;
        let row = index / columns;
        Some([*self.x.get(column)?, *self.y.get(row)?, *self.z.get(index)?])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Kind3D {
    Line,
    Scatter,
    Bars,
    Surface,
}

/// One named 3D data set. Build it with [`Series3D::line`], [`Series3D::scatter`],
/// [`Series3D::bars`], [`Series3D::surface`], or [`Series3D::surface_fn`], then add it
/// to a [`super::Plot3D`].
#[derive(Clone, Debug)]
pub struct Series3D<'a> {
    pub(super) name: String,
    pub(super) data: Cow<'a, [[f64; 3]]>,
    pub(super) surface: Option<Surface<'a>>,
    pub(super) kind: Kind3D,
    pub(super) color: Option<Color32>,
    pub(super) legend: bool,
    pub(super) width: f32,
    pub(super) line_style: LineStyle,
    pub(super) marker: Option<Marker>,
    pub(super) radius: f32,
    pub(super) bar_width: Option<f64>,
    pub(super) base: f64,
}

impl<'a> Series3D<'a> {
    fn new(name: impl Into<String>, data: Cow<'a, [[f64; 3]]>, kind: Kind3D) -> Self {
        Self {
            name: name.into(),
            data,
            surface: None,
            kind,
            color: None,
            legend: true,
            width: 1.6,
            line_style: LineStyle::Solid,
            marker: None,
            radius: 3.0,
            bar_width: None,
            base: 0.0,
        }
    }

    /// A polyline through `[x, y, z]` points, in order.
    pub fn line(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 3]]>>) -> Self {
        Self::new(name, data.into(), Kind3D::Line)
    }

    /// Unconnected 3D markers.
    pub fn scatter(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 3]]>>) -> Self {
        Self::new(name, data.into(), Kind3D::Scatter)
    }

    /// Vertical bars from the z base (0 by default) to each `[x, y, z]` point.
    pub fn bars(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 3]]>>) -> Self {
        Self::new(name, data.into(), Kind3D::Bars)
    }

    /// A surface over the grid `x × y` with row-major `z`.
    ///
    /// ```
    /// # use egui_workbench::components::plot::{Plot3D, Plot3DState, Series3D};
    /// # let ctx = egui::Context::default();
    /// # let mut icons = egui_workbench::Icons::new();
    /// let x = vec![-1.0, 0.0, 1.0];
    /// let y = vec![-1.0, 0.0, 1.0];
    /// let z = vec![0.0, 1.0, 0.0, 1.0, 2.0, 1.0, 0.0, 1.0, 0.0];
    /// let mut state = Plot3DState::default();
    /// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
    /// Plot3D::new("terrain")
    ///     .series(Series3D::surface("terrain", x.clone(), y.clone(), z.clone()))
    ///     .show(ui, &mut icons, &mut state);
    /// # });
    /// # output.textures_delta.clear();
    /// ```
    pub fn surface(
        name: impl Into<String>,
        x: impl Into<Cow<'a, [f64]>>,
        y: impl Into<Cow<'a, [f64]>>,
        z: impl Into<Cow<'a, [f64]>>,
    ) -> Self {
        let surface = Surface {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        };
        let mut series = Self::new(name, Cow::Owned(Vec::new()), Kind3D::Surface);
        series.surface = Some(surface);
        series
    }

    /// A surface sampled `columns × rows` times over two ranges.
    pub fn surface_fn(
        name: impl Into<String>,
        x_range: std::ops::RangeInclusive<f64>,
        y_range: std::ops::RangeInclusive<f64>,
        samples: [usize; 2],
        f: impl Fn(f64, f64) -> f64,
    ) -> Series3D<'static> {
        let columns = samples[0].max(2);
        let rows = samples[1].max(2);
        let (x0, x1) = (*x_range.start(), *x_range.end());
        let (y0, y1) = (*y_range.start(), *y_range.end());
        let x: Vec<f64> = (0..columns)
            .map(|index| x0 + (x1 - x0) * index as f64 / (columns - 1) as f64)
            .collect();
        let y: Vec<f64> = (0..rows)
            .map(|index| y0 + (y1 - y0) * index as f64 / (rows - 1) as f64)
            .collect();
        let mut z = Vec::with_capacity(columns * rows);
        for &y_value in &y {
            for &x_value in &x {
                z.push(f(x_value, y_value));
            }
        }
        Series3D::surface(name, x, y, z)
    }

    /// Explicit color; the theme palette is used in order otherwise.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Line width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(0.5);
        self
    }

    /// Solid, dashed, or dotted lines.
    pub fn style(mut self, style: LineStyle) -> Self {
        self.line_style = style;
        self
    }

    /// A marker on every line vertex, or the scatter marker.
    pub fn marker(mut self, marker: Marker) -> Self {
        self.marker = Some(marker);
        self
    }

    /// Marker radius in points.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(1.0);
        self
    }

    /// Bar footprint in data units; the smallest x/y gap × 0.7 when `None`.
    pub fn bar_width(mut self, width: f64) -> Self {
        self.bar_width = Some(width);
        self
    }

    /// Where bars start (default 0).
    pub fn base(mut self, base: f64) -> Self {
        self.base = base;
        self
    }

    /// Leave the series out of the legend.
    pub fn hide_from_legend(mut self) -> Self {
        self.legend = false;
        self
    }

    /// The series name (also the legend label).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The points, for line, scatter, and bar series (empty for surfaces).
    pub fn data(&self) -> &[[f64; 3]] {
        &self.data
    }

    /// The surface grid, when this is a surface series.
    pub fn surface_grid(&self) -> Option<&Surface<'a>> {
        self.surface.as_ref()
    }

    /// One data point by index: a grid vertex for surfaces.
    pub fn point(&self, index: usize) -> Option<[f64; 3]> {
        match &self.surface {
            Some(surface) => surface.point(index),
            None => self.data.get(index).copied(),
        }
    }

    /// How many points the series has, including surface vertices.
    pub fn len(&self) -> usize {
        match &self.surface {
            Some(surface) => surface.z.len(),
            None => self.data.len(),
        }
    }

    /// Whether the series has no points.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(super) fn has_direction(&self) -> bool {
        self.kind == Kind3D::Line
    }

    pub(super) fn bar_width_value(&self) -> f64 {
        self.bar_width
            .filter(|width| width.is_finite() && *width > 0.0)
            .unwrap_or_else(|| self.min_spacing() * 0.7)
    }

    /// The smallest positive x or y step (1 when there is none).
    pub(super) fn min_spacing(&self) -> f64 {
        let points: Vec<[f64; 3]> = match &self.surface {
            Some(surface) => surface
                .z
                .iter()
                .enumerate()
                .filter_map(|(index, _)| surface.point(index))
                .collect(),
            None => self.data.iter().copied().collect(),
        };
        let x = smallest_step(points.iter().map(|point| point[0]));
        let y = smallest_step(points.iter().map(|point| point[1]));
        x.min(y)
    }

    /// Data extent in scaled space (log axes ignore values ≤ 0).
    pub(super) fn bounds(&self, x_scale: Scale, y_scale: Scale, z_scale: Scale) -> Bounds3D {
        let mut bounds = Bounds3D::NOTHING;
        if let Some(surface) = &self.surface {
            for index in 0..surface.z.len() {
                if let Some(point) = surface.point(index) {
                    bounds.extend([
                        x_scale.forward(point[0]),
                        y_scale.forward(point[1]),
                        z_scale.forward(point[2]),
                    ]);
                }
            }
            return bounds;
        }
        for point in self.data.iter() {
            bounds.extend([
                x_scale.forward(point[0]),
                y_scale.forward(point[1]),
                z_scale.forward(point[2]),
            ]);
        }
        if self.kind == Kind3D::Bars && bounds.is_valid() {
            let half = self.bar_width_value() * 0.5;
            bounds.extend([bounds.min_x - half, bounds.min_y - half, z_scale.forward(self.base)]);
            bounds.extend([bounds.max_x + half, bounds.max_y + half, z_scale.forward(self.base)]);
        }
        bounds
    }

    /// A cheap key for [`Self::bounds`]: buffer addresses and lengths, the kind, the
    /// scales, and sampled values.
    pub(super) fn fingerprint(&self, x_scale: Scale, y_scale: Scale, z_scale: Scale) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (self.data.as_ptr() as usize, self.data.len()).hash(&mut hasher);
        if let Some(surface) = &self.surface {
            for values in [&surface.x, &surface.y, &surface.z] {
                (values.as_ptr() as usize, values.len()).hash(&mut hasher);
                let step = (values.len() / 8).max(1);
                for value in values.iter().step_by(step) {
                    value.to_bits().hash(&mut hasher);
                }
            }
        }
        self.kind.hash(&mut hasher);
        (x_scale, y_scale, z_scale).hash(&mut hasher);
        let step = (self.data.len() / 16).max(1);
        for point in self.data.iter().step_by(step).chain(self.data.last()) {
            for value in point {
                value.to_bits().hash(&mut hasher);
            }
        }
        hasher.finish()
    }
}

fn smallest_step(values: impl Iterator<Item = f64>) -> f64 {
    let mut values: Vec<f64> = values.filter(|value| value.is_finite()).collect();
    values.sort_by(f64::total_cmp);
    let mut step = f64::INFINITY;
    for pair in values.windows(2) {
        let delta = pair[1] - pair[0];
        if delta > 0.0 {
            step = step.min(delta);
        }
    }
    if step.is_finite() { step } else { 1.0 }
}
