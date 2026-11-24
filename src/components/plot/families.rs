//! Chart families built on the core line, bar, and heatmap series: histograms, box
//! plots, violins, hexbins, contours, polar, and Smith charts.

use super::{Colormap, Series};

/// A histogram of `values` as centered bars.
pub fn histogram(name: impl Into<String>, values: &[f64], bins: usize) -> Series<'static> {
    let bins = bins.max(1);
    let finite: Vec<f64> = values.iter().copied().filter(|value| value.is_finite()).collect();
    if finite.is_empty() {
        return Series::bars(name, Vec::new());
    }
    let min = finite.iter().copied().fold(f64::INFINITY, f64::min);
    let max = finite.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(1e-12);
    let width = span / bins as f64;
    let mut counts = vec![0.0; bins];
    for value in finite {
        let index = (((value - min) / width) as usize).min(bins - 1);
        counts[index] += 1.0;
    }
    let points: Vec<[f64; 2]> = counts
        .iter()
        .enumerate()
        .map(|(index, count)| [min + (index as f64 + 0.5) * width, *count])
        .collect();
    Series::bars(name, points).bar_width(width * 0.9)
}

/// A histogram normalized so the bar areas sum to one; a density estimate appears as a
/// line over the same bars.
pub fn histogram_density(name: impl Into<String>, values: &[f64], bins: usize) -> Vec<Series<'static>> {
    let bins = bins.max(1);
    let finite: Vec<f64> = values.iter().copied().filter(|value| value.is_finite()).collect();
    if finite.is_empty() {
        return Vec::new();
    }
    let min = finite.iter().copied().fold(f64::INFINITY, f64::min);
    let max = finite.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(1e-12);
    let width = span / bins as f64;
    let mut counts = vec![0.0; bins];
    for value in finite.iter().copied() {
        counts[(((value - min) / width) as usize).min(bins - 1)] += 1.0;
    }
    let total = finite.len() as f64 * width;
    let bars: Vec<[f64; 2]> = counts
        .iter()
        .enumerate()
        .map(|(index, count)| [min + (index as f64 + 0.5) * width, count / total])
        .collect();
    let name = name.into();
    let curve = Series::line(format!("{name} density"), bars.clone()).smooth(super::Smoothing::Spline);
    vec![Series::bars(name, bars).bar_width(width * 0.9), curve]
}

/// A box plot at x = 0: box, median, whiskers, and caps.
pub fn box_plot(name: impl Into<String>, values: &[f64]) -> Vec<Series<'static>> {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|value| value.is_finite()).collect();
    if sorted.is_empty() {
        return Vec::new();
    }
    sorted.sort_by(f64::total_cmp);
    let quantile = |q: f64| {
        let position = q.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
        let low = position.floor() as usize;
        let high = (low + 1).min(sorted.len() - 1);
        sorted[low] + (sorted[high] - sorted[low]) * (position - low as f64)
    };
    let q1 = quantile(0.25);
    let median = quantile(0.5);
    let q3 = quantile(0.75);
    let iqr = (q3 - q1).max(1e-12);
    let low = sorted
        .iter()
        .copied()
        .filter(|value| *value >= q1 - 1.5 * iqr)
        .fold(q1, f64::min);
    let high = sorted
        .iter()
        .copied()
        .filter(|value| *value <= q3 + 1.5 * iqr)
        .fold(q3, f64::max);
    let name = name.into();
    let half = 0.25;
    let box_outline = vec![[-half, q1], [half, q1], [half, q3], [-half, q3], [-half, q1]];
    vec![
        Series::line(format!("{name} box"), box_outline),
        Series::line(format!("{name} median"), vec![[-half, median], [half, median]]),
        Series::line(format!("{name} whisker"), vec![[0.0, low], [0.0, high]]),
        Series::line(format!("{name} low cap"), vec![[-half * 0.6, low], [half * 0.6, low]]),
        Series::line(
            format!("{name} high cap"),
            vec![[-half * 0.6, high], [half * 0.6, high]],
        ),
    ]
}

