//! Host runtime, protocol, schema, data, and the conformance fixture.

use std::sync::Arc;

use egui_workbench::data::{Column, Dataset};
use egui_workbench_host::data::dataset_from_columns;
use egui_workbench_host::{Host, HostSpec, Message, Patch, Request, Response, SPEC_VERSION, WidgetSpec};
use serde_json::json;

#[test]
fn a_default_spec_creates_a_host_and_a_schema() {
    let host = Host::new(HostSpec::default()).expect("host");
    assert_eq!(host.spec().version, SPEC_VERSION);
    assert_eq!(host.revision(), 0);
    let schema = HostSpec::schema();
    assert!(schema.get("$schema").is_some());
    assert!(schema.to_string().contains("HostSpec"));
}

#[test]
fn patches_update_the_spec_and_protocol_requests_respond() {
    let mut host = Host::new(HostSpec::default()).expect("host");
    host.apply(&Patch::Set {
        path: "/widgets/0/text".to_owned(),
        value: json!("Patched"),
    })
    .expect("patch");
    assert_eq!(host.json()["widgets"][0]["text"], "Patched");
    assert!(host.revision() > 0);
    let response = host.handle(Message::Request {
        id: 7,
        request: Request::GetSpec,
    });
    match response {
        Message::Response {
            id: 7,
            response: Response::Spec { spec },
        } => assert_eq!(spec["widgets"][0]["text"], "Patched"),
        other => panic!("unexpected response: {other:?}"),
    }
    let response = host.handle(Message::Request {
        id: 8,
        request: Request::RunCommand {
            id: "file.save".to_owned(),
        },
    });
    assert!(matches!(response, Message::Response { id: 8, .. }));
    assert!(
        host.poll_events()
            .iter()
            .any(|event| format!("{event:?}").contains("file.save"))
    );
}

#[test]
fn data_columns_feed_a_dataset_and_widget() {
    let columns = vec![
        egui_workbench_host::data::BufferColumn::Float {
            name: "x".to_owned(),
            values: vec![0.0, 1.0],
            validity: None,
        },
        egui_workbench_host::data::BufferColumn::Int {
            name: "y".to_owned(),
            values: vec![1, 2],
            validity: None,
        },
    ];
    let dataset = dataset_from_columns(columns);
    let mut host = Host::new(HostSpec::default()).expect("host");
    host.set_dataset("trace", dataset);
    assert_eq!(host.dataset("trace").map(Dataset::row_count), Some(2));
    let mut direct = Dataset::from_columns([Column::floats("v", vec![1.0])]);
    direct.set_metadata("source", "direct");
    host.set_dataset("direct", direct);
    assert!(host.dataset("direct").is_some());
    let _ = Arc::new(());
}

#[test]
fn the_conformance_fixture_applies() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/basic.json");
    let fixture: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    let spec: HostSpec = serde_json::from_value(fixture["spec"].clone()).expect("spec");
    let mut host = Host::new(spec).expect("host");
    let patches: Vec<Patch> = serde_json::from_value(fixture["patches"].clone()).expect("patches");
    for patch in patches {
        host.apply(&patch).expect("patch");
    }
    assert_eq!(host.json()["widgets"][0]["value"], "Ada");
    assert_eq!(host.json()["widgets"][2]["kind"], "label");
    assert!(
        host.spec()
            .widgets
            .iter()
            .any(|widget| matches!(widget, WidgetSpec::Label { text, .. } if text == "Hello, Ada"))
    );
}

#[test]
fn a_spec_file_loads_and_reloads() {
    let path = std::env::temp_dir().join(format!("egui-workbench-host-{}.json", std::process::id()));
    let spec = HostSpec {
        version: SPEC_VERSION,
        ..HostSpec::default()
    };
    std::fs::write(&path, serde_json::to_string_pretty(&spec).expect("json")).expect("write");
    let mut host = Host::load(&path).expect("load");
    assert!(host.source().is_some());
    assert!(!host.reload_if_changed());
    let _ = std::fs::remove_file(path);
}

#[test]
fn queued_actions_apply_on_the_next_frame() {
    use egui_workbench_host::{HostAction, ToastSpec};

    let spec = HostSpec {
        version: SPEC_VERSION,
        widgets: vec![WidgetSpec::Label {
            text: "before".to_owned(),
            size: 13.0,
            tone: "default".to_owned(),
            strong: false,
        }],
        ..HostSpec::default()
    };
    let mut host = Host::new(spec).expect("host");
    let actions = host.actions();
    actions.push(HostAction::Patch(Patch::Set {
        path: "/widgets/0/text".to_owned(),
        value: json!("after"),
    }));
    actions.push(HostAction::Toast(ToastSpec {
        kind: "success".to_owned(),
        title: "Saved".to_owned(),
        description: None,
    }));
    assert!(!host.apply_actions());
    assert_eq!(host.json()["widgets"][0]["text"], "after");
    let _ = host.frame();
}

#[test]
fn initial_toasts_are_seeded_from_the_spec() {
    use egui_workbench_host::ToastSpec;

    let spec = HostSpec {
        version: SPEC_VERSION,
        toasts: vec![ToastSpec {
            kind: "success".to_owned(),
            title: "Ready".to_owned(),
            description: None,
        }],
        ..HostSpec::default()
    };
    let host = Host::new(spec).expect("host");
    assert_eq!(host.toast_count(), 1);
}

#[test]
fn a_newer_spec_version_is_rejected() {
    let spec = HostSpec {
        version: SPEC_VERSION + 1,
        ..HostSpec::default()
    };
    assert!(matches!(
        Host::new(spec),
        Err(egui_workbench_host::HostError::UnsupportedVersion(_))
    ));
}
