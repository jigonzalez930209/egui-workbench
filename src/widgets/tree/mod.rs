//! The explorer: a section header and a keyboard-navigable tree.
//!
//! Rows are 21 pt high with a 4 pt radius, inset 5 pt, indented 12 pt per depth. Each
//! row has a 14 pt chevron slot (empty for leaves, so siblings align), a 16 pt icon,
//! and a 12.5 pt label truncated with "…". The selected row takes the selection fill;
//! the row of the active document takes 45 % of it plus a 2 pt focus-colored bar inside
//! its left edge; hovered rows take the hover fill. Context menus open under the row.

use std::hash::Hash;

use egui::{Color32, Rect, RichText, Sense, TextureId, Ui, Vec2};

use crate::FULL_UV;
use crate::Icons;
use crate::metrics::{CHEVRON_SIZE, ICON_MEDIUM, TREE_INDENT};

mod keyboard;

/// The icon of a tree row.
#[derive(Clone, Debug, PartialEq)]
pub enum RowIcon {
    /// No icon; the slot stays empty.
    None,
    /// A tinted UI glyph.
    Glyph(String),
    /// A full-color texture.
    Texture(TextureId),
    /// The Material icon of a file path.
    #[cfg(feature = "material-icons")]
    MaterialFile(std::path::PathBuf),
    /// The Material icon of a folder name; open and closed variants follow `expanded`.
    #[cfg(feature = "material-icons")]
    MaterialFolder(String),
}

/// One node of the tree. `children: None` is a leaf; `Some(vec![])` is an empty folder.
#[derive(Clone, Debug)]
pub struct TreeNode<K> {
    /// Stable key, usually a path.
    pub key: K,
    /// Visible label.
    pub label: String,
    /// Icon.
    pub icon: RowIcon,
    /// Children of a folder.
    pub children: Option<Vec<TreeNode<K>>>,
    /// Whether a folder shows its children.
    pub expanded: bool,
    /// Hover text.
    pub tooltip: Option<String>,
}

impl<K> TreeNode<K> {
    /// A leaf.
    pub fn leaf(key: K, label: impl Into<String>, icon: RowIcon) -> Self {
        Self {
            key,
            label: label.into(),
            icon,
            children: None,
            expanded: false,
            tooltip: None,
        }
    }

    /// A folder.
    pub fn folder(key: K, label: impl Into<String>, icon: RowIcon, expanded: bool, children: Vec<Self>) -> Self {
        Self {
            key,
            label: label.into(),
            icon,
            children: Some(children),
            expanded,
            tooltip: None,
        }
    }

    /// Whether the node is a folder.
    pub fn is_folder(&self) -> bool {
        self.children.is_some()
    }
}

/// Something the application must apply to its model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeEvent<K> {
    /// Expand or collapse a folder (click, Enter, Left, Right).
    Toggle(K),
    /// Open a leaf (click or Enter).
    Open(K),
    /// Rename the selected node (F2).
    Rename(K),
}

/// Persistent tree state.
#[derive(Clone, Debug)]
pub struct TreeState<K> {
    /// Selected node.
    pub selected: Option<K>,
    /// Scroll the selected row into view on the next frame.
    pub scroll_to_selected: bool,
    last_active: Option<K>,
}

impl<K> Default for TreeState<K> {
    fn default() -> Self {
        Self {
            selected: None,
            scroll_to_selected: false,
            last_active: None,
        }
    }
}

/// Options of one tree frame.
pub struct TreeView<'a, K> {
    /// Id salt; one per tree.
    pub id_salt: &'a str,
    /// The node of the active document, highlighted and followed by the selection.
    pub active: Option<&'a K>,
    /// Light theme, for Material icon variants.
    pub light: bool,
}

