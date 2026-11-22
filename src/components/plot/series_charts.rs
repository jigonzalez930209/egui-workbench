//! Constructors for the chart families beyond lines, bars, and scatter.

use std::borrow::Cow;

use super::series::{Kind, Series};

impl<'a> Series<'a> {
    /// One slice of a pie or donut chart. Add one series per slice; the values are
    /// normalized to the visible slices.
    ///
    /// ```
    /// # use egui_workbench::components::plot::{PieConfig, Plot, Series};
    /// # let ctx = egui::Context::default();
    /// # let mut icons = egui_workbench::Icons::new();
    /// # let mut state = egui_workbench::components::plot::PlotState::default();
    /// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
    /// let mut config = egui_workbench::components::plot::PlotConfig::default();
    /// config.pie.inner_radius = 0.55;
    /// Plot::new("share")
    ///     .config(config)
    ///     .series(Series::pie("Desktop", 62.0))
    ///     .series(Series::pie("Mobile", 38.0))
    ///     .show(ui, &mut icons, &mut state);
    /// # });
    /// # output.textures_delta.clear();
    /// ```
    pub fn pie(name: impl Into<String>, value: f64) -> Series<'static> {
        Series::new(name, Cow::Owned(Vec::new()), Kind::Pie { value })
    }

    /// Open–high–low–close candles: `[x, open, high, low, close]` rows.
    pub fn candles(name: impl Into<String>, rows: impl Into<Cow<'a, [[f64; 5]]>>) -> Series<'a> {
        let rows = rows.into();
        let close = rows.iter().map(|row| [row[0], row[4]]).collect();
        let mut series = Series::new(name, Cow::Owned(close), Kind::Candles { width: None });
        series.ohlc = Some(rows);
        series
    }

    /// A heatmap from `[x, y, value]` cells. Cell size follows the smallest gap on each
    /// axis; the color scale spans the visible values.
    pub fn heatmap(name: impl Into<String>, cells: impl Into<Cow<'a, [[f64; 3]]>>) -> Series<'a> {
        let cells = cells.into();
        let points = cells.iter().map(|cell| [cell[0], cell[1]]).collect();
        let mut series = Series::new(name, Cow::Owned(points), Kind::Heatmap);
        series.cells = Some(cells);
        series
    }

    /// A radar polygon over the axes named in [`RadarConfig::labels`](super::RadarConfig::labels).
    /// Values are indexed in order around the ring; zero is the center.
    pub fn radar(name: impl Into<String>, values: impl AsRef<[f64]>) -> Series<'static> {
        let data = values
            .as_ref()
            .iter()
            .enumerate()
            .map(|(index, value)| [index as f64, *value])
            .collect();
        Series::new(name, Cow::Owned(data), Kind::Radar)
    }

    /// One gauge with its own range.
    pub fn gauge(name: impl Into<String>, value: f64, range: std::ops::RangeInclusive<f64>) -> Series<'static> {
        let (mut min, mut max) = (*range.start(), *range.end());
        if !(min.is_finite() && max.is_finite()) || min >= max {
            min = min.min(0.0);
            max = min + 1.0;
        }
        let value = if value.is_finite() { value } else { min };
        Series::new(name, Cow::Owned(Vec::new()), Kind::Gauge { value, min, max })
    }

    /// A waterfall: each `[x, change]` bar starts where the previous one ended.
    pub fn waterfall(name: impl Into<String>, data: impl Into<Cow<'a, [[f64; 2]]>>) -> Series<'a> {
        Series::new(name, data.into(), Kind::Waterfall { width: None })
    }
}
