//! SVG and PNG export from the same scene the screen draws.

use std::fmt::Write as _;
use std::sync::Arc;

use egui::{Color32, Pos2};

use egui::{Context, Rect, Vec2};

use super::axis::Transform;
use super::scene::{self, Dash, Measure, Prim, Scene};
use super::{Plot, PlotState, overlay};

/// Common export sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExportPreset {
    /// The on-screen default, 960 × 540 pt.
    #[default]
    Screen,
    /// Half a letter page, 468 × 360 pt.
    HalfPage,
    /// A letter page, 612 × 792 pt.
    Letter,
    /// A single-column journal figure, 504 × 360 pt.
    Journal,
}

impl ExportPreset {
    /// Width and height in points.
    pub const fn size(self) -> [f32; 2] {
        match self {
            Self::Screen => [960.0, 540.0],
            Self::HalfPage => [468.0, 360.0],
            Self::Letter => [612.0, 792.0],
            Self::Journal => [504.0, 360.0],
        }
    }
}

/// Export settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExportOptions {
    /// Width in points (SVG user units).
    pub width: f32,
    /// Height in points.
    pub height: f32,
    /// PNG pixels per point (2 = retina-sharp).
    pub scale: f32,
    /// Leave the area around the data frame transparent instead of the theme's
    /// background.
    pub transparent: bool,
}

impl ExportOptions {
    /// The screen size, 2× scale, opaque background.
    pub fn for_preset(preset: ExportPreset) -> Self {
        let [width, height] = preset.size();
        Self {
            width,
            height,
            ..Self::default()
        }
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            width: 960.0,
            height: 540.0,
            scale: 2.0,
            transparent: false,
        }
    }
}

/// The font family written into SVG text; the PNG renderer loads egui's own fonts
/// under this family, so the raster matches the screen.
const FONT_FAMILY: &str = "Ubuntu, 'Ubuntu Light', 'Segoe UI', 'Helvetica Neue', Arial, sans-serif";

fn color(color: Color32) -> (String, f32) {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    (format!("#{r:02x}{g:02x}{b:02x}"), f32::from(a) / 255.0)
}

fn paint_attr(name: &str, value: Color32) -> String {
    let (hex, alpha) = color(value);
    if alpha <= 0.0 {
        return format!(" {name}=\"none\"");
    }
    if alpha >= 0.999 {
        format!(" {name}=\"{hex}\"")
    } else {
        format!(" {name}=\"{hex}\" {name}-opacity=\"{alpha:.3}\"")
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn points(points: &[Pos2]) -> String {
    let mut out = String::with_capacity(points.len() * 14);
    for (index, point) in points.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        let _ = write!(out, "{:.2},{:.2}", point.x, point.y);
    }
    out
}

fn prim(out: &mut String, prim: &Prim) {
    match prim {
        Prim::Line {
            points: line,
            width,
            color,
            dash,
        } => {
            let dash = match dash {
                Dash::Solid => String::new(),
                Dash::Dashed(dash, gap) => format!(" stroke-dasharray=\"{dash:.2} {gap:.2}\""),
                Dash::Dotted(spacing) => {
                    format!(" stroke-dasharray=\"0 {spacing:.2}\" stroke-linecap=\"round\"")
                }
                Dash::Marching { dash, gap, offset } => {
                    format!(" stroke-dasharray=\"{dash:.2} {gap:.2}\" stroke-dashoffset=\"{offset:.2}\"")
                }
            };
            let _ = writeln!(
                out,
                "<polyline points=\"{}\" fill=\"none\"{} stroke-width=\"{width:.2}\" stroke-linejoin=\"round\"{dash}/>",
                points(line),
                paint_attr("stroke", *color),
            );
        }
        Prim::Area {
            points: line,
            base_y,
            color,
        } => {
            if let (Some(first), Some(last)) = (line.first(), line.last()) {
                let mut polygon = line.clone();
                polygon.push(Pos2::new(last.x, *base_y));
                polygon.push(Pos2::new(first.x, *base_y));
                let _ = writeln!(
                    out,
                    "<polygon points=\"{}\"{}/>",
                    points(&polygon),
                    paint_attr("fill", *color)
                );
            }
        }
        Prim::Polygon { points: shape, color } => {
            let _ = writeln!(
                out,
                "<polygon points=\"{}\"{}/>",
                points(shape),
                paint_attr("fill", *color)
            );
        }
        Prim::Quad { points: shape, color } => {
            let _ = writeln!(
                out,
                "<polygon points=\"{}\"{}/>",
                points(shape),
                paint_attr("fill", *color)
            );
        }
        Prim::Fan {
            center,
            points: shape,
            color,
        } => {
            let mut polygon = Vec::with_capacity(shape.len() + 1);
            polygon.push(*center);
            polygon.extend_from_slice(shape);
            let _ = writeln!(
                out,
                "<polygon points=\"{}\"{}/>",
                points(&polygon),
                paint_attr("fill", *color)
            );
        }
        Prim::Rect {
            rect,
            radius,
            fill,
            stroke,
        } => {
            // Strokes are inside the rectangle, as on screen.
            let inset = stroke.map_or(0.0, |(width, _)| width * 0.5);
            let rect = rect.shrink(inset);
            let stroke = stroke.map_or_else(String::new, |(width, color)| {
                format!("{} stroke-width=\"{width:.2}\"", paint_attr("stroke", color))
            });
            let _ = writeln!(
                out,
                "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"{radius:.2}\"{}{stroke}/>",
                rect.left(),
                rect.top(),
                rect.width().max(0.0),
                rect.height().max(0.0),
                paint_attr("fill", *fill),
            );
        }
        Prim::Circle {
            center,
            radius,
            fill,
            stroke,
        } => {
            let stroke = stroke.map_or_else(String::new, |(width, color)| {
                format!("{} stroke-width=\"{width:.2}\"", paint_attr("stroke", color))
            });
            let _ = writeln!(
                out,
                "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{radius:.2}\"{}{stroke}/>",
                center.x,
                center.y,
                paint_attr("fill", *fill),
            );
        }
        Prim::Text {
            pos,
            text,
            size,
            color,
            baseline,
            angle,
        } => {
            let transform = if *angle == 0.0 {
                String::new()
            } else {
                format!(
                    " transform=\"rotate({:.3} {:.2} {:.2})\"",
                    angle.to_degrees(),
                    pos.x,
                    pos.y
                )
            };
            let _ = writeln!(
                out,
                "<text xml:space=\"preserve\" x=\"{:.2}\" y=\"{:.2}\" font-size=\"{size:.2}\"{}{transform}>{}</text>",
                pos.x,
                pos.y + baseline,
                paint_attr("fill", *color),
                escape(text),
            );
        }
    }
}

/// Write a scene as a standalone SVG document. `canvas` fills the whole image.
pub(super) fn svg(scene: &Scene, canvas: Option<Color32>) -> String {
    let (width, height) = (scene.size.x, scene.size.y);
    let mut out = String::with_capacity(64 * 1024);
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" \
         viewBox=\"0 0 {width:.2} {height:.2}\" font-family=\"{FONT_FAMILY}\" font-weight=\"300\">"
    );
    let frame = scene.frame;
    let _ = writeln!(
        out,
        "<defs><clipPath id=\"plot-frame\"><rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/></clipPath></defs>",
        frame.left(),
        frame.top(),
        frame.width(),
        frame.height()
    );
    if let Some(canvas) = canvas {
        let _ = writeln!(
            out,
            "<rect width=\"100%\" height=\"100%\"{}/>",
            paint_attr("fill", canvas)
        );
    }
    for primitive in &scene.under {
        prim(&mut out, primitive);
    }
    out.push_str("<g clip-path=\"url(#plot-frame)\">\n");
    for primitive in &scene.data {
        prim(&mut out, primitive);
    }
    out.push_str("</g>\n");
    for primitive in &scene.over {
        prim(&mut out, primitive);
    }
    out.push_str("</svg>\n");
    out
}