/// Draw the tree in a vertical scroll area filling the panel and return its events.
///
/// `context_menu` fills the right-click menu of a node; use [`crate::compact_menu`] rows
/// from [`crate::menu_item`] and record the chosen action in captured state.
pub fn show_tree<K: Clone + PartialEq + Hash + std::fmt::Debug>(
    ui: &mut Ui,
    icons: &mut Icons,
    nodes: &[TreeNode<K>],
    state: &mut TreeState<K>,
    view: &TreeView<'_, K>,
    mut context_menu: impl FnMut(&mut Ui, &mut Icons, &TreeNode<K>),
) -> Vec<TreeEvent<K>> {
    if state.last_active.as_ref() != view.active {
        // The selection follows a *change* of active document only, so clicking a folder
        // is not overwritten on the next frame.
        state.last_active = view.active.cloned();
        if let Some(active) = view.active {
            state.selected = Some(active.clone());
            state.scroll_to_selected = true;
        }
    }
    let focus_id = ui.scope_id().with((view.id_salt, "tree_focus"));
    let mut events = Vec::new();
    let mut focus_requested = false;
    let scroll = egui::ScrollArea::vertical()
        .id_salt(view.id_salt)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(2.0);
            let mut rows = Rows {
                icons,
                state: &mut *state,
                view,
                events: &mut events,
                focus_requested: &mut focus_requested,
                menu: &mut context_menu,
            };
            for node in nodes {
                rows.node(ui, node, 0);
            }
            ui.add_space(12.0);
        });
    state.scroll_to_selected = false;
    // The tree takes keyboard focus as one widget; egui drops focus from an id that no
    // widget registers in a frame, so register it every frame.
    ui.interact(scroll.inner_rect, focus_id, Sense::focusable_noninteractive());
    if focus_requested {
        ui.memory_mut(|memory| memory.request_focus(focus_id));
    }
    if ui.memory(|memory| memory.has_focus(focus_id)) {
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                focus_id,
                egui::EventFilter {
                    tab: false,
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    escape: false,
                },
            );
        });
        keyboard::navigate(ui, nodes, state, focus_id, &mut events);
    }
    if !events.is_empty() {
        ui.ctx().request_repaint();
    }
    events
}

struct Rows<'a, 'v, K, M> {
    icons: &'a mut Icons,
    state: &'a mut TreeState<K>,
    view: &'a TreeView<'v, K>,
    events: &'a mut Vec<TreeEvent<K>>,
    focus_requested: &'a mut bool,
    menu: &'a mut M,
}

impl<K: Clone + PartialEq + Hash + std::fmt::Debug, M: FnMut(&mut Ui, &mut Icons, &TreeNode<K>)> Rows<'_, '_, K, M> {
    fn node(&mut self, ui: &mut Ui, node: &TreeNode<K>, depth: usize) {
        let response = tree_row(ui, self.icons, node, depth, self.state, self.view);
        let response = match &node.tooltip {
            Some(tooltip) => response.on_hover_text(tooltip),
            None => response,
        };
        if response.clicked() {
            self.state.selected = Some(node.key.clone());
            *self.focus_requested = true;
            self.events.push(if node.is_folder() {
                TreeEvent::Toggle(node.key.clone())
            } else {
                TreeEvent::Open(node.key.clone())
            });
        }
        if self.state.scroll_to_selected && self.state.selected.as_ref() == Some(&node.key) {
            // Minimal scroll: a visible row stays put; arrows never recentre the tree.
            response.scroll_to_me(None);
            self.state.scroll_to_selected = false;
        }
        let icons = &mut *self.icons;
        let menu = &mut *self.menu;
        egui::Popup::context_menu(&response)
            .at_position(egui::pos2(response.rect.left(), response.rect.bottom() + 1.0))
            .show(|ui| {
                crate::compact_menu(ui);
                ui.set_min_width(196.0);
                // Rows fill the available width; without a cap the menu spreads.
                ui.set_max_width(220.0);
                menu(ui, icons, node);
            });
        if node.expanded
            && let Some(children) = &node.children
        {
            for child in children {
                self.node(ui, child, depth + 1);
            }
        }
    }
}

