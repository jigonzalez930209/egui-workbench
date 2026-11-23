//! Interactive overlays: legend toggles, the toolbar with its export menu, the hover
//! highlight, and the hover popup.

use egui::epaint::Shadow;
use egui::{Align2, Color32, LayerId, Order, Painter, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2};

use super::axis::{Transform, format_readout};
use super::legend::LegendLayout;
use super::{ExportFormat, Plot, PlotState, PlotTool, YAxis};
use crate::Icons;

/// Toggle series from legend rows and move the legend by dragging it. Returns the
/// legend rectangle (blocks hover).
pub(super) fn legend(ui: &mut Ui, plot: &Plot<'_>, legend: &LegendLayout, frame: Rect, state: &mut PlotState) -> Rect {
    // Under the rows: rows take clicks, this takes drags (egui hit-tests the two apart).
    let handle = ui.interact(legend.rect, plot.id.with("legend_drag"), Sense::drag());
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
        let response = ui.interact(row.rect, plot.id.with(("legend", row.series)), Sense::click());
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
        // Only the hovered row animates, and only while the pointer is on it.
        ui.ctx().request_repaint();
    }
    // An ordinary pointer over the legend, a closed hand while moving it.
    if handle.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    } else if over {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
    }
    legend.rect
}

/// What the toolbar asked for this frame.
#[derive(Default)]
pub(super) struct ToolbarOutput {
    pub rect: Option<Rect>,
    pub settings: bool,
    pub export: Option<ExportFormat>,
}

const BUTTON: f32 = 22.0;

const SEPARATOR: f32 = 7.0;

/// Where the toolbar goes, or `None` when it is off or the frame is too narrow. The
/// legend uses it to stay clear of the buttons.
pub(super) fn toolbar_rect(plot: &Plot<'_>, frame: Rect) -> Option<Rect> {
    if !plot.config.toolbar {
        return None;
    }
    let tools = if plot.chart_mode() == super::ChartMode::Cartesian {
        6
    } else {
        0
    };
    let buttons = tools + usize::from(plot.settings_button) + usize::from(plot.export_button);
    if buttons == 0 {
        return None;
    }
    let groups = if tools > 0 && (plot.settings_button || plot.export_button) {
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

/// The tool buttons in the top-right corner of the frame.
pub(super) fn toolbar(
    ui: &mut Ui,
    icons: &mut Icons,
    plot: &Plot<'_>,
    frame: Rect,
    transform: &Transform,
    state: &mut PlotState,
) -> ToolbarOutput {
    let mut output = ToolbarOutput::default();
    if !plot.config.toolbar {
        return output;
    }
    let mut buttons: Vec<(&str, &str, Option<PlotTool>)> = Vec::new();
    if plot.chart_mode() == super::ChartMode::Cartesian {
        buttons.extend([
            ("lucide-hand", "Pan (drag) · wheel zooms", Some(PlotTool::Pan)),
            (
                "lucide-scan-search",
                "Zoom to box (or right-drag)",
                Some(PlotTool::Zoom),
            ),
            (
                "lucide-square-dashed-mouse-pointer",
                "Select area (or Shift-drag)",
                Some(PlotTool::Select),
            ),
            ("lucide-zoom-in", "Zoom in", None),
            ("lucide-zoom-out", "Zoom out", None),
            ("lucide-maximize", "Fit data (double-click)", None),
        ]);
    }
    if plot.settings_button {
        buttons.push(("lucide-settings-2", "Chart settings", None));
    }
    if plot.export_button {
        buttons.push(("lucide-download", "Export as SVG or PNG", None));
    }
    let separator = SEPARATOR;
    let Some(rect) = toolbar_rect(plot, frame) else {
        return output;
    };
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
        if index == 3 || index == 6 {
            let line_x = painter.round_to_pixel_center(x + separator * 0.5);
            painter.vline(
                line_x,
                rect.top() + 5.0..=rect.bottom() - 5.0,
                crate::style::get(ui.ctx()).border(visuals.window_stroke.color),
            );
            x += separator;
        }
        let button = Rect::from_min_size(pos2(x, rect.top() + 2.0), vec2(BUTTON, BUTTON));
        x += BUTTON;
        let response = ui.interact(button, plot.id.with(("tool", *icon)), Sense::click());
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
                ("lucide-zoom-in", _) => {
                    super::interact::zoom_at(state, transform, frame.center(), 1.25, super::interact::ZoomAxes::ALL);
                }
                ("lucide-zoom-out", _) => {
                    super::interact::zoom_at(state, transform, frame.center(), 0.8, super::interact::ZoomAxes::ALL);
                }
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
        output.export = export_menu(ui, plot, button, state);
    }
    output
}

/// A two-row menu under the export button. Closes on a choice or a click elsewhere.
fn export_menu(ui: &mut Ui, plot: &Plot<'_>, button: Rect, state: &mut PlotState) -> Option<ExportFormat> {
    export_menu_for(ui, plot.id, button, &mut state.export_menu)
}

/// The export menu shared by the 2D and 3D toolbars. `open` is cleared when the menu
/// chooses a format or the pointer clicks outside it.
pub(super) fn export_menu_for(ui: &mut Ui, id: egui::Id, button: Rect, open: &mut bool) -> Option<ExportFormat> {
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

/// A ring around the hovered point and, optionally, dashed guides to both axes.
pub(super) fn highlight(
    painter: &Painter,
    frame: Rect,
    position: Pos2,
    color: Color32,
    fill: Color32,
    guide: Option<Color32>,
    right_axis: bool,
) {
    let x = painter.round_to_pixel_center(position.x);
    let y = painter.round_to_pixel_center(position.y);
    if let Some(guide) = guide {
        let stroke = Stroke::new(1.0, guide.gamma_multiply(0.6));
        painter.extend(Shape::dashed_line(
            &[pos2(x, y), pos2(x, frame.bottom())],
            stroke,
            3.0,
            3.0,
        ));
        let edge = if right_axis { frame.right() } else { frame.left() };
        painter.extend(Shape::dashed_line(&[pos2(edge, y), pos2(x, y)], stroke, 3.0, 3.0));
    }
    painter.circle(pos2(x, y), 5.0, fill, Stroke::new(2.0, color));
}

/// The popup next to the hovered point: series name, then x and y.
pub(super) fn tooltip(
    ui: &Ui,
    plot: &Plot<'_>,
    name: &str,
    color: Color32,
    point: [f64; 2],
    anchor: Pos2,
    axis: YAxis,
) {
    const PAD: f32 = 8.0;
    let visuals = ui.visuals();
    let painter = ui
        .ctx()
        .layer_painter(LayerId::new(Order::Tooltip, plot.id.with("tooltip")));
    let label = |text: String, size: f32, color: Color32| {
        painter.layout_no_wrap(text, crate::style::get(ui.ctx()).font(size), color)
    };
    let title = label(name.to_owned(), 12.0, visuals.text_color());
    let x_name = plot.config.x.label.clone().unwrap_or_else(|| "x".to_owned());
    let y_name = match axis {
        YAxis::Left => plot.config.y.label.clone().unwrap_or_else(|| "y".to_owned()),
        YAxis::Right => plot.config.y2.label.clone().unwrap_or_else(|| "y₂".to_owned()),
    };
    let rows = [
        (
            label(x_name, 11.5, visuals.weak_text_color()),
            label(format_readout(point[0]), 11.5, visuals.text_color()),
        ),
        (
            label(y_name, 11.5, visuals.weak_text_color()),
            label(format_readout(point[1]), 11.5, visuals.text_color()),
        ),
    ];
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
    // The dot shares the title's midline.
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
