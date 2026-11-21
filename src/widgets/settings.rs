//! The settings dialog scaffold: a modal with a tab rail and grouped rows.
//!
//! The dialog is 560–620 pt wide (window width minus 48), with an 18 pt bold title and
//! "×"; a 128 pt rail of right-aligned 28 pt tab buttons; a separator; a scrolling body
//! 240–352 pt high (window height minus 150); then a separator and a footer with a small
//! weak note on the left (the app version) and the primary "Done" button on the right.
//! Each row pairs a 12.5 pt title and a small weak description (260 pt column) with one
//! control aligned right.

use egui::{Color32, Context, Frame, Id, Key, Modal, RichText, Role, ScrollArea, Sense, Ui, WidgetInfo};

use crate::{close_glyph, dialog_frame, primary_button, switch};

/// Translated strings and tabs of the dialog.
#[derive(Clone, Copy, Debug)]
pub struct SettingsDialog<'a> {
    /// Title, "Settings".
    pub title: &'a str,
    /// Tab labels, in rail order.
    pub tabs: &'a [&'a str],
    /// Footer note, usually "App 1.2.3".
    pub note: &'a str,
    /// Primary button label, "Done".
    pub done: &'a str,
    /// Close tooltip.
    pub close: &'a str,
}

/// Show the dialog while `open` is true. `body` draws the selected tab (by index) and
/// returns whatever the application wants back from this frame.
pub fn show_settings_dialog<R>(
    ctx: &Context,
    open: &mut bool,
    dialog: &SettingsDialog<'_>,
    body: impl FnOnce(&mut Ui, usize) -> R,
) -> Option<R> {
    if !*open {
        return None;
    }
    let tab_id = Id::unique(("workbench_settings_tab", dialog.title));
    let mut tab: usize = ctx.data_mut(|data| *data.get_temp_mut_or_default(tab_id));
    let mut keep_open = true;
    let mut result = None;
    let response = Modal::new(Id::unique("workbench_settings"))
        .frame(dialog_frame(ctx))
        .show(ctx, |ui| {
            let available = ctx.content_rect().size();
            // Preferred 560–620 × 240–352; never larger than the window minus the 18 pt frame
            // padding, a 16 pt gutter, and the header and footer (~110 pt).
            let width = (available.x - 48.0)
                .clamp(560.0, 620.0)
                .min((available.x - 2.0 * (18.0 + 16.0)).max(280.0));
            let height = (available.y - 150.0)
                .clamp(240.0, 352.0)
                .min((available.y - 2.0 * (18.0 + 16.0) - 110.0).max(120.0));
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(dialog.title)
                        .size(crate::style::get(ui.ctx()).font_size(18.0))
                        .strong()
                        .color(ui.visuals().text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if close_glyph(ui, 18.0, dialog.close) {
                        keep_open = false;
                    }
                });
            });
            ui.add_space(10.0);
            let rail = 128.0;
            let content = width - rail - 36.0;
            ui.allocate_ui_with_layout(
                egui::vec2(width, height),
                egui::Layout::left_to_right(egui::Align::Min),
                |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(rail, height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(rail);
                            ui.spacing_mut().item_spacing.y = 2.0;
                            for (index, label) in dialog.tabs.iter().enumerate() {
                                if tab_button(ui, label, index == tab) {
                                    tab = index;
                                }
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(10.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(content, height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(content);
                            ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .max_width(content)
                                .max_height(height)
                                .show(ui, |ui| {
                                    ui.set_width(content - 10.0);
                                    result = Some(body(ui, tab));
                                });
                        },
                    );
                },
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(dialog.note).small().weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, dialog.done, false).clicked() {
                        keep_open = false;
                    }
                });
            });
        });
    ctx.data_mut(|data| data.insert_temp(tab_id, tab));
    if response.backdrop_response.clicked() || ctx.input(|input| input.key_pressed(Key::Escape)) {
        keep_open = false;
    }
    *open = keep_open;
    result
}

/// A section heading: 15 pt bold title, a small weak description, 10 pt below.
pub fn settings_section(ui: &mut Ui, title: &str, description: &str) {
    ui.label(
        RichText::new(title)
            .size(crate::style::get(ui.ctx()).font_size(15.0))
            .strong()
            .color(ui.visuals().text_color()),
    );
    ui.label(RichText::new(description).small().weak());
    ui.add_space(10.0);
}

/// A row: title and description on the left (260 pt), `control` aligned right, 7 pt
/// vertical padding.
pub fn settings_row(ui: &mut Ui, title: &str, description: &str, control: impl FnOnce(&mut Ui)) {
    Frame::new().inner_margin(egui::Margin::symmetric(0, 7)).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(260.0);
                ui.set_max_width(260.0);
                ui.label(RichText::new(title).size(crate::style::get(ui.ctx()).font_size(12.5)));
                ui.label(RichText::new(description).small().weak());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), control);
        });
    });
}

/// A row with a [`switch`]; returns true when it was toggled.
pub fn settings_switch(ui: &mut Ui, title: &str, description: &str, value: &mut bool) -> bool {
    let mut changed = false;
    settings_row(ui, title, description, |ui| changed = switch(ui, value, title));
    changed
}

/// A 28 pt rail tab: right-aligned label, selection fill and a 2 pt underline when
/// selected, hover fill otherwise.
fn tab_button(ui: &mut Ui, label: &str, selected: bool) -> bool {
    let width = ui.available_width().clamp(96.0, 126.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 28.0), Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::Tab, true, selected, label));
    let visuals = ui.visuals();
    let fill = if selected {
        visuals.selection.bg_fill
    } else if response.hovered() {
        visuals.widgets.hovered.weak_bg_fill
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, crate::style::get(ui.ctx()).item(), fill);
    if selected {
        let underline = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 5.0, rect.bottom() - 2.0),
            egui::pos2(rect.right() - 5.0, rect.bottom()),
        );
        ui.painter().rect_filled(
            underline,
            crate::style::get(ui.ctx()).mark(),
            visuals.selection.stroke.color,
        );
    }
    let color = if selected {
        visuals.selection.stroke.color
    } else {
        visuals.text_color()
    };
    ui.painter().text(
        egui::pos2(rect.right() - 14.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        label,
        crate::style::get(ui.ctx()).font(12.5),
        color,
    );
    response.clicked()
}
