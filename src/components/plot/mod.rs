//! A fast, interactive 2D plot in the spirit of matplotlib.
//!
//! - Series: lines, areas, steps, scatter with seven markers, bars, error bars,
//!   functions, and horizontal/vertical reference lines; plus pie/donut slices,
//!   candlesticks, heatmaps, radar polygons, gauges, and waterfalls.
//! - Axes: linear or log10 scales, 1-2-5 ticks that never overlap, minor log ticks,
//!   grid, and a title and axis titles with their own size, color, and alignment; every line is snapped to the pixel grid.
//! - Two y axes: series marked [`YAxis::Right`] get their own axis on the right, with
//!   its own range, scale, ticks, and title.
//! - Legend hover animations: 13 effects with user-defined easing, plus a direction
//!   arrow beside the newest sample of streaming line series.
//! - Interaction: drag to pan, wheel to zoom around the cursor (Shift: x only, Alt: y
//!   only; with two y axes, Alt zooms the left axis over the left half of the frame and
//!   the right axis over the right half), right-drag or the zoom tool to zoom to a box, Shift-drag or the select tool
//!   to select an area, double-click to fit the data again, and a hover popup with the
//!   nearest point. A small toolbar switches tools; the legend toggles series and can be
//!   dragged anywhere inside the frame (the position lives in the [`PlotState`]).
//! - Lines can be smoothed with a spline or a monotone cubic ([`Smoothing`]).
//! - Analysis: least-squares fits ([`Fit`]) with their equation and R².
//! - Speed: dense sorted lines are decimated per pixel column, so a million points
//!   draw in a few milliseconds; nothing repaints while idle.
//!
//! [`Plot3D`] is the 3D sibling: an orbit camera, x/y/z axes, lines, scatter, bars, and
//! surfaces, with the same legend, hover animations, direction arrow, themes, and
//! SVG/PNG export.
//!
//! The view, the selection, the active tool, and hidden series live in a
//! [`PlotState`] the caller owns, like every other component.

mod analysis;
mod animation;
mod annotations;
mod axis;
mod builder;
mod chart_config;
mod colormap;
mod config;
mod dataset;
mod decimate;
mod direction;
mod export;
mod extra;
mod families;
mod filter;
mod fit;
#[cfg(feature = "gpu")]
mod gpu;
#[cfg(feature = "gpu")]
mod gpu_line;
mod grid;
mod history;
mod interact;
mod layout;
mod legend;
mod math_text;
mod measure;
mod overlay;
mod paint;
mod pdf;
mod pipeline;
mod scene;
mod series;
mod series_bounds;
mod series_charts;
mod smooth;
mod snap;
mod special;
mod three;
mod transform;
mod transforms;
mod types;
mod widgets;
use crate::Icons;
use crate::components::IconComponent;
pub use analysis::{
    Complex, Model, ModelFit, Peak, Statistics, derivative, fft, find_peaks, fit_model, integral, remove_baseline,
    resample, rolling_baseline, savgol, savgol_derivative, spectrum,
};
pub use animation::{AnimationEasing, LegendAnimation, LegendHoverConfig};
pub use annotations::{PlotAnnotation, region_size};
pub use axis::{Bounds, Scale};
use axis::{SecondAxis, Transform};
pub use chart_config::{DirectionArrowConfig, GaugeConfig, HeatmapConfig, PieConfig, RadarConfig, WaterfallConfig};
pub use colormap::Colormap;
pub use config::{AxisConfig, Minor, PlotColors, PlotConfig, Renderer, SeriesStyle, TitleAlign};
pub use dataset::{DatasetPlot, DatasetPlotKind};
use egui::{Color32, Id, Pos2, Rect, Sense, Ui, vec2};
pub use export::{ExportOptions, ExportPreset};
pub use families::{box_plot, contour, hexbin, histogram, histogram_density, polar, show_colorbar, smith, violin};
pub use filter::{FilterKind, FilterSettings};
pub use fit::{Fit, FitKind};
#[cfg(feature = "gpu")]
pub use gpu::set_output_format;
pub use grid::PlotGrid;
pub use history::ViewHistory;
pub use math_text::to_unicode_math;
pub use measure::{
    CurveAreaSummary, DistanceSummary, Extremum, MeasurementAreaStyle, MeasurementConfig, MeasurementMode,
    PlotMeasurement, SeriesMeasurement, curve_area_summary, region_measurements,
};
pub use pipeline::Pipeline;
use scene::Measure;
pub use series::{LineStyle, Marker, Series, Smoothing, YAxis, points_in};
pub use three::{
    Bounds3D, Camera3D, Plot3D, Plot3DConfig, Plot3DHover, Plot3DResponse, Plot3DState, Plot3DTool, Projection,
    Series3D, Series3DStyle, Surface, Surface3DConfig,
};
pub use transforms::{TransformKind, TransformSettings};
use types::Drag;
pub use types::{ChartMode, ExportFormat, LegendPosition, PlotHover, PlotResponse, PlotTool};

