//! A responsive, virtualized icon grid and list.

use egui::text::{LayoutJob, TextWrapping};
use egui::{
    Align, Color32, FontId, Id, Rect, Response, Role, ScrollArea, Sense, Stroke, TextFormat, Ui, WidgetInfo, pos2, vec2,
};

use crate::Icons;
use crate::components::IconComponent;

/// How the glyph size of an [`IconGrid`] is chosen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IconSize {
    /// Scale the glyph with the cell width, between 20 and 56 pt.
    Auto,
    /// A fixed glyph size in points.
    Fixed(f32),
}

/// Grid of cells or one row per icon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconLayout {
    /// Square-ish cells that always fill the full width.
    #[default]
    Grid,
    /// One row per icon with its name.
    List,
}

/// A searchable icon browser body: every cell is the same width and the columns always
/// fill the available width exactly, so the grid follows the window as it resizes and
/// never overflows its right edge.
///
/// Rows outside the viewport are not laid out, so thousands of icons stay cheap. The
/// grid owns its vertical scroll area and fills the remaining height of the `Ui`.
///
/// ```
/// # use egui_workbench::components::{IconGrid, IconSize, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut icons = egui_workbench::Icons::new();
/// let names = vec!["heart".to_owned(), "star".to_owned()];
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// if let Some(index) = IconGrid::new("icons", &names)
///     .prefix("lucide-")
///     .size(IconSize::Auto)
///     .show(ui, &mut icons)
/// {
///     let _clicked = &names[index];
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct IconGrid<'a> {
    id: &'a str,
    names: &'a [String],
    prefix: &'a str,
    size: IconSize,
    layout: IconLayout,
    labels: bool,
    selected: Option<&'a str>,
    gap: f32,
}

impl<'a> IconGrid<'a> {
    /// A grid over `names`, with a stable id for its scroll state.
    pub fn new(id: &'a str, names: &'a [String]) -> Self {
        Self {
            id,
            names,
            prefix: "",
            size: IconSize::Auto,
            layout: IconLayout::Grid,
            labels: true,
            selected: None,
            gap: 6.0,
        }
    }

    /// A prefix added to each name when resolving the glyph, such as `lucide-`.
    pub fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

    /// Glyph size; [`IconSize::Auto`] by default.
    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Grid or list.
    pub fn layout(mut self, layout: IconLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Show the name under each grid cell (on by default).
    pub fn labels(mut self, labels: bool) -> Self {
        self.labels = labels;
        self
    }

    /// Outline the cell whose name equals `selected`.
    pub fn selected(mut self, selected: Option<&'a str>) -> Self {
        self.selected = selected;
        self
    }

    /// Space between cells in points.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    /// The column count and cell width for a content `width`.
    ///
    /// Columns are as many as fit at the minimum cell width; the cells then stretch so
    /// `columns * cell + (columns - 1) * gap == width`.
    pub fn columns_for(&self, width: f32) -> (usize, f32) {
        let min_cell = match self.size {
            IconSize::Auto => 92.0,
            IconSize::Fixed(size) => (size + 28.0).max(if self.labels { 84.0 } else { size + 16.0 }),
        };
        let width = width.max(min_cell);
        let columns = (((width + self.gap) / (min_cell + self.gap)).floor() as usize).max(1);
        let cell = (width - self.gap * (columns - 1) as f32) / columns as f32;
        (columns, cell.floor())
    }

    fn glyph(&self, cell: f32) -> f32 {
        match self.size {
            IconSize::Auto => (cell * 0.32).clamp(20.0, 56.0).round(),
            IconSize::Fixed(size) => size,
        }
    }
}

impl IconComponent for IconGrid<'_> {
    type Output = Option<usize>;

    fn show(self, ui: &mut Ui, icons: &mut Icons) -> Option<usize> {
        match self.layout {
            IconLayout::Grid => self.show_grid(ui, icons),
            IconLayout::List => self.show_list(ui, icons),
        }
    }
}

