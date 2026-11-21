//! The explorer: section header, rows, clicks, the keyboard, the active document, and
//! context menus.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_workbench::egui::{Key, Role, accesskit::Toggled};
use egui_workbench::metrics::{TREE_INDENT, TREE_ROW_HEIGHT};
use egui_workbench::{Icons, RowIcon, TreeEvent, TreeNode, TreeState, TreeView};

/// A small in-memory model: (key, parent, is_folder).
struct Model {
    expanded: Vec<&'static str>,
}

impl Model {
    fn nodes(&self) -> Vec<TreeNode<&'static str>> {
        let folder = |key: &'static str, children| {
            TreeNode::folder(
                key,
                key.rsplit('/').next().unwrap_or(key),
                RowIcon::MaterialFolder(key.into()),
                self.expanded.contains(&key),
                children,
            )
        };
        let file = |key: &'static str| {
            TreeNode::leaf(
                key,
                key.rsplit('/').next().unwrap_or(key),
                RowIcon::MaterialFile(key.into()),
            )
        };
        vec![
            folder(
                "docs",
                vec![
                    file("docs/guide.md"),
                    folder("docs/img", vec![file("docs/img/logo.svg")]),
                ],
            ),
            folder("src", vec![file("src/main.rs")]),
            file("README.md"),
            file("Cargo.toml"),
        ]
    }
}

struct State {
    icons: Icons,
    model: Model,
    tree: TreeState<&'static str>,
    active: Option<&'static str>,
    events: Vec<TreeEvent<&'static str>>,
    menu: Vec<String>,
    header: Vec<&'static str>,
}

fn explorer<'a>() -> Harness<'a, State> {
    let state = State {
        icons: Icons::new(),
        model: Model { expanded: Vec::new() },
        tree: TreeState::default(),
        active: None,
        events: Vec::new(),
        menu: Vec::new(),
        header: Vec::new(),
    };
    common::harness([320.0, 480.0], state, |ui, state: &mut State| {
        let mut header = Vec::new();
        egui_workbench::section_header(ui, &mut state.icons, "workspace", |ui, icons| {
            if egui_workbench::small_button(ui, icons, "collapse", "Collapse folders") {
                header.push("collapse");
            }
            if egui_workbench::small_button(ui, icons, "file-plus", "New file") {
                header.push("new file");
            }
        });
        state.header.extend(header);
        let nodes = state.model.nodes();
        let view = TreeView {
            id_salt: "explorer",
            active: state.active.as_ref(),
            light: true,
        };
        let mut menu = Vec::new();
        let events = egui_workbench::show_tree(
            ui,
            &mut state.icons,
            &nodes,
            &mut state.tree,
            &view,
            |ui, icons, node| {
                if egui_workbench::menu_item(ui, icons, "edit", "Rename…", Some("F2"), false) {
                    menu.push(format!("rename {}", node.key));
                    ui.close();
                }
                if egui_workbench::menu_item(ui, icons, "close", "Delete…", None, true) {
                    menu.push(format!("delete {}", node.key));
                    ui.close();
                }
            },
        );
        state.menu.extend(menu);
        for event in &events {
            if let TreeEvent::Toggle(key) = event {
                if let Some(position) = state.model.expanded.iter().position(|open| open == key) {
                    state.model.expanded.remove(position);
                } else {
                    state.model.expanded.push(key);
                }
            }
        }
        state.events.extend(events);
    })
}

fn row<'t>(harness: &'t Harness<'_, State>, label: &'t str) -> egui_kittest::Node<'t> {
    harness.get_by_role_and_label(Role::TreeItem, label)
}

fn is_selected(harness: &Harness<'_, State>, label: &str) -> bool {
    row(harness, label).accesskit_node().toggled() == Some(Toggled::True)
}

#[test]
fn the_header_shows_the_root_in_capitals_with_its_actions() {
    let mut harness = explorer();
    harness.run();
    assert!(harness.query_by_label("WORKSPACE").is_some());
    harness.get_by_role_and_label(Role::Button, "New file").click();
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Collapse folders").click();
    harness.run();
    assert_eq!(harness.state().header, ["new file", "collapse"]);
}

