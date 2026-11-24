#![allow(clippy::needless_range_loop)]

//! Signal analysis helpers used by the plot UI and by applications directly.
//!
//! Every function is a pure transform over slices: peak detection, baseline removal,
//! Savitzky–Golay smoothing/derivatives, integration, an FFT spectrum, resampling, a
//! Gauss–Newton fit with confidence bands, and selection statistics.

use std::ops::Range;

mod fit;

/// One detected peak: index and height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Peak {
    /// Sample index.
    pub index: usize,
    /// Sample value.
    pub value: f64,
}

/// Detect local maxima above `min_height`, at least `min_distance` samples apart.
pub fn find_peaks(values: &[f64], min_height: f64, min_distance: usize) -> Vec<Peak> {
    let distance = min_distance.max(1);
    let mut peaks: Vec<Peak> = Vec::new();
    for index in 1..values.len().saturating_sub(1) {
        let value = values[index];
        if !value.is_finite() || value < min_height {
            continue;
        }
        if value < values[index - 1] || value < values[index + 1] {
            continue;
        }
        if let Some(last) = peaks.last()
            && index - last.index < distance
        {
            if value > last.value {
                peaks.pop();
            } else {
                continue;
            }
        }
        peaks.push(Peak { index, value });
    }
    peaks
}

/// A rolling-minimum baseline. `window` is clamped to an odd number of samples.
pub fn rolling_baseline(values: &[f64], window: usize) -> Vec<f64> {
    let window = window.max(1) | 1;
    let half = window / 2;
    (0..values.len())
        .map(|index| {
            let start = index.saturating_sub(half);
            let end = (index + half + 1).min(values.len());
            values[start..end]
                .iter()
                .copied()
                .filter(|value| value.is_finite())
                .fold(f64::INFINITY, f64::min)
        })
        .collect()
}

/// Subtract a rolling baseline in place.
pub fn remove_baseline(values: &mut [f64], window: usize) {
    let baseline = rolling_baseline(values, window);
    for (value, base) in values.iter_mut().zip(baseline) {
        *value -= base;
    }
}

/// Savitzky–Golay smoothing coefficients for the center of an odd window.
fn savgol_coefficients(window: usize, order: usize, derivative: usize) -> Option<Vec<f64>> {
    let window = window.max(1) | 1;
    let order = order.min(window - 1);
    let half = window / 2;
    let size = order + 1;
    if derivative > order {
        return None;
    }
    let mut matrix = vec![vec![0.0; size + 1]; size];
    for row in 0..size {
        for column in 0..size {
            let power = row + column;
            matrix[row][column] = (-(half as i64)..=(half as i64))
                .map(|offset| (offset as f64).powi(power as i32))
                .sum();
        }
    }
    matrix[derivative][size] = factorial(derivative) as f64;
    for pivot in 0..size {
        let mut best = pivot;
        for row in pivot + 1..size {
            if matrix[row][pivot].abs() > matrix[best][pivot].abs() {
                best = row;
            }
        }
        if matrix[best][pivot].abs() < 1e-12 {
            return None;
        }
        matrix.swap(pivot, best);
        let divisor = matrix[pivot][pivot];
        for column in pivot..=size {
            matrix[pivot][column] /= divisor;
        }
        for row in 0..size {
            if row != pivot {
                let factor = matrix[row][pivot];
                for column in pivot..=size {
                    matrix[row][column] -= factor * matrix[pivot][column];
                }
            }
        }
    }
    let coefficients: Vec<f64> = (0..window)
        .map(|index| {
            let offset = index as i64 - half as i64;
            (0..size)
                .map(|power| matrix[power][size] * (offset as f64).powi(power as i32))
                .sum()
        })
        .collect();
    Some(coefficients)
}

/// Savitzky–Golay smoothing. Returns the same length; the edges are clamped.
pub fn savgol(values: &[f64], window: usize, order: usize) -> Vec<f64> {
    let Some(coefficients) = savgol_coefficients(window, order, 0) else {
        return values.to_vec();
    };
    let half = coefficients.len() / 2;
    (0..values.len())
        .map(|index| {
            coefficients
                .iter()
                .enumerate()
                .map(|(offset, weight)| {
                    let source = (index as i64 + offset as i64 - half as i64)
                        .clamp(0, values.len().saturating_sub(1) as i64) as usize;
                    values[source] * weight
                })
                .sum()
        })
        .collect()
}

/// Savitzky–Golay first derivative, per sample step.
pub fn savgol_derivative(values: &[f64], window: usize, order: usize, step: f64) -> Vec<f64> {
    let step = if step.abs() > f64::EPSILON { step } else { 1.0 };
    let Some(coefficients) = savgol_coefficients(window, order, 1) else {
        return vec![0.0; values.len()];
    };
    let half = coefficients.len() / 2;
    (0..values.len())
        .map(|index| {
            coefficients
                .iter()
                .enumerate()
                .map(|(offset, weight)| {
                    let source = (index as i64 + offset as i64 - half as i64)
                        .clamp(0, values.len().saturating_sub(1) as i64) as usize;
                    values[source] * weight
                })
                .sum::<f64>()
                / step
        })
        .collect()
}

