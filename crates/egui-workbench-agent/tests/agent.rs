//! MCP, permissions, accessibility, and automation.

use egui_workbench_agent::{Automation, AutomationStep, McpServer, Permissions, accessibility_tree, find_node};
use egui_workbench_host::{CommandSpec, Host, HostSpec, Patch, SPEC_VERSION, WidgetSpec};
use serde_json::{Value, json};

fn spec() -> HostSpec {
    HostSpec {
        version: SPEC_VERSION,
        commands: vec![
            CommandSpec {
                id: "save".to_owned(),
                title: "Save".to_owned(),
                category: "file".to_owned(),
                shortcut: Some("Ctrl+S".to_owned()),
                ..CommandSpec::default()
            },
            CommandSpec {
                id: "delete".to_owned(),
                title: "Delete".to_owned(),
                category: "destructive".to_owned(),
                ..CommandSpec::default()
            },
        ],
        widgets: vec![
            WidgetSpec::Label {
                text: "before".to_owned(),
                size: 13.0,
                tone: "default".to_owned(),
                strong: false,
            },
            WidgetSpec::Button {
                id: "save".to_owned(),
                label: "Save".to_owned(),
                variant: "primary".to_owned(),
                enabled: true,
            },
            WidgetSpec::TextField {
                id: "name".to_owned(),
                label: "Name".to_owned(),
                value: String::new(),
                hint: String::new(),
            },
        ],
        ..HostSpec::default()
    }
}

fn call(server: &mut McpServer, id: u64, name: &str, arguments: Value) -> Value {
    server
        .handle(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments }
        }))
        .expect("response")
}

#[test]
fn initialize_and_list_tools() {
    let mut server = McpServer::new(Host::new(spec()).expect("host"));
    let initialized = server
        .handle(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}))
        .expect("response");
    assert_eq!(initialized["result"]["serverInfo"]["name"], "egui-workbench");
    let tools = server
        .handle(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}))
        .expect("response");
    let names = tools["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"apply_patch"));
    assert!(names.contains(&"run_command"));
    assert!(names.contains(&"get_accessibility_tree"));
}

#[test]
fn read_only_denies_writes_and_logs_them() {
    let mut server = McpServer::new(Host::new(spec()).expect("host"));
    let response = call(
        &mut server,
        1,
        "apply_patch",
        json!({"patch":{"op":"set","path":"/widgets/0/text","value":"after"}}),
    );
    assert_eq!(response["result"]["isError"], true);
    assert!(
        response["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("read-only")
    );
    assert_eq!(server.host().json()["widgets"][0]["text"], "before");
    assert_eq!(server.activity().len(), 1);
    assert!(!server.activity()[0].allowed);
}

#[test]
fn writes_commands_and_confirmation_are_enforced() {
    let permissions = Permissions::read_write()
        .allow_commands(["save", "delete"])
        .confirm(["delete"], "token");
    let mut server = McpServer::new(Host::new(spec()).expect("host")).with_permissions(permissions);
    let patched = call(
        &mut server,
        1,
        "apply_patch",
        json!({"patch":{"op":"set","path":"/widgets/0/text","value":"after"}}),
    );
    assert_eq!(patched["result"]["isError"], false);
    assert_eq!(server.host().json()["widgets"][0]["text"], "after");

    let denied = call(&mut server, 2, "run_command", json!({"id":"unknown"}));
    assert_eq!(denied["result"]["isError"], true);
    let needs_confirmation = call(&mut server, 3, "run_command", json!({"id":"delete"}));
    assert_eq!(needs_confirmation["result"]["isError"], true);
    let confirmed = call(
        &mut server,
        4,
        "run_command",
        json!({"id":"delete","confirmation":"token"}),
    );
    assert_eq!(confirmed["result"]["isError"], false);
    assert!(server.activity().iter().all(|entry| entry.sequence > 0));
}

#[test]
fn a_screenshot_provider_is_exposed() {
    let mut server =
        McpServer::new(Host::new(spec()).expect("host")).with_screenshot(|_host| Ok(vec![137, 80, 78, 71]));
    let response = call(&mut server, 1, "get_screenshot", json!({}));
    assert_eq!(response["result"]["isError"], false);
    assert!(
        response["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .starts_with("data:image/png;base64,")
    );
    let mut bare = McpServer::new(Host::new(spec()).expect("host"));
    let response = call(&mut bare, 2, "get_screenshot", json!({}));
    assert_eq!(response["result"]["isError"], true);
}

#[test]
fn accessibility_tree_has_roles_and_names() {
    let tree = accessibility_tree(&spec());
    let button = find_node(&tree, "Save", Some("button")).expect("button");
    assert_eq!(button.id.as_deref(), Some("save"));
    let field = find_node(&tree, "Name", Some("text_field")).expect("field");
    assert_eq!(field.id.as_deref(), Some("name"));
}

#[test]
fn automation_records_commands_from_the_journal() {
    use egui_workbench::app::CommandJournal;

    let mut journal = CommandJournal::default();
    journal.append("save");
    journal.append("refresh");
    let flow = Automation::from_journal(&journal);
    assert_eq!(flow.steps.len(), 2);
    assert_eq!(flow.steps[0], AutomationStep::Command { id: "save".to_owned() });
}

#[test]
fn automation_replays_commands_and_patches() {
    let automation = Automation {
        steps: vec![
            AutomationStep::Command { id: "save".to_owned() },
            AutomationStep::Patch {
                patch: Patch::Set {
                    path: "/widgets/0/text".to_owned(),
                    value: json!("replayed"),
                },
            },
        ],
    };
    let mut host = Host::new(spec()).expect("host");
    let events = automation.replay(&mut host).expect("replay");
    assert!(events.iter().any(|event| format!("{event:?}").contains("save")));
    assert_eq!(host.json()["widgets"][0]["text"], "replayed");
}
