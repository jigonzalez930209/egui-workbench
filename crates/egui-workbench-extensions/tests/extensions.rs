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
fn the_manifest_entry_cannot_escape_or_use_a_platform_prefix() {
    let base = egui_workbench_extensions::ExtensionManifest {
        id: "com.example.entry".to_owned(),
        version: "0.1.0".to_owned(),
        name: "Entry".to_owned(),
        api: egui_workbench_extensions::manifest::ApiRange {
            min: "0.15".to_owned(),
            max: "0.16".to_owned(),
        },
        entry: "module.wasm".to_owned(),
        checksum: None,
        signature: None,
        capabilities: Default::default(),
        author: None,
        description: None,
    };
    for entry in [r"C:\x.wasm", r"\\srv\x.wasm", "a/../b.wasm", "/x.wasm", "", r"a\b.wasm"] {
        let mut manifest = base.clone();
        manifest.entry = entry.to_owned();
        assert!(manifest.validate().is_err(), "`{entry}` must be rejected");
    }
    let mut nested = base;
    nested.entry = "modules/hello.wasm".to_owned();
    nested.validate().expect("a nested module path is valid");
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

fn command_module(body: &str) -> Vec<u8> {
    let json = r#"{"id":"read","title":"Read"}"#;
    let data = json.replace('\\', "\\\\").replace('"', "\\\"");
    let source = format!(
        r#"(module
  (import "workbench" "host_register" (func $register (param i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "{data}")
  (func (export "init") (result i32)
    (drop (call $register (i32.const 0) (i32.const 0) (i32.const {len})))
    (i32.const 0))
  {body})"#,
        len = json.len(),
        body = body
    );
    wat::parse_str(source).expect("wat")
}

fn reader_module(path: &std::path::Path, out_cap: i32) -> Vec<u8> {
    let json = r#"{"id":"read","title":"Read"}"#;
    let json_data = json.replace('\\', "\\\\").replace('"', "\\\"");
    let raw_path = path.display().to_string();
    let path_len = raw_path.len();
    let path = raw_path.replace('\\', "\\\\").replace('"', "\\\"");
    let source = format!(
        r#"(module
  (import "workbench" "host_register" (func $register (param i32 i32 i32) (result i32)))
  (import "workbench" "host_read_file" (func $read (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "{json_data}")
  (data (i32.const 2048) "{path}")
  (func (export "init") (result i32)
    (drop (call $register (i32.const 0) (i32.const 0) (i32.const {json_len})))
    (i32.const 0))
  (func (export "on_command") (param i32 i32) (result i32)
    (call $read (i32.const 2048) (i32.const {path_len}) (i32.const 4096) (i32.const {out_cap}))))"#,
        json_len = json.len(),
        path_len = path_len,
    );
    wat::parse_str(source).expect("wat")
}

fn install_module(module: &[u8], grant: &std::path::Path) -> ExtensionManager {
    let state = tempfile::tempdir().expect("temp");
    let directory = state.path().join("extension");
    std::fs::create_dir_all(&directory).expect("directory");
    let manifest = serde_json::json!({
        "id": "com.example.reader",
        "version": "0.1.0",
        "name": "Reader",
        "api": { "min": "0.15", "max": "0.16" },
        "entry": "ext.wasm",
        "capabilities": ["commands", format!("filesystem:read:{}", grant.display())],
    });
    std::fs::write(
        directory.join("extension.json"),
        serde_json::to_vec(&manifest).expect("json"),
    )
    .expect("manifest");
    std::fs::write(directory.join("ext.wasm"), module).expect("module");
    let mut manager = ExtensionManager::new(state.keep().join("state")).expect("manager");
    manager.install(directory).expect("install");
    manager.grant("com.example.reader", "commands").expect("grant commands");
    manager
        .grant("com.example.reader", &format!("filesystem:read:{}", grant.display()))
        .expect("grant filesystem");
    manager.enable("com.example.reader").expect("enable");
    manager
}

#[test]
fn a_granted_file_is_read() {
    let state = tempfile::tempdir().expect("temp");
    let grant = state.path().join("grant");
    std::fs::create_dir_all(&grant).expect("grant");
    std::fs::write(grant.join("allowed.txt"), b"data").expect("file");
    let mut manager = install_module(&reader_module(&grant.join("allowed.txt"), 65536), &grant);
    assert_eq!(manager.invoke("read").expect("invoke"), Some(4));
}

#[test]
fn a_traversal_outside_the_grant_is_denied() {
    let state = tempfile::tempdir().expect("temp");
    let grant = state.path().join("grant");
    std::fs::create_dir_all(&grant).expect("grant");
    std::fs::write(state.path().join("secret.txt"), b"secret").expect("secret");
    let path = grant.join("../secret.txt");
    let mut manager = install_module(&reader_module(&path, 65536), &grant);
    assert_eq!(manager.invoke("read").expect("invoke"), Some(-3));
}

#[test]
fn a_sibling_prefix_is_denied() {
    let state = tempfile::tempdir().expect("temp");
    let grant = state.path().join("grant");
    let sibling = state.path().join("grant-sibling");
    std::fs::create_dir_all(&grant).expect("grant");
    std::fs::create_dir_all(&sibling).expect("sibling");
    std::fs::write(sibling.join("x"), b"secret").expect("file");
    let mut manager = install_module(&reader_module(&sibling.join("x"), 65536), &grant);
    assert_eq!(manager.invoke("read").expect("invoke"), Some(-3));
}

#[cfg(unix)]
#[test]
fn a_symlink_inside_the_grant_cannot_escape() {
    let state = tempfile::tempdir().expect("temp");
    let grant = state.path().join("grant");
    let outside = state.path().join("outside");
    std::fs::create_dir_all(&grant).expect("grant");
    std::fs::create_dir_all(&outside).expect("outside");
    std::fs::write(outside.join("target.txt"), b"secret").expect("target");
    std::os::unix::fs::symlink(outside.join("target.txt"), grant.join("link")).expect("symlink");
    let mut manager = install_module(&reader_module(&grant.join("link"), 65536), &grant);
    assert_eq!(manager.invoke("read").expect("invoke"), Some(-3));
}

#[test]
fn a_file_larger_than_the_output_cap_is_denied_before_reading() {
    let state = tempfile::tempdir().expect("temp");
    let grant = state.path().join("grant");
    std::fs::create_dir_all(&grant).expect("grant");
    std::fs::write(grant.join("big.bin"), vec![0u8; 1024 * 1024]).expect("file");
    let mut manager = install_module(&reader_module(&grant.join("big.bin"), 4), &grant);
    assert_eq!(manager.invoke("read").expect("invoke"), Some(-5));
}

#[test]
fn an_infinite_loop_traps_within_the_fuel_budget() {
    let state = tempfile::tempdir().expect("temp");
    let grant = state.path().join("grant");
    std::fs::create_dir_all(&grant).expect("grant");
    let body = r#"(func (export "on_command") (param i32 i32) (result i32)
    (loop $forever (br $forever))
    (unreachable))"#;
    let mut manager = install_module(&command_module(body), &grant);
    let start = std::time::Instant::now();
    let error = manager.invoke("read").expect_err("must trap");
    assert!(start.elapsed() < std::time::Duration::from_secs(1));
    assert!(matches!(error, egui_workbench_extensions::ExtensionError::Runtime(_)));
}

#[test]
fn memory_growth_past_the_limit_traps() {
    let state = tempfile::tempdir().expect("temp");
    let grant = state.path().join("grant");
    std::fs::create_dir_all(&grant).expect("grant");
    let body = r#"(func (export "init") (result i32)
    (drop (memory.grow (i32.const 2000)))
    (i32.store8 (i32.const 68157440) (i32.const 1))
    (i32.const 0))"#;
    let state_root = tempfile::tempdir().expect("temp");
    let directory = state_root.path().join("extension");
    std::fs::create_dir_all(&directory).expect("directory");
    let manifest = serde_json::json!({
        "id": "com.example.reader",
        "version": "0.1.0",
        "name": "Reader",
        "api": { "min": "0.15", "max": "0.16" },
        "entry": "ext.wasm",
        "capabilities": ["commands"],
    });
    std::fs::write(
        directory.join("extension.json"),
        serde_json::to_vec(&manifest).expect("json"),
    )
    .expect("manifest");
    std::fs::write(directory.join("ext.wasm"), command_module(body)).expect("module");
    let mut manager = ExtensionManager::new(state_root.path().join("state")).expect("manager");
    manager.install(directory).expect("install");
    manager.grant("com.example.reader", "commands").expect("grant");
    let error = manager.enable("com.example.reader").expect_err("must trap");
    assert!(
        matches!(error, egui_workbench_extensions::ExtensionError::Runtime(_)),
        "{error:?}"
    );
}
