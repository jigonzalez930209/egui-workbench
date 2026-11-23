//! SVG and PNG export from the same 3D scene the screen draws.

use egui::{Color32, Context, Pos2, Rect, Vec2};

use super::super::export::{png, svg};
use super::super::scene::{Measure, Scene};
use super::super::{ExportOptions, layout::Colors};
use super::camera::Projector;
use super::layout;
use super::{Plot3D, Plot3DState};

impl Plot3D<'_> {
    fn export_scene(&self, ctx: &Context, state: &Plot3DState, options: &ExportOptions) -> (Scene, Option<Color32>) {
        let measure = Measure {
            ctx: ctx.clone(),
            pixels_per_point: 1.0,
        };
        let rect = Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(options.width.max(120.0), options.height.max(120.0)),
        );
        let layout = layout::layout(&measure, &self.config, rect);
        let bounds = self.auto_bounds(state);
        let camera = self.camera(state, bounds, layout.frame);
        let projector = Projector::new(camera, layout.frame);
        let visuals = ctx.style_of(ctx.theme()).visuals.clone();
        let mut colors = Colors::from_colors(&self.config.colors, &visuals);
        let canvas = if options.transparent {
            colors.canvas.take()
        } else {
            Some(colors.canvas.take().unwrap_or(visuals.panel_fill))
        };
        let theme = crate::theme::current(ctx);
        let (scene, _) = super::scene::build_scene(
            &measure, self, &layout, &projector, bounds, state, &colors, &theme, true, 0.0, false,
        );
        (scene, canvas)
    }

    /// The 3D plot as a standalone SVG document.
    pub fn to_svg(&self, ctx: &Context, state: &Plot3DState, options: &ExportOptions) -> String {
        let (scene, canvas) = self.export_scene(ctx, state, options);
        svg(&scene, canvas)
    }

    /// The 3D plot as PNG bytes at `options.scale` pixels per point.
    ///
    /// # Errors
    /// When the image is too large to allocate or encoding fails.
    pub fn to_png(&self, ctx: &Context, state: &Plot3DState, options: &ExportOptions) -> Result<Vec<u8>, String> {
        png(&self.to_svg(ctx, state, options), options.scale)
    }
}
