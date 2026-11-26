//! Regression tests for the permission model.

use egui_workbench_agent::{McpServer, Permissions};
use egui_workbench_host::{Host, HostSpec};
use serde_json::{Value, json};

fn host() -> Host {
    let spec = HostSpec::from_json(r#"{"commands":[{"id":"danger","title":"Danger","enabled":true}]}"#).expect("spec");
    Host::new(spec).expect("host")
}

fn call(server: &mut McpServer, id: u64, name: &str, arguments: Value) -> Value {
    server
        .handle(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":arguments}}))
        .expect("response")
}

fn is_error(response: &Value) -> bool {
    response["result"]["isError"].as_bool().unwrap_or(false)
}

#[test]
fn an_empty_confirmation_token_never_matches() {
    let host = host();
    let permissions = Permissions::read_write().confirm(["danger"], "");
    let mut server = McpServer::new(host).with_permissions(permissions);
    let response = call(
        &mut server,
        1,
        "run_command",
        json!({"id": "danger", "confirmation": ""}),
    );
    assert!(is_error(&response), "{response}");
}

#[test]
fn the_right_token_still_confirms() {
    let host = host();
    let permissions = Permissions::read_write().confirm(["danger"], "s3cret");
    let mut server = McpServer::new(host).with_permissions(permissions);
    assert!(is_error(&call(
        &mut server,
        1,
        "run_command",
        json!({"id": "danger", "confirmation": "s3cre"})
    )));
    assert!(!is_error(&call(
        &mut server,
        2,
        "run_command",
        json!({"id": "danger", "confirmation": "s3cret"})
    )));
}

#[test]
fn the_audit_log_keeps_short_details() {
    let host = Host::new(HostSpec::default()).expect("host");
    let mut server = McpServer::new(host);
    call(&mut server, 1, "get_state", json!({}));
    assert!(
        server
            .activity()
            .iter()
            .all(|entry| entry.detail.chars().count() <= 241)
    );
}

#[test]
fn an_unknown_method_uses_the_method_not_found_code() {
    let host = Host::new(HostSpec::default()).expect("host");
    let mut server = McpServer::new(host);
    let response = server
        .handle(json!({"jsonrpc":"2.0","id":1,"method":"nope"}))
        .expect("response");
    assert_eq!(response["error"]["code"], -32601);
}

#[test]
fn undeclared_commands_are_refused_without_an_allow_list() {
    let mut server = McpServer::new(host()).with_permissions(Permissions::read_write());
    assert!(is_error(&call(
        &mut server,
        1,
        "run_command",
        json!({"id": "invented"})
    )));
    assert!(!is_error(&call(&mut server, 2, "run_command", json!({"id": "danger"}))));
}

#[test]
fn reading_state_does_not_drain_events() {
    let mut server = McpServer::new(host()).with_permissions(Permissions::read_write());
    call(&mut server, 1, "run_command", json!({"id": "danger"}));
    server.host_mut().handle(egui_workbench_host::Message::Request {
        id: 0,
        request: egui_workbench_host::Request::RunCommand {
            id: "danger".to_owned(),
        },
    });
    call(&mut server, 2, "get_state", json!({}));
    assert_eq!(server.host_mut().poll_events().len(), 1);
}
