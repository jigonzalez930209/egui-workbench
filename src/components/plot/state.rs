//! The interactive state of one plot.
//!
//! The application owns it, like every other component state.

use super::PlotAnnotation;
use super::axis::{Bounds, Scale};
use super::filter::FilterKind;
use super::measure::{MeasurementConfig, PlotMeasurement};
use super::pipeline::Pipeline;
use super::series::Series;
use super::transforms::TransformKind;
use super::types::{Drag, PlotTool};

/// The interactive state of one plot. Keep it in the application model.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct PlotState {
    pub(super) view: Option<Bounds>,
    /// The tool the primary button uses.
    pub tool: PlotTool,
    /// The selected area in data coordinates.
    pub selection: Option<Bounds>,
    pub(super) hidden: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) drag: Option<Drag>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) export_menu: bool,
    /// The right axis range in scaled units, once zoomed or panned.
    pub(super) y2_view: Option<[f64; 2]>,
    /// Where the dragged legend sits: 0–1 across the room left in the frame.
    pub(super) legend_pos: Option<[f32; 2]>,
    /// Series extents by [`Series::fingerprint`], so the automatic view does not walk
    /// every point of every series each frame.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) extents: Vec<(u64, Bounds)>,
    /// The series whose legend row the pointer is over.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) legend_hover: Option<usize>,
    /// When that hover began, for the animation clock.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) legend_hover_start: f64,
    /// Text notes, arrows, spans, and regions.
    #[cfg_attr(feature = "serde", serde(default))]
    pub annotations: Vec<PlotAnnotation>,
    /// Distance and area measurements drawn over the data.
    #[cfg_attr(feature = "serde", serde(default))]
    pub measurements: Vec<PlotMeasurement>,
    /// Filters and transforms applied to the visible series.
    #[cfg_attr(feature = "serde", serde(default))]
    pub pipeline: Pipeline,
    /// Live measurement settings. The chart config seeds them until the user edits them.
    #[cfg_attr(feature = "serde", serde(default))]
    pub measurement: MeasurementConfig,
    /// Whether the user changed the measurement settings from the panel.
    #[cfg_attr(feature = "serde", serde(default))]
    pub(super) measurement_customized: bool,
    /// Whether the settings panel is pinned open.
    #[cfg_attr(feature = "serde", serde(default))]
    pub settings_pinned: bool,
    /// Transient hover state for the settings panel.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) settings_hovered: bool,
    /// Dragged offset of the settings panel.
    #[cfg_attr(feature = "serde", serde(default))]
    pub settings_offset: [f32; 2],
    /// Whether the tool ribbon stays unrolled.
    #[cfg_attr(feature = "serde", serde(default))]
    pub toolbar_pinned: bool,
    /// Ribbon reveal fraction, animated.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) toolbar_reveal: f32,
    /// Dragged ribbon position relative to the plot frame.
    #[cfg_attr(feature = "serde", serde(default))]
    pub toolbar_anchor: Option<[f32; 2]>,
}

impl Default for PlotState {
    fn default() -> Self {
        Self {
            view: None,
            tool: PlotTool::default(),
            selection: None,
            hidden: Vec::new(),
            drag: None,
            export_menu: false,
            y2_view: None,
            legend_pos: None,
            extents: Vec::new(),
            legend_hover: None,
            legend_hover_start: 0.0,
            annotations: Vec::new(),
            measurements: Vec::new(),
            pipeline: Pipeline::default(),
            measurement: MeasurementConfig::default(),
            measurement_customized: false,
            settings_pinned: false,
            settings_hovered: false,
            settings_offset: [0.0; 2],
            // Keep the tools visible by default; the pin collapses the ribbon and the
            // hover animation reopens it.
            toolbar_pinned: true,
            toolbar_reveal: 1.0,
            toolbar_anchor: None,
        }
    }
}

impl PlotState {
    /// The cached extent of `series`, if it is current.
    pub(super) fn cached_bounds(&self, series: &Series<'_>, x_scale: Scale, y_scale: Scale) -> Bounds {
        let key = series.fingerprint(x_scale, y_scale);
        self.extents
            .iter()
            .find(|(cached, _)| *cached == key)
            .map_or_else(|| series.bounds(x_scale, y_scale), |(_, bounds)| *bounds)
    }

    pub(super) fn legend_hover(&self) -> Option<usize> {
        self.legend_hover
    }

    pub(super) fn legend_hover_start(&self) -> f64 {
        self.legend_hover_start
    }

    pub(super) fn set_legend_hover(&mut self, series: Option<usize>, now: f64) {
        if series != self.legend_hover {
            self.legend_hover = series;
            self.legend_hover_start = now;
        }
    }

    /// The series whose legend row is under the pointer, if any.
    pub fn hovered_series(&self) -> Option<usize> {
        self.legend_hover
    }
}