#[test]
fn clicking_a_folder_toggles_it_and_a_file_opens() {
    let mut harness = explorer();
    harness.run();
    assert!(harness.query_by_role_and_label(Role::TreeItem, "guide.md").is_none());
    row(&harness, "docs").click();
    harness.run();
    assert_eq!(harness.state().events, [TreeEvent::Toggle("docs")]);
    row(&harness, "guide.md").click();
    harness.run();
    assert_eq!(harness.state().events.last(), Some(&TreeEvent::Open("docs/guide.md")));
    assert!(is_selected(&harness, "guide.md"));
    row(&harness, "docs").click();
    harness.run();
    assert!(
        harness.query_by_role_and_label(Role::TreeItem, "guide.md").is_none(),
        "collapsed again"
    );
}

#[test]
fn rows_have_the_fixed_height_and_indent_by_depth() {
    let mut harness = explorer();
    harness.state_mut().model.expanded = vec!["docs", "docs/img"];
    harness.run();
    let docs = row(&harness, "docs").rect();
    let guide = row(&harness, "guide.md").rect();
    let logo = row(&harness, "logo.svg").rect();
    assert_eq!(docs.height(), TREE_ROW_HEIGHT);
    // 21 pt rows plus the 4 pt item spacing: a 25 pt pitch, as in MD View.
    assert_eq!(guide.top() - docs.top(), TREE_ROW_HEIGHT + 4.0);
    // The text of each depth starts 12 pt further right; rows span the full width.
    let text_left = |label: &str| {
        harness
            .get_all_by_label(label)
            .find(|node| node.accesskit_node().role() == Role::Label)
            .expect("label")
            .rect()
            .left()
    };
    assert_eq!(text_left("guide.md") - text_left("docs"), TREE_INDENT);
    assert_eq!(text_left("logo.svg") - text_left("guide.md"), TREE_INDENT);
    assert_eq!(docs.width(), logo.width());
}

#[test]
fn the_keyboard_moves_expands_collapses_opens_and_renames() {
    let mut harness = explorer();
    harness.run();
    row(&harness, "docs").click();
    harness.run();
    // docs is selected and expanded; the tree has focus.
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert!(is_selected(&harness, "guide.md"));
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().events.last(), Some(&TreeEvent::Open("docs/guide.md")));
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert!(is_selected(&harness, "img"));
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert!(
        harness.query_by_role_and_label(Role::TreeItem, "logo.svg").is_some(),
        "Right expands"
    );
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert!(is_selected(&harness, "logo.svg"), "Right steps into the first child");
    harness.key_press(Key::ArrowLeft);
    harness.run();
    assert!(is_selected(&harness, "img"), "Left goes to the parent");
    harness.key_press(Key::ArrowLeft);
    harness.run();
    assert!(
        harness.query_by_role_and_label(Role::TreeItem, "logo.svg").is_none(),
        "Left collapses"
    );
    harness.key_press(Key::F2);
    harness.run();
    assert_eq!(harness.state().events.last(), Some(&TreeEvent::Rename("docs/img")));
    harness.key_press(Key::ArrowUp);
    harness.key_press(Key::ArrowUp);
    harness.run();
    harness.key_press(Key::ArrowUp);
    harness.run();
    assert!(is_selected(&harness, "docs"), "Up stops at the first row");
}

#[test]
fn the_selection_follows_a_new_active_document_only() {
    let mut harness = explorer();
    harness.run();
    harness.state_mut().active = Some("README.md");
    harness.run();
    assert!(is_selected(&harness, "README.md"));
    row(&harness, "src").click();
    harness.run();
    assert!(
        is_selected(&harness, "src"),
        "a click is not overwritten by the unchanged active row"
    );
    assert!(!is_selected(&harness, "README.md"));
}

#[test]
fn the_context_menu_runs_its_items() {
    let mut harness = explorer();
    harness.run();
    row(&harness, "README.md").click_secondary();
    harness.run();
    let rename = harness.get_by_role_and_label(Role::MenuItem, "Rename…");
    let readme = row(&harness, "README.md").rect();
    assert!(
        (rename.rect().top() - readme.bottom()).abs() <= 6.0,
        "the menu opens under the row"
    );
    rename.click();
    harness.run();
    assert_eq!(harness.state().menu, ["rename README.md"]);
    assert!(harness.query_by_role_and_label(Role::MenuItem, "Rename…").is_none());

    row(&harness, "src").click_secondary();
    harness.run();
    harness.get_by_role_and_label(Role::MenuItem, "Delete…").click();
    harness.run();
    assert_eq!(harness.state().menu, ["rename README.md", "delete src"]);
}

#[test]
fn leaf_and_folder_nodes_report_their_kind() {
    let model = Model { expanded: vec![] };
    let nodes = model.nodes();
    assert!(nodes[0].is_folder());
    assert!(!nodes[2].is_folder());
}
