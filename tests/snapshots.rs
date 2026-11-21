//! Visual references of a complete workbench frame and its overlays.
//!
//! Refresh with `UPDATE_SNAPSHOTS=1 cargo test --test snapshots`, and look at every
//! changed PNG in `tests/snapshots/` before committing it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use egui_workbench::egui::{self, Role};
use egui_workbench::metrics::{SIDEBAR_WIDTH, STATUS_HEIGHT, TAB_HEIGHT, TITLE_HEIGHT};
use egui_workbench::{
    IconState, Icons, PaletteItem, PaletteLabels, PaletteState, Pill, RowIcon, Segment, SettingsDialog, StatusItem,
    TabIcon, TabItem, ThemeId, TitleBar, TreeNode, TreeState, TreeView, theme,
};

struct Frame {
    icons: Icons,
    tree: TreeState<&'static str>,
    light: bool,
    settings: bool,
    palette: PaletteState,
    pill: bool,
}

fn nodes() -> Vec<TreeNode<&'static str>> {
    let file = |key: &'static str| {
        TreeNode::leaf(
            key,
            key.rsplit('/').next().unwrap_or(key),
            RowIcon::MaterialFile(key.into()),
        )
    };
    vec![
        TreeNode::folder(
            "assets",
            "assets",
            RowIcon::MaterialFolder("assets".into()),
            false,
            vec![],
        ),
        TreeNode::folder(
            "docs",
            "docs",
            RowIcon::MaterialFolder("docs".into()),
            true,
            vec![file("docs/guide.md"), file("docs/notes.md")],
        ),
        TreeNode::folder("src", "src", RowIcon::MaterialFolder("src".into()), false, vec![]),
        file("Cargo.toml"),
        file("README.md"),
        file("main.rs"),
    ]
}

fn draw(ui: &mut egui::Ui, state: &mut Frame) {
    let icons = &mut state.icons;
    egui::Panel::top("title").exact_size(TITLE_HEIGHT).show(ui, |ui| {
        let mut bar = TitleBar::new("guide.md");
        bar.dirty = true;
        if state.pill {
            bar.pill = Some(Pill {
                icon: "folder-open",
                label: "Open Folder",
                tooltip: "",
            });
        }
        egui_workbench::show_title_bar(
            ui,
            icons,
            &bar,
            |ui, icons| {
                ui.horizontal(|ui| {
                    for (label, width) in [
                        ("File", 230.0),
                        ("Edit", 220.0),
                        ("View", 230.0),
                        ("Go", 200.0),
                        ("Help", 210.0),
                    ] {
                        egui_workbench::title_menu(ui, label, width, |ui| {
                            egui_workbench::menu_button(ui, "Item");
                        });
                    }
                });
                ui.add_space(6.0);
                egui_workbench::action_button(ui, icons, "back", "Back", IconState::enabled(false));
                egui_workbench::action_button(ui, icons, "forward", "Forward", IconState::NORMAL);
            },
            |ui, icons| {
                egui_workbench::action_button(ui, icons, "settings", "Settings", IconState::NORMAL);
                egui_workbench::action_button(ui, icons, "command", "Command Palette", IconState::NORMAL);
                egui_workbench::action_button(ui, icons, "search", "Search", IconState::NORMAL);
                egui_workbench::action_button(ui, icons, "sidebar", "Explorer", IconState::active(true));
            },
        );
    });
    egui::Panel::left("explorer")
        .resizable(false)
        .exact_size(SIDEBAR_WIDTH)
        .frame(egui::Frame::new().fill(ui.visuals().window_fill))
        .show(ui, |ui| {
            egui_workbench::section_header(ui, icons, "workspace", |ui, icons| {
                egui_workbench::small_button(ui, icons, "collapse", "Collapse folders");
                egui_workbench::small_button(ui, icons, "refresh", "Refresh");
                egui_workbench::small_button(ui, icons, "folder-plus", "New folder");
                egui_workbench::small_button(ui, icons, "file-plus", "New file");
            });
            let view = TreeView {
                id_salt: "explorer",
                active: Some(&"docs/guide.md"),
                light: state.light,
            };
            egui_workbench::show_tree(ui, icons, &nodes(), &mut state.tree, &view, |ui, icons, _| {
                egui_workbench::menu_item(ui, icons, "eye", "Open", Some("Enter"), false);
                egui_workbench::menu_item(ui, icons, "folder-open", "Reveal", None, false);
                ui.separator();
                egui_workbench::menu_item(ui, icons, "edit", "Rename…", Some("F2"), false);
                egui_workbench::menu_item(ui, icons, "close", "Delete…", None, true);
            });
        });
    egui::Panel::top("tabs").exact_size(TAB_HEIGHT).show(ui, |ui| {
        let tabs = [
            TabItem::new("README.md", TabIcon::Glyph("files")),
            TabItem::new("guide.md", TabIcon::Glyph("files")),
            TabItem {
                dirty: true,
                ..TabItem::new("notes.md", TabIcon::Glyph("files"))
            },
            TabItem {
                warning: true,
                ..TabItem::new("main.rs", TabIcon::Glyph("files"))
            },
        ];
        egui_workbench::show_tab_strip(ui, icons, &tabs, 1, "Close", 276.0, |ui, icons| {
            egui_workbench::tool_button(ui, icons, "history", "Quick Open", false);
            egui_workbench::menu_tool_button(ui, icons, "more", "Tab actions", 210.0, |_, _| {});
            egui_workbench::tool_button(ui, icons, "plus", "New tab", false);
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
                    enabled: false,
                },
            ];
            egui_workbench::segmented(ui, icons, "mode", &segments, 1);
        });
    });
    egui::Panel::bottom("status").exact_size(STATUS_HEIGHT).show(ui, |ui| {
        let left = [
            StatusItem::new("…/docs/guide.md"),
            StatusItem::new("Modified"),
            StatusItem::new("Ln 4, Col 12"),
        ];
        let right = [
            StatusItem::new("UTF-8"),
            StatusItem::new("LF"),
            StatusItem::new("Spaces: 4"),
            StatusItem::new("Markdown"),
        ];
        egui_workbench::show_status_bar(ui, &left, &right);
    });
    egui::CentralPanel::default().show(ui, |ui| {
        let warn = ui.visuals().warn_fg_color;
        egui_workbench::banner(ui, warn, "guide.md changed on disk.", |ui| {
            let _ = ui.button("Reload");
            let _ = ui.button("Keep mine");
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new("# Guide\n\nThe workbench shell, reusable.").monospace());
    });
    if state.settings {
        let dialog = SettingsDialog {
            title: "Settings",
            tabs: &["Appearance", "Editor", "About"],
            note: "App 1.0.0",
            done: "Done",
            close: "Close",
        };
        let mut open = true;
        let mut follow = true;
        let mut zoom = 1.0;
        egui_workbench::show_settings_dialog(ui.ctx(), &mut open, &dialog, |ui, _| {
            egui_workbench::settings_section(ui, "Appearance", "Theme, motion, and text size.");
            let mut id = ThemeId::GithubLight;
            let options: Vec<(ThemeId, String)> = ThemeId::ALL.iter().map(|id| (*id, id.label().to_owned())).collect();
            egui_workbench::settings_row(ui, "Theme", "Color family for the whole workbench.", |ui| {
                egui_workbench::combo(ui, "Theme", &mut id, &options);
            });
            egui_workbench::settings_switch(ui, "Follow system", "Match the desktop preference.", &mut follow);
            egui_workbench::settings_row(ui, "Text scale", "From 80% to 200%.", |ui| {
                egui_workbench::numeric_stepper(ui, icons, &mut zoom, 0.8..=2.0, 0.1, 112.0, |value| {
                    format!("{:.0}%", value * 100.0)
                });
            });
        });
    }
    let items: Vec<PaletteItem<u8>> = [("Save", "Ctrl+S"), ("Search", "Ctrl+F"), ("Settings…", "Ctrl+,")]
        .into_iter()
        .map(|(label, detail)| PaletteItem {
            label: label.into(),
            detail: detail.into(),
            value: 0,
        })
        .collect();
    let labels = PaletteLabels {
        title: "Command Palette",
        hint: "Type to filter…",
        empty: "No matches",
        close: "Close",
    };
    egui_workbench::show_palette(ui.ctx(), &mut state.palette, labels, &items);
}

