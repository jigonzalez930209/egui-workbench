//! Layouts as data: groups, tab moves, keyboard actions, named layouts, persistence.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_workbench::app::{EditorGroupId, Layout, LayoutAction, LayoutTab, SplitDirection, Workspace};

#[test]
fn three_groups_from_two_splits_and_a_keyboard_move() {
    let mut layout = Layout::with_tabs(vec![LayoutTab::new("a", "A"), LayoutTab::new("b", "B").pinned()]);
    let right = layout
        .split(EditorGroupId(0), SplitDirection::Horizontal)
        .expect("split");
    let bottom = layout.split(EditorGroupId(0), SplitDirection::Vertical).expect("split");
    assert_eq!(layout.groups().len(), 3);
    assert!(layout.apply(LayoutAction::FocusNext));
    assert!(layout.focus(EditorGroupId(0)));
    assert!(layout.apply(LayoutAction::MoveTabToNextGroup));
    assert!(layout.group(right).is_some() || layout.group(bottom).is_some());
    assert!(layout.apply(LayoutAction::ToggleMaximize));
    assert!(layout.maximized.is_some());
    assert!(layout.apply(LayoutAction::ToggleZen));
    assert!(layout.zen);
}

#[test]
fn a_layout_survives_the_workspace() {
    let mut workspace = Workspace::new();
    workspace.layout = Layout::with_tabs(vec![LayoutTab::new("a", "A")]);
    let right = workspace
        .layout
        .split(EditorGroupId(0), SplitDirection::Horizontal)
        .expect("split");
    workspace.layout.focus(right);
    workspace.save_layout("Analysis");
    let json = workspace.to_json().expect("serialize");
    let mut back = Workspace::from_json(&json).expect("parse");
    assert_eq!(back.layout, workspace.layout);
    assert_eq!(back.active_layout, "Analysis");
    assert!(back.switch_layout("Analysis"));
}

#[test]
fn closed_tabs_reopen_and_groups_promote_their_sibling() {
    let mut layout = Layout::with_tabs(vec![LayoutTab::new("a", "A")]);
    let right = layout
        .split(EditorGroupId(0), SplitDirection::Horizontal)
        .expect("split");
    let closed = layout.close_tab(EditorGroupId(0), 0).expect("close");
    assert_eq!(closed.title, "A");
    assert_eq!(layout.reopen_closed().map(|tab| tab.title), Some("A".to_owned()));
    assert!(layout.close_group(right));
    assert_eq!(layout.groups(), vec![EditorGroupId(0)]);
}
