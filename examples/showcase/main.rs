//! A complete workbench skeleton: undecorated window, title bar with menus, explorer,
//! tab strip with the mode switch, editor area, status bar, settings, palette, Quick
//! Open, search popover, keyboard menu, prompts, and the unsaved-changes guard.
//!
//! Run with `cargo run --example showcase -- [folder]`.

mod dialogs;
mod menus;
mod model;

use std::path::PathBuf;

use egui_workbench::app::{CloseGuard, Documents, ThemeState, WorkbenchApp, show_workbench};
use egui_workbench::egui;
use egui_workbench::{
    IconState, Icons, KeyboardMenuState, PaletteState, Pill, Segment, StatusItem, TabEvent, TabIcon, TabItem, ThemeId,
    TitleBar, TreeEvent, TreeState, TreeView, WindowAction, action_button, menu_item, menu_tool_button, segmented,
    small_button, tool_button,
};

use model::{Cmd, Doc, FsNode};

fn main() -> eframe::Result {
    let root = std::env::args().nth(1).map(PathBuf::from);
    eframe::run_native(
        "Workbench",
        egui_workbench::native_options("Workbench"),
        Box::new(move |cc| Ok(Box::new(Showcase::new(&cc.egui_ctx, root)))),
    )
}

use egui_workbench::eframe;

/// Which overlay or prompt is showing.
#[derive(Default)]
enum Overlay {
    #[default]
    None,
    Rename(usize, String),
    Delete(usize),
    ClosePrompt,
    About,
}

struct Showcase {
    icons: Icons,
    theme: ThemeState,
    sidebar_open: bool,
    root: Option<FsNode>,
    tree: TreeState<PathBuf>,
    docs: Documents<Doc>,
    mode: usize,
    split_ratio: f32,
    untitled: usize,
    palette: PaletteState,
    quick_open: PaletteState,
    search: Option<String>,
    keyboard_menu: KeyboardMenuState,
    settings_open: bool,
    overlay: Overlay,
    guard: CloseGuard,
}

impl Showcase {
    fn new(ctx: &egui::Context, root: Option<PathBuf>) -> Self {
        let root = root
            .or_else(|| std::env::current_dir().ok())
            .map(|path| FsNode::root(&path));
        Self {
            icons: Icons::new(),
            theme: ThemeState::new(ctx, ThemeId::GithubDark, false),
            sidebar_open: true,
            root,
            tree: TreeState::default(),
            docs: Documents::new(),
            mode: 1,
            split_ratio: 0.5,
            untitled: 0,
            palette: PaletteState::default(),
            quick_open: PaletteState::default(),
            search: None,
            keyboard_menu: KeyboardMenuState::default(),
            settings_open: false,
            overlay: Overlay::None,
            guard: CloseGuard::new(),
        }
    }

    fn light(&self) -> bool {
        self.theme.is_light()
    }

    fn handle(&mut self, ctx: &egui::Context, cmd: Cmd) {
        match cmd {
            Cmd::New => {
                self.untitled += 1;
                self.docs.push_active(Doc::untitled(self.untitled));
            }
            Cmd::Save => {
                if let Some(doc) = self.docs.active_mut()
                    && let Some(path) = &doc.path
                    && std::fs::write(path, &doc.text).is_ok()
                {
                    doc.dirty = false;
                }
            }
            Cmd::CloseTab => self.close(self.docs.active_index()),
            Cmd::CloseOthers => {
                let mut index = self.docs.len();
                while index > 0 {
                    index -= 1;
                    if index != self.docs.active_index() && !self.docs.get(index).is_some_and(|doc| doc.dirty) {
                        self.docs.remove(index);
                    }
                }
            }
            Cmd::NextTab => {
                self.docs.select_next();
            }
            Cmd::PreviousTab => {
                self.docs.select_previous();
            }
            Cmd::Rename => {
                if let Some(doc) = self.docs.active() {
                    self.overlay = Overlay::Rename(self.docs.active_index(), doc.name.clone());
                }
            }
            Cmd::Delete => self.overlay = Overlay::Delete(self.docs.active_index()),
            Cmd::ToggleSidebar => self.sidebar_open = !self.sidebar_open,
            Cmd::SetMode(mode) => self.mode = mode,
            Cmd::Palette => self.palette.open(),
            Cmd::QuickOpen => self.quick_open.open(),
            Cmd::Search => {
                self.search = if self.search.is_some() {
                    None
                } else {
                    Some(String::new())
                };
            }
            Cmd::Settings => self.settings_open = true,
            Cmd::SetTheme(id) => self.theme.set_requested(ctx, id),
            Cmd::ToggleFollowSystem => self.theme.set_follow_system(ctx, !self.theme.follow_system()),
            Cmd::About => self.overlay = Overlay::About,
            Cmd::Quit => egui_workbench::apply_window_action(ctx, WindowAction::Close),
        }
    }

