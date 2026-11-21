//! Least-squares fits and smoothing for plot series.

use super::axis::format_readout;
use super::series::{LineStyle, Series};

/// The model to fit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FitKind {
    /// `y = a + b·x`.
    Linear,
    /// `y = c0 + c1·x + … + cn·x^n` (degree 1–8).
    Polynomial(u8),
    /// `y = a·e^(b·x)`, fitted on `ln y` (points with `y ≤ 0` are skipped).
    Exponential,
    /// `y = a·x^b`, fitted on `ln x`, `ln y` (points with `x ≤ 0` or `y ≤ 0` skipped).
    Power,
    /// `y = a + b·ln x` (points with `x ≤ 0` skipped).
    Logarithmic,
    /// Centered moving average over `n` points (smoothing, not a model).
    MovingAverage(u16),
}

/// A fitted model: its coefficients, its goodness of fit, and an evaluator.
#[derive(Clone, Debug, PartialEq)]
pub struct Fit {
    /// The model.
    pub kind: FitKind,
    /// Coefficients: `[a, b]` for two-parameter models, `c₀…cₙ` for polynomials.
    pub coefficients: Vec<f64>,
    /// Coefficient of determination on the original data (1 = perfect).
    pub r_squared: f64,
    /// Smallest and largest x of the fitted points.
    pub x_range: (f64, f64),
    smoothed: Vec<[f64; 2]>,
}

impl Fit {
    /// Fit `kind` to `data`. Returns `None` for too few usable points or a singular
    /// system.
    pub fn new(kind: FitKind, data: &[[f64; 2]]) -> Option<Self> {
        let finite: Vec<[f64; 2]> = data
            .iter()
            .copied()
            .filter(|point| point[0].is_finite() && point[1].is_finite())
            .collect();
        let x_range = finite.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |range, point| {
            (range.0.min(point[0]), range.1.max(point[0]))
        });
        let (coefficients, smoothed) = match kind {
            FitKind::Linear => (polynomial(&finite, 1)?, Vec::new()),
            FitKind::Polynomial(degree) => (polynomial(&finite, usize::from(degree.clamp(1, 8)))?, Vec::new()),
            FitKind::Exponential => {
                let logged: Vec<[f64; 2]> = finite
                    .iter()
                    .filter(|p| p[1] > 0.0)
                    .map(|p| [p[0], p[1].ln()])
                    .collect();
                let line = polynomial(&logged, 1)?;
                (vec![line[0].exp(), line[1]], Vec::new())
            }
            FitKind::Power => {
                let logged: Vec<[f64; 2]> = finite
                    .iter()
                    .filter(|p| p[0] > 0.0 && p[1] > 0.0)
                    .map(|p| [p[0].ln(), p[1].ln()])
                    .collect();
                let line = polynomial(&logged, 1)?;
                (vec![line[0].exp(), line[1]], Vec::new())
            }
            FitKind::Logarithmic => {
                let logged: Vec<[f64; 2]> = finite
                    .iter()
                    .filter(|p| p[0] > 0.0)
                    .map(|p| [p[0].ln(), p[1]])
                    .collect();
                (polynomial(&logged, 1)?, Vec::new())
            }
            FitKind::MovingAverage(window) => (Vec::new(), moving_average(&finite, usize::from(window.max(1)))),
        };
        let mut fit = Self {
            kind,
            coefficients,
            r_squared: f64::NAN,
            x_range,
            smoothed,
        };
        if !matches!(kind, FitKind::MovingAverage(_)) {
            fit.r_squared = r_squared(&finite, |x| fit.eval(x));
        }
        Some(fit)
    }

    /// The model value at `x` (`NaN` outside the model's domain).
    pub fn eval(&self, x: f64) -> f64 {
        let c = &self.coefficients;
        match self.kind {
            FitKind::Linear | FitKind::Polynomial(_) => {
                c.iter().rev().fold(0.0, |acc, coefficient| acc * x + coefficient)
            }
            FitKind::Exponential => c[0] * (c[1] * x).exp(),
            FitKind::Power => {
                if x > 0.0 {
                    c[0] * x.powf(c[1])
                } else {
                    f64::NAN
                }
            }
            FitKind::Logarithmic => {
                if x > 0.0 {
                    c[0] + c[1] * x.ln()
                } else {
                    f64::NAN
                }
            }
            FitKind::MovingAverage(_) => {
                let index = self.smoothed.partition_point(|point| point[0] < x);
                self.smoothed.get(index).map_or(f64::NAN, |point| point[1])
            }
        }
    }

    /// A short equation such as `y = 2.01x + 0.498`, with `R² = 0.998` when defined.
    pub fn equation(&self) -> String {
        let c = &self.coefficients;
        let model = match self.kind {
            FitKind::Linear | FitKind::Polynomial(_) => {
                let mut terms = Vec::new();
                for (power, coefficient) in c.iter().enumerate().rev() {
                    if *coefficient == 0.0 && c.len() > 1 {
                        continue;
                    }
                    let magnitude = format_readout(coefficient.abs());
                    let variable = match power {
                        0 => String::new(),
                        1 => "x".to_owned(),
                        _ => format!("x{}", superscript(power)),
                    };
                    let sign = if *coefficient < 0.0 { "-" } else { "+" };
                    if terms.is_empty() {
                        let lead = if *coefficient < 0.0 { "-" } else { "" };
                        terms.push(format!("{lead}{magnitude}{variable}"));
                    } else {
                        terms.push(format!("{sign} {magnitude}{variable}"));
                    }
                }
                format!("y = {}", terms.join(" "))
            }
            FitKind::Exponential => format!("y = {}·e^({}x)", format_readout(c[0]), format_readout(c[1])),
            FitKind::Power => format!("y = {}·x^{}", format_readout(c[0]), format_readout(c[1])),
            FitKind::Logarithmic => format!("y = {} + {}·ln x", format_readout(c[0]), format_readout(c[1])),
            FitKind::MovingAverage(window) => return format!("moving average ({window})"),
        };
        if self.r_squared.is_finite() {
            format!("{model}   R² = {:.4}", self.r_squared)
        } else {
            model
        }
    }

    /// The fitted curve as a dashed line over the data's x range (`samples` points), or
    /// the smoothed points for a moving average.
    pub fn series(&self, name: impl Into<String>, samples: usize) -> Series<'static> {
        let series = if let FitKind::MovingAverage(_) = self.kind {
            Series::line(name, self.smoothed.clone())
        } else {
            let (start, end) = self.x_range;
            Series::function(name, start..=end, samples, |x| self.eval(x))
        };
        series.style(LineStyle::Dashed).width(1.5)
    }
}