fn workbench<'a>(id: ThemeId) -> Harness<'a, Frame> {
    let state = Frame {
        icons: Icons::new(),
        tree: TreeState::default(),
        light: id.mode() == egui_workbench::ThemeMode::Light,
        settings: false,
        palette: PaletteState::default(),
        pill: false,
    };
    Harness::builder()
        .with_size(egui::vec2(980.0, 600.0))
        .with_max_steps(64)
        .with_context_setup(move |ctx| theme::apply_id(ctx, id))
        .build_ui_state(draw, state)
}

fn snapshot_workbench(id: ThemeId, name: &str) {
    let mut harness = workbench(id);
    harness.run();
    harness.snapshot(name);
}

#[test]
fn workbench_github_light() {
    snapshot_workbench(ThemeId::GithubLight, "workbench-github-light");
}

#[test]
fn workbench_github_dark() {
    snapshot_workbench(ThemeId::GithubDark, "workbench-github-dark");
}

#[test]
fn workbench_dracula() {
    snapshot_workbench(ThemeId::Dracula, "workbench-dracula");
}

#[test]
fn settings_dialog() {
    let mut harness = workbench(ThemeId::GithubLight);
    harness.state_mut().settings = true;
    harness.run();
    harness.snapshot("settings-github-light");
}

#[test]
fn settings_dialog_dark() {
    let mut harness = workbench(ThemeId::GithubDark);
    harness.state_mut().settings = true;
    harness.run();
    harness.snapshot("settings-github-dark");
}

#[test]
fn command_palette() {
    let mut harness = workbench(ThemeId::OneDarkPro);
    harness.state_mut().palette.open();
    harness.run();
    harness.snapshot("palette-one-dark-pro");
}

#[test]
fn tree_context_menu_and_pill() {
    let mut harness = workbench(ThemeId::GithubDark);
    harness.state_mut().pill = true;
    harness.run();
    harness
        .get_by_role_and_label(Role::TreeItem, "README.md")
        .click_secondary();
    harness.run();
    harness.snapshot("context-menu-github-dark");
}
