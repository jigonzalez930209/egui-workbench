//! The MCP stdio server and its permission model.

use std::collections::{BTreeMap, HashSet};
use std::io::{self, BufRead, Write};
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use egui_workbench_host::{Host, HostEvent, Message, Patch, Request, Response};

use crate::access::{AccessNode, accessibility_tree, to_json};
use crate::automation::Automation;
pub use crate::permissions::Permissions;

fn queue_command(host: &mut Host, id: &str) {
    let _ = host.handle(Message::Request {
        id: 0,
        request: Request::RunCommand { id: id.to_owned() },
    });
}

/// The MCP protocol version this server implements.
pub const MCP_VERSION: &str = "2024-11-05";

/// One audited tool decision.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Activity {
    /// Monotonic sequence.
    pub sequence: u64,
    /// Tool name.
    pub tool: String,
    /// Whether it ran.
    pub allowed: bool,
    /// Denial reason or a short description.
    pub detail: String,
}

/// Why a tool call failed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToolError {
    /// The tool does not exist.
    UnknownTool(String),
    /// The session is read-only.
    ReadOnly,
    /// The command is not on the allow-list.
    CommandDenied(String),
    /// The command needs a confirmation value.
    ConfirmationRequired(String),
    /// The arguments are malformed.
    Arguments(String),
    /// The host rejected the operation.
    Host(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTool(tool) => write!(formatter, "unknown tool `{tool}`"),
            Self::ReadOnly => formatter.write_str("the session is read-only"),
            Self::CommandDenied(command) => write!(formatter, "command `{command}` is not allowed"),
            Self::ConfirmationRequired(command) => write!(formatter, "command `{command}` needs confirmation"),
            Self::Arguments(error) => formatter.write_str(error),
            Self::Host(error) => formatter.write_str(error),
        }
    }
}

type CommandHandler = Box<dyn FnMut(&mut Host, &str) + Send>;

/// Render the current host to bytes (usually PNG). The host does not own a rasterizer;
/// the embedding application supplies one when it wants `get_screenshot`.
pub type ScreenshotProvider = Arc<dyn Fn(&Host) -> Result<Vec<u8>, String> + Send + Sync>;

/// A host plus the agent policy.
pub struct McpServer {
    host: Host,
    permissions: Permissions,
    activity: Vec<Activity>,
    sequence: u64,
    command_handler: Option<CommandHandler>,
    screenshot: Option<ScreenshotProvider>,
}

impl McpServer {
    /// A read-only server.
    pub fn new(host: Host) -> Self {
        Self {
            host,
            permissions: Permissions::default(),
            activity: Vec::new(),
            sequence: 0,
            command_handler: None,
            screenshot: None,
        }
    }

    /// Replace the permissions.
    pub fn with_permissions(mut self, permissions: Permissions) -> Self {
        self.permissions = permissions;
        self
    }

    /// Install the command handler used by `run_command`.
    pub fn on_command(mut self, handler: impl FnMut(&mut Host, &str) + Send + 'static) -> Self {
        self.command_handler = Some(Box::new(handler));
        self
    }

    /// Add a screenshot provider for `get_screenshot`.
    pub fn with_screenshot(
        mut self,
        provider: impl Fn(&Host) -> Result<Vec<u8>, String> + Send + Sync + 'static,
    ) -> Self {
        self.screenshot = Some(Arc::new(provider));
        self
    }

    /// The current host.
    pub fn host(&self) -> &Host {
        &self.host
    }

    /// The current host, mutably.
    pub fn host_mut(&mut self) -> &mut Host {
        &mut self.host
    }

    /// The audit log.
    pub fn activity(&self) -> &[Activity] {
        &self.activity
    }