/// Superscripts from Latin-1 only (² and ³), so the default fonts can draw them.
fn superscript(power: usize) -> String {
    match power {
        2 => "²".to_owned(),
        3 => "³".to_owned(),
        _ => format!("^{power}"),
    }
}

/// Least-squares polynomial coefficients `c₀…c_degree` through the normal equations,
/// with x centered and scaled for conditioning.
fn polynomial(data: &[[f64; 2]], degree: usize) -> Option<Vec<f64>> {
    if data.len() <= degree {
        return None;
    }
    let n = data.len() as f64;
    let mean = data.iter().map(|p| p[0]).sum::<f64>() / n;
    let scale = data
        .iter()
        .map(|p| (p[0] - mean).abs())
        .fold(0.0, f64::max)
        .max(f64::MIN_POSITIVE);
    let size = degree + 1;
    let mut matrix = vec![vec![0.0; size + 1]; size];
    for point in data {
        let t = (point[0] - mean) / scale;
        let mut powers = vec![1.0; 2 * size];
        for index in 1..2 * size {
            powers[index] = powers[index - 1] * t;
        }
        for row in 0..size {
            for column in 0..size {
                matrix[row][column] += powers[row + column];
            }
            matrix[row][size] += powers[row] * point[1];
        }
    }
    let scaled = solve(matrix)?;
    // Expand Σ sᵢ·((x − mean)/scale)ⁱ back into powers of x.
    let mut coefficients = vec![0.0; size];
    for (power, s) in scaled.iter().enumerate() {
        let factor = s / scale.powi(power as i32);
        for (k, coefficient) in coefficients.iter_mut().enumerate().take(power + 1) {
            *coefficient += factor * binomial(power, k) * (-mean).powi((power - k) as i32);
        }
    }
    Some(coefficients)
}

