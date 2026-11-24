//! Manifest validation, sandbox capabilities, and a compiled C extension.

use std::collections::BTreeMap;
use std::path::PathBuf;

use egui_workbench_extensions::ExtensionManager;
use egui_workbench_extensions::manifest::Capability;

fn example() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello-c")
}

#[test]
fn the_c_extension_registers_commands_panels_and_viewers() {
    let directory = tempfile::tempdir().expect("temp");
    let mut manager = ExtensionManager::new(directory.path()).expect("manager");
    let installed = manager.install(example()).expect("install");
    assert_eq!(installed.id, "com.example.hello");
    assert!(!installed.enabled);

    for capability in ["commands", "panels", "viewers", "data", "selection"] {
        manager.grant("com.example.hello", capability).expect("grant");
    }
    let info = manager.enable("com.example.hello").expect("enable");
    assert!(info.enabled);
    assert_eq!(info.commands[0].id, "hello.run");
    assert_eq!(info.panels[0].id, "hello.panel");
    assert_eq!(info.viewers[0].extensions, ["hello"]);

    let code = manager.invoke("hello.run").expect("invoke");
    assert_eq!(code, Some(0));
    assert!(manager.logs().iter().any(|log| log.message == "hello.run"));
}

#[test]
fn the_rust_extension_uses_the_same_host_api() {
    let directory = tempfile::tempdir().expect("temp");
    let mut manager = ExtensionManager::new(directory.path()).expect("manager");
    manager
        .install(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello-rust"))
        .expect("install");
    manager.grant("com.example.hello-rust", "commands").expect("grant");
    manager.grant("com.example.hello-rust", "panels").expect("grant");
    let info = manager.enable("com.example.hello-rust").expect("enable");
    assert_eq!(info.commands[0].id, "rust.run");
    manager.invoke("rust.run").expect("invoke");
}

#[test]
fn enabling_without_capabilities_is_refused() {
    let directory = tempfile::tempdir().expect("temp");
    let mut manager = ExtensionManager::new(directory.path()).expect("manager");
    manager.install(example()).expect("install");
    let error = manager.enable("com.example.hello").expect_err("must fail");
    assert!(format!("{error}").contains("missing capabilities"));
}

#[test]
fn reload_rebuilds_an_enabled_runtime() {
    let directory = tempfile::tempdir().expect("temp");
    let mut manager = ExtensionManager::new(directory.path()).expect("manager");
    manager.install(example()).expect("install");
    for capability in ["commands", "panels", "viewers", "data", "selection"] {
        manager.grant("com.example.hello", capability).expect("grant");
    }
    manager.enable("com.example.hello").expect("enable");
    let info = manager.reload("com.example.hello").expect("reload");
    assert!(info.enabled);
    assert_eq!(info.commands.len(), 1);
}

#[test]
fn capabilities_parse_and_the_host_api_range_is_checked() {
    assert!(
        matches!(Capability::parse("filesystem:read:notes").expect("cap"), Capability::FilesystemRead { prefix } if prefix == "notes")
    );
    assert!(Capability::parse("network").is_err());
    let manifest = egui_workbench_extensions::ExtensionManifest {
        id: "com.example.future".to_owned(),
        version: "0.1.0".to_owned(),
        name: "Future".to_owned(),
        api: egui_workbench_extensions::manifest::ApiRange {
            min: "0.16".to_owned(),
            max: "0.17".to_owned(),
        },
        entry: "future.wasm".to_owned(),
        checksum: None,
        signature: None,
        capabilities: Default::default(),
        author: None,
        description: None,
    };
    assert!(manifest.validate().is_err());
}

#[test]
fn datasets_are_metadata_only() {
    let directory = tempfile::tempdir().expect("temp");
    let mut manager = ExtensionManager::new(directory.path()).expect("manager");
    let mut datasets = BTreeMap::new();
    datasets.insert("trace".to_owned(), 42);
    manager.set_datasets(datasets);
    manager.set_selection("hello");
    assert_eq!(manager.root(), directory.path());
    assert!(manager.logs().is_empty());
    assert!(manager.denied().is_empty());
}