/// Central-difference derivative; the ends use one-sided differences.
pub fn derivative(values: &[f64], step: f64) -> Vec<f64> {
    let step = if step.abs() > f64::EPSILON { step } else { 1.0 };
    (0..values.len())
        .map(|index| {
            if values.len() < 2 {
                0.0
            } else if index == 0 {
                (values[1] - values[0]) / step
            } else if index + 1 == values.len() {
                (values[index] - values[index - 1]) / step
            } else {
                (values[index + 1] - values[index - 1]) / (2.0 * step)
            }
        })
        .collect()
}

/// Trapezoidal integral, with the running area at every sample.
pub fn integral(values: &[f64], step: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(values.len());
    let mut total = 0.0;
    out.push(0.0);
    for window in values.windows(2) {
        total += (window[0] + window[1]) * 0.5 * step;
        out.push(total);
    }
    out
}

/// A complex sample for the FFT.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    /// Real part.
    pub re: f64,
    /// Imaginary part.
    pub im: f64,
}

/// In-place radix-2 FFT. The length must be a power of two.
pub fn fft(values: &mut [Complex]) {
    let length = values.len();
    if length <= 1 || !length.is_power_of_two() {
        return;
    }
    let mut j = 0;
    for i in 1..length {
        let mut bit = length >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            values.swap(i, j);
        }
    }
    let mut size = 2;
    while size <= length {
        let angle = -std::f64::consts::TAU / size as f64;
        let (sin, cos) = angle.sin_cos();
        for start in (0..length).step_by(size) {
            let mut w = Complex { re: 1.0, im: 0.0 };
            for offset in 0..size / 2 {
                let even = values[start + offset];
                let odd = values[start + offset + size / 2];
                let product = Complex {
                    re: odd.re * w.re - odd.im * w.im,
                    im: odd.re * w.im + odd.im * w.re,
                };
                values[start + offset] = Complex {
                    re: even.re + product.re,
                    im: even.im + product.im,
                };
                values[start + offset + size / 2] = Complex {
                    re: even.re - product.re,
                    im: even.im - product.im,
                };
                w = Complex {
                    re: w.re * cos - w.im * sin,
                    im: w.re * sin + w.im * cos,
                };
            }
        }
        size <<= 1;
    }
}

/// A one-sided magnitude spectrum: `[frequency, magnitude]` up to Nyquist.
pub fn spectrum(values: &[f64], sample_rate: f64) -> Vec<[f64; 2]> {
    let mut length = 1;
    while length * 2 <= values.len() {
        length *= 2;
    }
    if length < 2 {
        return Vec::new();
    }
    let mut buffer: Vec<Complex> = values[..length]
        .iter()
        .map(|value| Complex { re: *value, im: 0.0 })
        .collect();
    fft(&mut buffer);
    (0..=length / 2)
        .map(|index| {
            let magnitude = buffer[index].re.hypot(buffer[index].im) / length as f64;
            [index as f64 * sample_rate / length as f64, magnitude]
        })
        .collect()
}

/// Linear resampling to `len` samples.
pub fn resample(values: &[f64], len: usize) -> Vec<f64> {
    if len == 0 || values.is_empty() {
        return Vec::new();
    }
    if len == 1 {
        return vec![values[0]];
    }
    (0..len)
        .map(|index| {
            let position = index as f64 * (values.len() - 1) as f64 / (len - 1) as f64;
            let low = position.floor() as usize;
            let high = (low + 1).min(values.len() - 1);
            let fraction = position - low as f64;
            values[low] * (1.0 - fraction) + values[high] * fraction
        })
        .collect()
}

pub use fit::{Model, ModelFit, fit_model};

/// Summary statistics of a selection.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Statistics {
    /// Number of finite values.
    pub count: usize,
    /// Minimum.
    pub min: f64,
    /// Maximum.
    pub max: f64,
    /// Mean.
    pub mean: f64,
    /// Population standard deviation.
    pub stddev: f64,
    /// Median.
    pub median: f64,
}

impl Statistics {
    /// Statistics of a slice, ignoring non-finite values.
    pub fn of(values: &[f64]) -> Self {
        let mut finite: Vec<f64> = values.iter().copied().filter(|value| value.is_finite()).collect();
        if finite.is_empty() {
            return Self::default();
        }
        finite.sort_by(f64::total_cmp);
        let count = finite.len();
        let mean = finite.iter().sum::<f64>() / count as f64;
        let variance = finite.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / count as f64;
        let median = if count.is_multiple_of(2) {
            (finite[count / 2 - 1] + finite[count / 2]) * 0.5
        } else {
            finite[count / 2]
        };
        Self {
            count,
            min: finite[0],
            max: finite[count - 1],
            mean,
            stddev: variance.sqrt(),
            median,
        }
    }

    /// Statistics of a slice of samples in `range`.
    pub fn range(values: &[f64], range: Range<usize>) -> Self {
        let start = range.start.min(values.len());
        let end = range.end.min(values.len()).max(start);
        Self::of(&values[start..end])
    }
}

fn factorial(value: usize) -> u64 {
    (1..=value as u64).product::<u64>().max(1)
}

#[cfg(test)]
mod tests;
