//! Interactive overlays: legend toggles, the toolbar with its export menu, the hover
//! highlight, and the hover popup.

use egui::epaint::Shadow;
use egui::{Color32, LayerId, Order, Painter, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2};

use super::axis::{Transform, format_readout};
use super::legend::LegendLayout;
use super::{ExportFormat, Plot, PlotState, PlotTool, YAxis};
use crate::Icons;

mod export;
mod settings;

pub(super) use export::export_menu_for;

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
pub(super) fn toolbar_rect(plot: &Plot<'_>, frame: Rect, state: &PlotState) -> Option<Rect> {
    if !plot.config.toolbar {
        return None;
    }
    let tools = if plot.chart_mode() == super::ChartMode::Cartesian {
        9
    } else {
        0
    };
    let has_clear = state.has_overlays();
    let buttons =
        tools + usize::from(plot.settings_button) + usize::from(plot.export_button) + usize::from(has_clear) + 1;
    if buttons == 0 {
        return None;
    }
    let extras = plot.settings_button || plot.export_button || has_clear;
    let groups = if tools > 0 && extras { 2.0 } else { 1.0 };
    let width = BUTTON * buttons as f32 + SEPARATOR * groups + 4.0;
    let origin = state.toolbar_anchor.map_or_else(
        || pos2(frame.right() - 6.0 - width, frame.top() + 6.0),
        |[x, y]| frame.min + vec2(x, y),
    );
    let rect = Rect::from_min_size(origin, vec2(width, BUTTON + 4.0));
    (rect.left() >= frame.left() + 40.0 && rect.right() <= frame.right()).then_some(rect)
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
            (
                "lucide-ruler",
                "Measure distance and slope (drag)",
                Some(PlotTool::MeasureDistance),
            ),
            (
                "lucide-square-dashed",
                if plot.config.measurement.mode == super::MeasurementMode::CurveSegment {
                    "Measure curve area against its chord (drag)"
                } else {
                    "Measure area, extrema, and integrals (drag)"
                },
                Some(PlotTool::MeasureArea),
            ),
            ("lucide-sliders-horizontal", "Filters and transforms (analysis)", None),
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
    if state.has_overlays() {
        buttons.push(("lucide-eraser", "Clear measurements, analysis, and annotations", None));
    }
    buttons.push((
        "lucide-pin",
        "Toolbar (hover to unroll) · click to pin · drag to move",
        None,
    ));
    let separator = SEPARATOR;
    let Some(full) = toolbar_rect(plot, frame, state) else {
        return output;
    };
    let pin_rect = Rect::from_min_size(
        pos2(full.right() - 2.0 - BUTTON, full.top() + 2.0),
        vec2(BUTTON, BUTTON),
    );
    let pointer = ui.ctx().pointer_hover_pos();
    let reveal_before = state.toolbar_reveal.clamp(0.0, 1.0);
    let visible_left = full.right() - (BUTTON + 4.0).max(full.width() * reveal_before);
    let over = pointer.is_some_and(|pointer| {
        (pointer.x >= visible_left - 2.0
            && pointer.x <= full.right()
            && pointer.y >= full.top()
            && pointer.y <= full.bottom())
            || pin_rect.expand(2.0).contains(pointer)
    });
    let target = if state.toolbar_pinned || over { 1.0 } else { 0.0 };
    let dt = ui.input(|input| input.stable_dt).min(0.1);
    let speed = 1.0 - (-dt * 22.0).exp();
    if (target - state.toolbar_reveal).abs() > 0.001 {
        state.toolbar_reveal += (target - state.toolbar_reveal) * speed;
        ui.ctx().request_repaint();
    } else {
        state.toolbar_reveal = target;
    }
    let reveal = state.toolbar_reveal.clamp(0.0, 1.0);
    let visible = Rect::from_min_max(
        pos2(full.right() - (BUTTON + 4.0).max(full.width() * reveal), full.top()),
        full.max,
    );
    let visuals = ui.visuals().clone();
    let painter = ui.painter_at(frame);
    painter.rect(
        visible,
        crate::style::get(ui.ctx()).control(),
        visuals.window_fill.gamma_multiply(0.94),
        crate::style::get(ui.ctx()).border(visuals.window_stroke.color),
        StrokeKind::Inside,
    );
    let mut x = full.left() + 2.0;
    let mut export_button = None;
    let mut analysis_button = None;
    let mut pin_button = None;
    let mut settings_button = None;
    let mut gear_hovered = false;
    for (index, (icon, tooltip, tool)) in buttons.iter().enumerate() {
        if index == 6 || index == 9 {
            if x >= visible.left() {
                let line_x = painter.round_to_pixel_center(x + separator * 0.5);
                painter.vline(
                    line_x,
                    full.top() + 5.0..=full.bottom() - 5.0,
                    crate::style::get(ui.ctx()).border(visuals.window_stroke.color),
                );
            }
            x += separator;
        }
        let button = Rect::from_min_size(pos2(x, full.top() + 2.0), vec2(BUTTON, BUTTON));
        x += BUTTON;
        let is_export = *icon == "lucide-download";
        let is_analysis = *icon == "lucide-sliders-horizontal";
        let is_pin = *icon == "lucide-pin";
        let is_settings = *icon == "lucide-settings-2";
        if !is_pin && button.left() < visible.left() - 0.5 {
            continue;
        }
        let sense = if is_pin {
            Sense::click_and_drag()
        } else {
            Sense::click()
        };
        let response = ui.interact(button, plot.id.with(("tool", *icon)), sense);
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, *tooltip));
        let active = tool.is_some_and(|tool| tool == state.tool)
            || (is_export && state.export_menu)
            || (is_analysis && state.pipeline.is_active())
            || (is_pin && state.toolbar_pinned)
            || (is_settings && state.settings_pinned);
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
        if is_pin && response.dragged() {
            let delta = response.drag_delta();
            let max_x = (frame.width() - full.width()).max(0.0);
            let max_y = (frame.height() - full.height()).max(0.0);
            state.toolbar_anchor = Some([
                (full.left() - frame.left() + delta.x).clamp(0.0, max_x),
                (full.top() - frame.top() + delta.y).clamp(0.0, max_y),
            ]);
            state.toolbar_reveal = 1.0;
            ui.ctx().request_repaint();
        }
        if response.clicked() {
            match (*icon, tool) {
                (_, Some(tool)) => state.tool = *tool,
                ("lucide-sliders-horizontal", _) => {}
                ("lucide-pin", _) => {
                    state.toolbar_pinned = !state.toolbar_pinned;
                    if state.toolbar_pinned {
                        state.toolbar_reveal = 1.0;
                    }
                }
                ("lucide-settings-2", _) => {
                    state.settings_pinned = !state.settings_pinned;
                    state.settings_hovered = true;
                    output.settings = true;
                }
                ("lucide-eraser", _) => state.clear_overlays(),
                ("lucide-zoom-in", _) => {
                    super::interact::zoom_at(state, transform, frame.center(), 1.25, super::interact::ZoomAxes::ALL);
                }
                ("lucide-zoom-out", _) => {
                    super::interact::zoom_at(state, transform, frame.center(), 0.8, super::interact::ZoomAxes::ALL);
                }
                ("lucide-download", _) => state.export_menu = !state.export_menu,
                _ => state.reset_view(),
            }
            ui.ctx().request_repaint();
        }
        if is_export {
            export_button = Some(button);
        }
        if is_analysis {
            analysis_button = Some(response.clone());
        }
        if is_pin {
            pin_button = Some(button);
        }
        if is_settings {
            settings_button = Some(button);
            gear_hovered = response.hovered();
            if gear_hovered {
                state.settings_hovered = true;
            }
        }
        response.on_hover_text(*tooltip);
    }
    output.rect = Some(full);
    if let Some(response) = analysis_button {
        egui::Popup::menu(&response).show(|ui| settings::analysis_menu(ui, state));
    }
    if state.settings_pinned || state.settings_hovered {
        let anchor = settings_button.or(pin_button);
        let panel_hovered = anchor.is_some_and(|anchor| settings::panel(ui, state, anchor));
        state.settings_hovered = gear_hovered || panel_hovered;
    } else {
        state.settings_hovered = false;
    }
    if state.export_menu
        && let Some(button) = export_button
    {
        output.export = export::export_menu(ui, plot, button, state);
    }
    output
}

/// A two-row menu under the export button. Closes on a choice or a click elsewhere.
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
