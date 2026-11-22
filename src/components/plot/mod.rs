//! A fast, interactive 2D plot in the spirit of matplotlib.
//!
//! - Series: lines, areas, steps, scatter with seven markers, bars, error bars,
//!   functions, and horizontal/vertical reference lines.
//! - Axes: linear or log10 scales, 1-2-5 ticks that never overlap, minor log ticks,
//!   grid, and a title and axis titles with their own size, color, and alignment; every line is snapped to the pixel grid.
//! - Two y axes: series marked [`YAxis::Right`] get their own axis on the right, with
//!   its own range, scale, ticks, and title.
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
//! The view, the selection, the active tool, and hidden series live in a
//! [`PlotState`] the caller owns, like every other component.

mod axis;
mod builder;
mod config;
mod export;
mod fit;
mod interact;
mod legend;
mod overlay;
mod paint;
mod scene;
mod series;
mod smooth;
mod transform;
mod widgets;

use egui::{Color32, Id, Pos2, Rect, Response, Sense, Ui, vec2};

pub use axis::{Bounds, Scale};
pub use config::{AxisConfig, Minor, PlotColors, PlotConfig, SeriesStyle, TitleAlign};
pub use export::ExportOptions;
pub use fit::{Fit, FitKind};
pub use series::{LineStyle, Marker, Series, Smoothing, YAxis, points_in};

use crate::Icons;
use crate::components::IconComponent;
use axis::{SecondAxis, Transform};
use scene::Measure;

/// A file format the toolbar's export menu asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    /// Scalable vector graphics.
    Svg,
    /// A raster image.
    Png,
}

/// What a primary-button drag does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlotTool {
    /// Drag moves the view.
    #[default]
    Pan,
    /// Drag draws a box and zooms to it.
    Zoom,
    /// Drag draws a box and reports it as the selection.
    Select,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Drag {
    Pan,
    Box { start: Pos2, select: bool },
}

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

/// The nearest data point under the pointer.
#[derive(Clone, Debug, PartialEq)]
pub struct PlotHover {
    /// Series name.
    pub series: String,
    /// Index into the series data.
    pub index: usize,
    /// The point, in data coordinates.
    pub point: [f64; 2],
    /// The y axis the series is measured on.
    pub axis: YAxis,
}

/// What happened in the plot this frame.
pub struct PlotResponse {
    /// The response of the whole plot area.
    pub response: Response,
    /// The pointer position in data coordinates, while over the frame.
    pub pointer: Option<[f64; 2]>,
    /// The nearest point within 12 pt of the pointer.
    pub hovered: Option<PlotHover>,
    /// A point that was clicked.
    pub clicked: Option<PlotHover>,
    /// The selection changed (drawn, or cleared with a click).
    pub selection_changed: bool,
    /// The visible range in data coordinates (x and the left y axis).
    pub view: Bounds,
    /// The visible range of the right y axis in data coordinates, when it is shown.
    pub y2_view: Option<[f64; 2]>,
    /// The data area on screen (inside the axes).
    pub frame: Rect,
    /// The toolbar's settings button was clicked: open your settings dialog.
    pub settings_requested: bool,
    /// The toolbar's export menu chose a format: call [`Plot::to_svg`] or
    /// [`Plot::to_png`] and save the result.
    pub export_requested: Option<ExportFormat>,
    transform: Transform,
}

impl PlotResponse {
    /// Where a data point lands on screen this frame, for annotations drawn on top.
    pub fn screen_pos(&self, point: [f64; 2]) -> Pos2 {
        self.transform.to_screen(point)
    }

    /// Where a point measured on `axis` lands on screen this frame.
    pub fn screen_pos_on(&self, axis: YAxis, point: [f64; 2]) -> Pos2 {
        self.transform.for_axis(axis).to_screen(point)
    }

    /// The data coordinates of a screen position.
    pub fn data_pos(&self, pos: Pos2) -> [f64; 2] {
        self.transform.to_data(pos)
    }
}

/// Where the legend sits inside the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LegendPosition {
    /// Top-left corner.
    #[default]
    TopLeft,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    BottomRight,
    /// No legend.
    Hidden,
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
            ctx: ui.ctx(),
            pixels_per_point: ui.ctx().pixels_per_point(),
        };
        let view = state.view.unwrap_or_else(|| self.auto_bounds(state));
        let y2 = self.second_axis(state);
        let layout = overlay::layout(&measure, &self.config, rect, view, y2);
        let transform = Transform {
            frame: layout.frame,
            view,
            x_scale: self.config.x.scale,
            y_scale: self.config.y.scale,
            y2,
        };

        let mut selection_changed = false;
        let pointer = response.hover_pos().filter(|pos| layout.frame.contains(*pos));
        self.interact(ui, &response, &transform, state, pointer, &mut selection_changed);
        // The transform may have changed; paint with the new view this same frame.
        let view = state.view.unwrap_or(view);
        let y2 = y2.map(|axis| match state.y2_view {
            Some([min, max]) => SecondAxis { min, max, ..axis },
            None => axis,
        });
        let transform = Transform { view, y2, ..transform };

        let theme = crate::theme::current(ui.ctx());
        let colors = overlay::Colors::resolve(&self.config, ui.visuals());
        let scene = overlay::build_scene(&measure, &self, &layout, &transform, state, &colors, &theme, false);
        let painter = ui.painter_at(rect);
        scene::paint(&painter, &painter.with_clip_rect(layout.frame), &scene);
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

        let legend = legend::layout(&measure, &self, layout.frame, state, true)
            .map(|legend| widgets::legend(ui, &self, &legend, layout.frame, state));
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
            .and_then(|pointer| self.hover(ui, &transform, state, pointer, &theme));

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
