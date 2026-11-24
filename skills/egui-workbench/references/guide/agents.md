# Agents and DevTools

> Drive a workbench from an MCP client, approve proposed changes, replay flows headlessly, and inspect the running UI. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/agents.html>

0.14 closes the loop between an agent that writes an application and an agent that uses
it. The MCP server exposes the same commands, spec, and accessible names a person uses;
the permission model is read-only until the embedding application opts in; DevTools and
the automation driver make the result inspectable and testable.

## MCP server

```bash
cargo run -p egui-workbench-agent --bin egui-workbench-mcp -- \
  --spec app.json --read-only
```

The server speaks MCP over stdio: newline-delimited JSON-RPC 2.0, `initialize`,
`tools/list`, and `tools/call`. It binds no socket.

```json
{
  "mcpServers": {
    "workbench": {
      "command": "egui-workbench-mcp",
      "args": ["--spec", "app.json", "--read-only"]
    }
  }
}
```

| Tool | Permission | Result |
|---|---|---|
| `get_spec` | read | Current host spec |
| `get_state` | read | Spec, pending events, revision |
| `list_commands` | read | Commands with ids, shortcuts, enabled state |
| `get_accessibility_tree` | read | Roles, accessible names, widget ids |
| `list_activity` | read | Every allowed and denied tool call |
| `get_screenshot` | read | PNG data URL when the application installs a screenshot provider |
| `apply_patch` | write | JSON-pointer patch |
| `run_command` | write + allow-list | Command id and resulting events |

Read-only is the default. `--read-write` enables mutations, `--allow save` limits the
commands, and `--confirm token` requires `arguments.confirmation == "token"` for commands
whose category is `destructive`.

```rust
use egui_workbench_agent::{McpServer, Permissions};

let server = McpServer::new(host)
    .with_permissions(
        Permissions::read_write()
            .allow_commands(["save", "increment"])
            .confirm(["delete"], "one-time-code"),
    )
    .on_command(|host, id| {
        // The embedding application decides what a command means.
        let _ = (host, id);
    });
```

`get_screenshot` is optional because the host does not carry a rasterizer: install a
provider with `McpServer::with_screenshot(|host| render_png(host))`. Without one, the tool
reports that no provider is configured.

The permission model is enforced before the host sees a request, and every decision is
appended to `server.activity()`. See the [threat model](../reference/agent-threat-model.md)
for the assumptions and the limits.

## Accessibility tree

`accessibility_tree(&spec)` turns the widget tree into stable roles and names. An agent
uses it to find a control without guessing CSS-like paths:

```json
[
  { "role": "text_field", "name": "Name", "id": "name", "children": [] },
  { "role": "button", "name": "Save", "id": "save", "children": [{ "role": "state", "name": "enabled", "id": null, "children": [] }] }
]
```

`find_node(&tree, "Save", Some("button"))` returns the node; `Automation::target` turns it
into the command id for a replay.

## Automation

```rust
use egui_workbench_agent::{Automation, AutomationStep};
use egui_workbench_host::Patch;

let flow = Automation {
    steps: vec![
        AutomationStep::Command { id: "save".to_owned() },
        AutomationStep::Patch {
            patch: Patch::Set {
                path: "/widgets/0/text".to_owned(),
                value: serde_json::json!("replayed"),
            },
        },
    ],
};
let events = flow.replay(&mut host)?;
assert!(events.iter().any(|event| format!("{event:?}").contains("save")));
```

`replay` runs headless against a `Host`, so it works in CI without a display. Flows are
serializable (`serde`); record them from the journal and check the spec or a screenshot
after each step.

## Agent panel

`components::show_agent_panel` is a chat view with streaming messages, Markdown headings,
bullets, fenced code, tool-call cards, and approve/reject buttons. It returns an
`AgentAction`; the application owns the model and the transport.

```rust
use egui_workbench::components::{
    AgentAction, AgentPanelState, ChatMessage, ToolCall, show_agent_panel,
};

let mut panel = AgentPanelState::new();
panel.push(ChatMessage::user("Rename the label"));
panel.push(ChatMessage::assistant("## Plan\n- patch the label", true));
panel.propose(ToolCall::new("apply_patch", "Rename the first label").diff("+Title\n-Old"));
if let Some(action) = show_agent_panel(ui, &mut panel) {
    match action {
        AgentAction::Submit(text) => send_to_model(text),
        AgentAction::Approve(index) => run_tool(index),
        AgentAction::Reject(index) => reject_tool(index),
        _ => {}
    }
}
```

## DevTools

F12 now has six tabs:

| Tab | What it shows |
|---|---|
| Journal | The command timeline from 0.5 |
| State | Any serialized application state, filterable |
| Inspector | Widget tree with ids and accessible names; select a widget for its JSON |
| Theme | Live color tokens with WCAG AA checks; edits call `apply_tokens` |
| Performance | Frame-time sparkline and the last frame's repaint causes |
| Accessibility | Missing labels, empty labels, untitled plots and grids |

```rust
use egui_workbench::app::{DevToolsState, show_devtools};

let mut devtools = DevToolsState::default();
show_devtools(ctx, &mut devtools, &journal, &snapshot, Default::default());
```

`DevToolsState` also stores the live `Theme` and `PerformanceState`; `open_inspector()`,
`open_state()`, and `open_journal()` are available for commands and tests.
