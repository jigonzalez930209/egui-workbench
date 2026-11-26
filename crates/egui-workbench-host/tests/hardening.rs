//! Regression tests for the protocol and runtime.

use std::io::Cursor;

use egui_workbench_host::{FrameError, Host, HostAction, HostSpec, Patch, read_frame};
use serde_json::json;

#[test]
fn an_empty_stream_is_a_clean_close() {
    assert_eq!(read_frame(Cursor::new(Vec::new())).unwrap_err(), FrameError::Closed);
}

#[test]
fn a_truncated_frame_is_an_io_error() {
    let mut bytes = 100u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(b"{}");
    assert!(matches!(read_frame(Cursor::new(bytes)), Err(FrameError::Io(_))));
}

#[test]
fn a_huge_announced_length_does_not_reserve_memory() {
    let bytes = (60u32 * 1024 * 1024).to_be_bytes().to_vec();
    assert!(read_frame(Cursor::new(bytes)).is_err());
}

#[test]
fn a_patch_cannot_nest_the_spec_past_the_depth_limit() {
    let mut host = Host::new(HostSpec::default()).expect("host");
    let mut row = json!({"kind": "separator"});
    for _ in 0..300 {
        row = json!({"kind": "row", "children": [row], "gap": 0.0});
    }
    let patch = Patch::Set {
        path: "/widgets".to_owned(),
        value: json!([row]),
    };
    let error = host.apply(&patch).expect_err("too deep");
    assert!(error.to_string().contains("nested deeper"), "{error}");
    // The failed patch left the spec untouched.
    assert_eq!(host.spec().widgets.len(), 1);
}

#[test]
fn moderate_nesting_is_still_accepted() {
    let mut host = Host::new(HostSpec::default()).expect("host");
    let mut row = json!({"kind": "separator"});
    for _ in 0..10 {
        row = json!({"kind": "row", "children": [row], "gap": 0.0});
    }
    let patch = Patch::Set {
        path: "/widgets".to_owned(),
        value: json!([row]),
    };
    host.apply(&patch).expect("accepted");
}

#[test]
fn the_action_queue_survives_a_reload() {
    let directory = std::env::temp_dir().join(format!("wb-reload-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("dir");
    let path = directory.join("spec.json");
    std::fs::write(&path, r#"{"version":1}"#).expect("write");
    let mut host = Host::load(&path).expect("load");
    let queue = host.actions();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    std::fs::write(&path, r#"{"version":1,"window":{"title":"after"}}"#).expect("rewrite");
    assert!(host.reload_if_changed());
    queue.push(HostAction::SetTitle("pushed".to_owned()));
    host.apply_actions();
    assert_eq!(host.spec().window.title, "pushed");
    let _ = std::fs::remove_dir_all(&directory);
}
