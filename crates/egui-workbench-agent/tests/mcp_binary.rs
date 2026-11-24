//! The MCP binary on its stdio transport.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

#[test]
fn the_binary_answers_initialize_and_tools_list() {
    let spec = std::env::temp_dir().join("egui-workbench-mcp-binary.json");
    std::fs::write(
        &spec,
        r#"{"version":1,"widgets":[{"kind":"label","text":"mcp","size":13.0,"tone":"default","strong":false}]}"#,
    )
    .expect("spec");
    let mut child = Command::new(env!("CARGO_BIN_EXE_egui-workbench-mcp"))
        .args(["--spec", spec.to_str().expect("path"), "--read-only"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn");
    let mut stdin = child.stdin.take().expect("stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));
    let mut request = |value: Value| {
        writeln!(stdin, "{value}").expect("write");
        stdin.flush().expect("flush");
        let mut line = String::new();
        stdout.read_line(&mut line).expect("read");
        serde_json::from_str::<Value>(&line).expect("json")
    };
    let initialized = request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
    assert_eq!(initialized["result"]["serverInfo"]["version"], "1.0.0");
    let tools = request(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    assert!(
        tools["result"]["tools"]
            .as_array()
            .expect("tools")
            .iter()
            .any(|tool| tool["name"] == "get_accessibility_tree")
    );
    drop(stdin);
    let _ = child.wait();
    let _ = std::fs::remove_file(spec);
}