impl PlotState {
    /// Fit the view to the data again (also bound to double-click and the toolbar).
    pub fn reset_view(&mut self) {
        self.view = None;
        self.y2_view = None;
    }

    /// Whether the view follows the data (no zoom or pan since the last reset).
    pub fn is_auto(&self) -> bool {
        self.view.is_none()
    }

    /// The explicit view, or `None` while automatic.
    pub fn view(&self) -> Option<Bounds> {
        self.view
    }

    /// Show `bounds` (data coordinates, linear axes) on the x and left y axes.
    pub fn set_view(&mut self, bounds: Bounds) {
        self.view = Some(bounds);
    }

    /// Show `min..=max` (data coordinates, linear axis) on the right y axis.
    pub fn set_y2_view(&mut self, min: f64, max: f64) {
        self.y2_view = Some([min.min(max), min.max(max)]);
    }

    /// Every distance and area measurement, in the order they were added.
    pub fn measurements(&self) -> &[PlotMeasurement] {
        &self.measurements
    }

    /// Add a measurement. The plot draws it on the next frame.
    pub fn add_measurement(&mut self, measurement: PlotMeasurement) {
        self.measurements.push(measurement);
    }

    /// Remove the measurement at `index`.
    pub fn remove_measurement(&mut self, index: usize) -> Option<PlotMeasurement> {
        (index < self.measurements.len()).then(|| self.measurements.remove(index))
    }

    /// Remove the newest measurement and return it.
    pub fn pop_measurement(&mut self) -> Option<PlotMeasurement> {
        self.measurements.pop()
    }

    /// Remove every measurement.
    pub fn clear_measurements(&mut self) {
        self.measurements.clear();
    }

    /// The filter/transform pipeline.
    pub fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    /// The filter/transform pipeline, mutably.
    pub fn pipeline_mut(&mut self) -> &mut Pipeline {
        &mut self.pipeline
    }

    /// Enable or disable one filter.
    pub fn toggle_filter(&mut self, filter: FilterKind) {
        self.pipeline.toggle_filter(filter);
    }

    /// Enable or disable one filter explicitly.
    pub fn set_filter(&mut self, filter: FilterKind, enabled: bool) {
        self.pipeline.set_filter(filter, enabled);
    }

    /// Replace the transform. A new transform clears measurements because the data
    /// domain changed.
    pub fn set_transform(&mut self, transform: Option<TransformKind>) {
        if self.pipeline.transform != transform {
            self.pipeline.set_transform(transform);
            self.measurements.clear();
        }
    }

    /// Disable every filter and transform.
    pub fn clear_analysis(&mut self) {
        self.pipeline.clear();
    }

    /// Replace the live measurement settings and mark them user-edited.
    pub fn set_measurement(&mut self, measurement: MeasurementConfig) {
        self.measurement = measurement;
        self.measurement_customized = true;
    }

    /// Clear every painted overlay: measurements, selection, annotations, and analysis.
    pub fn clear_overlays(&mut self) {
        self.measurements.clear();
        self.selection = None;
        self.annotations.clear();
        self.pipeline.clear();
    }

    /// Whether a clear action would remove anything.
    pub fn has_overlays(&self) -> bool {
        !self.measurements.is_empty()
            || !self.annotations.is_empty()
            || self.pipeline.is_active()
            || self.selection.is_some()
    }

    /// Pin the tool ribbon open or let it unroll on hover.
    pub fn pin_toolbar(&mut self, pinned: bool) {
        self.toolbar_pinned = pinned;
        if pinned {
            self.toolbar_reveal = 1.0;
        }
    }

    /// Collapse the ribbon and unpin it.
    pub fn collapse_toolbar(&mut self) {
        self.toolbar_pinned = false;
        self.toolbar_reveal = 0.0;
    }

    /// Where the legend was dragged to, as fractions (0 = left/top, 1 = right/bottom)
    /// of the room the frame leaves around it; `None` uses the configured corner.
    pub fn legend_position(&self) -> Option<[f32; 2]> {
        self.legend_pos
    }

    /// Place the legend (see [`Self::legend_position`]); `None` returns it to its corner.
    pub fn set_legend_position(&mut self, position: Option<[f32; 2]>) {
        self.legend_pos = position.map(|[x, y]| [x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)]);
    }

    /// Whether the legend has hidden the series called `name`.
    pub fn is_hidden(&self, name: &str) -> bool {
        self.hidden.iter().any(|hidden| hidden == name)
    }

    /// Show or hide a series by name.
    pub fn set_hidden(&mut self, name: &str, hidden: bool) {
        self.hidden.retain(|entry| entry != name);
        if hidden {
            self.hidden.push(name.to_owned());
        }
    }
}
