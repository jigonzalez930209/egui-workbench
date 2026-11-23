//! Bounds, axis scales, the data ↔ screen transform, and tick generation.

pub(super) use super::transform::{SecondAxis, Transform};

/// An axis-aligned range in data space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    /// Smallest x.
    pub min_x: f64,
    /// Largest x.
    pub max_x: f64,
    /// Smallest y.
    pub min_y: f64,
    /// Largest y.
    pub max_y: f64,
}

impl Bounds {
    /// Bounds from two corners in any order.
    pub fn from_corners(a: [f64; 2], b: [f64; 2]) -> Self {
        Self {
            min_x: a[0].min(b[0]),
            max_x: a[0].max(b[0]),
            min_y: a[1].min(b[1]),
            max_y: a[1].max(b[1]),
        }
    }

    pub(super) const NOTHING: Self = Self {
        min_x: f64::INFINITY,
        max_x: f64::NEG_INFINITY,
        min_y: f64::INFINITY,
        max_y: f64::NEG_INFINITY,
    };

    pub(super) fn is_valid(&self) -> bool {
        self.min_x.is_finite()
            && self.max_x.is_finite()
            && self.min_y.is_finite()
            && self.max_y.is_finite()
            && self.min_x <= self.max_x
            && self.min_y <= self.max_y
    }

    pub(super) fn extend(&mut self, point: [f64; 2]) {
        if point[0].is_finite() && point[1].is_finite() {
            self.min_x = self.min_x.min(point[0]);
            self.max_x = self.max_x.max(point[0]);
            self.min_y = self.min_y.min(point[1]);
            self.max_y = self.max_y.max(point[1]);
        }
    }

    pub(super) fn union(&mut self, other: &Self) {
        if other.is_valid() {
            self.extend([other.min_x, other.min_y]);
            self.extend([other.max_x, other.max_y]);
        }
    }

    /// Width in data units.
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Height in data units.
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Whether `point` lies inside (edges included).
    pub fn contains(&self, point: [f64; 2]) -> bool {
        (self.min_x..=self.max_x).contains(&point[0]) && (self.min_y..=self.max_y).contains(&point[1])
    }
}

/// How values map onto an axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Scale {
    /// Evenly spaced values.
    #[default]
    Linear,
    /// Decades evenly spaced; values ≤ 0 are not drawn.
    Log10,
}

impl Scale {
    pub(super) fn forward(self, value: f64) -> f64 {
        match self {
            Self::Linear => value,
            Self::Log10 => {
                if value > 0.0 {
                    value.log10()
                } else {
                    f64::NAN
                }
            }
        }
    }

    pub(super) fn inverse(self, value: f64) -> f64 {
        match self {
            Self::Linear => value,
            Self::Log10 => 10f64.powf(value),
        }
    }
}

/// One tick: its position in scaled space, its label, and whether it is a major tick.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Tick {
    pub value: f64,
    pub label: String,
    pub major: bool,
}

/// Overrides for tick placement, from [`super::AxisConfig`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct TickRules {
    /// Fixed major step (decades on log axes).
    pub step: Option<f64>,
    pub minor: super::config::Minor,
    pub decimals: Option<u8>,
}

impl TickRules {
    pub fn of(axis: &super::config::AxisConfig) -> Self {
        Self {
            step: axis.major_step,
            minor: axis.minor,
            decimals: axis.decimals,
        }
    }
}

/// "Nice" ticks (1, 2, 5 × 10ⁿ) in `[min, max]` (scaled space), about `pixels / spacing`
/// of them, with labels formatted to the step's precision, plus minor ticks.
pub(super) fn ticks(scale: Scale, min: f64, max: f64, pixels: f32, spacing: f32, rules: TickRules) -> Vec<Tick> {
    if max <= min || !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }
    let target = f64::from((pixels / spacing).max(2.0));
    match scale {
        Scale::Linear => linear_ticks(min, max, target, pixels, rules),
        Scale::Log10 => log_ticks(min, max, target, rules),
    }
}

