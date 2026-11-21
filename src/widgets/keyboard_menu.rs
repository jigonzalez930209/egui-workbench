//! A keyboard-first view of the title-bar menus.
//!
//! F10 or Alt+letter opens a group; Left/Right switch groups, Up/Down move between
//! items, Enter runs the item, Esc closes. The overlay is a 330 pt popover anchored at
//! (12, 40) from the window's top-left corner.

use egui::{Context, Key, RichText, Sense};

/// One item.
#[derive(Clone, Debug)]
pub struct MenuEntry<T> {
    /// Visible label.
    pub label: String,
    /// Value returned on Enter or click.
    pub value: T,
}

/// One top-level group ("File", "Edit", …).
#[derive(Clone, Debug)]
pub struct MenuGroup<T> {
    /// Visible group label.
    pub label: String,
    /// Items in display order.
    pub entries: Vec<MenuEntry<T>>,
}

/// Persistent selection state.
#[derive(Clone, Debug, Default)]
pub struct KeyboardMenuState {
    /// Whether the overlay is visible.
    pub open: bool,
    /// Selected group.
    pub group: usize,
    /// Selected item.
    pub item: usize,
}

impl KeyboardMenuState {
    /// Open a group at its first item.
    pub fn open(&mut self, group: usize) {
        self.open = true;
        self.group = group;
        self.item = 0;
    }
}

/// Open the menu on F10 (first group) or on Alt plus one of `letters` (one per group).
/// Consumes the key and the text event Alt+letter produces on some backends.
pub fn open_from_keyboard(ctx: &Context, state: &mut KeyboardMenuState, letters: &[Key]) -> bool {
    let group = letters
        .iter()
        .position(|key| ctx.input_mut(|input| input.consume_key(egui::Modifiers::ALT, *key)))
        .or_else(|| {
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::F10))
                .then_some(0)
        });
    let Some(group) = group else {
        return false;
    };
    state.open(group);
    ctx.input_mut(|input| input.events.retain(|event| !matches!(event, egui::Event::Text(_))));
    true
}

/// Draw the overlay and return the value of the item that ran.
pub fn show_keyboard_menu<T: Clone>(
    ctx: &Context,
    state: &mut KeyboardMenuState,
    groups: &[MenuGroup<T>],
    hint: &str,
) -> Option<T> {
    if !state.open || groups.is_empty() {
        return None;
    }
    let mut chosen = None;
    let position = egui::pos2(ctx.content_rect().left() + 12.0, ctx.content_rect().top() + 40.0);
    egui::Area::new(egui::Id::unique("workbench_keyboard_menu"))
        .order(egui::Order::Foreground)
        .fixed_pos(position)
        .show(ctx, |ui| {
            crate::popover_frame(ui)
                .corner_radius(crate::style::get(ui.ctx()).container())
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.set_width(330.0);
                    ui.horizontal_wrapped(|ui| {
                        for (index, group) in groups.iter().enumerate() {
                            if ui.selectable_label(index == state.group, &group.label).clicked() {
                                state.group = index;
                                state.item = 0;
                            }
                        }
                    });
                    ui.separator();
                    let group = &groups[state.group.min(groups.len() - 1)];
                    state.item = state.item.min(group.entries.len().saturating_sub(1));
                    for (index, entry) in group.entries.iter().enumerate() {
                        let selected = index == state.item;
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), Sense::click());
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(egui::Role::MenuItem, true, selected, &entry.label)
                        });
                        let visuals = ui.visuals();
                        if selected {
                            ui.painter().rect_filled(
                                rect,
                                crate::style::get(ui.ctx()).inner(),
                                visuals.selection.bg_fill,
                            );
                        } else if response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                crate::style::get(ui.ctx()).inner(),
                                visuals.widgets.hovered.weak_bg_fill,
                            );
                        }
                        let color = if selected {
                            visuals.selection.stroke.color
                        } else {
                            visuals.text_color()
                        };
                        ui.painter().text(
                            egui::pos2(rect.left() + 8.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &entry.label,
                            crate::style::get(ui.ctx()).font(12.0),
                            color,
                        );
                        if response.clicked() {
                            state.item = index;
                            chosen = Some(entry.value.clone());
                        }
                    }
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(hint)
                            .size(crate::style::get(ui.ctx()).font_size(10.0))
                            .weak(),
                    );
                });
        });
    let pressed = |key| ctx.input(|input| input.key_pressed(key));
    if pressed(Key::Escape) {
        state.open = false;
        return None;
    }
    if pressed(Key::ArrowRight) {
        state.group = (state.group + 1) % groups.len();
        state.item = 0;
    }
    if pressed(Key::ArrowLeft) {
        state.group = (state.group + groups.len() - 1) % groups.len();
        state.item = 0;
    }
    let group = &groups[state.group.min(groups.len() - 1)];
    if pressed(Key::ArrowDown) {
        state.item = (state.item + 1).min(group.entries.len().saturating_sub(1));
    }
    if pressed(Key::ArrowUp) {
        state.item = state.item.saturating_sub(1);
    }
    if pressed(Key::Enter)
        && let Some(entry) = group.entries.get(state.item)
    {
        chosen = Some(entry.value.clone());
    }
    if chosen.is_some() {
        state.open = false;
    }
    chosen
}