/// Most pixels a PNG export may allocate (about 400 MB of RGBA).
const MAX_PIXELS: u64 = 100_000_000;

/// Rasterize an SVG produced by [`svg`] at `scale` pixels per point, with egui's
/// default fonts registered for the text.
pub(super) fn png(svg: &str, scale: f32) -> Result<Vec<u8>, String> {
    let mut options = usvg::Options::default();
    {
        let database = Arc::make_mut(&mut options.fontdb);
        for data in egui::FontDefinitions::default().font_data.values() {
            database.load_font_data((*data.font).as_ref().to_vec());
        }
    }
    options.font_family = "Ubuntu".to_owned();
    let tree = usvg::Tree::from_str(svg, &options).map_err(|error| format!("invalid SVG: {error}"))?;
    let size = tree.size();
    let scale = scale.clamp(0.25, 8.0);
    let width = (size.width() * scale).round().max(1.0) as u32;
    let height = (size.height() * scale).round().max(1.0) as u32;
    // `width * scale` is caller-controlled; 8x of a 100000-point export is 640 gigapixels.
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err("the image is too large".to_owned());
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or("the image is too large")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap
        .encode_png()
        .map_err(|error| format!("PNG encoding failed: {error}"))
}

impl Plot<'_> {
    /// The scene for an export of `options.width × options.height` points.
    fn export_scene(
        &self,
        ctx: &Context,
        state: &PlotState,
        options: &ExportOptions,
    ) -> (scene::Scene, Option<Color32>) {
        let measure = Measure {
            ctx: ctx.clone(),
            pixels_per_point: 1.0,
        };
        let rect = Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(options.width.max(120.0), options.height.max(120.0)),
        );
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
        let visuals = ctx.style_of(ctx.theme()).visuals.clone();
        let mut colors = overlay::Colors::resolve(&self.config, &visuals);
        let canvas = if options.transparent {
            colors.canvas.take()
        } else {
            Some(colors.canvas.take().unwrap_or(visuals.panel_fill))
        };
        let theme = crate::theme::current(ctx);
        let mut scene = overlay::build_scene(
            &measure, self, &layout, &transform, state, &colors, &theme, true, mode, 0.0, false, false,
        );
        super::annotations::paint(&mut scene.data, &transform, &state.annotations, &theme);
        (scene, canvas)
    }

    /// The plot as a standalone SVG document: the same view, colors, legend, and
    /// selection as on screen, without the toolbar and hover popup.
    pub fn to_svg(&self, ctx: &Context, state: &PlotState, options: &ExportOptions) -> String {
        let (scene, canvas) = self.export_scene(ctx, state, options);
        svg(&scene, canvas)
    }

    /// The plot as a one-page vector PDF.
    pub fn to_pdf(&self, ctx: &Context, state: &PlotState, options: &ExportOptions) -> Vec<u8> {
        let (scene, _) = self.export_scene(ctx, state, options);
        super::pdf::write(&scene, options)
    }

    /// The plot as PNG bytes at `options.scale` pixels per point, rendered from the SVG
    /// with egui's fonts.
    ///
    /// # Errors
    /// When the image is too large to allocate or encoding fails.
    pub fn to_png(&self, ctx: &Context, state: &PlotState, options: &ExportOptions) -> Result<Vec<u8>, String> {
        png(&self.to_svg(ctx, state, options), options.scale)
    }
}
