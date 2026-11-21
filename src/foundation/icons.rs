//! Monochrome UI icons, rasterized from SVG once and tinted at draw time.
//!
//! Every glyph is a 24×24 SVG drawn in white (`#fff`) strokes or fills, so a single
//! texture can be tinted with any token color. Applications add their own glyphs with
//! [`Icons::register`] (the product `logo` is the usual one); a registered name
//! replaces a bundled glyph of the same name.

use std::collections::HashMap;

use egui::{Color32, Context, Image, Pos2, Rect, TextureHandle, TextureOptions, Ui, Vec2};
use resvg::tiny_skia::{Pixmap, Transform};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/ui-icons"]
struct UiAssets;

/// Pixel size of every UI glyph texture: crisp up to 2× at 16 pt.
const GLYPH_PIXELS: u32 = 48;

/// The UV rectangle of a whole texture.
pub const FULL_UV: Rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));

/// UI glyph cache, plus the Material file icons when that feature is on.
pub struct Icons {
    custom: HashMap<String, Vec<u8>>,
    textures: HashMap<String, TextureHandle>,
    /// File and folder icons from the VS Code Material Icon Theme.
    #[cfg(feature = "material-icons")]
    pub material: crate::MaterialIcons,
}

impl Default for Icons {
    fn default() -> Self {
        Self::new()
    }
}

impl Icons {
    /// An empty cache over the bundled glyphs.
    pub fn new() -> Self {
        Self {
            custom: HashMap::new(),
            textures: HashMap::new(),
            #[cfg(feature = "material-icons")]
            material: crate::MaterialIcons::new(),
        }
    }

    /// The names of the bundled UI glyphs, sorted.
    pub fn bundled() -> Vec<String> {
        let mut names: Vec<String> = UiAssets::iter()
            .filter_map(|path| path.strip_suffix(".svg").map(str::to_owned))
            .collect();
        names.sort();
        names
    }

    /// Add or replace a glyph. Draw it in white so tinting works.
    pub fn register(&mut self, name: &str, svg: impl Into<Vec<u8>>) {
        self.custom.insert(name.to_owned(), svg.into());
        let sized = format!("{name}@");
        self.textures.retain(|key, _| key != name && !key.starts_with(&sized));
    }

    /// The names of the bundled Lucide icons, sorted.
    ///
    /// Lucide names are resolved after the UI glyphs, so `save` keeps the workbench
    /// glyph and `lucide-save` forces Lucide. `trash-2` (a name only Lucide has) also
    /// resolves. Both are drawn through the same white-tint pipeline.
    #[cfg(feature = "lucide-icons")]
    pub fn lucide_names() -> Vec<String> {
        crate::foundation::lucide::names()
    }

    /// Lucide icons matching `query`, best match first. Exact substring matches rank
    /// before subsequence matches; an empty query returns every name.
    #[cfg(feature = "lucide-icons")]
    pub fn lucide_search(query: &str) -> Vec<String> {
        crate::foundation::lucide::search(query)
    }

    /// The texture of a glyph, rasterized on first use.
    ///
    /// Resolution order: registered custom SVG, bundled UI glyph, Lucide icon. Prefix a
    /// name with `lucide-` to skip the UI set when both have it.
    pub fn texture(&mut self, ctx: &Context, name: &str) -> Option<TextureHandle> {
        self.texture_px(ctx, name, GLYPH_PIXELS)
    }

    /// The texture of a glyph that stays crisp at `size` points on the current display.
    ///
    /// Small glyphs share the 48 px texture; larger ones (gallery previews, empty
    /// states, toasts on HiDPI screens) are rasterized again at 96, 192, or 384 px.
    pub fn texture_sized(&mut self, ctx: &Context, name: &str, size: f32) -> Option<TextureHandle> {
        let needed = size * ctx.pixels_per_point();
        let pixels = [GLYPH_PIXELS, GLYPH_PIXELS * 2, GLYPH_PIXELS * 4, GLYPH_PIXELS * 8]
            .into_iter()
            .find(|&pixels| pixels as f32 >= needed)
            .unwrap_or(GLYPH_PIXELS * 8);
        self.texture_px(ctx, name, pixels)
    }

    fn texture_px(&mut self, ctx: &Context, name: &str, pixels: u32) -> Option<TextureHandle> {
        let key = if pixels == GLYPH_PIXELS {
            name.to_owned()
        } else {
            format!("{name}@{pixels}")
        };
        if let Some(texture) = self.textures.get(&key) {
            return Some(texture.clone());
        }
        let bytes = match self.custom.get(name) {
            Some(bytes) => bytes.clone(),
            None => bundled_svg(name)?,
        };
        let image = rasterize(&bytes, pixels)?;
        let texture = ctx.load_texture(format!("workbench-icon-{key}"), image, TextureOptions::LINEAR);
        self.textures.insert(key, texture.clone());
        Some(texture)
    }

