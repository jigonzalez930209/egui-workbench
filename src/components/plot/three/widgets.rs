//! The 3D toolbar, legend interaction, hover ring, and tooltip.

use egui::epaint::Shadow;
use egui::{Color32, LayerId, Order, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use super::super::ExportFormat;
use super::super::axis::format_readout;
use super::super::legend::LegendLayout;
use super::super::widgets::export_menu_for;
use super::camera::{Camera3D, Projector};
use super::{Plot3D, Plot3DState, Plot3DTool};
use crate::Icons;

const BUTTON: f32 = 22.0;
const SEPARATOR: f32 = 7.0;

#[derive(Default)]
pub(super) struct ToolbarOutput {
    pub rect: Option<Rect>,
    pub settings: bool,
    pub export: Option<ExportFormat>,
}

/// Where the 3D toolbar goes, or `None` when it is off or the frame is too narrow.
pub(super) fn toolbar_rect(plot: &Plot3D<'_>, frame: Rect) -> Option<Rect> {
    if !plot.config.toolbar {
        return None;
    }
    let tools = 5;
    let buttons = tools + usize::from(plot.settings_button) + usize::from(plot.export_button);
    let groups = if plot.settings_button || plot.export_button {
        2.0
    } else {
        1.0
    };
    let width = BUTTON * buttons as f32 + SEPARATOR * groups + 4.0;
    let rect = Rect::from_min_size(
        pos2(frame.right() - 6.0 - width, frame.top() + 6.0),
        vec2(width, BUTTON + 4.0),
    );
    (rect.left() >= frame.left() + 40.0).then_some(rect)
}

pub(super) fn toolbar(
    ui: &mut Ui,
    icons: &mut Icons,
    plot: &Plot3D<'_>,
    frame: Rect,
    camera: Camera3D,
    state: &mut Plot3DState,
) -> ToolbarOutput {
    let mut output = ToolbarOutput::default();
    let Some(rect) = toolbar_rect(plot, frame) else {
        return output;
    };
    let mut buttons: Vec<(&str, &str, Option<Plot3DTool>)> = vec![
        ("lucide-rotate-3d", "Orbit (drag)", Some(Plot3DTool::Orbit)),
        ("lucide-hand", "Pan (drag or Shift-drag)", Some(Plot3DTool::Pan)),
        ("lucide-zoom-in", "Zoom in", None),
        ("lucide-zoom-out", "Zoom out", None),
        ("lucide-maximize", "Fit data (double-click)", None),
    ];
    if plot.settings_button {
        buttons.push(("lucide-settings-2", "Chart settings", None));
    }
    if plot.export_button {
        buttons.push(("lucide-download", "Export as SVG or PNG", None));
    }
    let visuals = ui.visuals().clone();
    let painter = ui.painter_at(frame);
    painter.rect(
        rect,
        crate::style::get(ui.ctx()).control(),
        visuals.window_fill.gamma_multiply(0.94),
        crate::style::get(ui.ctx()).border(visuals.window_stroke.color),
        StrokeKind::Inside,
    );
    let mut x = rect.left() + 2.0;
    let mut export_button = None;
    for (index, (icon, tooltip, tool)) in buttons.iter().enumerate() {
        if index == 5 {
            let line_x = painter.round_to_pixel_center(x + SEPARATOR * 0.5);
            painter.vline(
                line_x,
                rect.top() + 5.0..=rect.bottom() - 5.0,
                crate::style::get(ui.ctx()).border(visuals.window_stroke.color),
            );
            x += SEPARATOR;
        }
        let button = Rect::from_min_size(pos2(x, rect.top() + 2.0), vec2(BUTTON, BUTTON));
        x += BUTTON;
        let response = ui.interact(button, plot.id.with(("tool3d", *icon)), Sense::click());
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, *tooltip));
        let is_export = *icon == "lucide-download";
        let active = tool.is_some_and(|tool| tool == state.tool) || (is_export && state.export_menu);
        let fill = if active {
            visuals.selection.bg_fill
        } else if response.hovered() {
            visuals.widgets.hovered.weak_bg_fill
        } else {
            Color32::TRANSPARENT
        };
        painter.rect_filled(button, crate::style::get(ui.ctx()).inner(), fill);
        let color = if active || response.hovered() {
            visuals.text_color()
        } else {
            visuals.weak_text_color()
        };
        icons.paint(ui, icon, button.center(), 14.0, color);
        if response.clicked() {
            match (*icon, tool) {
                (_, Some(tool)) => state.tool = *tool,
                ("lucide-zoom-in", _) => state.set_camera(camera.zoom(1.25)),
                ("lucide-zoom-out", _) => state.set_camera(camera.zoom(0.8)),
                ("lucide-settings-2", _) => output.settings = true,
                ("lucide-download", _) => state.export_menu = !state.export_menu,
                _ => state.reset_view(),
            }
            ui.ctx().request_repaint();
        }
        if is_export {
            export_button = Some(button);
        }
        response.on_hover_text(*tooltip);
    }
    output.rect = Some(rect);
    if state.export_menu
        && let Some(button) = export_button
    {
        output.export = export_menu_for(ui, plot.id, button, &mut state.export_menu);
    }
    output
}