/// Paint one row and return its click response.
fn tree_row<K: PartialEq + Hash + std::fmt::Debug>(
    ui: &mut Ui,
    icons: &mut Icons,
    node: &TreeNode<K>,
    depth: usize,
    state: &TreeState<K>,
    view: &TreeView<'_, K>,
) -> egui::Response {
    let selected = state.selected.as_ref() == Some(&node.key);
    let active = view.active == Some(&node.key);
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), crate::style::get(ui.ctx()).tree_row_height()),
        Sense::hover(),
    );
    let visuals = ui.visuals();
    let fill = if selected {
        visuals.selection.bg_fill
    } else if active {
        visuals.selection.bg_fill.gamma_multiply(0.45)
    } else if ui.rect_contains_pointer(rect) {
        visuals.widgets.hovered.weak_bg_fill
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, crate::style::get(ui.ctx()).inner(), fill);
    if active {
        let bar = Rect::from_min_size(
            egui::pos2(rect.left() + 4.0, rect.top() + 4.0),
            egui::vec2(2.0, rect.height() - 8.0),
        );
        ui.painter().rect_filled(
            bar,
            crate::style::get(ui.ctx()).mark(),
            visuals.widgets.active.bg_stroke.color,
        );
    }
    let inner = rect.shrink2(egui::vec2(5.0, 0.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.add_space(depth as f32 * TREE_INDENT + 2.0);
            if node.is_folder() {
                let chevron = if node.expanded { "chevron-down" } else { "chevron-right" };
                icons.show_tinted(ui, chevron, CHEVRON_SIZE, ui.visuals().weak_text_color());
            } else {
                ui.allocate_exact_size(Vec2::splat(CHEVRON_SIZE), Sense::hover());
            }
            row_icon(ui, icons, &node.icon, node.expanded, view.light);
            ui.add(
                egui::Label::new(
                    RichText::new(&node.label)
                        .size(crate::style::get(ui.ctx()).font_size(12.5))
                        .color(ui.visuals().text_color()),
                )
                .truncate(),
            );
        },
    );
    let response = ui.interact(rect, ui.scope_id().with(("tree_row", &node.key)), Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::TreeItem, true, selected, &node.label));
    response.on_hover_cursor(egui::CursorIcon::Default)
}

#[cfg_attr(not(feature = "material-icons"), allow(unused_variables))]
fn row_icon(ui: &mut Ui, icons: &mut Icons, icon: &RowIcon, expanded: bool, light: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(ICON_MEDIUM), Sense::hover());
    let texture = match icon {
        RowIcon::None => None,
        RowIcon::Glyph(name) => {
            icons.paint(ui, name, rect.center(), ICON_MEDIUM, ui.visuals().text_color());
            None
        }
        RowIcon::Texture(texture) => Some(*texture),
        #[cfg(feature = "material-icons")]
        RowIcon::MaterialFile(path) => {
            let id = icons.material.file_icon(path, light);
            icons.material.texture(ui.ctx(), &id, light).map(|texture| texture.id())
        }
        #[cfg(feature = "material-icons")]
        RowIcon::MaterialFolder(name) => {
            let id = icons.material.folder_icon(name, expanded, false, light);
            icons.material.texture(ui.ctx(), &id, light).map(|texture| texture.id())
        }
    };
    if let Some(texture) = texture {
        ui.painter().image(texture, rect, FULL_UV, Color32::WHITE);
    }
}

/// The explorer section header: the root name in 11.5 pt bold muted capitals, with
/// [`crate::small_button`]s added right to left by `actions`.
pub fn section_header(ui: &mut Ui, icons: &mut Icons, title: &str, actions: impl FnOnce(&mut Ui, &mut Icons)) {
    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        ui.label(
            RichText::new(title.to_uppercase())
                .size(crate::style::get(ui.ctx()).font_size(11.5))
                .strong()
                .color(ui.visuals().weak_text_color()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(4.0);
            actions(ui, icons);
        });
    });
    ui.add_space(4.0);
}
