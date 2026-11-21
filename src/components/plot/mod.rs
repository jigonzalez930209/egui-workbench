//! A fast, interactive 2D plot in the spirit of matplotlib.
//!
//! - Series: lines, areas, steps, scatter with seven markers, bars, error bars,
//!   functions, and horizontal/vertical reference lines.
//! - Axes: linear or log10 scales, 1-2-5 ticks that never overlap, minor log ticks,
//!   grid, title, and axis labels; every line is snapped to the pixel grid.
//! - Interaction: drag to pan, wheel to zoom around the cursor (Shift: x only, Alt: y
//!   only), right-drag or the zoom tool to zoom to a box, Shift-drag or the select tool
//!   to select an area, double-click to fit the data again, and a hover popup with the
//!   nearest point. A small toolbar switches tools; the legend toggles series.
//! - Analysis: least-squares fits ([`Fit`]) with their equation and R².
//! - Speed: dense sorted lines are decimated per pixel column, so a million points
//!   draw in a few milliseconds; nothing repaints while idle.
//!
//! The view, the selection, the active tool, and hidden series live in a
//! [`PlotState`] the caller owns, like every other component.

mod axis;
mod config;
mod export;
mod fit;
mod interact;
mod legend;
mod overlay;
mod paint;
mod scene;
mod series;
mod widgets;

use egui::{Color32, Id, Pos2, Rect, Response, Sense, Ui, vec2};

pub use axis::{Bounds, Scale};
pub use config::{AxisConfig, Minor, PlotColors, PlotConfig, SeriesStyle};
pub use export::ExportOptions;
pub use fit::{Fit, FitKind};
pub use series::{LineStyle, Marker, Series, points_in};

use crate::Icons;
use crate::components::IconComponent;
use axis::Transform;
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
}

impl PlotState {
    /// Fit the view to the data again (also bound to double-click and the toolbar).
    pub fn reset_view(&mut self) {
        self.view = None;
    }

    /// Whether the view follows the data (no zoom or pan since the last reset).
    pub fn is_auto(&self) -> bool {
        self.view.is_none()
    }

    /// Show `bounds` (data coordinates, linear axes).
    pub fn set_view(&mut self, bounds: Bounds) {
        self.view = Some(bounds);
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
    /// The visible range in data coordinates.
    pub view: Bounds,
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
    /// A plot with a stable id.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            id: Id::unique(("workbench_plot", id)),
            series: Vec::new(),
            config: PlotConfig::default(),
            width: None,
            height: None,
            x_window: None,
            include: Bounds::NOTHING,
            settings_button: false,
            export_button: false,
        }
    }

    /// Replace the whole appearance (text, sizes, colors, axes, grid, legend, and
    /// per-series styles). Call it first; the other builder methods edit it.
    pub fn config(mut self, config: PlotConfig) -> Self {
        self.config = config;
        self
    }

    /// The current appearance.
    pub fn current_config(&self) -> &PlotConfig {
        &self.config
    }

    /// The names of the series, in drawing order (for a settings dialog).
    pub fn series_names(&self) -> Vec<String> {
        self.series.iter().map(|series| series.name.clone()).collect()
    }

    /// Fixed x limits in data units (`None` keeps that side automatic).
    pub fn x_limits(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        (self.config.x.min, self.config.x.max) = (min, max);
        self
    }

    /// Fixed y limits in data units (`None` keeps that side automatic).
    pub fn y_limits(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        (self.config.y.min, self.config.y.max) = (min, max);
        self
    }

    /// Show a settings button in the toolbar; see [`PlotResponse::settings_requested`].
    pub fn settings_button(mut self, show: bool) -> Self {
        self.settings_button = show;
        self
    }

    /// Show an SVG/PNG export menu in the toolbar; see
    /// [`PlotResponse::export_requested`].
    pub fn export_button(mut self, show: bool) -> Self {
        self.export_button = show;
        self
    }

    /// Add a series. Series draw in order; the palette color follows the order too.
    pub fn series(mut self, series: Series<'a>) -> Self {
        self.series.push(series);
        self
    }

    /// Fit `kind` to `data` and add the dashed curve, named with its equation and R².
    /// Nothing is added when the fit fails (too few points, singular system).
    pub fn fit(mut self, kind: FitKind, data: &[[f64; 2]]) -> Self {
        if let Some(fit) = Fit::new(kind, data) {
            self.series.push(fit.series(fit.equation(), 200));
        }
        self
    }

    /// A title above the frame.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    /// The x axis label (also used in the hover popup).
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.config.x.label = Some(label.into());
        self
    }

    /// The y axis label, drawn vertically.
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.config.y.label = Some(label.into());
        self
    }

    /// Linear or log10 x axis.
    pub fn x_scale(mut self, scale: Scale) -> Self {
        self.config.x.scale = scale;
        self
    }

    /// Linear or log10 y axis.
    pub fn y_scale(mut self, scale: Scale) -> Self {
        self.config.y.scale = scale;
        self
    }

    /// Fixed width; fills the available width otherwise.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Fixed height; fills the available height otherwise (320 pt in scroll areas).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Legend corner, or [`LegendPosition::Hidden`].
    pub fn legend(mut self, legend: LegendPosition) -> Self {
        self.config.legend = legend;
        self
    }

    /// Draw the grid (on by default).
    pub fn grid(mut self, grid: bool) -> Self {
        for axis in [&mut self.config.x, &mut self.config.y] {
            axis.grid_major = grid;
            axis.grid_minor = grid;
        }
        self
    }

    /// Show the tool buttons in the top-right corner (on by default).
    pub fn toolbar(mut self, toolbar: bool) -> Self {
        self.config.toolbar = toolbar;
        self
    }

    /// For streaming data: while the view is automatic, show only the last `width`
    /// x units.
    pub fn x_window(mut self, width: f64) -> Self {
        self.x_window = Some(width);
        self
    }

    /// Keep `point` inside the automatic view (for example `[0.0, 0.0]`).
    pub fn include(mut self, point: [f64; 2]) -> Self {
        self.include.extend(point);
        self
    }

    pub(super) fn visible<'s>(&'s self, state: &'s PlotState) -> impl Iterator<Item = &'s Series<'a>> {
        self.series.iter().filter(|series| !state.is_hidden(&series.name))
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
        let layout = overlay::layout(&measure, &self.config, rect, view);
        let transform = Transform {
            frame: layout.frame,
            view,
            x_scale: self.config.x.scale,
            y_scale: self.config.y.scale,
        };

        let mut selection_changed = false;
        let pointer = response.hover_pos().filter(|pos| layout.frame.contains(*pos));
        self.interact(ui, &response, &transform, state, pointer, &mut selection_changed);
        // The transform may have changed; paint with the new view this same frame.
        let view = state.view.unwrap_or(view);
        let transform = Transform { view, ..transform };

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
            .map(|legend| widgets::legend(ui, &self, &legend, state));
        let toolbar = widgets::toolbar(ui, icons, &self, layout.frame, &transform, state);
        let blocked = |pos: Pos2| {
            legend.is_some_and(|rect| rect.contains(pos)) || toolbar.rect.is_some_and(|rect| rect.contains(pos))
        };
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
