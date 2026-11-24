//! Layout and workspace tests.

use super::*;

fn layout() -> Layout {
    Layout::with_tabs(vec![LayoutTab::new("a", "A"), LayoutTab::new("b", "B")])
}

#[test]
fn split_creates_a_second_group_and_focuses_it() {
    let mut layout = layout();
    let right = layout
        .split(EditorGroupId(0), SplitDirection::Horizontal)
        .expect("split");
    assert_eq!(layout.groups(), vec![EditorGroupId(0), right]);
    assert_eq!(layout.active, right);
    assert_eq!(layout.group(EditorGroupId(0)).map(|group| group.tabs.len()), Some(2));
    assert!(layout.group(right).is_some_and(|group| group.tabs.is_empty()));
}

#[test]
fn tabs_move_between_groups_and_reopen() {
    let mut layout = layout();
    let right = layout.split(EditorGroupId(0), SplitDirection::Vertical).expect("split");
    assert!(layout.move_tab(EditorGroupId(0), 0, right, 0));
    assert_eq!(layout.group(right).map(|group| group.tabs[0].title.as_str()), Some("A"));
    let closed = layout.close_tab(right, 0).expect("close");
    assert_eq!(closed.title, "A");
    assert_eq!(layout.reopen_closed().map(|tab| tab.title), Some("A".to_owned()));
}

#[test]
fn closing_a_group_promotes_its_sibling() {
    let mut layout = layout();
    let right = layout
        .split(EditorGroupId(0), SplitDirection::Horizontal)
        .expect("split");
    assert!(layout.close_group(right));
    assert_eq!(layout.groups(), vec![EditorGroupId(0)]);
    assert!(!layout.close_group(EditorGroupId(0)));
}

#[test]
fn keyboard_actions_drive_every_operation() {
    let mut layout = layout();
    assert!(layout.apply(LayoutAction::SplitRight));
    assert_eq!(layout.groups().len(), 2);
    assert!(layout.apply(LayoutAction::FocusNext));
    assert!(layout.apply(LayoutAction::MoveTabToNextGroup));
    assert!(layout.apply(LayoutAction::ToggleMaximize));
    assert!(layout.maximized.is_some());
    assert!(layout.apply(LayoutAction::ToggleZen));
    assert!(layout.zen);
    assert!(layout.apply(LayoutAction::CloseGroup));
    assert_eq!(layout.groups().len(), 1);
}

#[test]
#[cfg(feature = "serde")]
fn layouts_round_trip_and_normalize() {
    let mut layout = layout();
    let right = layout
        .split(EditorGroupId(0), SplitDirection::Horizontal)
        .expect("split");
    layout.maximize(Some(right));
    layout.set_zen(true);
    let json = layout.to_json().expect("serialize");
    let mut back = Layout::from_json(&json).expect("parse");
    assert_eq!(back, layout);
    back.active = EditorGroupId(99);
    back.normalize();
    assert!(back.group(back.active).is_some());
    assert!(back.maximized.is_some());
}

#[test]
#[cfg(feature = "serde")]
fn a_newer_layout_version_is_rejected() {
    let json = r#"{"version": 9, "root": {"kind": "group", "id": 0, "tabs": [], "active": 0}}"#;
    assert!(matches!(
        Layout::from_json(json),
        Err(LayoutError::UnsupportedVersion(9))
    ));
}
