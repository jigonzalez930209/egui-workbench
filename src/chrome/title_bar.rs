//! The 34 pt workbench title bar.
//!
//! Left to right: a 10 pt gap, the 20 pt product mark, the text menus (File, Edit, …),
//! history buttons; centered: the document title with a "•" dirty marker over a 140 pt
//! drag region; right to left: window controls (close, maximize/restore, minimize), a
//! separator, then the workbench toggles (settings, palette, search, sidebar).

use egui::{Align, Align2, Color32, Image, Layout, Rect, Role, Sense, Ui, UiBuilder, Vec2, WidgetInfo};

use crate::metrics::{ICON_LARGE, ICON_SMALL, WINDOW_BUTTON_HEIGHT, WINDOW_BUTTON_WIDTH};
use crate::{Icons, WindowAction};

/// Translated tooltips of the window controls.
#[derive(Clone, Copy, Debug)]
pub struct WindowLabels<'a> {
    /// "Minimize".
    pub minimize: &'a str,
    /// "Maximize".
    pub maximize: &'a str,
    /// "Restore".
    pub restore: &'a str,
    /// "Close".
    pub close: &'a str,
}

impl Default for WindowLabels<'_> {
    fn default() -> Self {
        Self {
            minimize: "Minimize",
            maximize: "Maximize",
            restore: "Restore",
            close: "Close",
        }
    }
}

/// A rounded pill button placed just left of the right-hand actions ("Open Folder").
#[derive(Clone, Copy, Debug)]
pub struct Pill<'a> {
    /// Glyph name.
    pub icon: &'a str,
    /// Visible label.
    pub label: &'a str,
    /// Tooltip; empty for none.
    pub tooltip: &'a str,
}

/// Inputs of the title bar.
#[derive(Clone, Copy, Debug)]
pub struct TitleBar<'a> {
    /// Centered document title; empty hides it.
    pub title: &'a str,
    /// Appends " •" to the title.
    pub dirty: bool,
    /// Switches the maximize glyph to "restore".
    pub maximized: bool,
    /// Product mark glyph (register your own as `logo`).
    pub logo: &'a str,
    /// Window control tooltips.
    pub labels: WindowLabels<'a>,
    /// Optional pill button.
    pub pill: Option<Pill<'a>>,
}

impl<'a> TitleBar<'a> {
    /// A title bar with the default logo and English labels.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            dirty: false,
            maximized: false,
            logo: "logo",
            labels: WindowLabels::default(),
            pill: None,
        }
    }
}

/// What happened in the title bar this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TitleBarOutput {
    /// A window control was clicked (double-clicking the drag region is handled here).
    pub window: Option<WindowAction>,
    /// The pill was clicked.
    pub pill_clicked: bool,
}

/// Draw the title bar inside a `Panel::top` of [`crate::metrics::TITLE_HEIGHT`].
///
/// `left` runs in a left-to-right row after the logo: add [`title_menu`]s and history
/// [`crate::action_button`]s. `right` runs in a right-to-left row after the window
/// controls, so add its buttons from the rightmost to the leftmost.
pub fn show_title_bar(
    ui: &mut Ui,
    icons: &mut Icons,
    bar: &TitleBar<'_>,
    left: impl FnOnce(&mut Ui, &mut Icons),
    right: impl FnOnce(&mut Ui, &mut Icons),
) -> TitleBarOutput {
    let rect = ui.max_rect();
    let mut output = TitleBarOutput::default();
    ui.painter().rect_filled(rect, 0.0, ui.visuals().panel_fill);

    let left_edge = ui
        .scope_builder(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                flat_menu_style(ui);
                ui.add_space(10.0);
                if let Some(texture) = icons.texture(ui.ctx(), bar.logo) {
                    ui.add(Image::new(&texture).fit_to_exact_size(Vec2::splat(ICON_LARGE)));
                }
                ui.add_space(8.0);
                left(ui, icons);
            },
        )
        .response
        .rect
        .right();

    let right_edge = ui
        .scope_builder(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
                ui.add_space(1.0);
                let labels = bar.labels;
                if window_button(ui, icons, "close", true, labels.close) {
                    output.window = Some(WindowAction::Close);
                }
                let (icon, tooltip) = if bar.maximized {
                    ("restore", labels.restore)
                } else {
                    ("maximize", labels.maximize)
                };
                if window_button(ui, icons, icon, false, tooltip) {
                    output.window = Some(WindowAction::ToggleMaximize);
                }
                if window_button(ui, icons, "minimize", false, labels.minimize) {
                    output.window = Some(WindowAction::Minimize);
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);
                right(ui, icons);
                ui.add_space(2.0);
            },
        )
        .response
        .rect
        .left();

    // Only the center is a drag surface: covering the whole bar stole clicks from the
    // history and workbench buttons.
    let center = rect.center().x;
    let drag_left = (center - 70.0).max(rect.left() + 8.0);
    let drag_right = (center + 70.0).min(rect.right() - 8.0).max(drag_left);
    let drag = Rect::from_min_max(egui::pos2(drag_left, rect.top()), egui::pos2(drag_right, rect.bottom()));
    let response = ui.interact(drag, ui.scope_id().with("title_bar_drag"), Sense::click_and_drag());
    if response.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if response.double_clicked() {
        output.window = Some(WindowAction::ToggleMaximize);
    }

    if let Some(pill) = bar.pill {
        let center = egui::pos2((right_edge - 58.0).max(left_edge + 12.0), rect.center().y);
        output.pill_clicked = pill_button(ui, icons, center, &pill);
    }
    if !bar.title.is_empty() {
        let marker = if bar.dirty { " •" } else { "" };
        ui.painter().text(
            drag.center(),
            Align2::CENTER_CENTER,
            format!("{}{marker}", bar.title),
            crate::style::get(ui.ctx()).font(12.5),
            ui.visuals().text_color(),
        );
    }
    output
}

