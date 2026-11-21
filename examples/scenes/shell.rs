//! The whole workbench: title bar with menus, explorer tree, tab strip, editor, status
//! bar, palette, and settings, wired to plain application state.

use egui_workbench::components::prelude::*;
use egui_workbench::egui;
use egui_workbench::metrics::{STATUS_HEIGHT, TAB_HEIGHT, TITLE_HEIGHT};
use egui_workbench::{IconState, Segment, StatusItem, ThemeId, TitleBar, TreeView};

use super::{Host, Scene};

const FILES: [(&str, &str); 6] = [
    (
        "docs/guide.md",
        "# Guide\n\nThe workbench shell, reusable.\n\n- Title bar, menus, and window controls\n- Explorer tree with keyboard navigation\n- Tabs with dirty markers\n- Status bar\n",
    ),
    (
        "docs/notes.md",
        "# Notes\n\nEdit this tab: the dot in the tab means unsaved.\n",
    ),
    (
        "src/main.rs",
        "fn main() -> eframe::Result {\n    eframe::run_native(\"App\", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))\n}\n",
    ),
    (
        "src/app.rs",
        "pub struct App {\n    theme: ThemeState,\n    tabs: Vec<Tab>,\n}\n",
    ),
    ("Cargo.toml", "[package]\nname = \"my-app\"\nversion = \"0.1.0\"\n"),
    ("README.md", "# my-app\n\nBuilt on egui-workbench.\n"),
];

struct Tab {
    key: &'static str,
    text: String,
    dirty: bool,
}

/// The interactive workbench.
pub struct Shell {
    tabs: Vec<Tab>,
    active: usize,
    tree: TreeState<&'static str>,
    expanded: Vec<&'static str>,
    explorer: bool,
    mode: usize,
    palette: PaletteState,
    settings_open: bool,
}

impl Default for Shell {
    fn default() -> Self {
        let mut shell = Self {
            tabs: Vec::new(),
            active: 0,
            tree: TreeState::default(),
            expanded: vec!["docs", "src"],
            explorer: true,
            mode: 1,
            palette: PaletteState::default(),
            settings_open: false,
        };
        shell.open("README.md");
        shell.open("docs/guide.md");
        shell
    }
}

impl Shell {
    fn open(&mut self, key: &'static str) {
        if let Some(index) = self.tabs.iter().position(|tab| tab.key == key) {
            self.active = index;
            return;
        }
        let text = FILES
            .iter()
            .find(|(name, _)| *name == key)
            .map_or("", |(_, text)| *text);
        self.tabs.push(Tab {
            key,
            text: text.to_owned(),
            dirty: false,
        });
        self.active = self.tabs.len() - 1;
    }

    fn nodes(&self) -> Vec<TreeNode<&'static str>> {
        let leaf = |key: &'static str| TreeNode::leaf(key, name(key), RowIcon::Glyph("files".to_owned()));
        let folder = |key: &'static str, children: Vec<TreeNode<&'static str>>| {
            let open = self.expanded.contains(&key);
            let icon = if open { "folder-open" } else { "folder" };
            TreeNode::folder(key, key, RowIcon::Glyph(icon.to_owned()), open, children)
        };
        vec![
            folder("docs", vec![leaf("docs/guide.md"), leaf("docs/notes.md")]),
            folder("src", vec![leaf("src/app.rs"), leaf("src/main.rs")]),
            leaf("Cargo.toml"),
            leaf("README.md"),
        ]
    }

    fn title_bar(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let title = self.tabs.get(self.active).map_or("Workbench", |tab| name(tab.key));
        let mut bar = TitleBar::new(title);
        bar.dirty = self.tabs.get(self.active).is_some_and(|tab| tab.dirty);
        let mut chosen: Option<String> = None;
        let explorer = self.explorer;
        let mut toggle_explorer = false;
        let mut palette = false;
        let mut settings = false;
        let mut save = false;
        let mut palette_button = false;
        let mut explorer_button = false;
        let theme = host.theme.requested();
        let mut new_theme = None;
        egui_workbench::show_title_bar(
            ui,
            host.icons,
            &bar,
            |ui, icons| {
                egui_workbench::title_menu(ui, "File", 220.0, |ui| {
                    if egui_workbench::menu_item(ui, icons, "file-plus", "New file", Some("Ctrl+N"), false) {
                        chosen = Some("New file".to_owned());
                    }
                    save = egui_workbench::menu_item(ui, icons, "save", "Save", Some("Ctrl+S"), false);
                });
                egui_workbench::title_menu(ui, "View", 230.0, |ui| {
                    if egui_workbench::menu_toggle(ui, "Explorer", explorer, true) {
                        toggle_explorer = true;
                    }
                    if egui_workbench::menu_button(ui, "Command palette") {
                        palette = true;
                    }
                    ui.separator();
                    for id in [
                        ThemeId::GithubLight,
                        ThemeId::GithubDark,
                        ThemeId::Dracula,
                        ThemeId::OneDarkPro,
                    ] {
                        if egui_workbench::menu_toggle(ui, id.label(), theme == id, true) {
                            new_theme = Some(id);
                        }
                    }
                });
                egui_workbench::title_menu(ui, "Help", 200.0, |ui| {
                    if egui_workbench::menu_button(ui, "About") {
                        chosen = Some("About".to_owned());
                    }
                });
            },
            |ui, icons| {
                settings = egui_workbench::action_button(ui, icons, "settings", "Settings", IconState::NORMAL);
                palette_button =
                    egui_workbench::action_button(ui, icons, "command", "Command Palette", IconState::NORMAL);
                explorer_button =
                    egui_workbench::action_button(ui, icons, "sidebar", "Explorer", IconState::active(explorer));
            },
        );
        if toggle_explorer || explorer_button {
            self.explorer = !self.explorer;
        }
        if palette || palette_button {
            self.palette.open = true;
        }
        if save {
            self.save(host);
        }
        if settings {
            self.settings_open = true;
        }
        if let Some(id) = new_theme {
            host.theme.set_requested(ui.ctx(), id);
            chosen = Some(format!("Theme: {}", id.label()));
        }
        if let Some(chosen) = chosen {
            host.report(chosen);
        }
    }
}

