//! Bounds, axis scales, the data ↔ screen transform, and tick generation.

pub(super) use super::transform::{SecondAxis, Transform};

/// An axis-aligned range in data space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scale {
    /// Evenly spaced values.
    #[default]
    Linear,
    /// Decades evenly spaced; values ≤ 0 are not drawn.
    Log10,
    /// A signed log scale: zero is the origin and both signs expand logarithmically.
    SymmetricLog,
    /// Seconds since the Unix epoch, formatted as calendar tick labels in UTC.
    Time,
    /// Evenly spaced category indices, labeled from [`super::AxisConfig::categories`].
    Category,
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
            Self::SymmetricLog => value.signum() * (1.0 + value.abs()).log10(),
            Self::Time | Self::Category => value,
        }
    }

    pub(super) fn inverse(self, value: f64) -> f64 {
        match self {
            Self::Linear | Self::Time | Self::Category => value,
            Self::Log10 => 10f64.powf(value),
            Self::SymmetricLog => value.signum() * (10f64.powf(value.abs()) - 1.0),
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
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct TickRules<'a> {
    /// Fixed major step (decades on log axes).
    pub step: Option<f64>,
    pub minor: super::config::Minor,
    pub decimals: Option<u8>,
    /// Category labels.
    pub categories: Option<&'a [String]>,
}

impl<'a> TickRules<'a> {
    pub fn of(axis: &'a super::config::AxisConfig) -> Self {
        Self {
            step: axis.major_step,
            minor: axis.minor,
            decimals: axis.decimals,
            categories: axis.categories.as_deref(),
        }
    }
}

/// "Nice" ticks (1, 2, 5 × 10ⁿ) in `[min, max]` (scaled space), about `pixels / spacing`
/// of them, with labels formatted to the step's precision, plus minor ticks.
pub(super) fn ticks<'a>(
    scale: Scale,
    min: f64,
    max: f64,
    pixels: f32,
    spacing: f32,
    rules: TickRules<'a>,
) -> Vec<Tick> {
    if max <= min || !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }
    let target = f64::from((pixels / spacing).max(2.0));
    match scale {
        Scale::Linear => linear_ticks(min, max, target, pixels, rules),
        Scale::Log10 => log_ticks(min, max, target, rules),
        Scale::SymmetricLog => symmetric_log_ticks(min, max, target, pixels, rules),
        Scale::Time => time_ticks(min, max, target, rules),
        Scale::Category => category_ticks(min, max, rules),
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

fn symmetric_log_ticks(min: f64, max: f64, target: f64, pixels: f32, rules: TickRules<'_>) -> Vec<Tick> {
    let decimals = rules.decimals.unwrap_or(3) as usize;
    let mut ticks = linear_ticks(min, max, target, pixels, rules);
    for tick in &mut ticks {
        let value = Scale::SymmetricLog.inverse(tick.value);
        tick.label = format_number(value, decimals);
    }
    ticks
}

fn time_ticks(min: f64, max: f64, target: f64, rules: TickRules<'_>) -> Vec<Tick> {
    let span = max - min;
    let step = rules
        .step
        .filter(|step| *step > 0.0 && span / *step <= 500.0)
        .unwrap_or_else(|| nice_time_step(span, target));
    let first = (min / step).ceil() as i64;
    let last = (max / step).floor() as i64;
    if last - first > 1000 {
        return Vec::new();
    }
    let mut ticks = Vec::new();
    for index in first..=last {
        let value = index as f64 * step;
        ticks.push(Tick {
            value,
            label: format_timestamp(value, span),
            major: true,
        });
    }
    ticks
}

fn nice_time_step(span: f64, target: f64) -> f64 {
    const STEPS: [f64; 19] = [
        1.0,
        2.0,
        5.0,
        10.0,
        15.0,
        30.0,
        60.0,
        300.0,
        900.0,
        1800.0,
        3600.0,
        10_800.0,
        21_600.0,
        43_200.0,
        86_400.0,
        172_800.0,
        604_800.0,
        2_592_000.0,
        31_536_000.0,
    ];
    let ideal = span / target.max(1.0);
    STEPS
        .iter()
        .copied()
        .find(|step| *step >= ideal)
        .unwrap_or(31_536_000.0)
}

fn format_timestamp(seconds: f64, span: f64) -> String {
    let Some(time) = jiff::Timestamp::from_second(seconds as i64).ok() else {
        return format_number(seconds, 0);
    };
    let format = if span >= 2.0 * 86_400.0 {
        "%Y-%m-%d"
    } else if span >= 86_400.0 {
        "%m-%d %H:%M"
    } else if span >= 3_600.0 {
        "%H:%M"
    } else {
        "%H:%M:%S"
    };
    time.strftime(format).to_string()
}

fn format_number(value: f64, decimals: usize) -> String {
    if decimals == 0 {
        format!("{value:.0}")
    } else {
        format!("{value:.decimals$}")
    }
}

fn category_ticks(min: f64, max: f64, rules: TickRules<'_>) -> Vec<Tick> {
    let first = min.floor().max(0.0) as usize;
    let last = max.ceil().max(0.0) as usize;
    if last.saturating_sub(first) > 500 {
        return Vec::new();
    }
    (first..=last)
        .map(|index| Tick {
            value: index as f64,
            label: rules
                .categories
                .and_then(|categories| categories.get(index))
                .cloned()
                .unwrap_or_else(|| index.to_string()),
            major: true,
        })
        .collect()
}

fn linear_ticks(min: f64, max: f64, target: f64, pixels: f32, rules: TickRules<'_>) -> Vec<Tick> {
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
mod tests;