/// A violin outline from a Gaussian kernel density estimate, centered at x = 0.
pub fn violin(name: impl Into<String>, values: &[f64], samples: usize) -> Vec<Series<'static>> {
    let finite: Vec<f64> = values.iter().copied().filter(|value| value.is_finite()).collect();
    if finite.is_empty() {
        return Vec::new();
    }
    let min = finite.iter().copied().fold(f64::INFINITY, f64::min);
    let max = finite.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(1e-12);
    let bandwidth = (span / finite.len() as f64).sqrt().max(span / 100.0);
    let samples = samples.clamp(16, 512);
    let mut outline = Vec::with_capacity(samples * 2 + 1);
    let mut density = Vec::with_capacity(samples);
    for index in 0..samples {
        let x = min + span * index as f64 / (samples - 1) as f64;
        let value: f64 = finite
            .iter()
            .map(|sample| (-((x - sample) / bandwidth).powi(2) * 0.5).exp())
            .sum::<f64>()
            / finite.len() as f64;
        density.push((x, value));
    }
    let peak = density.iter().map(|(_, value)| *value).fold(0.0, f64::max).max(1e-12);
    for (x, value) in &density {
        outline.push([value / peak * 0.35, *x]);
    }
    for (x, value) in density.iter().rev() {
        outline.push([-value / peak * 0.35, *x]);
    }
    if let Some(first) = outline.first().copied() {
        outline.push(first);
    }
    let name = name.into();
    vec![
        Series::line(format!("{name} violin"), outline),
        Series::line(
            format!("{name} median"),
            vec![[-0.35, median_of(&finite)], [0.35, median_of(&finite)]],
        ),
    ]
}

fn median_of(sorted: &[f64]) -> f64 {
    let mut values = sorted.to_vec();
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) * 0.5
    } else {
        values[middle]
    }
}

/// A regular hexbin/2D-density grid of `[x, y, count]` cells, ready for `Series::heatmap`.
pub fn hexbin(name: impl Into<String>, points: &[[f64; 2]], bins: usize) -> Series<'static> {
    let bins = bins.max(2);
    let finite: Vec<[f64; 2]> = points
        .iter()
        .copied()
        .filter(|point| point[0].is_finite() && point[1].is_finite())
        .collect();
    if finite.is_empty() {
        return Series::heatmap(name, Vec::new());
    }
    let (min_x, max_x) = finite
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), point| {
            (min.min(point[0]), max.max(point[0]))
        });
    let (min_y, max_y) = finite
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), point| {
            (min.min(point[1]), max.max(point[1]))
        });
    let width = ((max_x - min_x) / bins as f64).max(1e-12);
    let height = ((max_y - min_y) / bins as f64).max(1e-12);
    let mut counts = vec![0.0; bins * bins];
    for point in finite {
        let x = (((point[0] - min_x) / width) as usize).min(bins - 1);
        let y = (((point[1] - min_y) / height) as usize).min(bins - 1);
        counts[y * bins + x] += 1.0;
    }
    let cells: Vec<[f64; 3]> = counts
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0.0)
        .map(|(index, count)| {
            let x = min_x + (index % bins) as f64 * width + width * 0.5;
            let y = min_y + (index / bins) as f64 * height + height * 0.5;
            [x, y, *count]
        })
        .collect();
    let name = name.into();
    Series::heatmap(name, cells)
}

/// A contour line of a regular grid at `level`, as one series with NaN breaks.
pub fn contour(
    name: impl Into<String>,
    grid: &[Vec<f64>],
    level: f64,
    origin: [f64; 2],
    step: [f64; 2],
) -> Series<'static> {
    let mut points: Vec<[f64; 2]> = Vec::new();
    for row in 0..grid.len().saturating_sub(1) {
        for column in 0..grid[row].len().saturating_sub(1) {
            let values = [
                grid[row][column],
                grid[row][column + 1],
                grid[row + 1][column + 1],
                grid[row + 1][column],
            ];
            let corners = [
                [origin[0] + column as f64 * step[0], origin[1] + row as f64 * step[1]],
                [
                    origin[0] + (column + 1) as f64 * step[0],
                    origin[1] + row as f64 * step[1],
                ],
                [
                    origin[0] + (column + 1) as f64 * step[0],
                    origin[1] + (row + 1) as f64 * step[1],
                ],
                [
                    origin[0] + column as f64 * step[0],
                    origin[1] + (row + 1) as f64 * step[1],
                ],
            ];
            let mut crossings = Vec::new();
            for edge in 0..4 {
                let next = (edge + 1) % 4;
                let (a, b) = (values[edge], values[next]);
                if (a < level && b >= level) || (a >= level && b < level) {
                    let t = (level - a) / (b - a);
                    let point = [
                        corners[edge][0] + (corners[next][0] - corners[edge][0]) * t,
                        corners[edge][1] + (corners[next][1] - corners[edge][1]) * t,
                    ];
                    crossings.push(point);
                }
            }
            if crossings.len() >= 2 {
                if !points.is_empty() {
                    points.push([f64::NAN, f64::NAN]);
                }
                points.push(crossings[0]);
                points.push(crossings[1]);
            }
        }
    }
    Series::line(name, points)
}

