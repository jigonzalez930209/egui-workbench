//! The export menu shared by the 2D and 3D toolbars.

use egui::epaint::Shadow;
use egui::{Align2, Color32, LayerId, Order, Rect, Sense, StrokeKind, Ui, pos2, vec2};

use super::{ExportFormat, Plot, PlotState};

pub(super) fn export_menu(ui: &mut Ui, plot: &Plot<'_>, button: Rect, state: &mut PlotState) -> Option<ExportFormat> {
    export_menu_for(ui, plot.id, button, &mut state.export_menu)
}

/// The export menu shared by the 2D and 3D toolbars. `open` is cleared when the menu
/// chooses a format or the pointer clicks outside it.
pub(crate) fn export_menu_for(ui: &mut Ui, id: egui::Id, button: Rect, open: &mut bool) -> Option<ExportFormat> {
    let width = 150.0;
    let row = 26.0;
    let rect = Rect::from_min_size(
        pos2(button.right() - width + 4.0, button.bottom() + 6.0),
        vec2(width, 2.0 * row + 8.0),
    );
    let layer = LayerId::new(Order::Foreground, id.with("export_menu"));
    let visuals = ui.visuals().clone();
    let painter = ui.ctx().layer_painter(layer);
    painter.add(
        Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(40),
        }
        .as_shape(rect, crate::style::get(ui.ctx()).control()),
    );
    painter.rect(
        rect,
        crate::style::get(ui.ctx()).control(),
        visuals.window_fill,
        crate::style::get(ui.ctx()).border(visuals.window_stroke.color),
        StrokeKind::Inside,
    );
    let mut chosen = None;
    for (index, (label, detail, format)) in [
        ("Export SVG", "vector", ExportFormat::Svg),
        ("Export PDF", "file-text", ExportFormat::Pdf),
        ("Export PNG", "image", ExportFormat::Png),
    ]
    .into_iter()
    .enumerate()
    {
        let item = Rect::from_min_size(
            pos2(rect.left() + 4.0, rect.top() + 4.0 + index as f32 * row),
            vec2(width - 8.0, row),
        );
        let response = egui::Area::new(id.with(("export_item", index)))
            .order(Order::Foreground)
            .fixed_pos(item.min)
            .show(ui.ctx(), |ui| ui.allocate_exact_size(item.size(), Sense::click()).1)
            .inner;
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::MenuItem, true, label));
        if response.hovered() {
            painter.rect_filled(
                item,
                crate::style::get(ui.ctx()).inner(),
                visuals.widgets.hovered.weak_bg_fill,
            );
        }
        let middle = painter.round_to_pixel_center(item.center().y);
        painter.text(
            pos2(item.left() + 10.0, middle),
            Align2::LEFT_CENTER,
            label,
            crate::style::get(ui.ctx()).font(12.5),
            visuals.text_color(),
        );
        painter.text(
            pos2(item.right() - 8.0, middle),
            Align2::RIGHT_CENTER,
            detail,
            crate::style::get(ui.ctx()).font(11.0),
            visuals.weak_text_color(),
        );
        if response.clicked() {
            chosen = Some(format);
        }
    }
    let clicked_elsewhere = ui.input(|input| {
        input.pointer.any_click()
            && input
                .pointer
                .interact_pos()
                .is_some_and(|pos| !rect.contains(pos) && !button.contains(pos))
    });
    if chosen.is_some() || clicked_elsewhere || ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        *open = false;
    }
    chosen
}
