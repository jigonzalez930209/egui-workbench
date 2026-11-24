//! Plots bound to dataset columns: `Plot::from_dataset(&data).x("t").y("v")`.

use egui::{Color32, Ui};

use super::{Bounds, Plot, PlotResponse, PlotState, Series};
use crate::Icons;
use crate::data::{Dataset, LinkHub, LinkMode};

/// How a dataset series is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum DatasetPlotKind {
    /// A line through the points.
    #[default]
    Line,
    /// One marker per point.
    Scatter,
}

/// A plot builder over two numeric dataset columns.
pub struct DatasetPlot<'a> {
    data: &'a Dataset,
    x: String,
    y: Option<String>,
    color_by: Option<String>,
    title: Option<String>,
    kind: DatasetPlotKind,
    height: f32,
    visible: Option<Vec<usize>>,
}

impl<'a> DatasetPlot<'a> {
    /// A line plot over `x` and `y`.
    pub fn new(data: &'a Dataset, x: impl Into<String>, y: Option<impl Into<String>>) -> Self {
        Self {
            data,
            x: x.into(),
            y: y.map(Into::into),
            color_by: None,
            title: None,
            kind: DatasetPlotKind::Line,
            height: 260.0,
            visible: None,
        }
    }

    /// Set the x column.
    pub fn x(mut self, x: impl Into<String>) -> Self {
        self.x = x.into();
        self
    }

    /// Set the y column.
    pub fn y(mut self, y: impl Into<String>) -> Self {
        self.y = Some(y.into());
        self
    }

    /// Split the series by a text/category column.
    pub fn color_by(mut self, column: impl Into<String>) -> Self {
        self.color_by = Some(column.into());
        self
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Draw markers instead of a line.
    pub fn scatter(mut self) -> Self {
        self.kind = DatasetPlotKind::Scatter;
        self
    }

    /// Set the height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// The rows whose x and y values fall inside `bounds`.
    pub fn rows_in(&self, bounds: Bounds) -> Vec<usize> {
        let Some(y) = &self.y else {
            return Vec::new();
        };
        self.data.rows_in_bounds(&self.x, y, bounds)
    }

    /// Show only `rows` of the dataset.
    pub fn with_rows(mut self, rows: impl IntoIterator<Item = usize>) -> Self {
        self.visible = Some(rows.into_iter().collect());
        self
    }

    /// Show the plot and publish its brush to a selection bus.
    pub fn show_linked(
        mut self,
        ui: &mut Ui,
        icons: &mut Icons,
        state: &mut PlotState,
        hub: &mut LinkHub,
    ) -> PlotResponse {
        if hub.mode() != LinkMode::Highlight && !hub.selection().is_empty() {
            self.visible = Some(hub.selection().rows.iter().copied().collect());
        }
        let selected_rows = state.selection.map(|selection| self.rows_in(selection));
        let response = self.show(ui, icons, state);
        if let Some(rows) = selected_rows {
            hub.set_rows(rows, "plot");
        } else if hub.selection().source.as_deref() == Some("plot") {
            hub.clear();
        }
        response
    }

    /// Show the plot.
    pub fn show(self, ui: &mut Ui, icons: &mut Icons, state: &mut PlotState) -> PlotResponse {
        let Some(y) = self.y.as_deref() else {
            return Plot::new("dataset-empty").height(self.height).show(ui, icons, state);
        };
        let palette = [
            Color32::from_rgb(0x58, 0xa6, 0xff),
            Color32::from_rgb(0xf7, 0x78, 0xba),
            Color32::from_rgb(0x3f, 0xd0, 0x8a),
            Color32::from_rgb(0xe3, 0xb3, 0x41),
            Color32::from_rgb(0xb0, 0x8c, 0xff),
            Color32::from_rgb(0x56, 0xd4, 0xdd),
        ];
        let mut groups: Vec<(String, Vec<[f64; 2]>)> = Vec::new();
        let rows: Vec<usize> = self
            .visible
            .clone()
            .unwrap_or_else(|| (0..self.data.row_count()).collect());
        if let Some(color_by) = &self.color_by
            && let Some(column) = self.data.column_named(color_by)
        {
            for row in rows {
                let Some([x, y]) = self.data.point_at_row(&self.x, y, row) else {
                    continue;
                };
                let key = column.cell(row).to_text();
                if let Some((_, points)) = groups.iter_mut().find(|(name, _)| name == &key) {
                    points.push([x, y]);
                } else {
                    groups.push((key, vec![[x, y]]));
                }
            }
        } else {
            let points: Vec<[f64; 2]> = rows
                .into_iter()
                .filter_map(|row| self.data.point_at_row(&self.x, y, row))
                .collect();
            groups.push((y.to_owned(), points));
        }
        let mut plot = Plot::new("dataset").x_label(&self.x).y_label(y).height(self.height);
        if let Some(title) = &self.title {
            plot = plot.title(title);
        }
        for (index, (name, points)) in groups.iter().enumerate() {
            let series = match self.kind {
                DatasetPlotKind::Line => Series::line(name.clone(), points.as_slice()),
                DatasetPlotKind::Scatter => Series::scatter(name.clone(), points.as_slice()),
            };
            plot = plot.series(series.color(palette[index % palette.len()]));
        }
        plot.show(ui, icons, state)
    }
}

impl Plot<'_> {
    /// Start a plot bound to dataset columns.
    pub fn from_dataset(data: &Dataset) -> DatasetPlot<'_> {
        DatasetPlot::new(data, "", None::<String>)
    }
}
