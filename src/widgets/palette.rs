//! The command palette and quick open.
//!
//! A 560 pt modal: a 15 pt bold title with "×", a focused filter field, and up to 12
//! matching 26 pt rows in a 300 pt scroll area. The first match is highlighted with
//! the selection fill and runs on Enter; a click runs any row. Esc or a backdrop click
//! closes it. Matching is a case-insensitive substring test on label and detail.

use egui::epaint::Shadow;
use egui::{Color32, Context, Frame, Id, Key, Modal, RichText, ScrollArea, Sense};

use crate::{close_glyph, text_field};

/// One palette row.
#[derive(Clone, Debug, PartialEq)]
pub struct PaletteItem<T> {
    /// Display label.
    pub label: String,
    /// Secondary text, shown after the label (a shortcut or a path).
    pub detail: String,
    /// Value returned when the row runs.
    pub value: T,
}

/// Persistent palette state.
#[derive(Clone, Debug, Default)]
pub struct PaletteState {
    /// Whether the palette is visible.
    pub open: bool,
    /// The filter text.
    pub query: String,
}

impl PaletteState {
    /// Open with an empty filter.
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
    }
}

/// Translated strings of the palette.
#[derive(Clone, Copy, Debug)]
pub struct PaletteLabels<'a> {
    /// Title, such as "Command Palette" or "Quick Open".
    pub title: &'a str,
    /// Filter hint.
    pub hint: &'a str,
    /// Shown when nothing matches.
    pub empty: &'a str,
    /// Close tooltip.
    pub close: &'a str,
}

/// Items whose label or detail contains the query, at most 12.
pub fn filter<'a, T>(items: &'a [PaletteItem<T>], query: &str) -> Vec<&'a PaletteItem<T>> {
    let needle = query.to_lowercase();
    items
        .iter()
        .filter(|item| {
            needle.is_empty()
                || item.label.to_lowercase().contains(&needle)
                || item.detail.to_lowercase().contains(&needle)
        })
        .take(12)
        .collect()
}

/// Show the palette and return the value of the row that ran.
pub fn show_palette<T: Clone>(
    ctx: &Context,
    state: &mut PaletteState,
    labels: PaletteLabels<'_>,
    items: &[PaletteItem<T>],
) -> Option<T> {
    if !state.open {
        return None;
    }
    let mut chosen = None;
    let mut keep_open = true;
    let visuals = ctx.style_of(ctx.theme()).visuals.clone();
    let frame = Frame::new()
        .fill(visuals.window_fill)
        .stroke(crate::style::get(ctx).border(visuals.window_stroke.color))
        .corner_radius(crate::style::get(ctx).dialog())
        .inner_margin(egui::Margin::same(14))
        .shadow(Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(60),
        });
    let dialog = Modal::new(Id::unique("workbench_palette"))
        .frame(frame)
        .show(ctx, |ui| {
            ui.set_width(560.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(labels.title)
                        .size(crate::style::get(ui.ctx()).font_size(15.0))
                        .strong()
                        .color(ui.visuals().text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if close_glyph(ui, 14.0, labels.close) {
                        keep_open = false;
                    }
                });
            });
            ui.add_space(8.0);
            // The field spans the same width as the result rows below it.
            text_field(ui, &mut state.query, labels.hint, ui.available_width()).request_focus();
            ui.add_space(8.0);
            let matches = filter(items, &state.query);
            ScrollArea::vertical()
                .max_height(300.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    if matches.is_empty() {
                        ui.label(RichText::new(labels.empty).weak());
                    }
                    for (index, item) in matches.iter().enumerate() {
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), crate::style::get(ui.ctx()).list_row_height()),
                            Sense::click(),
                        );
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(egui::Role::ListBoxOption, true, index == 0, &item.label)
                        });
                        if index == 0 {
                            ui.painter().rect_filled(
                                rect,
                                crate::style::get(ui.ctx()).item(),
                                ui.visuals().selection.bg_fill,
                            );
                        } else if response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                crate::style::get(ui.ctx()).item(),
                                ui.visuals().widgets.hovered.weak_bg_fill,
                            );
                        }
                        // Label on the left, detail (usually the shortcut) right-aligned and
                        // muted, like the menus.
                        ui.painter().text(
                            egui::pos2(rect.left() + 10.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &item.label,
                            crate::style::get(ui.ctx()).font(12.5),
                            ui.visuals().text_color(),
                        );
                        if !item.detail.is_empty() {
                            ui.painter().text(
                                egui::pos2(rect.right() - 10.0, rect.center().y),
                                egui::Align2::RIGHT_CENTER,
                                &item.detail,
                                crate::style::get(ui.ctx()).font(11.5),
                                ui.visuals().weak_text_color(),
                            );
                        }
                        if response.clicked() {
                            chosen = Some(item.value.clone());
                            keep_open = false;
                        }
                    }
                });
            if ctx.input(|input| input.key_pressed(Key::Enter))
                && let Some(item) = matches.first()
            {
                chosen = Some(item.value.clone());
                keep_open = false;
            }
        });
    if dialog.backdrop_response.clicked() || ctx.input(|input| input.key_pressed(Key::Escape)) {
        keep_open = false;
    }
    state.open = keep_open;
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtering_matches_label_and_detail_case_insensitively() {
        let item = |label: &str, detail: &str| PaletteItem {
            label: label.to_owned(),
            detail: detail.to_owned(),
            value: label.to_owned(),
        };
        let items = vec![
            item("Save", "Ctrl+S"),
            item("Open Folder…", "Ctrl+Shift+O"),
            item("Quit", ""),
        ];
        let labels = |query: &str| {
            filter(&items, query)
                .iter()
                .map(|item| item.label.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(labels("").len(), 3);
        assert_eq!(labels("folder"), ["Open Folder…"]);
        assert_eq!(labels("ctrl+s"), ["Save", "Open Folder…"]);
    }
}
