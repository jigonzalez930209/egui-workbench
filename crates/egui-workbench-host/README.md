# egui-workbench-host

A declarative host for egui-workbench. An application is a versioned JSON `HostSpec`:
window, theme, commands, menus, explorer, editor groups, panels, status bar, and a widget
tree. The host renders it with the egui-workbench components, applies JSON-pointer
patches, and emits events.

```rust
use egui_workbench_host::{Host, HostSpec};

let mut host = Host::new(HostSpec::default())?;
host.apply(&egui_workbench_host::Patch::Set {
    path: "/widgets/0/text".to_owned(),
    value: serde_json::json!("Hello"),
})?;
host.run()?;
```

- `HostSpec::schema()` generates the JSON Schema.
- `Host::apply`, `Host::poll_events`, and `Host::run`.
- `protocol` has the versioned messages and a four-byte length-prefixed frame for
  stdio, sockets, and WebSocket binary messages.
- The `egui-workbench` binary runs a spec, speaks the protocol with `--stdio`, prints the
  schema with `--schema`, and hot-reloads the spec file.
- `custom::CustomWidget` is the native Rust escape hatch.
- `conformance/basic.json` is the language-neutral fixture used by the host tests.

The crate is published separately from `egui-workbench`; the host depends on the root
crate by version.