mod state;

pub use state::PlotState;

/// The plot builder. Add series, then call `show` with the icon cache and a
/// [`PlotState`].
///
/// ```
/// # use egui_workbench::components::plot::{Plot, PlotState, Series, Fit, FitKind};
/// # let ctx = egui::Context::default();
/// # let mut icons = egui_workbench::Icons::new();
/// let data: Vec<[f64; 2]> = (0..100).map(|i| [i as f64 * 0.1, (i as f64 * 0.1).sin()]).collect();
/// let mut state = PlotState::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let response = Plot::new("signal")
///     .title("Signal")
///     .x_label("t (s)")
///     .y_label("V")
///     .height(280.0)
///     .series(Series::line("sin", &data))
///     .series(Series::scatter("samples", &data[..10]))
///     .show(ui, &mut icons, &mut state);
/// if let Some(hover) = response.hovered {
///     let _ = hover.point;
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Clone)]
pub struct Plot<'a> {
    id: Id,
    series: Vec<Series<'a>>,
    config: PlotConfig,
    width: Option<f32>,
    height: Option<f32>,
    x_window: Option<f64>,
    include: Bounds,
    settings_button: bool,
    export_button: bool,
}

impl<'a> Plot<'a> {
    pub(super) fn visible<'s>(&'s self, state: &'s PlotState) -> impl Iterator<Item = &'s Series<'a>> {
        self.series.iter().filter(|series| !state.is_hidden(&series.name))
    }

    /// The chart family: the first special series wins (pie, then radar, then gauge).
    /// Mixing families in one plot is not meaningful; cartesian series need no marker.
    pub fn chart_mode(&self) -> ChartMode {
        let mut mode = ChartMode::Cartesian;
        for series in &self.series {
            match series.kind {
                series::Kind::Pie { .. } => return ChartMode::Pie,
                series::Kind::Radar if mode == ChartMode::Cartesian => mode = ChartMode::Radar,
                series::Kind::Gauge { .. } if mode == ChartMode::Cartesian => mode = ChartMode::Gauge,
                _ => {}
            }
        }
        mode
    }

    /// Whether the direction arrow draws for series `index` after the overrides.
    pub(super) fn draws_direction(&self, index: usize) -> bool {
        self.config
            .series_style(&self.series[index].name)
            .and_then(|style| style.direction_arrow)
            .unwrap_or(self.config.direction.enabled)
    }

    /// The axis series `index` is measured on, after the config override.
    fn axis_of(&self, series: &Series<'_>) -> YAxis {
        self.config
            .series_style(&series.name)
            .and_then(|style| style.y_axis)
            .unwrap_or(series.axis)
    }

    /// Whether the right axis is shown (hidden series keep it, so the layout holds
    /// still while toggling them).
    fn has_y2(&self) -> bool {
        self.series.iter().any(|series| self.axis_of(series) == YAxis::Right)
    }

    /// The right axis for this frame: the state's range, or the automatic one.
    /// Keep `state.extents` to one current entry per series, computing only new ones.
    fn refresh_extents(&self, state: &mut PlotState) {
        let mut extents = Vec::with_capacity(self.series.len());
        for series in &self.series {
            let y_scale = match self.axis_of(series) {
                YAxis::Left => self.config.y.scale,
                YAxis::Right => self.config.y2.scale,
            };
            let key = series.fingerprint(self.config.x.scale, y_scale);
            let bounds = state
                .extents
                .iter()
                .find(|(cached, _)| *cached == key)
                .map_or_else(|| series.bounds(self.config.x.scale, y_scale), |(_, bounds)| *bounds);
            extents.push((key, bounds));
        }
        state.extents = extents;
    }

    fn second_axis(&self, state: &PlotState) -> Option<SecondAxis> {
        if !self.has_y2() {
            return None;
        }
        let [min, max] = state.y2_view.unwrap_or_else(|| self.auto_y2(state));
        Some(SecondAxis {
            min,
            max,
            scale: self.config.y2.scale,
        })
    }
}