impl Shell {
    fn save(&mut self, host: &mut Host<'_>) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.dirty = false;
            host.toasts
                .add(Toast::new(ToastKind::Success, format!("Saved “{}”", name(tab.key))));
        }
    }

    fn settings(&mut self, ctx: &egui::Context, host: &mut Host<'_>) {
        let dialog = egui_workbench::SettingsDialog {
            title: "Settings",
            tabs: &["Appearance", "About"],
            note: "egui-workbench",
            done: "Done",
            close: "Close",
        };
        let theme = &mut *host.theme;
        egui_workbench::show_settings_dialog(ctx, &mut self.settings_open, &dialog, |ui, tab| {
            if tab == 0 {
                egui_workbench::settings_section(ui, "Appearance", "Applied immediately.");
                let mut id = theme.requested();
                let options: Vec<(ThemeId, String)> =
                    ThemeId::ALL.iter().map(|id| (*id, id.label().to_owned())).collect();
                egui_workbench::settings_row(ui, "Theme", "Color family for the whole workbench.", |ui| {
                    egui_workbench::combo(ui, "Theme", &mut id, &options);
                });
                if id != theme.requested() {
                    theme.set_requested(ui.ctx(), id);
                }
            } else {
                egui_workbench::settings_section(ui, "About", "A live egui-workbench shell.");
            }
        });
    }
}

fn name(key: &str) -> &str {
    key.rsplit('/').next().unwrap_or(key)
}

