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

mod animation;
mod axis;
mod builder;
mod chart_config;
mod config;
mod decimate;
mod direction;
mod export;
mod extra;
mod fit;
mod interact;
mod layout;
mod legend;
mod overlay;
mod paint;
mod scene;
mod series;
mod series_bounds;
mod series_charts;
mod smooth;
mod special;
mod three;
mod transform;
mod types;
mod widgets;

use egui::{Color32, Id, Pos2, Rect, Sense, Ui, vec2};

pub use animation::{AnimationEasing, LegendAnimation, LegendHoverConfig};
pub use axis::{Bounds, Scale};
pub use chart_config::{DirectionArrowConfig, GaugeConfig, HeatmapConfig, PieConfig, RadarConfig, WaterfallConfig};
pub use config::{AxisConfig, Minor, PlotColors, PlotConfig, SeriesStyle, TitleAlign};
pub use export::ExportOptions;
pub use fit::{Fit, FitKind};
pub use series::{LineStyle, Marker, Series, Smoothing, YAxis, points_in};
pub use three::{
    Bounds3D, Camera3D, Plot3D, Plot3DConfig, Plot3DHover, Plot3DResponse, Plot3DState, Plot3DTool, Projection,
    Series3D, Series3DStyle, Surface, Surface3DConfig,
};

pub use types::{ChartMode, ExportFormat, LegendPosition, PlotHover, PlotResponse, PlotTool};

use crate::Icons;
use crate::components::IconComponent;
use axis::{SecondAxis, Transform};
use scene::Measure;
use types::Drag;

/// The interactive state of one plot. Keep it in the application model.
#[derive(Clone, Debug, Default)]
pub struct PlotState {
    view: Option<Bounds>,
    /// The tool the primary button uses.
    pub tool: PlotTool,
    /// The selected area in data coordinates.
    pub selection: Option<Bounds>,
    hidden: Vec<String>,
    drag: Option<Drag>,
    export_menu: bool,
    /// The right axis range in scaled units, once zoomed or panned.
    y2_view: Option<[f64; 2]>,
    /// Where the dragged legend sits: 0–1 across the room left in the frame.
    legend_pos: Option<[f32; 2]>,
    /// Series extents by [`Series::fingerprint`], so the automatic view does not walk
    /// every point of every series each frame.
    extents: Vec<(u64, Bounds)>,
    /// The series whose legend row the pointer is over.
    legend_hover: Option<usize>,
    /// When that hover began, for the animation clock.
    legend_hover_start: f64,
}

impl PlotState {
    /// The cached extent of `series`, if it is current.
    fn cached_bounds(&self, series: &Series<'_>, x_scale: Scale, y_scale: Scale) -> Bounds {
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

    /// Show `bounds` (data coordinates, linear axes) on the x and left y axes.
    pub fn set_view(&mut self, bounds: Bounds) {
        self.view = Some(bounds);
    }

    /// Show `min..=max` (data coordinates, linear axis) on the right y axis.
    pub fn set_y2_view(&mut self, min: f64, max: f64) {
        self.y2_view = Some([min.min(max), min.max(max)]);
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
            &measure, &self, &layout, &transform, state, &colors, &theme, false, mode, now, motion,
        );
        // The points each series drew; hover searches these instead of every point.
        let drawn = std::mem::take(&mut scene.drawn);
        let painter = ui.painter_at(rect);
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
mod tests {
    use super::*;

    #[test]
    fn special_series_choose_the_chart_mode() {
        let pie = Plot::new("pie").series(Series::pie("a", 1.0));
        assert_eq!(pie.chart_mode(), ChartMode::Pie);
        let radar = Plot::new("radar").series(Series::radar("a", [1.0, 2.0, 3.0]));
        assert_eq!(radar.chart_mode(), ChartMode::Radar);
        let gauge = Plot::new("gauge").series(Series::gauge("a", 4.0, 0.0..=10.0));
        assert_eq!(gauge.chart_mode(), ChartMode::Gauge);
        let candles = Plot::new("candles").series(Series::candles("a", vec![[0.0, 1.0, 2.0, 0.5, 1.5]]));
        assert_eq!(candles.chart_mode(), ChartMode::Cartesian);
    }

    #[test]
    fn direction_and_legend_hover_builders_write_the_config() {
        let plot = Plot::new("p")
            .direction_arrow(true)
            .direction_points(9)
            .legend_animation(LegendAnimation::DashMarch);
        assert!(plot.config.direction.enabled);
        assert_eq!(plot.config.direction.points, 9);
        assert_eq!(plot.config.legend_hover.animation, LegendAnimation::DashMarch);
    }
}
