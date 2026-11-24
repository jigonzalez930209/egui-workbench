//! The 34 pt document tab strip.
//!
//! The strip uses the raised surface (`window_fill`); the active tab uses the editor
//! background (`panel_fill`) so it merges with the document below, as in VS Code. Tabs
//! are 96–212 pt wide (label + 58 pt), have 4 pt top corners, a 16 pt icon at x+15,
//! the label at x+28 truncated with "…", and a 16 pt close button at the right edge
//! that fades in on hover and stays on the active tab. A dirty, inactive, unhovered tab
//! shows a 3 pt dot where the close button would be. The mouse wheel scrolls the strip
//! horizontally, and a newly activated tab is scrolled into view.

use egui::{Color32, CornerRadius, FontId, Rect, Sense, TextureId, Ui, Vec2};

use crate::FULL_UV;
use crate::Icons;
use crate::metrics::{ICON_MEDIUM, TAB_HEIGHT, TAB_STRIP_PADDING};

/// Tab label size; the width is measured with the same font the label is painted with.
const TAB_FONT_SIZE: f32 = 12.5;

/// The icon in front of a tab label.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TabIcon<'a> {
    /// No icon; the slot stays empty.
    None,
    /// A tinted UI glyph.
    Glyph(&'a str),
    /// A full-color texture, such as a Material file icon.
    Texture(TextureId),
}

/// One tab.
#[derive(Clone, Copy, Debug)]
pub struct TabItem<'a> {
    /// Label, usually the file name.
    pub label: &'a str,
    /// Icon.
    pub icon: TabIcon<'a>,
    /// Unsaved changes.
    pub dirty: bool,
    /// Paints the label in the warning color (a conflict with the disk).
    pub warning: bool,
    /// Strikes the label through (the file is gone).
    pub missing: bool,
}

impl<'a> TabItem<'a> {
    /// A clean tab with an icon.
    pub fn new(label: &'a str, icon: TabIcon<'a>) -> Self {
        Self {
            label,
            icon,
            dirty: false,
            warning: false,
            missing: false,
        }
    }
}

/// A tab interaction.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TabEvent {
    /// The tab was clicked.
    Select(usize),
    /// The close button or a middle click closed the tab.
    Close(usize),
}

/// Draw the strip inside a `Panel::top` of [`TAB_HEIGHT`].
///
/// It paints its whole `ui.max_rect()` as the strip surface, so call it inside a
/// fixed-height panel (or a sized container), never in a scrolling page.
///
/// `controls` runs right-to-left in the last `controls_width` points (276 in the reference shell:
/// the segmented mode switch, "+", "…", and history). `close_label` ("Close") is the
/// tooltip of the close buttons; each is announced as `Close <tab label>`.
pub fn show_tab_strip(
    ui: &mut Ui,
    icons: &mut Icons,
    tabs: &[TabItem<'_>],
    active: usize,
    close_label: &str,
    controls_width: f32,
    controls: impl FnOnce(&mut Ui, &mut Icons),
) -> Option<TabEvent> {
    let bar = ui.max_rect();
    ui.painter().rect_filled(bar, 0.0, ui.visuals().window_fill);
    let tabs_rect = Rect::from_min_max(
        egui::pos2(bar.left() + TAB_STRIP_PADDING, bar.top()),
        egui::pos2((bar.right() - controls_width).max(bar.left() + 80.0), bar.bottom()),
    );
    let mut event = None;
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(tabs_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| event = scrolling_tabs(ui, icons, tabs, active, close_label, tabs_rect),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(bar)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.add_space(2.0);
            controls(ui, icons);
        },
    );
    event
}

fn scrolling_tabs(
    ui: &mut Ui,
    icons: &mut Icons,
    tabs: &[TabItem<'_>],
    active: usize,
    close_label: &str,
    area: Rect,
) -> Option<TabEvent> {
    ui.spacing_mut().item_spacing.x = 0.0;
    let scroll_id = ui.scope_id().with("workbench_tab_scroll");
    let mut offset: f32 = ui.ctx().data_mut(|data| *data.get_temp_mut_or_default(scroll_id));
    if ui.rect_contains_pointer(area) {
        // The raw wheel events, not the smoothed scroll delta: the smoothed value keeps
        // moving after the wheel stops and overshot the end of the strip.
        let wheel = ui.input(|input| {
            input
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::MouseWheel {
                        unit, delta, modifiers, ..
                    } if !modifiers.command && !modifiers.alt => Some(match unit {
                        egui::MouseWheelUnit::Point => -delta.y,
                        egui::MouseWheelUnit::Line => -delta.y * 20.0,
                        egui::MouseWheelUnit::Page => -delta.y * area.width(),
                    }),
                    _ => None,
                })
                .sum::<f32>()
        });
        if wheel.abs() > 0.0 {
            offset = (offset + wheel).max(0.0);
            ui.ctx().request_repaint();
        }
    }
    let revealed_id = ui.scope_id().with("workbench_tab_revealed");
    let revealed: Option<usize> = ui.ctx().data(|data| data.get_temp(revealed_id));
    let mut active_span = None;
    let mut event = None;
    let output = egui::ScrollArea::horizontal()
        .auto_shrink([false, true])
        .scroll_source(egui::scroll_area::ScrollSource::SCROLL_BAR)
        .horizontal_scroll_offset(offset)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let origin = ui.cursor().min.x;
                for (index, tab) in tabs.iter().enumerate() {
                    let start = ui.cursor().min.x - origin;
                    let hit = tab_button(ui, icons, tab, index, index == active, close_label);
                    if index == active {
                        active_span = Some((start, ui.min_rect().right() - origin));
                    }
                    event = event.or(hit);
                }
            });
        });
    let mut stored = output.state.clamped_offset().x;
    if revealed != Some(active)
        && let Some((start, end)) = active_span
    {
        let visible = output.inner_rect.width();
        let max = (output.content_size.x - visible).max(0.0);
        let target = if start < stored {
            start
        } else if end > stored + visible {
            end - visible
        } else {
            stored
        };
        stored = target.clamp(0.0, max);
        if (stored - output.state.clamped_offset().x).abs() > 0.5 {
            ui.ctx().request_repaint();
        }
        ui.ctx().data_mut(|data| data.insert_temp(revealed_id, active));
    }
    ui.ctx().data_mut(|data| data.insert_temp(scroll_id, stored));
    event
}