/// Convert polar samples `(theta, radius)` to a cartesian line.
pub fn polar(name: impl Into<String>, theta: &[f64], radius: &[f64]) -> Series<'static> {
    let points = theta
        .iter()
        .zip(radius)
        .filter(|(theta, radius)| theta.is_finite() && radius.is_finite())
        .map(|(theta, radius)| [radius * theta.cos(), radius * theta.sin()])
        .collect::<Vec<_>>();
    Series::line(name, points)
}

/// Convert impedance samples `(resistance, reactance)` to a Smith-chart reflection line.
pub fn smith(name: impl Into<String>, resistance: &[f64], reactance: &[f64]) -> Series<'static> {
    let points = resistance
        .iter()
        .zip(reactance)
        .filter(|(resistance, reactance)| resistance.is_finite() && reactance.is_finite())
        .map(|(resistance, reactance)| {
            let (zr, zi) = (*resistance, *reactance);
            let denominator = (zr + 1.0).powi(2) + zi * zi;
            [
                ((zr * zr + zi * zi - 1.0) / denominator.max(1e-12)),
                (2.0 * zi / denominator.max(1e-12)),
            ]
        })
        .collect::<Vec<_>>();
    Series::line(name, points)
}

/// A colorbar widget for an existing colormap, drawn at the current cursor.
pub fn show_colorbar(ui: &mut egui::Ui, colormap: Colormap, min: f64, max: f64, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, height), egui::Sense::hover());
    let bands = 32;
    let theme = crate::theme::current(ui.ctx());
    for index in 0..bands {
        let t = index as f32 / (bands - 1) as f32;
        let band = egui::Rect::from_min_size(
            egui::pos2(
                rect.left() + 4.0,
                rect.bottom() - (index + 1) as f32 * rect.height() / bands as f32,
            ),
            egui::vec2(10.0, rect.height() / bands as f32 + 1.0),
        );
        ui.painter().rect_filled(band, 0.0, colormap.sample(1.0 - t, &theme));
    }
    ui.painter().text(
        egui::pos2(rect.right() + 4.0, rect.top()),
        egui::Align2::LEFT_TOP,
        format!("{max:.3}"),
        crate::style::get(ui.ctx()).font(9.0),
        ui.visuals().weak_text_color(),
    );
    ui.painter().text(
        egui::pos2(rect.right() + 4.0, rect.bottom()),
        egui::Align2::LEFT_BOTTOM,
        format!("{min:.3}"),
        crate::style::get(ui.ctx()).font(9.0),
        ui.visuals().weak_text_color(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_bars_sum_to_the_sample_count() {
        let values = [0.1, 0.2, 0.25, 0.8, 0.95];
        let series = histogram("h", &values, 5);
        let total: f64 = series.data().iter().map(|point| point[1]).sum();
        assert_eq!(total, values.len() as f64);
    }

    #[test]
    fn box_and_violin_have_the_expected_parts() {
        let values = [1.0, 2.0, 3.0, 4.0, 100.0];
        assert_eq!(box_plot("b", &values).len(), 5);
        assert_eq!(violin("v", &values, 32).len(), 2);
    }

    #[test]
    fn hexbin_and_contour_produce_geometry() {
        let points = [[0.0, 0.0], [0.1, 0.1], [1.0, 1.0]];
        let hex = hexbin("h", &points, 4);
        assert!(hex.cells().is_some_and(|cells| !cells.is_empty()));
        let grid = vec![vec![0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 0.0]];
        let contour = contour("c", &grid, 0.5, [0.0, 0.0], [1.0, 1.0]);
        assert!(contour.data().len() >= 2);
    }

    #[test]
    fn polar_and_smith_map_domain_values() {
        let polar = polar("p", &[0.0, std::f64::consts::FRAC_PI_2], &[1.0, 1.0]);
        assert_eq!(polar.data()[0], [1.0, 0.0]);
        let smith = smith("s", &[1.0], &[0.0]);
        assert!((smith.data()[0][0] - 0.0).abs() < 1e-9);
        assert!((smith.data()[0][1] - 0.0).abs() < 1e-9);
    }
}
