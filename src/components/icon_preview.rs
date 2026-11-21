//! Arbitrary-size icon preview, with the same cache as `IconButton`.

use egui::{Color32, Response, Role, Sense, Ui, Vec2, WidgetInfo};

use crate::Icons;
use crate::components::IconComponent;

/// A clickable icon preview at any pixel size.
///
/// `IconButton` uses the three chrome sizes; this component is for galleries, palettes,
/// and icon pickers where the product chooses the size.
///
/// ```
/// # use egui_workbench::components::{IconPreview, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut icons = egui_workbench::Icons::new();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// if IconPreview::new("lucide-heart").size(32.0).show(ui, &mut icons).clicked() {
///     // selected
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct IconPreview<'a> {
    name: &'a str,
    size: f32,
    color: Option<Color32>,
}

impl<'a> IconPreview<'a> {
    /// An icon at the default 24 pt size.
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            size: 24.0,
            color: None,
        }
    }

    /// Icon size in points.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Explicit tint; defaults to the current text color.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl IconComponent for IconPreview<'_> {
    type Output = Response;

    fn show(self, ui: &mut Ui, icons: &mut Icons) -> Response {
        let side = self.size + 12.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
        response.widget_info(|| WidgetInfo::labeled(Role::Button, true, self.name));
        let color = self.color.unwrap_or_else(|| ui.visuals().text_color());
        icons.paint(ui, self.name, rect.center(), self.size, color);
        response.on_hover_text(self.name)
    }
}
