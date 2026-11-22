//! Data extents for every series kind (the automatic view and caching use them).

use super::axis::{Bounds, Scale};
use super::series::{Kind, Series};

impl Series<'_> {
    /// Data extent in scaled space (log axes ignore values ≤ 0).
    pub(super) fn bounds(&self, x_scale: Scale, y_scale: Scale) -> Bounds {
        match &self.kind {
            Kind::Candles { width } => return self.candle_bounds(x_scale, y_scale, *width),
            Kind::Heatmap => return self.heatmap_bounds(x_scale, y_scale),
            Kind::Waterfall { width } => return self.waterfall_bounds(x_scale, y_scale, *width),
            Kind::Pie { .. } | Kind::Radar | Kind::Gauge { .. } => return Bounds::NOTHING,
            _ => {}
        }
        let mut bounds = Bounds::NOTHING;
        let error = |index: usize| self.y_errors.as_ref().and_then(|errors| errors.get(index)).copied();
        for (index, point) in self.data.iter().enumerate() {
            let x = x_scale.forward(point[0]);
            match error(index) {
                Some(error) => {
                    bounds.extend([x, y_scale.forward(point[1] - error)]);
                    bounds.extend([x, y_scale.forward(point[1] + error)]);
                }
                None => bounds.extend([x, y_scale.forward(point[1])]),
            }
        }
        if !bounds.is_valid() {
            return bounds;
        }
        match self.kind {
            Kind::Bars { base, width } => {
                bounds.extend([bounds.min_x, y_scale.forward(base)]);
                let half = width.unwrap_or_else(|| self.min_spacing() * 0.8) * 0.5;
                if x_scale == Scale::Linear {
                    bounds.min_x -= half;
                    bounds.max_x += half;
                }
            }
            Kind::Line {
                fill_to: Some(base), ..
            } => bounds.extend([bounds.min_x, y_scale.forward(base)]),
            _ => {}
        }
        bounds
    }

    /// The smallest positive x step (1 for a single point).
    pub(super) fn min_spacing(&self) -> f64 {
        smallest_step(self.data.iter().map(|point| point[0]))
    }

    fn candle_bounds(&self, x_scale: Scale, y_scale: Scale, width: Option<f64>) -> Bounds {
        let Some(rows) = &self.ohlc else {
            return Bounds::NOTHING;
        };
        let mut bounds = Bounds::NOTHING;
        for row in rows.iter() {
            let x = x_scale.forward(row[0]);
            for value in &row[1..] {
                bounds.extend([x, y_scale.forward(*value)]);
            }
        }
        if bounds.is_valid() && x_scale == Scale::Linear {
            let half = width.unwrap_or_else(|| self.min_spacing() * 0.65) * 0.5;
            bounds.min_x -= half;
            bounds.max_x += half;
        }
        bounds
    }

    fn heatmap_bounds(&self, x_scale: Scale, y_scale: Scale) -> Bounds {
        let Some(cells) = &self.cells else {
            return Bounds::NOTHING;
        };
        let mut bounds = Bounds::NOTHING;
        for cell in cells.iter() {
            bounds.extend([x_scale.forward(cell[0]), y_scale.forward(cell[1])]);
        }
        if bounds.is_valid() {
            let x_half = smallest_step(cells.iter().map(|cell| cell[0])) * 0.5;
            let y_half = smallest_step(cells.iter().map(|cell| cell[1])) * 0.5;
            bounds.min_x -= x_half;
            bounds.max_x += x_half;
            bounds.min_y -= y_half;
            bounds.max_y += y_half;
        }
        bounds
    }

    fn waterfall_bounds(&self, x_scale: Scale, y_scale: Scale, width: Option<f64>) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        let mut base = 0.0;
        for point in self.data.iter() {
            let next = base + point[1];
            let x = x_scale.forward(point[0]);
            bounds.extend([x, y_scale.forward(base)]);
            bounds.extend([x, y_scale.forward(next)]);
            base = next;
        }
        if bounds.is_valid() && x_scale == Scale::Linear {
            bounds.extend([bounds.min_x, y_scale.forward(0.0)]);
            let half = width.unwrap_or_else(|| self.min_spacing() * 0.65) * 0.5;
            bounds.min_x -= half;
            bounds.max_x += half;
        }
        bounds
    }

    /// Whether x never decreases (enables binary search and decimation).
    pub(super) fn sorted_x(&self) -> bool {
        self.data
            .windows(2)
            .all(|pair| pair[1][0] >= pair[0][0] || pair[1][0].is_nan() || pair[0][0].is_nan())
    }

    /// The OHLC rows, when this is a candlestick series.
    pub(super) fn ohlc(&self) -> Option<&[[f64; 5]]> {
        self.ohlc.as_deref()
    }

    /// The heatmap cells, when this is a heatmap.
    pub(super) fn cells(&self) -> Option<&[[f64; 3]]> {
        self.cells.as_deref()
    }
}

/// The smallest positive step between values (1 when there is none).
fn smallest_step(values: impl Iterator<Item = f64>) -> f64 {
    let mut previous: Option<f64> = None;
    let mut smallest = f64::INFINITY;
    for value in values {
        if !value.is_finite() {
            continue;
        }
        if let Some(previous) = previous {
            let delta = (value - previous).abs();
            if delta > 0.0 {
                smallest = smallest.min(delta);
            }
        }
        previous = Some(value);
    }
    if smallest.is_finite() { smallest } else { 1.0 }
}
