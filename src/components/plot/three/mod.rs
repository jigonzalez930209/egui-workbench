//! A native 3D plot: orbit camera, x/y/z axes, surfaces, bars, lines, and scatter.
//!
//! It shares the crate's themes, icon cache, legend hover animations, direction arrow,
//! hover popup, and SVG/PNG export with [`super::Plot`], but has its own camera state.
//!
//! ```
//! use egui_workbench::components::plot::{Plot3D, Plot3DState, Series3D};
//! # let ctx = egui::Context::default();
//! # let mut icons = egui_workbench::Icons::new();
//! let mut state = Plot3DState::default();
//! let points: Vec<[f64; 3]> = (0..80).map(|index| {
//!     let angle = index as f64 * 0.2;
//!     [angle.cos() * 2.0, angle.sin() * 2.0, index as f64 * 0.08]
//! }).collect();
//! # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
//! Plot3D::new("helix")
//!     .title("Helix")
//!     .series(Series3D::line("orbit", &points))
//!     .show(ui, &mut icons, &mut state);
//! # });
//! # output.textures_delta.clear();
//! ```

mod axes;
mod builder;
mod camera;
mod config;
mod export;
mod interact;
mod layout;
mod legend;
mod paint;
mod scene;
mod series;
mod types;
mod widgets;

use egui::{Id, Pos2, Sense, Ui, vec2};

use super::layout::Colors;
use super::scene::Measure;
use crate::Icons;
use crate::components::IconComponent;
use camera::Projector;
pub use camera::{Camera3D, Projection};
pub use config::{Plot3DConfig, Series3DStyle, Surface3DConfig};
pub use series::{Series3D, Surface};
pub use types::{Bounds3D, Plot3DHover, Plot3DResponse, Plot3DState, Plot3DTool};

/// The 3D plot builder. Add series, then call `show` with the icon cache and a
/// [`Plot3DState`].
#[derive(Clone)]
pub struct Plot3D<'a> {
    pub(super) id: Id,
    pub(super) series: Vec<Series3D<'a>>,
    pub(super) config: Plot3DConfig,
    width: Option<f32>,
    height: Option<f32>,
    include: Bounds3D,
    pub(super) settings_button: bool,
    pub(super) export_button: bool,
}

impl<'a> Plot3D<'a> {
    /// Paint the plot and handle its interaction.
    pub fn show(self, ui: &mut Ui, icons: &mut Icons, state: &mut Plot3DState) -> Plot3DResponse {
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
        let bounds = self.auto_bounds(state);
        let layout = layout::layout(&measure, &self.config, rect);
        let camera = self.camera(state, bounds, layout.frame);
        interact::interact(ui, &response, layout.frame, state, camera);
        let camera = self.camera(state, bounds, layout.frame);
        let projector = Projector::new(camera, layout.frame);
        let pointer = response.hover_pos().filter(|pos| layout.frame.contains(*pos));

        let now = ui.input(|input| input.time);
        let legend = legend::layout(&measure, &self, layout.frame, state, true);
        let legend_rect = legend
            .as_ref()
            .map(|legend| widgets::legend(ui, &self, legend, layout.frame, state));
        let motion = crate::theme::motion_enabled(ui.ctx());
        let theme = crate::theme::current(ui.ctx());
        let colors = Colors::from_colors(&self.config.colors, ui.visuals());
        #[allow(unused_mut)]
        let (mut scene, drawn) = scene::build_scene(
            &measure, &self, &layout, &projector, bounds, state, &colors, &theme, false, now, motion,
        );
        #[cfg(feature = "gpu")]
        if self.config.renderer == super::Renderer::Gpu {
            let triangles = super::gpu::scene_triangles(&mut scene);
            super::gpu::paint_triangles(ui, layout.frame, &triangles, self.id);
        }
        let painter = ui.painter_at(rect);
        super::scene::paint(&painter, &painter.with_clip_rect(layout.frame), scene);

        let toolbar = widgets::toolbar(ui, icons, &self, layout.frame, camera, state);
        let blocked = |pos: Pos2| {
            legend_rect.is_some_and(|rect| rect.contains(pos)) || toolbar.rect.is_some_and(|rect| rect.contains(pos))
        };
        if state.drag.is_none() && toolbar.rect.zip(pointer).is_some_and(|(rect, pos)| rect.contains(pos)) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
        }
        let hovered = pointer
            .filter(|pos| !blocked(*pos) && state.drag.is_none())
            .and_then(|pos| interact::hover(ui, &self, &projector, pos, &theme, bounds, &drawn));
        let clicked = (response.clicked() && state.drag.is_none())
            .then(|| hovered.clone())
            .flatten();
        let view = Bounds3D {
            min_x: self.config.x.scale.inverse(bounds.min_x),
            max_x: self.config.x.scale.inverse(bounds.max_x),
            min_y: self.config.y.scale.inverse(bounds.min_y),
            max_y: self.config.y.scale.inverse(bounds.max_y),
            min_z: self.config.z.scale.inverse(bounds.min_z),
            max_z: self.config.z.scale.inverse(bounds.max_z),
        };
        Plot3DResponse {
            response,
            hovered,
            clicked,
            view,
            camera,
            frame: layout.frame,
            settings_requested: toolbar.settings,
            export_requested: toolbar.export,
        }
    }
}