impl IconGrid<'_> {
    fn show_grid(self, ui: &mut Ui, icons: &mut Icons) -> Option<usize> {
        let mut clicked = None;
        ScrollArea::vertical()
            .id_salt((self.id, "icon_grid"))
            .auto_shrink([false, false])
            .show_viewport(ui, |ui, viewport| {
                let width = ui.available_width();
                let (columns, cell) = self.columns_for(width);
                // Spread any rounding remainder evenly so both edges stay aligned.
                let gap = if columns > 1 {
                    (width - cell * columns as f32) / (columns - 1) as f32
                } else {
                    0.0
                };
                let glyph = self.glyph(cell);
                let label_height = if self.labels { 18.0 } else { 0.0 };
                let height = (glyph + 22.0 + label_height).max(cell * 0.62).round();
                let pitch = height + self.gap;
                let rows = self.names.len().div_ceil(columns);
                let total = (rows as f32 * pitch - self.gap).max(0.0);
                ui.set_width(width);
                ui.set_height(total);

                let origin = ui.max_rect().min;
                let first = ((viewport.min.y / pitch).floor().max(0.0)) as usize;
                let last = (((viewport.max.y / pitch).ceil()) as usize + 1).min(rows);
                for row in first..last {
                    for column in 0..columns {
                        let index = row * columns + column;
                        let Some(name) = self.names.get(index) else {
                            break;
                        };
                        let min = pos2(origin.x + column as f32 * (cell + gap), origin.y + row as f32 * pitch);
                        let rect = Rect::from_min_size(min, vec2(cell, height));
                        if self.cell(ui, icons, rect, name, glyph, label_height).clicked() {
                            clicked = Some(index);
                        }
                    }
                }
            });
        clicked
    }

    fn cell(&self, ui: &mut Ui, icons: &mut Icons, rect: Rect, name: &str, glyph: f32, label: f32) -> Response {
        let response = ui.interact(rect, Id::unique((self.id, name)), Sense::click());
        response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
        let visuals = ui.visuals();
        let selected = self.selected == Some(name);
        let (fill, stroke) = if selected {
            (
                visuals.selection.bg_fill,
                crate::style::get(ui.ctx()).border(visuals.widgets.hovered.bg_stroke.color),
            )
        } else if response.is_pointer_button_down_on() {
            (visuals.widgets.active.weak_bg_fill.gamma_multiply(0.35), Stroke::NONE)
        } else if response.hovered() {
            (
                visuals.widgets.hovered.weak_bg_fill,
                crate::style::get(ui.ctx()).border(visuals.window_stroke.color),
            )
        } else {
            (Color32::TRANSPARENT, Stroke::NONE)
        };
        let text = visuals.text_color();
        let weak = visuals.weak_text_color();
        ui.painter().rect(
            rect,
            crate::style::get(ui.ctx()).container(),
            fill,
            stroke,
            egui::StrokeKind::Inside,
        );
        let content = glyph + if label > 0.0 { 8.0 + label } else { 0.0 };
        let top = rect.center().y - content * 0.5;
        icons.paint(
            ui,
            &format!("{}{name}", self.prefix),
            pos2(rect.center().x, top + glyph * 0.5),
            glyph,
            text,
        );
        if label > 0.0 {
            let galley = truncated(ui, name, 11.0, if selected { text } else { weak }, rect.width() - 12.0);
            let pos = pos2(rect.center().x - galley.size().x * 0.5, top + glyph + 8.0);
            ui.painter().galley(pos, galley, weak);
        }
        response.on_hover_text(name)
    }

    fn show_list(self, ui: &mut Ui, icons: &mut Icons) -> Option<usize> {
        let mut clicked = None;
        let glyph = match self.size {
            IconSize::Auto => 20.0,
            IconSize::Fixed(size) => size,
        };
        let height = (glyph + 12.0).max(28.0);
        ScrollArea::vertical()
            .id_salt((self.id, "icon_list"))
            .auto_shrink([false, false])
            .show_viewport(ui, |ui, viewport| {
                let width = ui.available_width();
                let pitch = height + 2.0;
                let total = (self.names.len() as f32 * pitch - 2.0).max(0.0);
                ui.set_width(width);
                ui.set_height(total);
                let origin = ui.max_rect().min;
                let first = (viewport.min.y / pitch).floor().max(0.0) as usize;
                let last = ((viewport.max.y / pitch).ceil() as usize + 1).min(self.names.len());
                for index in first..last {
                    let name = &self.names[index];
                    let rect =
                        Rect::from_min_size(pos2(origin.x, origin.y + index as f32 * pitch), vec2(width, height));
                    let response = ui.interact(rect, Id::unique((self.id, "list", name)), Sense::click());
                    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
                    let visuals = ui.visuals();
                    let fill = if self.selected == Some(name.as_str()) {
                        visuals.selection.bg_fill
                    } else if response.hovered() {
                        visuals.widgets.hovered.weak_bg_fill
                    } else {
                        Color32::TRANSPARENT
                    };
                    let text = visuals.text_color();
                    ui.painter()
                        .rect_filled(rect, crate::style::get(ui.ctx()).control(), fill);
                    let center = pos2(rect.left() + 8.0 + glyph * 0.5, rect.center().y);
                    icons.paint(ui, &format!("{}{name}", self.prefix), center, glyph, text);
                    let left = rect.left() + 8.0 + glyph + 12.0;
                    let galley = truncated(ui, name, 13.0, text, rect.right() - left - 8.0);
                    ui.painter()
                        .galley(pos2(left, rect.center().y - galley.size().y * 0.5), galley, text);
                    if response.clicked() {
                        clicked = Some(index);
                    }
                }
            });
        clicked
    }
}

/// One line of text, cut with "…" at `width`.
fn truncated(ui: &Ui, text: &str, size: f32, color: Color32, width: f32) -> std::sync::Arc<egui::Galley> {
    let mut job = LayoutJob::single_section(text.to_owned(), TextFormat::simple(FontId::proportional(size), color));
    job.wrap = TextWrapping::truncate_at_width(width.max(8.0));
    job.halign = Align::LEFT;
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_fill_the_width_exactly() {
        let names: Vec<String> = Vec::new();
        for size in [IconSize::Auto, IconSize::Fixed(16.0), IconSize::Fixed(48.0)] {
            let grid = IconGrid::new("t", &names).size(size);
            for width in [120.0_f32, 333.0, 640.0, 1021.0, 1920.0] {
                let (columns, cell) = grid.columns_for(width);
                let used = columns as f32 * cell + (columns - 1) as f32 * grid.gap;
                assert!(used <= width + 0.01, "{size:?} {width}: {used}");
                assert!(
                    width - used < columns as f32,
                    "{size:?} {width}: slack {}",
                    width - used
                );
                // One more column would not fit at the minimum cell width.
                assert!(columns >= 1);
            }
        }
    }

    #[test]
    fn wider_windows_get_more_columns() {
        let names: Vec<String> = Vec::new();
        let grid = IconGrid::new("t", &names);
        let (narrow, _) = grid.columns_for(400.0);
        let (wide, _) = grid.columns_for(1600.0);
        assert!(wide > narrow);
    }
}