    /// Paint a glyph centered in `rect` at `size`, tinted with `color`.
    pub fn paint(&mut self, ui: &Ui, name: &str, center: Pos2, size: f32, color: Color32) -> bool {
        let Some(texture) = self.texture_sized(ui.ctx(), name, size) else {
            return false;
        };
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        ui.painter().image(texture.id(), rect, FULL_UV, color);
        true
    }

    /// Place a glyph as a widget at `center` with `ui.put`, like MD View's title-bar and
    /// tab-strip buttons. Unlike [`Self::paint`], this moves the layout cursor to the
    /// glyph's edge, so the next button in the row starts there and overlaps the previous
    /// cell. That gives the workbench its compact button pitch (see `metrics`).
    pub fn put(&mut self, ui: &mut Ui, name: &str, center: Pos2, size: f32, color: Color32) -> bool {
        let Some(texture) = self.texture_sized(ui.ctx(), name, size) else {
            return false;
        };
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        ui.put(
            rect,
            Image::new(&texture).fit_to_exact_size(Vec2::splat(size)).tint(color),
        );
        true
    }

    /// Add a glyph at the cursor, tinted with the text color. Reserves the slot even
    /// when the glyph is missing, so rows never shift.
    pub fn show(&mut self, ui: &mut Ui, name: &str, size: f32) {
        self.show_tinted(ui, name, size, ui.visuals().text_color());
    }

    /// Add a glyph at the cursor with an explicit tint.
    pub fn show_tinted(&mut self, ui: &mut Ui, name: &str, size: f32, color: Color32) {
        if let Some(texture) = self.texture_sized(ui.ctx(), name, size) {
            ui.add(Image::new(&texture).fit_to_exact_size(Vec2::splat(size)).tint(color));
        } else {
            ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
        }
    }
}

fn bundled_svg(name: &str) -> Option<Vec<u8>> {
    if let Some(name) = name.strip_prefix("lucide-").or_else(|| name.strip_prefix("lucide/")) {
        return lucide_svg(name);
    }
    if let Some(file) = UiAssets::get(&format!("{name}.svg")) {
        return Some(file.data.into_owned());
    }
    lucide_svg(name)
}

#[cfg(feature = "lucide-icons")]
fn lucide_svg(name: &str) -> Option<Vec<u8>> {
    crate::foundation::lucide::svg(name)
}

#[cfg(not(feature = "lucide-icons"))]
fn lucide_svg(_name: &str) -> Option<Vec<u8>> {
    None
}

/// Rasterize an SVG to fit a `pixels`-square box, keeping its aspect ratio.
pub(crate) fn rasterize(bytes: &[u8], pixels: u32) -> Option<egui::ColorImage> {
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default()).ok()?;
    let size = tree.size();
    let scale = (pixels as f32 / size.width().max(1.0)).min(pixels as f32 / size.height().max(1.0));
    let width = (size.width() * scale).round().max(1.0) as u32;
    let height = (size.height() * scale).round().max(1.0) as u32;
    let mut pixmap = Pixmap::new(width, height)?;
    resvg::render(&tree, Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    let mut rgba = pixmap.take();
    for pixel in rgba.chunks_exact_mut(4) {
        let alpha = u16::from(pixel[3]);
        if alpha > 0 && alpha < 255 {
            for channel in &mut pixel[..3] {
                *channel = ((u16::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
            }
        }
    }
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &rgba,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "lucide-icons")]
    fn lucide_is_available_through_the_same_cache() {
        let names = Icons::lucide_names();
        assert!(names.len() > 2000, "{}", names.len());
        assert!(names.iter().any(|name| name == "trash-2"));
        let mut icons = Icons::new();
        let ctx = Context::default();
        for name in ["save", "lucide-save", "trash-2", "heart", "circle-help"] {
            let texture = icons.texture(&ctx, name).unwrap_or_else(|| panic!("{name}"));
            assert!(texture.size()[0] > 0, "{name}");
        }
    }

    #[test]
    fn every_bundled_glyph_rasterizes() {
        let names = Icons::bundled();
        assert!(names.len() >= 40, "{names:?}");
        for name in names {
            let file = UiAssets::get(&format!("{name}.svg")).expect("asset");
            let image = rasterize(&file.data, GLYPH_PIXELS).unwrap_or_else(|| panic!("{name}"));
            assert_eq!(image.size, [48, 48], "{name}");
            assert!(image.pixels.iter().any(|pixel| pixel.a() > 0), "{name} is empty");
        }
    }
}