impl Scene for Shell {
    fn full(&self) -> bool {
        true
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        egui::Panel::top("scene_title")
            .exact_size(TITLE_HEIGHT)
            .show(ui, |ui| self.title_bar(ui, host));
        egui::Panel::bottom("scene_status")
            .exact_size(STATUS_HEIGHT)
            .show(ui, |ui| {
                let tab = self.tabs.get(self.active);
                let left = [
                    StatusItem::new(tab.map_or("No file", |tab| tab.key)),
                    StatusItem::new(if tab.is_some_and(|tab| tab.dirty) {
                        "Modified"
                    } else {
                        "Saved"
                    }),
                ];
                let right = [
                    StatusItem::new("UTF-8"),
                    StatusItem::new("LF"),
                    StatusItem::new(host.theme.requested().label()),
                ];
                egui_workbench::show_status_bar(ui, &left, &right);
            });
        if self.explorer {
            let width = (ui.available_width() * 0.3).clamp(150.0, 240.0);
            egui::Panel::left("scene_explorer")
                .resizable(false)
                .exact_size(width)
                .frame(egui::Frame::new().fill(ui.visuals().window_fill))
                .show(ui, |ui| {
                    let mut collapse = false;
                    egui_workbench::section_header(ui, host.icons, "my-app", |ui, icons| {
                        collapse = egui_workbench::small_button(ui, icons, "collapse", "Collapse folders");
                    });
                    if collapse {
                        self.expanded.clear();
                    }
                    let active = self.tabs.get(self.active).map(|tab| tab.key);
                    let view = TreeView {
                        id_salt: "scene_explorer",
                        active: active.as_ref(),
                        light: !host.theme.is_dark(),
                    };
                    let nodes = self.nodes();
                    let events =
                        egui_workbench::show_tree(ui, host.icons, &nodes, &mut self.tree, &view, |ui, icons, _| {
                            egui_workbench::menu_item(ui, icons, "eye", "Open", Some("Enter"), false);
                            egui_workbench::menu_item(ui, icons, "edit", "Rename…", Some("F2"), false);
                        });
                    for event in events {
                        match event {
                            TreeEvent::Toggle(key) => {
                                if let Some(index) = self.expanded.iter().position(|open| *open == key) {
                                    self.expanded.remove(index);
                                } else {
                                    self.expanded.push(key);
                                }
                            }
                            TreeEvent::Open(key) => self.open(key),
                            TreeEvent::Rename(key) => host.report(format!("Rename {key}: the app shows a NameDialog")),
                        }
                    }
                });
        }
        egui::Panel::top("scene_tabs").exact_size(TAB_HEIGHT).show(ui, |ui| {
            let items: Vec<TabItem<'_>> = self
                .tabs
                .iter()
                .map(|tab| TabItem {
                    dirty: tab.dirty,
                    ..TabItem::new(name(tab.key), TabIcon::Glyph("files"))
                })
                .collect();
            let mut mode = None;
            let event =
                egui_workbench::show_tab_strip(ui, host.icons, &items, self.active, "Close", 100.0, |ui, icons| {
                    let segments = [
                        Segment {
                            icon: "eye",
                            tooltip: "Preview",
                            enabled: true,
                        },
                        Segment {
                            icon: "edit",
                            tooltip: "Editor",
                            enabled: true,
                        },
                    ];
                    mode = egui_workbench::segmented(ui, icons, "scene_mode", &segments, self.mode);
                });
            if let Some(mode) = mode {
                self.mode = mode;
            }
            match event {
                Some(TabEvent::Select(index)) => self.active = index,
                Some(TabEvent::Close(index)) => {
                    let tab = self.tabs.remove(index);
                    if tab.dirty {
                        host.toasts.add(Toast::new(
                            ToastKind::Warning,
                            format!("Closed “{}” without saving", name(tab.key)),
                        ));
                    }
                    self.active = self.active.min(self.tabs.len().saturating_sub(1));
                }
                None => {}
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            let Some(tab) = self.tabs.get_mut(self.active) else {
                ui.centered_and_justified(|ui| Label::new("Open a file from the explorer").tone(Tone::Muted).show(ui));
                return;
            };
            // A breadcrumb that also names the editor for screen readers.
            let path = ui.label(egui::RichText::new(tab.key.replace('/', " / ")).small().weak());
            ui.add_space(4.0);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                if self.mode == 0 {
                    for line in tab.text.lines() {
                        match line.strip_prefix("# ") {
                            Some(heading) => {
                                ui.heading(heading);
                            }
                            None => {
                                ui.label(line);
                            }
                        }
                    }
                } else {
                    let edit = egui::TextEdit::multiline(&mut tab.text)
                        .code_editor()
                        .frame(egui::Frame::NONE)
                        .desired_width(f32::INFINITY);
                    if ui.add(edit).labelled_by(path.id).changed() {
                        tab.dirty = true;
                    }
                }
            });
        });
        let items: Vec<PaletteItem<&'static str>> = FILES
            .iter()
            .map(|(key, _)| PaletteItem {
                label: name(key).to_owned(),
                detail: (*key).to_owned(),
                value: *key,
            })
            .collect();
        let labels = PaletteLabels {
            title: "Go to File",
            hint: "Type a file name…",
            empty: "No matches",
            close: "Close",
        };
        if let Some(key) = CommandPalette::new(&mut self.palette, labels, &items).show(ui.ctx()) {
            self.open(key);
        }
        let (palette, save) = ui.input_mut(|input| {
            (
                input.consume_key(egui::Modifiers::COMMAND, egui::Key::P),
                input.consume_key(egui::Modifiers::COMMAND, egui::Key::S),
            )
        });
        if palette {
            self.palette.open = !self.palette.open;
        }
        if save {
            self.save(host);
        }
        self.settings(ui.ctx(), host);
    }
}