    /// Handle one JSON-RPC request. Notifications return `None`.
    pub fn handle(&mut self, request: Value) -> Option<Value> {
        let id = request.get("id").cloned();
        let method = request.get("method").and_then(Value::as_str).unwrap_or_default();
        id.as_ref()?;
        let id = id.unwrap_or(Value::Null);
        let result = match method {
            "initialize" => Ok::<_, (i32, String)>(json!({
                "protocolVersion": MCP_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "egui-workbench", "version": env!("CARGO_PKG_VERSION") }
            })),
            "tools/list" => Ok(json!({ "tools": tools() })),
            "tools/call" => self
                .call_tool(request.get("params").cloned().unwrap_or(Value::Null))
                .map_err(|error| (-32602, error)),
            other => Err((-32601, format!("unknown method `{other}`"))),
        };
        Some(match result {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, error)) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": code, "message": error }
            }),
        })
    }

    /// Run the newline-delimited JSON-RPC stdio transport.
    ///
    /// # Errors
    /// When stdin or stdout fails.
    pub fn serve_stdio(mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let Ok(request) = serde_json::from_str::<Value>(&line) else {
                writeln!(
                    stdout,
                    "{}",
                    json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"parse error"}})
                )?;
                stdout.flush()?;
                continue;
            };
            if let Some(response) = self.handle(request) {
                writeln!(stdout, "{response}")?;
                stdout.flush()?;
            }
        }
        Ok(())
    }

    fn call_tool(&mut self, params: Value) -> Result<Value, String> {
        let name = params.get("name").and_then(Value::as_str).ok_or("missing tool name")?;
        let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
        let result = self.tool(name, &arguments);
        let (allowed, detail) = match &result {
            Ok(description) => (true, description.clone()),
            Err(error) => (false, error.to_string()),
        };
        self.audit(name, allowed, detail);
        match result {
            Ok(text) => Ok(json!({
                "content": [{ "type": "text", "text": text }],
                "isError": false
            })),
            Err(error) => Ok(json!({
                "content": [{ "type": "text", "text": error.to_string() }],
                "isError": true
            })),
        }
    }

    fn tool(&mut self, name: &str, arguments: &Value) -> Result<String, ToolError> {
        match name {
            "get_spec" => Ok(serde_json::to_string(self.host.json()).unwrap_or_default()),
            "get_state" => Ok(serde_json::to_string(&json!({
                "spec": self.host.json(),
                "events": self.host.peek_events(),
                "revision": self.host.revision()
            }))
            .unwrap_or_default()),
            "list_commands" => {
                let commands = self
                    .host
                    .spec()
                    .commands
                    .iter()
                    .map(|command| {
                        json!({
                            "id": command.id,
                            "title": command.title,
                            "shortcut": command.shortcut,
                            "enabled": command.enabled
                        })
                    })
                    .collect::<Vec<_>>();
                Ok(serde_json::to_string(&commands).unwrap_or_default())
            }
            "get_accessibility_tree" => Ok(to_json(&accessibility_tree(self.host.spec())).to_string()),
            "list_activity" => Ok(serde_json::to_string(&self.activity).unwrap_or_default()),
            "get_screenshot" => {
                let Some(provider) = &self.screenshot else {
                    return Err(ToolError::Host("no screenshot provider is configured".to_owned()));
                };
                let bytes = provider(&self.host).map_err(ToolError::Host)?;
                Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
            }
            "apply_patch" => {
                self.require_write()?;
                let patch: Patch = serde_json::from_value(arguments.get("patch").cloned().unwrap_or(Value::Null))
                    .map_err(|error| ToolError::Arguments(error.to_string()))?;
                self.host
                    .apply(&patch)
                    .map_err(|error| ToolError::Host(error.to_string()))?;
                Ok("patch applied".to_owned())
            }
            "run_command" => {
                self.require_write()?;
                let id = arguments
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| ToolError::Arguments("missing command id".to_owned()))?;
                self.require_command(id)?;
                if self.permissions.sensitive_commands.contains(id) {
                    let provided = arguments.get("confirmation").and_then(Value::as_str);
                    // An empty token would be matched by an empty `confirmation`.
                    match (
                        provided,
                        self.permissions
                            .confirmation_token
                            .as_deref()
                            .filter(|token| !token.is_empty()),
                    ) {
                        (Some(provided), Some(expected)) if constant_time_eq(provided, expected) => {}
                        _ => return Err(ToolError::ConfirmationRequired(id.to_owned())),
                    }
                }
                queue_command(&mut self.host, id);
                if let Some(handler) = &mut self.command_handler {
                    handler(&mut self.host, id);
                }
                let _ = self.host.apply_actions();
                let events = self.host.poll_events();
                Ok(serde_json::to_string(&json!({
                    "events": events,
                    "spec": self.host.json()
                }))
                .unwrap_or_default())
            }
            "get_automation" => Ok(serde_json::to_string(&Automation::new()).unwrap_or_default()),
            other => Err(ToolError::UnknownTool(other.to_owned())),
        }
    }

    /// Command ids the spec declares: commands, menu rows, and button widgets.
    fn declared_commands(&self) -> HashSet<String> {
        let spec = self.host.spec();
        let mut ids: HashSet<String> = spec.commands.iter().map(|command| command.id.clone()).collect();
        ids.extend(
            spec.menus
                .iter()
                .flat_map(|menu| menu.items.iter().filter_map(|item| item.command.clone())),
        );
        let mut nodes = vec![accessibility_tree(spec)];
        while let Some(level) = nodes.pop() {
            for node in level {
                if node.role == "button"
                    && let Some(id) = node.id
                {
                    ids.insert(id);
                }
                nodes.push(node.children);
            }
        }
        ids
    }

    fn require_write(&self) -> Result<(), ToolError> {
        if self.permissions.read_only {
            Err(ToolError::ReadOnly)
        } else {
            Ok(())
        }
    }

    fn require_command(&self, id: &str) -> Result<(), ToolError> {
        if let Some(allowed) = &self.permissions.allowed_commands {
            if !allowed.contains(id) {
                return Err(ToolError::CommandDenied(id.to_owned()));
            }
            return Ok(());
        }
        // With no allow-list, only commands the spec declares may run; an agent must not
        // invent ids for the embedding application's handler.
        if self.declared_commands().contains(id) {
            Ok(())
        } else {
            Err(ToolError::CommandDenied(id.to_owned()))
        }
    }

    fn audit(&mut self, tool: &str, allowed: bool, mut detail: String) {
        if detail.len() > MAX_AUDIT_DETAIL {
            let mut end = MAX_AUDIT_DETAIL;
            while !detail.is_char_boundary(end) {
                end -= 1;
            }
            detail.truncate(end);
            detail.push('…');
        }
        if self.activity.len() >= MAX_AUDIT_ENTRIES {
            self.activity.remove(0);
        }
        self.sequence = self.sequence.wrapping_add(1);
        self.activity.push(Activity {
            sequence: self.sequence,
            tool: tool.to_owned(),
            allowed,
            detail,
        });
    }
}