/// Flat, borderless text buttons for the menu row: 13 pt labels, 9×4 pt padding.
fn flat_menu_style(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);
    ui.spacing_mut().button_padding = Vec2::new(9.0, 4.0);
    let font = crate::style::get(ui.ctx()).font(13.0);
    ui.style_mut().text_styles.insert(egui::TextStyle::Button, font);
    let visuals = ui.visuals_mut();
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
}

/// A text menu in the title bar ("File", "Edit", …). The dropdown uses the compact
/// menu typography at `min_width` (200–230 pt in MD View).
pub fn title_menu(ui: &mut Ui, label: &str, min_width: f32, add: impl FnOnce(&mut Ui)) {
    ui.menu_button(label, |ui| {
        crate::compact_menu(ui);
        ui.set_min_width(min_width);
        add(ui);
    });
}

/// A 44×32 pt window control: transparent at rest, hover fill on hover; the close
/// button turns red (#C42B1C) with a white glyph.
fn window_button(ui: &mut Ui, icons: &mut Icons, icon: &str, close: bool, tooltip: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(WINDOW_BUTTON_WIDTH, WINDOW_BUTTON_HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, tooltip));
    let hovered = response.hovered();
    let fill = match (hovered, close) {
        (true, true) => Color32::from_rgb(196, 43, 28),
        (true, false) => ui.visuals().widgets.hovered.weak_bg_fill,
        (false, _) => Color32::TRANSPARENT,
    };
    ui.painter().rect_filled(rect, 0.0, fill);
    let color = if hovered && close {
        Color32::WHITE
    } else {
        ui.visuals().text_color()
    };
    icons.put(ui, icon, rect.center(), ICON_SMALL, color);
    response.on_hover_text(tooltip).clicked()
}

/// A 104×24 pt pill with a 12 pt radius, a 14 pt glyph, and a 12 pt label.
fn pill_button(ui: &mut Ui, icons: &mut Icons, center: egui::Pos2, pill: &Pill<'_>) -> bool {
    let rect = Rect::from_center_size(center, Vec2::new(104.0, 24.0));
    let response = ui.interact(rect, ui.scope_id().with("title_bar_pill"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, pill.label));
    let fill = if response.hovered() {
        ui.visuals().widgets.hovered.weak_bg_fill
    } else {
        ui.visuals().widgets.inactive.weak_bg_fill
    };
    ui.painter()
        .rect_filled(rect, crate::style::get(ui.ctx()).pill(rect.height()), fill);
    ui.painter().rect_stroke(
        rect,
        crate::style::get(ui.ctx()).pill(rect.height()),
        ui.visuals().widgets.noninteractive.bg_stroke,
        egui::StrokeKind::Inside,
    );
    let color = ui.visuals().text_color();
    icons.paint(
        ui,
        pill.icon,
        egui::pos2(rect.left() + 17.0, rect.center().y),
        ICON_SMALL,
        color,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 29.0, rect.center().y),
        Align2::LEFT_CENTER,
        pill.label,
        crate::style::get(ui.ctx()).font(12.0),
        color,
    );
    let response = if pill.tooltip.is_empty() {
        response
    } else {
        response.on_hover_text(pill.tooltip)
    };
    response.clicked()
}
