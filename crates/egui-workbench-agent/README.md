# egui-workbench-agent

The agent surface of egui-workbench:

- **MCP server** over stdio: `initialize`, `tools/list`, and `tools/call` with tools for
  the spec, state, commands, accessibility tree, patches, and activity.
- **Permissions**: read-only by default; opt in to writes, allow-list commands, and
  require a confirmation token for sensitive commands. Every decision is logged.
- **Accessibility tree** generated from the host spec (roles and accessible names).
- **Automation**: record commands and patches and replay them headlessly against a host.

Run the server for a spec:

```bash
cargo run -p egui-workbench-agent --bin egui-workbench-mcp -- --spec app.json --read-only
```

The server speaks newline-delimited JSON-RPC 2.0, the MCP stdio transport. It binds
nothing; local stdio is the only transport.

The permission model is deliberately separate from the host: an MCP client cannot apply
a patch unless the embedding application enables writes, and destructive commands can
require a confirmation value that the client must supply in the tool arguments.