/// Toggle series from legend rows and move the legend by dragging it.
pub(super) fn legend(
    ui: &mut Ui,
    plot: &Plot3D<'_>,
    legend: &LegendLayout,
    frame: Rect,
    state: &mut Plot3DState,
) -> Rect {
    let handle = ui.interact(legend.rect, plot.id.with("legend3d_drag"), Sense::drag());
    let mut over = handle.hovered();
    if handle.dragged() {
        let room = (frame.size() - legend.rect.size()).max(vec2(1.0, 1.0));
        let min = legend.rect.min + handle.drag_delta() - frame.min;
        state.set_legend_position(Some([min.x / room.x, min.y / room.y]));
        ui.ctx().request_repaint();
    }
    let now = ui.input(|input| input.time);
    let mut hovered = None;
    for row in &legend.rows {
        let name = &plot.series[row.series].name;
        let response = ui.interact(row.rect, plot.id.with(("legend3d", row.series)), Sense::click());
        if response.hovered() && !handle.dragged() {
            over = true;
            hovered = Some(row.series);
        }
        if response.clicked() {
            let hidden = state.is_hidden(name);
            state.set_hidden(name, !hidden);
        }
        response.on_hover_text(if row.hidden {
            "Show series (drag to move the legend)"
        } else {
            "Hide series (drag to move the legend)"
        });
    }
    state.set_legend_hover(hovered, now);
    if hovered.is_some() && plot.config.legend_hover.enabled && crate::theme::motion_enabled(ui.ctx()) {
        ui.ctx().request_repaint();
    }
    if handle.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    } else if over {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
    }
    legend.rect
}

/// A ring on the hovered point and a dashed drop line to the floor.
pub(super) fn highlight(
    painter: &Painter,
    projector: &Projector,
    point: [f64; 3],
    floor: f64,
    color: Color32,
    fill: Color32,
    guide: Option<Color32>,
) {
    let Some((pos, _)) = projector.project(point) else {
        return;
    };
    if let Some(guide) = guide
        && let Some((drop, _)) = projector.project([point[0], point[1], floor])
    {
        painter.extend(egui::Shape::dashed_line(
            &[pos, drop],
            Stroke::new(1.0, guide.gamma_multiply(0.6)),
            3.0,
            3.0,
        ));
    }
    painter.circle(
        pos2(
            painter.round_to_pixel_center(pos.x),
            painter.round_to_pixel_center(pos.y),
        ),
        5.0,
        fill,
        Stroke::new(2.0, color),
    );
}

/// The popup next to the hovered point: series, x, y, and z.
pub(super) fn tooltip(ui: &Ui, plot: &Plot3D<'_>, name: &str, color: Color32, point: [f64; 3], anchor: Pos2) {
    const PAD: f32 = 8.0;
    let visuals = ui.visuals();
    let painter = ui
        .ctx()
        .layer_painter(LayerId::new(Order::Tooltip, plot.id.with("tooltip3d")));
    let label = |text: String, size: f32, color: Color32| {
        painter.layout_no_wrap(text, crate::style::get(ui.ctx()).font(size), color)
    };
    let title = label(name.to_owned(), 12.0, visuals.text_color());
    let names = [
        plot.config.x.label.clone().unwrap_or_else(|| "x".to_owned()),
        plot.config.y.label.clone().unwrap_or_else(|| "y".to_owned()),
        plot.config.z.label.clone().unwrap_or_else(|| "z".to_owned()),
    ];
    let rows: Vec<_> = names
        .into_iter()
        .zip(point)
        .map(|(name, value)| {
            (
                label(name, 11.5, visuals.weak_text_color()),
                label(format_readout(value), 11.5, visuals.text_color()),
            )
        })
        .collect();
    let names = rows.iter().map(|(name, _)| name.size().x).fold(0.0, f32::max);
    let values = rows.iter().map(|(_, value)| value.size().x).fold(0.0, f32::max);
    let width = (title.size().x + 14.0).max(names + 12.0 + values) + 2.0 * PAD;
    let line = 16.0;
    let height = title.size().y + 4.0 + line * rows.len() as f32 + 2.0 * PAD - 2.0;
    let screen = ui.ctx().content_rect();
    let mut min = anchor + vec2(14.0, 14.0);
    if min.x + width > screen.right() - 4.0 {
        min.x = anchor.x - 14.0 - width;
    }
    if min.y + height > screen.bottom() - 4.0 {
        min.y = anchor.y - 14.0 - height;
    }
    let rect = Rect::from_min_size(min.round(), vec2(width.round(), height.round()));
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
    let mut y = rect.top() + PAD;
    let title_height = title.size().y;
    painter.circle_filled(
        pos2(rect.left() + PAD + 4.0, (y + title_height * 0.5).round()),
        4.0,
        color,
    );
    painter.galley(pos2(rect.left() + PAD + 14.0, y), title, visuals.text_color());
    y += title_height + 4.0;
    for (name, value) in rows {
        painter.galley(pos2(rect.left() + PAD, y), name, visuals.weak_text_color());
        let value_x = rect.right() - PAD - value.size().x;
        painter.galley(pos2(value_x, y), value, visuals.text_color());
        y += line;
    }
}
