//! Fourier and related transforms available from the analysis menu.

use serde::{Deserialize, Serialize};

use super::analysis::{Complex, fft};

/// A transform available from the analysis menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TransformKind {
    /// Magnitude spectrum.
    Fft,
    /// Inverse DFT of the series treated as a complex spectrum.
    InverseFft,
    /// One-sided power spectral density.
    PowerSpectrum,
    /// Normalized autocorrelation.
    Autocorrelation,
    /// Normalized cross-correlation of the first two series.
    CrossCorrelation,
    /// Analytic-signal envelope.
    HilbertEnvelope,
    /// Real cepstrum.
    Cepstrum,
}

impl TransformKind {
    /// Every transform, in menu order.
    pub const ALL: [Self; 7] = [
        Self::Fft,
        Self::InverseFft,
        Self::PowerSpectrum,
        Self::Autocorrelation,
        Self::CrossCorrelation,
        Self::HilbertEnvelope,
        Self::Cepstrum,
    ];

    /// The menu label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fft => "FFT magnitude",
            Self::InverseFft => "Inverse FFT",
            Self::PowerSpectrum => "Power spectral density",
            Self::Autocorrelation => "Autocorrelation",
            Self::CrossCorrelation => "Cross-correlation (2 series)",
            Self::HilbertEnvelope => "Hilbert envelope",
            Self::Cepstrum => "Cepstrum",
        }
    }

    /// Whether the transform needs a second series.
    pub const fn needs_pair(self) -> bool {
        matches!(self, Self::CrossCorrelation)
    }
}

/// Parameters shared by the transforms.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TransformSettings {
    /// Samples per x unit; `None` estimates it from the series.
    pub sample_rate: Option<f64>,
}

/// Apply `kind` to the first (and, for cross-correlation, second) series.
///
/// Returns an empty vector when the input is too short or a pair transform has fewer
/// than two series.
pub fn apply_transform(kind: TransformKind, inputs: &[&[[f64; 2]]], settings: &TransformSettings) -> Vec<[f64; 2]> {
    let Some(first) = inputs.first() else {
        return Vec::new();
    };
    if first.len() < 4 {
        return Vec::new();
    }
    let sample_rate = settings.sample_rate.unwrap_or_else(|| estimate_rate(first)).max(1.0e-9);
    let values = first.iter().map(|point| point[1]).collect::<Vec<_>>();
    match kind {
        TransformKind::Fft => magnitude(&values, sample_rate),
        TransformKind::InverseFft => inverse(&values, sample_rate),
        TransformKind::PowerSpectrum => power(&values, sample_rate),
        TransformKind::Autocorrelation => autocorrelation(&values, sample_rate),
        TransformKind::CrossCorrelation => {
            let Some(second) = inputs.get(1) else {
                return Vec::new();
            };
            cross_correlation(
                &values,
                &second.iter().map(|point| point[1]).collect::<Vec<_>>(),
                sample_rate,
            )
        }
        TransformKind::HilbertEnvelope => envelope(first, &values),
        TransformKind::Cepstrum => cepstrum(&values, sample_rate),
    }
}

fn estimate_rate(data: &[[f64; 2]]) -> f64 {
    let mut steps = data
        .windows(2)
        .map(|pair| pair[1][0] - pair[0][0])
        .filter(|step| step.is_finite() && *step > 0.0)
        .collect::<Vec<_>>();
    steps.sort_by(f64::total_cmp);
    steps.get(steps.len() / 2).map_or(1.0, |step| 1.0 / *step)
}

fn forward(values: &[f64]) -> Vec<Complex> {
    let length = next_power_of_two(values.len().max(2));
    let mut buffer = vec![Complex::default(); length];
    for (slot, value) in buffer.iter_mut().zip(values) {
        slot.re = *value;
    }
    fft(&mut buffer);
    buffer
}