fn binomial(n: usize, k: usize) -> f64 {
    (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
}

/// Gaussian elimination with partial pivoting on an augmented matrix.
fn solve(mut matrix: Vec<Vec<f64>>) -> Option<Vec<f64>> {
    let size = matrix.len();
    for column in 0..size {
        let pivot = (column..size).max_by(|&a, &b| matrix[a][column].abs().total_cmp(&matrix[b][column].abs()))?;
        if matrix[pivot][column].abs() < 1e-12 {
            return None;
        }
        matrix.swap(column, pivot);
        let pivot_row = matrix[column].clone();
        for row in matrix.iter_mut().skip(column + 1) {
            let factor = row[column] / pivot_row[column];
            for (value, pivot) in row.iter_mut().zip(&pivot_row).skip(column) {
                *value -= factor * pivot;
            }
        }
    }
    let mut result = vec![0.0; size];
    for row in (0..size).rev() {
        let sum: f64 = (row + 1..size).map(|k| matrix[row][k] * result[k]).sum();
        result[row] = (matrix[row][size] - sum) / matrix[row][row];
    }
    result.iter().all(|value| value.is_finite()).then_some(result)
}

fn r_squared(data: &[[f64; 2]], model: impl Fn(f64) -> f64) -> f64 {
    let n = data.len() as f64;
    let mean = data.iter().map(|p| p[1]).sum::<f64>() / n;
    let (mut residual, mut total) = (0.0, 0.0);
    for point in data {
        let predicted = model(point[0]);
        if predicted.is_finite() {
            residual += (point[1] - predicted).powi(2);
            total += (point[1] - mean).powi(2);
        }
    }
    if total > 0.0 { 1.0 - residual / total } else { 1.0 }
}

fn moving_average(data: &[[f64; 2]], window: usize) -> Vec<[f64; 2]> {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let half = window / 2;
    (0..sorted.len())
        .map(|index| {
            let start = index.saturating_sub(half);
            let end = (index + window - half).min(sorted.len());
            let slice = &sorted[start..end];
            [
                sorted[index][0],
                slice.iter().map(|p| p[1]).sum::<f64>() / slice.len() as f64,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6 * b.abs().max(1.0)
    }

    #[test]
    fn linear_fit_recovers_slope_and_intercept() {
        let data: Vec<[f64; 2]> = (0..20).map(|i| [f64::from(i), 2.5 * f64::from(i) - 3.0]).collect();
        let fit = Fit::new(FitKind::Linear, &data).unwrap_or_else(unreachable_fit);
        assert!(
            close(fit.coefficients[0], -3.0) && close(fit.coefficients[1], 2.5),
            "{:?}",
            fit.coefficients
        );
        assert!(close(fit.r_squared, 1.0));
        assert_eq!(fit.equation(), "y = 2.5x - 3   R² = 1.0000");
    }

    #[test]
    fn cubic_fit_recovers_coefficients_far_from_the_origin() {
        let f = |x: f64| 0.5 * x * x * x - 2.0 * x + 7.0;
        let data: Vec<[f64; 2]> = (0..40)
            .map(|i| 100.0 + f64::from(i) * 0.25)
            .map(|x| [x, f(x)])
            .collect();
        let fit = Fit::new(FitKind::Polynomial(3), &data).unwrap_or_else(unreachable_fit);
        for x in [100.0, 104.0, 109.75] {
            assert!(close(fit.eval(x), f(x)), "{x}: {} vs {}", fit.eval(x), f(x));
        }
    }

    #[test]
    fn exponential_and_power_fits() {
        let exp: Vec<[f64; 2]> = (0..10)
            .map(|i| [f64::from(i), 3.0 * (0.4 * f64::from(i)).exp()])
            .collect();
        let fit = Fit::new(FitKind::Exponential, &exp).unwrap_or_else(unreachable_fit);
        assert!(close(fit.coefficients[0], 3.0) && close(fit.coefficients[1], 0.4));
        let power: Vec<[f64; 2]> = (1..10).map(|i| [f64::from(i), 2.0 * f64::from(i).powf(1.5)]).collect();
        let fit = Fit::new(FitKind::Power, &power).unwrap_or_else(unreachable_fit);
        assert!(close(fit.coefficients[0], 2.0) && close(fit.coefficients[1], 1.5));
    }

    #[test]
    fn too_few_points_do_not_fit() {
        assert!(Fit::new(FitKind::Polynomial(3), &[[0.0, 1.0], [1.0, 2.0]]).is_none());
    }

    #[test]
    fn moving_average_smooths() {
        let data = [[0.0, 0.0], [1.0, 3.0], [2.0, 0.0], [3.0, 3.0]];
        let fit = Fit::new(FitKind::MovingAverage(2), &data).unwrap_or_else(unreachable_fit);
        assert!(close(fit.eval(1.0), 1.5));
    }

    fn unreachable_fit() -> Fit {
        Fit {
            kind: FitKind::Linear,
            coefficients: vec![f64::NAN; 2],
            r_squared: f64::NAN,
            x_range: (0.0, 0.0),
            smoothed: Vec::new(),
        }
    }
}