fn tab_button(
    ui: &mut Ui,
    icons: &mut Icons,
    tab: &TabItem<'_>,
    index: usize,
    active: bool,
    close_label: &str,
) -> Option<TabEvent> {
    let font = FontId::proportional(TAB_FONT_SIZE);
    let text_width = ui
        .painter()
        .layout_no_wrap(tab.label.to_owned(), font.clone(), Color32::WHITE)
        .rect
        .width();
    let width = (text_width + 58.0).clamp(96.0, 212.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, TAB_HEIGHT), Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, active, tab.label));
    let active_t = ui
        .ctx()
        .animate_bool_with_time(response.id.with("active"), active, 0.16);
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(response.id.with("hover"), response.hovered() && !active, 0.10);
    let radius = CornerRadius {
        nw: 4,
        ne: 4,
        sw: 0,
        se: 0,
    };
    if active_t > 0.01 {
        ui.painter()
            .rect_filled(rect, radius, ui.visuals().panel_fill.gamma_multiply(active_t));
    }
    if hover_t > 0.01 {
        let fill = ui.visuals().widgets.hovered.weak_bg_fill.gamma_multiply(hover_t);
        ui.painter().rect_filled(rect, radius, fill);
    }
    let icon_center = egui::pos2(rect.left() + 15.0, rect.center().y);
    match tab.icon {
        TabIcon::None => {}
        TabIcon::Glyph(name) => {
            icons.paint(ui, name, icon_center, ICON_MEDIUM, ui.visuals().text_color());
        }
        TabIcon::Texture(texture) => {
            let icon_rect = Rect::from_center_size(icon_center, Vec2::splat(ICON_MEDIUM));
            ui.painter().image(texture, icon_rect, FULL_UV, Color32::WHITE);
        }
    }
    let color = if tab.warning || tab.missing {
        ui.visuals().warn_fg_color
    } else if active {
        ui.visuals().text_color()
    } else {
        ui.visuals().weak_text_color()
    };
    let mut job = egui::text::LayoutJob::single_section(tab.label.to_owned(), egui::TextFormat::simple(font, color));
    job.wrap = egui::text::TextWrapping::truncate_at_width((rect.width() - 28.0 - 26.0).max(16.0));
    let galley = ui.painter().layout_job(job);
    let text_pos = egui::pos2(rect.left() + 28.0, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(text_pos, galley.clone(), color);
    if tab.missing {
        let y = rect.center().y;
        ui.painter().line_segment(
            [
                egui::pos2(text_pos.x, y),
                egui::pos2(text_pos.x + galley.rect.width(), y),
            ],
            crate::style::get(ui.ctx()).border(color),
        );
    }
    if tab.dirty && !response.hovered() && !active {
        ui.painter().circle_filled(
            egui::pos2(rect.right() - 12.0, rect.center().y),
            3.0,
            ui.visuals().weak_text_color(),
        );
    }
    let close_rect = Rect::from_center_size(egui::pos2(rect.right() - 13.0, rect.center().y), Vec2::splat(16.0));
    let close = ui.interact(
        close_rect,
        ui.scope_id().with(("workbench_tab_close", index)),
        Sense::click(),
    );
    close.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, format!("{close_label} {}", tab.label)));
    let close = close.on_hover_text(close_label);
    // The close button is its own widget: using only the tab response made it fade out
    // as soon as the pointer reached it.
    let visible = active || ui.rect_contains_pointer(rect) || close.hovered();
    let close_t = ui
        .ctx()
        .animate_bool_with_time(response.id.with("close"), visible, 0.12);
    if close_t > 0.05 {
        if close.hovered() {
            ui.painter().rect_filled(
                close_rect,
                crate::style::get(ui.ctx()).tight(),
                ui.visuals().widgets.hovered.weak_bg_fill,
            );
        }
        let tint = ui.visuals().text_color().gamma_multiply(close_t);
        icons.paint(ui, "close", close_rect.center(), 12.0, tint);
    }
    if close.clicked() || response.middle_clicked() {
        Some(TabEvent::Close(index))
    } else if response.clicked() {
        Some(TabEvent::Select(index))
    } else {
        None
    }
}