fn inverse_transform(buffer: &mut [Complex]) {
    let length = buffer.len();
    for value in buffer.iter_mut() {
        value.im = -value.im;
    }
    fft(buffer);
    for value in buffer.iter_mut() {
        value.im = -value.im / length as f64;
        value.re /= length as f64;
    }
}

fn magnitude(values: &[f64], sample_rate: f64) -> Vec<[f64; 2]> {
    let buffer = forward(values);
    let length = buffer.len();
    (0..=length / 2)
        .map(|index| {
            let scale = if index == 0 || index == length / 2 { 1.0 } else { 2.0 };
            [
                index as f64 * sample_rate / length as f64,
                scale * buffer[index].re.hypot(buffer[index].im) / length as f64,
            ]
        })
        .collect()
}

fn inverse(values: &[f64], sample_rate: f64) -> Vec<[f64; 2]> {
    let mut buffer = values
        .iter()
        .map(|value| Complex { re: *value, im: 0.0 })
        .collect::<Vec<_>>();
    inverse_transform(&mut buffer);
    buffer
        .iter()
        .enumerate()
        .map(|(index, value)| [index as f64 / sample_rate, value.re])
        .collect()
}

fn power(values: &[f64], sample_rate: f64) -> Vec<[f64; 2]> {
    let buffer = forward(values);
    let length = buffer.len();
    (0..=length / 2)
        .map(|index| {
            let scale = if index == 0 || index == length / 2 { 1.0 } else { 2.0 };
            [
                index as f64 * sample_rate / length as f64,
                scale * (buffer[index].re * buffer[index].re + buffer[index].im * buffer[index].im)
                    / (length as f64 * sample_rate),
            ]
        })
        .collect()
}

fn autocorrelation(values: &[f64], sample_rate: f64) -> Vec<[f64; 2]> {
    let length = next_power_of_two(values.len() * 2);
    let mut buffer = vec![Complex::default(); length];
    for (slot, value) in buffer.iter_mut().zip(values) {
        slot.re = *value;
    }
    fft(&mut buffer);
    let power = buffer
        .iter()
        .map(|value| value.re * value.re + value.im * value.im)
        .collect::<Vec<_>>();
    let mut result = power
        .iter()
        .map(|value| Complex { re: *value, im: 0.0 })
        .collect::<Vec<_>>();
    inverse_transform(&mut result);
    let zero = result[0].re.abs().max(1.0e-12);
    (0..=length / 2)
        .map(|lag| [lag as f64 / sample_rate, result[lag].re / zero])
        .collect()
}

fn cross_correlation(first: &[f64], second: &[f64], sample_rate: f64) -> Vec<[f64; 2]> {
    let length = next_power_of_two(first.len().max(second.len()) * 2);
    let mut a = vec![Complex::default(); length];
    let mut b = vec![Complex::default(); length];
    for (slot, value) in a.iter_mut().zip(first) {
        slot.re = *value;
    }
    for (slot, value) in b.iter_mut().zip(second) {
        slot.re = *value;
    }
    fft(&mut a);
    fft(&mut b);
    let mut product = a
        .iter()
        .zip(&b)
        .map(|(a, b)| Complex {
            re: a.re * b.re + a.im * b.im,
            im: a.im * b.re - a.re * b.im,
        })
        .collect::<Vec<_>>();
    inverse_transform(&mut product);
    let zero = (a[0].re.abs() * b[0].re.abs()).max(1.0e-12);
    (0..=length / 2)
        .map(|lag| [lag as f64 / sample_rate, product[lag].re / zero])
        .collect()
}

fn envelope(data: &[[f64; 2]], values: &[f64]) -> Vec<[f64; 2]> {
    let mut buffer = forward(values);
    let length = buffer.len();
    for value in buffer.iter_mut().take(length / 2).skip(1) {
        value.re *= 2.0;
        value.im *= 2.0;
    }
    for value in buffer.iter_mut().skip(length / 2 + 1) {
        value.re = 0.0;
        value.im = 0.0;
    }
    inverse_transform(&mut buffer);
    data.iter()
        .zip(&buffer)
        .map(|(point, value)| [point[0], value.re.hypot(value.im)])
        .collect()
}