    fn close(&mut self, index: usize) {
        self.docs.remove(index);
    }

    fn open_path(&mut self, path: &std::path::Path) {
        if self.docs.select_by(|doc| doc.path.as_deref() == Some(path)).is_none() {
            self.docs.push_active(Doc::open(path));
        }
    }

    fn draw_title_bar(&mut self, ui: &mut egui::Ui) -> Option<Cmd> {
        let (mut out, mut right_out) = (None, None);
        let doc = self.docs.active();
        let mut bar = TitleBar::new(doc.map_or("", |doc| doc.name.as_str()));
        bar.dirty = doc.is_some_and(|doc| doc.dirty);
        bar.maximized = egui_workbench::is_maximized(ui.ctx());
        if self.root.is_none() {
            bar.pill = Some(Pill {
                icon: "folder-open",
                label: "Open Folder",
                tooltip: "",
            });
        }
        let (mode, sidebar, theme_id, follow) = (
            self.mode,
            self.sidebar_open,
            self.theme.requested(),
            self.theme.follow_system(),
        );
        let (search_open, has_root, has_docs) = (self.search.is_some(), self.root.is_some(), self.docs.len() > 1);
        let result = egui_workbench::show_title_bar(
            ui,
            &mut self.icons,
            &bar,
            |ui, icons| {
                ui.horizontal(|ui| menus::title_menus(ui, mode, sidebar, theme_id, follow, &mut out));
                ui.add_space(6.0);
                if action_button(ui, icons, "back", "Previous Tab", IconState::enabled(has_docs)) {
                    out = Some(Cmd::PreviousTab);
                }
                if action_button(ui, icons, "forward", "Next Tab", IconState::enabled(has_docs)) {
                    out = Some(Cmd::NextTab);
                }
            },
            |ui, icons| {
                if action_button(ui, icons, "settings", "Settings", IconState::NORMAL) {
                    right_out = Some(Cmd::Settings);
                }
                if action_button(ui, icons, "command", "Command Palette", IconState::NORMAL) {
                    right_out = Some(Cmd::Palette);
                }
                if action_button(ui, icons, "search", "Search", IconState::active(search_open)) {
                    right_out = Some(Cmd::Search);
                }
                if has_root && action_button(ui, icons, "sidebar", "Explorer", IconState::active(sidebar)) {
                    right_out = Some(Cmd::ToggleSidebar);
                }
            },
        );
        if let Some(action) = result.window {
            egui_workbench::apply_window_action(ui.ctx(), action);
        }
        out.or(right_out)
    }

