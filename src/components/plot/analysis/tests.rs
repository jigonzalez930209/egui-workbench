//! Analysis tool tests.

use super::*;

#[test]
fn peaks_respect_height_and_distance() {
    let values = [0.0, 1.0, 0.2, 0.9, 0.0, 2.0, 0.3];
    let peaks = find_peaks(&values, 0.5, 3);
    assert_eq!(peaks.iter().map(|peak| peak.index).collect::<Vec<_>>(), [1, 5]);
}

#[test]
fn savgol_keeps_a_line_and_removes_noise() {
    let line: Vec<f64> = (0..31).map(|index| 2.0 * index as f64 + 1.0).collect();
    let smooth = savgol(&line, 7, 2);
    for index in 3..line.len() - 3 {
        assert!((line[index] - smooth[index]).abs() < 1e-9, "{index}");
    }
}

#[test]
fn derivative_and_integral_round_trip() {
    let values = [0.0, 1.0, 2.0, 3.0, 4.0];
    let derivative = derivative(&values, 1.0);
    assert_eq!(derivative[2], 1.0);
    let area = integral(&values, 1.0);
    assert_eq!(area.last().copied(), Some(8.0));
    let smooth = savgol_derivative(&values, 5, 2, 1.0);
    assert!((smooth[2] - 1.0).abs() < 1e-9);
}

#[test]
fn fft_finds_a_sine_frequency() {
    let sample_rate = 100.0;
    let values: Vec<f64> = (0..128)
        .map(|index| (std::f64::consts::TAU * 10.0 * index as f64 / sample_rate).sin())
        .collect();
    let spectrum = spectrum(&values, sample_rate);
    let peak = spectrum
        .iter()
        .max_by(|left, right| left[1].total_cmp(&right[1]))
        .copied()
        .expect("spectrum");
    assert!((peak[0] - 10.0).abs() < 1.0, "{peak:?}");
}

#[test]
fn resampling_and_statistics() {
    let values = [0.0, 1.0, 2.0, 3.0];
    let resampled = resample(&values, 7);
    assert_eq!(resampled.len(), 7);
    assert!((resampled[0] - 0.0).abs() < 1e-9);
    assert!((resampled[6] - 3.0).abs() < 1e-9);
    let stats = Statistics::of(&[1.0, 2.0, f64::NAN, 3.0]);
    assert_eq!(stats.count, 3);
    assert_eq!(stats.median, 2.0);
    assert!((stats.mean - 2.0).abs() < 1e-9);
}

#[test]
fn a_model_fit_recovers_parameters() {
    let model = |x: f64, p: &[f64]| p[0] * (-x / p[1]).exp() + p[2];
    let x: Vec<f64> = (0..40).map(|index| index as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|x| 2.0 * (-x / 1.5).exp() + 0.5).collect();
    let fit = fit_model(&x, &y, &[1.0, 1.0, 0.0], &model).expect("fit");
    assert!((fit.params[0] - 2.0).abs() < 1e-3);
    assert!((fit.params[1] - 1.5).abs() < 1e-3);
    assert!((fit.params[2] - 0.5).abs() < 1e-3);
    assert!(fit.errors.is_some());
}

#[test]
fn baseline_removal_subtracts_the_rolling_minimum() {
    let mut values = [1.0, 2.0, 1.5, 3.0, 2.5];
    remove_baseline(&mut values, 3);
    assert!(values[0].abs() < 1e-9);
    assert!((values[3] - 1.5).abs() < 1e-9);
}