/// The nice step for a span split into about `target` intervals.
pub(super) fn nice_step(span: f64, target: f64) -> f64 {
    let raw = span / target.max(1.0);
    let magnitude = 10f64.powf(raw.log10().floor());
    let residual = raw / magnitude;
    let nice = if residual <= 1.0 {
        1.0
    } else if residual <= 2.0 {
        2.0
    } else if residual <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

fn linear_ticks(min: f64, max: f64, target: f64, pixels: f32, rules: TickRules) -> Vec<Tick> {
    let auto = nice_step(max - min, target);
    // A fixed step that would draw thousands of ticks falls back to the automatic one.
    let step = rules.step.filter(|step| (max - min) / step <= 500.0).unwrap_or(auto);
    let decimals = rules
        .decimals
        .map_or_else(|| (-step.log10().floor()).max(0.0) as usize, usize::from);
    let first = (min / step).ceil() as i64;
    let last = (max / step).floor() as i64;
    if last - first > 1000 {
        return Vec::new();
    }
    let mut ticks: Vec<Tick> = (first..=last)
        .map(|index| {
            let value = index as f64 * step;
            Tick {
                value,
                label: format_value(value, decimals, step),
                major: true,
            }
        })
        .collect();
    let per_major = (step / (max - min)) as f32 * pixels;
    let divisions = match rules.minor {
        super::config::Minor::Off => 0,
        super::config::Minor::Divisions(divisions) => u32::from(divisions),
        super::config::Minor::Auto => {
            // 4 subdivisions for 2-steps, 5 otherwise; fewer when they would crowd.
            let digit = (step / 10f64.powf(step.log10().floor())).round();
            let preferred = if digit == 2.0 { 4 } else { 5 };
            [preferred, 2]
                .into_iter()
                .find(|divisions| per_major / *divisions as f32 >= 8.0)
                .unwrap_or(0)
        }
    };
    if divisions > 1 && (last - first + 2) * i64::from(divisions) <= 4000 {
        for index in first - 1..=last {
            for part in 1..divisions {
                let value = (index as f64 + f64::from(part) / f64::from(divisions)) * step;
                if (min..=max).contains(&value) {
                    ticks.push(Tick {
                        value,
                        label: String::new(),
                        major: false,
                    });
                }
            }
        }
    }
    ticks
}

fn log_ticks(min: f64, max: f64, target: f64, rules: TickRules) -> Vec<Tick> {
    // Decades are majors (every n-th labeled when crowded; the others keep their grid
    // line); 2…9 are minors when there is room for them.
    let first = min.floor() as i64;
    let last = max.ceil() as i64;
    if last - first > 300 {
        return Vec::new();
    }
    let decades = (last - first).max(1) as f64;
    let every = rules.step.map_or_else(
        || (decades / target).ceil().max(1.0) as i64,
        |step| step.round().max(1.0) as i64,
    );
    let minors = match rules.minor {
        super::config::Minor::Off => false,
        super::config::Minor::Divisions(_) => true,
        super::config::Minor::Auto => decades * 3.0 <= target,
    };
    // One notation for the whole axis: plain numbers only when every label is short.
    let plain = min.ceil() >= -3.0 && max.floor() <= 4.0;
    let mut ticks = Vec::new();
    for exponent in first..=last {
        let value = exponent as f64;
        if (min..=max).contains(&value) {
            let labeled = exponent.rem_euclid(every) == 0;
            ticks.push(Tick {
                value,
                label: if labeled {
                    format_power(exponent, plain)
                } else {
                    String::new()
                },
                major: labeled,
            });
        }
        if minors {
            for factor in 2..=9 {
                let value = exponent as f64 + f64::from(factor).log10();
                if (min..=max).contains(&value) {
                    ticks.push(Tick {
                        value,
                        label: String::new(),
                        major: false,
                    });
                }
            }
        }
    }
    ticks
}

fn format_power(exponent: i64, plain: bool) -> String {
    if !plain {
        return format!("1e{exponent}");
    }
    if exponent < 0 {
        format!("{:.*}", exponent.unsigned_abs() as usize, 10f64.powi(exponent as i32))
    } else {
        10i64.pow(exponent as u32).to_string()
    }
}

/// A value at the precision of `step`, switching to scientific notation for very large
/// or very small magnitudes. `-0` prints as `0`.
pub(super) fn format_value(value: f64, decimals: usize, step: f64) -> String {
    if value.abs() < step * 1e-9 {
        // Same precision as its neighbors: "0.0" next to "0.2", never "-0".
        return format!("{:.decimals$}", 0.0);
    }
    let magnitude = value.abs();
    if magnitude >= 1e6 || (magnitude < 1e-4 && magnitude > 0.0) {
        let digits = (step.abs().log10().floor() - magnitude.log10().floor()).abs().min(6.0) as usize;
        return format!("{value:.digits$e}");
    }
    format!("{value:.decimals$}")
}

/// A readable value for tooltips: about 4 significant digits.
pub(super) fn format_readout(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let magnitude = value.abs();
    if magnitude != 0.0 && !(1e-3..1e6).contains(&magnitude) {
        return format!("{value:.3e}");
    }
    let decimals = if magnitude == 0.0 {
        0
    } else {
        (3 - magnitude.log10().floor() as i32).clamp(0, 6) as usize
    };
    let text = format!("{value:.decimals$}");
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Rect, pos2};

    #[test]
    fn nice_steps_are_one_two_five() {
        assert_eq!(nice_step(10.0, 10.0), 1.0);
        assert_eq!(nice_step(10.0, 4.0), 5.0);
        assert!((nice_step(0.9, 5.0) - 0.2).abs() < 1e-12);
        assert_eq!(nice_step(1000.0, 5.0), 200.0);
    }

    #[test]
    fn linear_ticks_cover_the_range_with_clean_labels() {
        let ticks = ticks(Scale::Linear, -0.05, 1.05, 500.0, 50.0, TickRules::default());
        let labels: Vec<_> = ticks
            .iter()
            .filter(|tick| tick.major)
            .map(|tick| tick.label.as_str())
            .collect();
        assert_eq!(labels, ["0.0", "0.2", "0.4", "0.6", "0.8", "1.0"]);
    }

    #[test]
    fn log_ticks_are_decades() {
        let ticks = ticks(Scale::Log10, 0.0, 3.0, 300.0, 40.0, TickRules::default());
        let majors: Vec<_> = ticks
            .iter()
            .filter(|tick| tick.major)
            .map(|tick| tick.label.clone())
            .collect();
        assert_eq!(majors, ["1", "10", "100", "1000"]);
        let wide = super::ticks(Scale::Log10, 0.0, 6.0, 120.0, 40.0, TickRules::default());
        let labels: Vec<_> = wide
            .iter()
            .filter(|tick| tick.major)
            .map(|tick| tick.label.as_str())
            .collect();
        assert_eq!(labels, ["1e0", "1e2", "1e4", "1e6"], "one notation per axis");
        assert_eq!(
            wide.iter().filter(|tick| tick.value.fract() == 0.0).count(),
            7,
            "every decade keeps a grid line"
        );
    }

    #[test]
    fn fixed_steps_minor_divisions_and_decimals() {
        let rules = TickRules {
            step: Some(0.25),
            minor: super::super::config::Minor::Divisions(5),
            decimals: Some(3),
        };
        let ticks = ticks(Scale::Linear, 0.0, 1.0, 400.0, 50.0, rules);
        let majors: Vec<_> = ticks
            .iter()
            .filter(|tick| tick.major)
            .map(|tick| tick.label.as_str())
            .collect();
        assert_eq!(majors, ["0.000", "0.250", "0.500", "0.750", "1.000"]);
        assert_eq!(
            ticks.iter().filter(|tick| !tick.major).count(),
            16,
            "4 minors in each of 4 intervals"
        );
        let off = super::ticks(
            Scale::Linear,
            0.0,
            1.0,
            400.0,
            50.0,
            TickRules {
                minor: super::super::config::Minor::Off,
                ..TickRules::default()
            },
        );
        assert!(off.iter().all(|tick| tick.major));
    }

    #[test]
    fn transform_round_trips() {
        let transform = Transform {
            frame: Rect::from_min_max(pos2(10.0, 10.0), pos2(410.0, 210.0)),
            view: Bounds::from_corners([0.0, -1.0], [4.0, 1.0]),
            x_scale: Scale::Linear,
            y_scale: Scale::Linear,
            y2: Some(SecondAxis {
                min: 0.0,
                max: 100.0,
                scale: Scale::Linear,
            }),
        };
        let right = transform.for_axis(super::super::series::YAxis::Right);
        assert_eq!(right.to_screen([1.0, 75.0]), pos2(110.0, 60.0));
        let screen = transform.to_screen([1.0, 0.5]);
        assert_eq!(screen, pos2(110.0, 60.0));
        let back = transform.to_data(screen);
        assert!((back[0] - 1.0).abs() < 1e-6 && (back[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn readouts_are_short() {
        assert_eq!(format_readout(5.432_109), "5.432");
        assert_eq!(format_readout(1234.5678), "1235");
        assert_eq!(format_readout(0.0), "0");
        assert_eq!(format_readout(2.5e-7), "2.500e-7");
    }
}