    fn draw_sidebar(&mut self, ui: &mut egui::Ui) {
        let Some(root) = &mut self.root else {
            return;
        };
        let mut refresh = false;
        let mut collapse = false;
        egui_workbench::section_header(ui, &mut self.icons, &root.name, |ui, icons| {
            collapse = small_button(ui, icons, "collapse", "Collapse folders");
            refresh = small_button(ui, icons, "refresh", "Refresh");
            small_button(ui, icons, "folder-plus", "New folder");
            small_button(ui, icons, "file-plus", "New file");
        });
        if refresh {
            root.refresh();
        }
        if collapse {
            root.collapse_all();
        }
        let nodes = root.view();
        let active = self.docs.active().and_then(|doc| doc.path.clone());
        let view = TreeView {
            id_salt: "explorer",
            active: active.as_ref(),
            light: self.theme.is_light(),
        };
        let mut menu_open = None;
        let events =
            egui_workbench::show_tree(ui, &mut self.icons, &nodes, &mut self.tree, &view, |ui, icons, node| {
                if !node.is_folder() && menu_item(ui, icons, "eye", "Open", Some("Enter"), false) {
                    menu_open = Some(node.key.clone());
                    ui.close();
                }
                menu_item(ui, icons, "folder-open", "Reveal", None, false);
                ui.separator();
                menu_item(ui, icons, "edit", "Rename…", Some("F2"), false);
                menu_item(ui, icons, "close", "Delete…", None, true);
            });
        for event in events {
            match event {
                TreeEvent::Toggle(path) => {
                    root.toggle(&path);
                }
                TreeEvent::Open(path) => menu_open = Some(path),
                TreeEvent::Rename(_) => {}
            }
        }
        if let Some(path) = menu_open {
            self.open_path(&path);
        }
    }

    fn draw_tabs(&mut self, ui: &mut egui::Ui) -> Option<Cmd> {
        let light = self.light();
        let icon_ids: Vec<Option<egui::TextureId>> = self
            .docs
            .items()
            .iter()
            .map(|doc| {
                let id = doc.path.as_deref().map_or_else(
                    || "markdown".to_owned(),
                    |path| self.icons.material.file_icon(path, light),
                );
                self.icons
                    .material
                    .texture(ui.ctx(), &id, light)
                    .map(|texture| texture.id())
            })
            .collect();
        let tabs: Vec<TabItem<'_>> = self
            .docs
            .items()
            .iter()
            .zip(&icon_ids)
            .map(|(doc, icon)| TabItem {
                dirty: doc.dirty,
                ..TabItem::new(&doc.name, icon.map_or(TabIcon::Glyph("files"), TabIcon::Texture))
            })
            .collect();
        let mut out = None;
        let mode = self.mode;
        let event = egui_workbench::show_tab_strip(
            ui,
            &mut self.icons,
            &tabs,
            self.docs.active_index(),
            "Close",
            276.0,
            |ui, icons| {
                if tool_button(ui, icons, "history", "Quick Open", false) {
                    out = Some(Cmd::QuickOpen);
                }
                menu_tool_button(ui, icons, "more", "Tab actions", 210.0, |ui, _icons| {
                    for (label, cmd) in [
                        ("Close Tab", Cmd::CloseTab),
                        ("Close Other Tabs", Cmd::CloseOthers),
                        ("Save", Cmd::Save),
                    ] {
                        if egui_workbench::menu_button(ui, label) {
                            out = Some(cmd);
                        }
                    }
                });
                if tool_button(ui, icons, "plus", "New tab", false) {
                    out = Some(Cmd::New);
                }
                let segments = [
                    Segment {
                        icon: "eye",
                        tooltip: "View",
                        enabled: true,
                    },
                    Segment {
                        icon: "edit",
                        tooltip: "Editor",
                        enabled: true,
                    },
                    Segment {
                        icon: "split",
                        tooltip: "Split",
                        enabled: true,
                    },
                ];
                if let Some(index) = segmented(ui, icons, "mode", &segments, mode) {
                    out = Some(Cmd::SetMode(index));
                }
            },
        );
        match event {
            Some(TabEvent::Select(index)) => {
                self.docs.set_active(index);
            }
            Some(TabEvent::Close(index)) => self.close(index),
            None => {}
        }
        out
    }