impl IconComponent for Plot3D<'_> {
    type Output = Plot3DResponse;

    fn show(self, ui: &mut Ui, icons: &mut Icons) -> Plot3DResponse {
        let id = self.id;
        let mut state = ui
            .ctx()
            .data_mut(|data| data.get_temp::<Plot3DState>(id).unwrap_or_default());
        let response = Plot3D::show(self, ui, icons, &mut state);
        ui.ctx().data_mut(|data| data.insert_temp(id, state));
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plot<'a>() -> Plot3D<'a> {
        Plot3D::new("test")
            .series(Series3D::line("a", vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]))
            .series(Series3D::line("b", vec![[0.0, 1.0, 0.0], [1.0, 0.0, 1.0]]))
            .series(Series3D::line("c", vec![[1.0, 1.0, 1.0], [2.0, 2.0, 2.0]]))
    }

    #[test]
    fn the_hovered_series_draws_last_without_moving_the_legend() {
        let plot = plot();
        let mut state = Plot3DState::default();
        assert_eq!(plot.draw_order(&state), [0, 1, 2]);
        state.set_legend_hover(Some(0), 0.0);
        assert_eq!(plot.draw_order(&state), [1, 2, 0]);
        state.set_hidden("a", true);
        assert_eq!(plot.draw_order(&state), [0, 1, 2]);
    }

    #[test]
    fn auto_bounds_include_every_visible_series_and_the_include_point() {
        let plot = plot().include([4.0, 5.0, 6.0]);
        let mut state = Plot3DState::default();
        state.set_hidden("c", true);
        let bounds = plot.auto_bounds(&state);
        assert!(bounds.max_z > 6.0);
        assert!(bounds.max_x > 4.0 && bounds.max_y > 5.0);
    }

    #[test]
    fn builders_write_the_config() {
        let plot = Plot3D::new("builders")
            .title("3D")
            .x_label("x")
            .y_label("y")
            .z_label("z")
            .projection(Projection::Orthographic)
            .direction_arrow(true);
        assert_eq!(plot.config.title.as_deref(), Some("3D"));
        assert_eq!(plot.config.x.label.as_deref(), Some("x"));
        assert_eq!(plot.config.projection, Projection::Orthographic);
        assert!(plot.config.direction.enabled);
    }

    #[test]
    fn the_camera_round_trips_and_stays_above_the_data() {
        let bounds = Bounds3D::from_corners([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let frame = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(600.0, 400.0));
        let camera = Camera3D::auto(bounds, &Plot3DConfig::default(), frame);
        let projector = Projector::new(camera, frame);
        assert!(projector.project([0.0, 0.0, 0.0]).is_some());
        assert!(projector.eye()[2] > 0.0);
    }

    #[test]
    fn scene_primitives_stay_inside_the_frame() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        let measure = Measure {
            ctx: ctx.clone(),
            pixels_per_point: 1.0,
        };
        let plot = Plot3D::new("scene").series(Series3D::surface_fn(
            "terrain",
            -2.0..=2.0,
            -2.0..=2.0,
            [20, 20],
            |x, y| (x * x + y * y).sin(),
        ));
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(760.0, 520.0));
        let layout = layout::layout(&measure, &plot.config, rect);
        let bounds = plot.auto_bounds(&Plot3DState::default());
        let camera = Camera3D::auto(bounds, &plot.config, layout.frame);
        let projector = Projector::new(camera, layout.frame);
        let colors = Colors::resolve(&super::super::PlotConfig::default(), &ctx.style_of(ctx.theme()).visuals);
        let theme = crate::theme::Theme::dark();
        let (scene, _) = scene::build_scene(
            &measure,
            &plot,
            &layout,
            &projector,
            bounds,
            &Plot3DState::default(),
            &colors,
            &theme,
            false,
            0.0,
            false,
        );
        let mut largest = 0.0_f32;
        for prim in scene.under.iter().chain(&scene.data).chain(&scene.over) {
            if let super::super::scene::Prim::Line { points, .. } = prim {
                for pair in points.windows(2) {
                    largest = largest.max(pair[0].distance(pair[1]));
                }
            }
        }
        assert!(largest < layout.frame.width(), "a primitive spans {largest} pt");
    }

    #[test]
    fn edge_on_bar_faces_do_not_emit_spikes() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        let measure = Measure {
            ctx: ctx.clone(),
            pixels_per_point: 1.0,
        };
        let cells: Vec<[f64; 3]> = (0..8)
            .flat_map(|x| (0..8).map(move |y| [f64::from(x), f64::from(y), 1.0 + f64::from((x + y) % 5)]))
            .collect();
        let plot = Plot3D::new("bars").series(Series3D::bars("revenue", cells));
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1120.0, 760.0));
        let layout = layout::layout(&measure, &plot.config, rect);
        let bounds = plot.auto_bounds(&Plot3DState::default());
        let camera = Camera3D {
            projection: Projection::Orthographic,
            yaw: 0.02,
            pitch: 0.35,
            ..Camera3D::auto(bounds, &plot.config, layout.frame)
        };
        let projector = Projector::new(camera, layout.frame);
        let colors = Colors::resolve(&super::super::PlotConfig::default(), &ctx.style_of(ctx.theme()).visuals);
        let theme = crate::theme::Theme::dark();
        let (scene, _) = scene::build_scene(
            &measure,
            &plot,
            &layout,
            &projector,
            bounds,
            &Plot3DState::default(),
            &colors,
            &theme,
            false,
            0.0,
            false,
        );
        let limit = layout.frame.width().max(layout.frame.height()) * 1.5;
        for prim in scene.data.iter() {
            match prim {
                super::super::scene::Prim::Polygon { .. } => panic!("bar fills use quad meshes, not polygons"),
                super::super::scene::Prim::Line { points, .. } => {
                    for pair in points.windows(2) {
                        assert!(
                            pair[0].distance(pair[1]) < limit,
                            "a face outline spans {} pt",
                            pair[0].distance(pair[1])
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