/// Compare secrets without an early exit on the first differing byte.
fn constant_time_eq(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    let mut difference = left.len() ^ right.len();
    for index in 0..left.len().max(right.len()) {
        difference |= usize::from(left.get(index).copied().unwrap_or(0) ^ right.get(index).copied().unwrap_or(0));
    }
    difference == 0
}

/// Longest audit detail kept per call. A successful call's detail used to be its whole
/// result, which stored every screenshot and spec dump in the log.
const MAX_AUDIT_DETAIL: usize = 240;

/// Most audit entries kept in memory.
const MAX_AUDIT_ENTRIES: usize = 10_000;

/// The tool list, including JSON Schemas for the arguments.
pub fn tools() -> Vec<Value> {
    vec![
        tool(
            "get_spec",
            "Read the current host spec.",
            json!({"type":"object","properties":{}}),
        ),
        tool(
            "get_state",
            "Read the spec, pending events, and revision.",
            json!({"type":"object","properties":{}}),
        ),
        tool(
            "list_commands",
            "List the commands declared by the spec.",
            json!({"type":"object","properties":{}}),
        ),
        tool(
            "get_accessibility_tree",
            "Read roles and accessible names for every widget.",
            json!({"type":"object","properties":{}}),
        ),
        tool(
            "list_activity",
            "Read the permission audit log.",
            json!({"type":"object","properties":{}}),
        ),
        tool(
            "get_screenshot",
            "Render the host to a PNG data URL. The embedding application supplies the renderer.",
            json!({"type":"object","properties":{}}),
        ),
        tool(
            "apply_patch",
            "Apply a JSON-pointer patch. Requires write permission.",
            json!({"type":"object","required":["patch"],"properties":{"patch":{"type":"object"}}}),
        ),
        tool(
            "run_command",
            "Run a command id. Requires write permission and the command allow-list.",
            json!({"type":"object","required":["id"],"properties":{"id":{"type":"string"},"confirmation":{"type":"string"}}}),
        ),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({ "name": name, "description": description, "inputSchema": input_schema })
}

/// A JSON map of accessible names to ids, for quick lookups.
pub fn names(nodes: &[AccessNode]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for node in nodes {
        if let Some(id) = &node.id {
            map.insert(node.name.clone(), id.clone());
        }
        map.extend(names(&node.children));
    }
    map
}

/// Re-exported for callers that build a response by hand.
pub fn response_spec(host: &Host) -> Response {
    Response::Spec {
        spec: host.json().clone(),
    }
}

/// Re-exported event helper.
pub fn event_count(events: &[HostEvent]) -> usize {
    events.len()
}
