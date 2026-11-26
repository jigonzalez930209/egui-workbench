//! Filters that can be applied from the chart's analysis menu.

use serde::{Deserialize, Serialize};

use super::analysis::savgol;

/// A filter available from the analysis menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FilterKind {
    /// Savitzky-Golay polynomial smoothing.
    SavitzkyGolay,
    /// A centered moving average.
    MovingAverage,
    /// A Gaussian-weighted moving average.
    Gaussian,
    /// A centered median filter for impulsive noise.
    Median,
    /// A first-order exponential low-pass filter.
    LowPass,
}

impl FilterKind {
    /// Every filter, in menu order.
    pub const ALL: [Self; 5] = [
        Self::SavitzkyGolay,
        Self::MovingAverage,
        Self::Gaussian,
        Self::Median,
        Self::LowPass,
    ];

    /// The menu label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::SavitzkyGolay => "Savitzky–Golay",
            Self::MovingAverage => "Moving average",
            Self::Gaussian => "Gaussian",
            Self::Median => "Median",
            Self::LowPass => "Low-pass (exponential)",
        }
    }
}

/// Parameters shared by the filters.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FilterSettings {
    /// Savitzky-Golay window, forced odd and at least 3.
    pub savgol_window: usize,
    /// Savitzky-Golay polynomial order.
    pub savgol_order: usize,
    /// Moving-average window.
    pub moving_window: usize,
    /// Gaussian standard deviation in samples.
    pub gaussian_sigma: f64,
    /// Median window.
    pub median_window: usize,
    /// Exponential low-pass coefficient in `(0, 1]`.
    pub lowpass_alpha: f64,
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self {
            savgol_window: 11,
            savgol_order: 3,
            moving_window: 7,
            gaussian_sigma: 2.0,
            median_window: 5,
            lowpass_alpha: 0.25,
        }
    }
}

/// Apply `filters` to the y values, preserving x.
pub fn apply_filters(data: &[[f64; 2]], filters: &[FilterKind], settings: &FilterSettings) -> Vec<[f64; 2]> {
    if filters.is_empty() {
        return data.to_vec();
    }
    let mut values = data.iter().map(|point| point[1]).collect::<Vec<_>>();
    for filter in filters {
        values = apply(*filter, &values, settings);
    }
    data.iter()
        .zip(values)
        .map(|(point, value)| [point[0], value])
        .collect()
}

fn apply(filter: FilterKind, values: &[f64], settings: &FilterSettings) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    match filter {
        FilterKind::SavitzkyGolay => {
            let window = odd(settings.savgol_window);
            let order = settings.savgol_order.min(window.saturating_sub(2)).max(1);
            savgol(values, window, order)
        }
        FilterKind::MovingAverage => moving(values, odd(settings.moving_window), Window::Mean),
        FilterKind::Gaussian => gaussian(values, settings.gaussian_sigma.abs().max(0.25)),
        FilterKind::Median => moving(values, odd(settings.median_window), Window::Median),
        FilterKind::LowPass => {
            let alpha = settings.lowpass_alpha.clamp(0.01, 1.0);
            let mut out = Vec::with_capacity(values.len());
            let mut previous = values[0];
            for value in values {
                previous += alpha * (value - previous);
                out.push(previous);
            }
            out
        }
    }
}

enum Window {
    Mean,
    Median,
}

fn moving(values: &[f64], window: usize, kind: Window) -> Vec<f64> {
    let half = window / 2;
    (0..values.len())
        .map(|index| {
            let mut samples = Vec::with_capacity(window);
            for offset in 0..window {
                let source = (index as i64 + offset as i64 - half as i64)
                    .clamp(0, values.len().saturating_sub(1) as i64) as usize;
                samples.push(values[source]);
            }
            match kind {
                Window::Mean => samples.iter().sum::<f64>() / samples.len() as f64,
                Window::Median => {
                    samples.sort_by(f64::total_cmp);
                    samples[samples.len() / 2]
                }
            }
        })
        .collect()
}

fn gaussian(values: &[f64], sigma: f64) -> Vec<f64> {
    let radius = (3.0 * sigma).ceil() as usize;
    let weights = (-(radius as i64)..=radius as i64)
        .map(|offset| (-(offset as f64).powi(2) / (2.0 * sigma * sigma)).exp())
        .collect::<Vec<_>>();
    let total = weights.iter().sum::<f64>();
    (0..values.len())
        .map(|index| {
            let mut sum = 0.0;
            for (offset, weight) in (-(radius as i64)..=radius as i64).zip(&weights) {
                let source = (index as i64 + offset).clamp(0, values.len().saturating_sub(1) as i64) as usize;
                sum += values[source] * weight;
            }
            sum / total
        })
        .collect()
}

fn odd(window: usize) -> usize {
    let window = window.max(3);
    if window.is_multiple_of(2) { window + 1 } else { window }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spike() -> Vec<[f64; 2]> {
        (0..21)
            .map(|index| [f64::from(index), if index == 10 { 10.0 } else { 0.0 }])
            .collect()
    }

    #[test]
    fn every_filter_keeps_the_x_values_and_length() {
        let data = spike();
        let settings = FilterSettings::default();
        for filter in FilterKind::ALL {
            let filtered = apply_filters(&data, &[filter], &settings);
            assert_eq!(filtered.len(), data.len(), "{filter:?}");
            assert!(filtered.iter().zip(&data).all(|(a, b)| a[0] == b[0]));
        }
    }

    #[test]
    fn savgol_preserves_a_straight_line() {
        let data = (0..21)
            .map(|index| [f64::from(index), 2.0 * f64::from(index) + 1.0])
            .collect::<Vec<_>>();
        let filtered = apply_filters(&data, &[FilterKind::SavitzkyGolay], &FilterSettings::default());
        // The clamped window ends are the only places a straight line can bend.
        for (point, expected) in filtered.iter().zip(&data).skip(6).take(filtered.len() - 12) {
            assert!((point[1] - expected[1]).abs() < 1.0e-6);
        }
    }

    #[test]
    fn moving_average_and_gaussian_reduce_a_spike() {
        let settings = FilterSettings::default();
        for filter in [FilterKind::MovingAverage, FilterKind::Gaussian] {
            let filtered = apply_filters(&spike(), &[filter], &settings);
            assert!(filtered[10][1] < 10.0, "{filter:?}");
            assert!(filtered[10][1] > 0.0, "{filter:?}");
        }
    }

    #[test]
    fn median_removes_a_single_outlier() {
        let filtered = apply_filters(&spike(), &[FilterKind::Median], &FilterSettings::default());
        assert_eq!(filtered[10][1], 0.0);
    }

    #[test]
    fn the_low_pass_moves_toward_the_input() {
        let data = [[0.0, 0.0], [1.0, 10.0], [2.0, 10.0]];
        let filtered = apply_filters(&data, &[FilterKind::LowPass], &FilterSettings::default());
        assert!(filtered[1][1] > 0.0 && filtered[1][1] < 10.0);
        assert!(filtered[2][1] > filtered[1][1]);
    }
}