impl Plot<'_> {
    /// Paint the plot and handle its interaction.
    pub fn show(self, ui: &mut Ui, icons: &mut Icons, state: &mut PlotState) -> PlotResponse {
        if state.pipeline.is_active() {
            let series = state.pipeline.process(&self.series);
            if !series.is_empty() {
                let processed = Plot {
                    id: self.id,
                    series,
                    config: self.config,
                    width: self.width,
                    height: self.height,
                    x_window: self.x_window,
                    include: self.include,
                    settings_button: self.settings_button,
                    export_button: self.export_button,
                };
                return processed.show_inner(ui, icons, state);
            }
        }
        self.show_inner(ui, icons, state)
    }

    fn show_inner(self, ui: &mut Ui, icons: &mut Icons, state: &mut PlotState) -> PlotResponse {
        let available = ui.available_size();
        let width = self.width.unwrap_or(available.x).max(120.0);
        let height = self
            .height
            .unwrap_or(if available.y.is_finite() && available.y < 4000.0 {
                available.y
            } else {
                320.0
            })
            .max(120.0);
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());
        let measure = Measure {
            ctx: ui.ctx().clone(),
            pixels_per_point: ui.ctx().pixels_per_point(),
        };
        self.refresh_extents(state);
        let view = state.view.unwrap_or_else(|| self.auto_bounds(state));
        let y2 = self.second_axis(state);
        let mode = self.chart_mode();
        let layout = overlay::layout(&measure, &self.config, rect, view, y2, mode);
        let transform = Transform {
            frame: layout.frame,
            view,
            x_scale: self.config.x.scale,
            y_scale: self.config.y.scale,
            y2,
        };

        let mut selection_changed = false;
        let pointer = response.hover_pos().filter(|pos| layout.frame.contains(*pos));
        if mode == ChartMode::Cartesian {
            self.interact(ui, &response, &transform, state, pointer, &mut selection_changed);
        } else if pointer.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
        }
        // The transform may have changed; paint with the new view this same frame.
        let view = state.view.unwrap_or(view);
        let y2 = y2.map(|axis| match state.y2_view {
            Some([min, max]) => SecondAxis { min, max, ..axis },
            None => axis,
        });
        let transform = Transform { view, y2, ..transform };

        // The legend interaction runs before the scene: the hovered row feeds the
        // animation the data layer draws this same frame.
        let now = ui.input(|input| input.time);
        let legend = if let Some(legend) = legend::layout(&measure, &self, layout.frame, state, true) {
            Some(widgets::legend(ui, &self, &legend, layout.frame, state))
        } else {
            state.set_legend_hover(None, now);
            None
        };
        let motion = crate::theme::motion_enabled(ui.ctx());
        let theme = crate::theme::current(ui.ctx());
        let colors = overlay::Colors::resolve(&self.config, ui.visuals());
        let mut scene = overlay::build_scene(
            &measure,
            &self,
            &layout,
            &transform,
            state,
            &colors,
            &theme,
            false,
            mode,
            now,
            motion,
            {
                #[cfg(feature = "gpu")]
                {
                    self.config.renderer == Renderer::Gpu
                }
                #[cfg(not(feature = "gpu"))]
                {
                    false
                }
            },
        );
        // The points each series drew; hover searches these instead of every point.
        let drawn = std::mem::take(&mut scene.drawn);
        annotations::paint(&mut scene.data, &transform, &state.annotations, &theme);
        measure::paint(&mut scene, &measure, &transform, &self, state, &theme);
        let painter = ui.painter_at(rect);
        #[cfg(feature = "gpu")]
        let gpu = self.config.renderer == Renderer::Gpu;
        #[cfg(feature = "gpu")]
        if gpu {
            let lines = gpu::collect(&self, &transform, state, &theme);
            gpu::paint_lines(ui, layout.frame, &lines, self.id);
        }
        scene::paint(&painter, &painter.with_clip_rect(layout.frame), scene);
        if let (Some(Drag::Box { start, select }), Some(current)) = (state.drag, response.interact_pointer_pos()) {
            let fill = colors.selection.gamma_multiply(if select { 0.12 } else { 0.06 });
            painter.with_clip_rect(layout.frame).rect(
                Rect::from_two_pos(start, current).intersect(layout.frame),
                0.0,
                fill,
                egui::Stroke::new(1.0, colors.selection),
                egui::StrokeKind::Inside,
            );
        }
        if let (Some(Drag::Measure { start, distance }), Some(current)) = (state.drag, response.interact_pointer_pos())
        {
            let clipped = painter.with_clip_rect(layout.frame);
            if distance || self.config.measurement.mode == MeasurementMode::CurveSegment {
                clipped.line_segment([start, current], egui::Stroke::new(1.0, colors.selection));
            } else {
                clipped.rect(
                    Rect::from_two_pos(start, current).intersect(layout.frame),
                    0.0,
                    colors.selection.gamma_multiply(0.06),
                    egui::Stroke::new(1.0, colors.selection),
                    egui::StrokeKind::Inside,
                );
            }
        }

        let toolbar = widgets::toolbar(ui, icons, &self, layout.frame, &transform, state);
        let blocked = |pos: Pos2| {
            legend.is_some_and(|rect| rect.contains(pos)) || toolbar.rect.is_some_and(|rect| rect.contains(pos))
        };
        if state.drag.is_none() && toolbar.rect.zip(pointer).is_some_and(|(rect, pos)| rect.contains(pos)) {
            // Buttons get the ordinary pointer, not the frame's crosshair.
            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
        }
        let hovered = pointer
            .filter(|pos| !blocked(*pos) && state.drag.is_none())
            .and_then(|pointer| {
                (mode == ChartMode::Cartesian)
                    .then(|| self.hover(ui, &transform, state, pointer, &theme, &drawn))
                    .flatten()
            });

        let clicked = if response.clicked() && state.drag.is_none() {
            hovered.clone()
        } else {
            None
        };
        if response.clicked() && state.tool == PlotTool::Select && hovered.is_none() && state.selection.is_some() {
            state.selection = None;
            selection_changed = true;
        }

        let (x_scale, y_scale) = (self.config.x.scale, self.config.y.scale);
        let data_view = Bounds::from_corners(
            [x_scale.inverse(view.min_x), y_scale.inverse(view.min_y)],
            [x_scale.inverse(view.max_x), y_scale.inverse(view.max_y)],
        );
        PlotResponse {
            response,
            pointer: pointer.map(|pos| transform.to_data(pos)),
            hovered,
            clicked,
            selection_changed,
            view: data_view,
            y2_view: y2.map(|axis| [axis.scale.inverse(axis.min), axis.scale.inverse(axis.max)]),
            frame: transform.frame,
            settings_requested: toolbar.settings,
            export_requested: toolbar.export,
            transform,
        }
    }
}

/// The default series colors: theme tokens first, then fixed hues.
fn palette(theme: &crate::theme::Theme, index: usize) -> Color32 {
    let colors = [
        theme.link,
        theme.warning,
        theme.success,
        theme.danger,
        theme.done,
        Color32::from_rgb(0x1f, 0xb5, 0xb5),
        Color32::from_rgb(0xd9, 0x5d, 0xb4),
        Color32::from_rgb(0x8c, 0x8c, 0x3c),
    ];
    colors[index % colors.len()]
}

impl IconComponent for Plot<'_> {
    type Output = PlotResponse;

    fn show(self, ui: &mut Ui, icons: &mut Icons) -> PlotResponse {
        let id = self.id;
        let mut state = ui
            .ctx()
            .data_mut(|data| data.get_temp::<PlotState>(id).unwrap_or_default());
        let response = Plot::show(self, ui, icons, &mut state);
        ui.ctx().data_mut(|data| data.insert_temp(id, state));
        response
    }
}

#[cfg(test)]
mod tests;