fn cepstrum(values: &[f64], sample_rate: f64) -> Vec<[f64; 2]> {
    let mut buffer = forward(values);
    for value in &mut buffer {
        value.re = (value.re.hypot(value.im) + 1.0e-12).ln();
        value.im = 0.0;
    }
    inverse_transform(&mut buffer);
    (0..=buffer.len() / 2)
        .map(|index| [index as f64 / sample_rate, buffer[index].re])
        .collect()
}

fn next_power_of_two(value: usize) -> usize {
    value.max(2).next_power_of_two()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frequency: f64, rate: f64, len: usize) -> Vec<[f64; 2]> {
        (0..len)
            .map(|index| {
                let t = index as f64 / rate;
                [t, (std::f64::consts::TAU * frequency * t).sin()]
            })
            .collect()
    }

    fn peak(data: &[[f64; 2]]) -> [f64; 2] {
        data.iter()
            .copied()
            .max_by(|a, b| a[1].total_cmp(&b[1]))
            .unwrap_or([0.0, 0.0])
    }

    #[test]
    fn fft_finds_the_sine_frequency() {
        let data = sine(8.0, 128.0, 128);
        let spectrum = apply_transform(TransformKind::Fft, &[&data], &TransformSettings::default());
        assert!((peak(&spectrum)[0] - 8.0).abs() < 1.0);
    }

    #[test]
    fn the_inverse_transform_round_trips() {
        let values = [1.0, 2.0, -1.0, 0.5, 0.0, -2.0, 3.0, 1.0];
        let mut buffer = forward(&values);
        inverse_transform(&mut buffer);
        for (actual, expected) in buffer.iter().zip(values) {
            assert!((actual.re - expected).abs() < 1.0e-9);
        }
    }

    #[test]
    fn the_power_spectrum_peaks_at_the_signal_frequency() {
        let data = sine(5.0, 64.0, 128);
        let power = apply_transform(TransformKind::PowerSpectrum, &[&data], &TransformSettings::default());
        assert!((peak(&power)[0] - 5.0).abs() < 1.0);
    }

    #[test]
    fn autocorrelation_is_one_at_zero_lag() {
        let data = sine(4.0, 64.0, 128);
        let correlation = apply_transform(TransformKind::Autocorrelation, &[&data], &TransformSettings::default());
        assert!((correlation[0][1] - 1.0).abs() < 1.0e-9);
    }

    #[test]
    fn cross_correlation_finds_a_shifted_signal() {
        let first = sine(6.0, 64.0, 128);
        let shifted = first
            .iter()
            .map(|point| [point[0], (point[1] * 0.5) + 0.1])
            .collect::<Vec<_>>();
        let correlation = apply_transform(
            TransformKind::CrossCorrelation,
            &[&first, &shifted],
            &TransformSettings::default(),
        );
        assert!(correlation[0][1] > 0.9);
    }

    #[test]
    fn the_hilbert_envelope_of_a_sine_is_one() {
        let data = sine(5.0, 64.0, 128);
        let envelope = apply_transform(TransformKind::HilbertEnvelope, &[&data], &TransformSettings::default());
        assert_eq!(envelope.len(), data.len());
        for point in &envelope[10..110] {
            assert!((point[1] - 1.0).abs() < 0.05, "{point:?}");
        }
    }

    #[test]
    fn the_cepstrum_has_a_frequency_axis() {
        let data = sine(5.0, 64.0, 128);
        let cepstrum = apply_transform(TransformKind::Cepstrum, &[&data], &TransformSettings::default());
        assert!(cepstrum.len() > 2);
        assert!(cepstrum[1][0] > cepstrum[0][0]);
    }
}