    fn draw_status_bar(&self, ui: &mut egui::Ui) {
        let doc = self.docs.active();
        let path = doc
            .and_then(|doc| doc.path.as_ref())
            .map_or_else(|| "No file open".to_owned(), |path| path.display().to_string());
        let left = [
            StatusItem::new(egui_workbench::ellipsize_start(&path, 48)).tooltip(path),
            StatusItem::new(if doc.is_some_and(|doc| doc.dirty) {
                "Modified"
            } else {
                "Saved"
            }),
            StatusItem::new(format!("{} lines", doc.map_or(0, |doc| doc.text.lines().count()))),
        ];
        let right = [
            StatusItem::new("UTF-8"),
            StatusItem::new("LF"),
            StatusItem::new("Spaces: 4"),
            StatusItem::new("Markdown"),
        ];
        egui_workbench::show_status_bar(ui, &left, &right);
    }

    fn draw_center(&mut self, ui: &mut egui::Ui) {
        if self.docs.is_empty() {
            if egui_workbench::show_empty_state(
                ui,
                "Workbench",
                "A native egui workbench.",
                "Open file",
                "Open a file to begin.",
            ) {
                self.handle(ui.ctx(), Cmd::QuickOpen);
            }
            return;
        }
        let Some(doc) = self.docs.active_mut() else {
            return;
        };
        let editor = |ui: &mut egui::Ui, doc: &mut Doc| {
            egui_workbench::with_editor_scrollbars(ui, |ui| {
                egui::ScrollArea::both()
                    .id_salt("editor")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut doc.text)
                                .code_editor()
                                .frame(egui::Frame::NONE)
                                .desired_width(f32::INFINITY),
                        );
                        doc.dirty |= response.changed();
                    });
            });
        };
        let preview = |ui: &mut egui::Ui, text: &str| {
            egui::ScrollArea::vertical()
                .id_salt("preview")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(text).size(14.0));
                });
        };
        match self.mode {
            0 => preview(ui, &doc.text),
            1 => editor(ui, doc),
            _ => {
                // The preview shows the text as of the start of the frame.
                let snapshot = doc.text.clone();
                let mut ratio = self.split_ratio;
                egui_workbench::split_panes(ui, &mut ratio, |ui| editor(ui, doc), |ui| preview(ui, &snapshot));
                self.split_ratio = ratio;
            }
        }
    }
}

impl WorkbenchApp for Showcase {
    type Command = Cmd;

    fn sidebar_visible(&self) -> bool {
        self.sidebar_open && self.root.is_some()
    }

    fn tab_strip_visible(&self) -> bool {
        true
    }

    fn status_bar_visible(&self) -> bool {
        true
    }

    fn title_bar(&mut self, ui: &mut egui::Ui) -> Vec<Cmd> {
        self.draw_title_bar(ui).into_iter().collect()
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) -> Vec<Cmd> {
        self.draw_sidebar(ui);
        Vec::new()
    }

    fn tab_strip(&mut self, ui: &mut egui::Ui) -> Vec<Cmd> {
        self.draw_tabs(ui).into_iter().collect()
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) -> Vec<Cmd> {
        self.draw_status_bar(ui);
        Vec::new()
    }

    fn central(&mut self, ui: &mut egui::Ui) -> Vec<Cmd> {
        self.draw_center(ui);
        Vec::new()
    }

    fn overlays(&mut self, ui: &mut egui::Ui) -> Vec<Cmd> {
        dialogs::show(self, ui.ctx())
    }
}

impl eframe::App for Showcase {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
        if self.guard.should_prompt(ctx, self.docs.any_dirty()) {
            self.overlay = Overlay::ClosePrompt;
        }
        if matches!(self.overlay, Overlay::None)
            && !self.palette.open
            && !self.quick_open.open
            && !self.settings_open
            && let Some(cmd) = menus::shortcut(ctx, &mut self.keyboard_menu, self.mode)
        {
            self.handle(ctx, cmd);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let commands = show_workbench(ui, self);
        for cmd in commands {
            self.handle(&ctx, cmd);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}
