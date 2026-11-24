# Host

> Run an application described as a versioned spec, apply patches, and speak the host protocol from any language. Live page: <https://jigonzalez930209.github.io/egui-workbench/app/host.html>

`egui-workbench-host` runs an application described as data. A `HostSpec` carries the
window, theme, commands, menus, explorer, editor groups, panels, status bar, and widget
tree. The host renders it with the component layer, accepts patches, and emits events.

`Cargo.toml`:

```toml
[dependencies]
egui-workbench-host = "0.11"
```

## Spec

```rust
use egui_workbench_host::{Host, HostSpec, Patch};
use serde_json::json;

let spec = HostSpec::from_json(r#"{
  "version": 1,
  "window": { "title": "My App", "width": 900, "height": 640 },
  "widgets": [
    { "kind": "text_field", "id": "name", "label": "Name", "value": "", "hint": "Type…" },
    { "kind": "button", "id": "greet", "label": "Greet", "variant": "primary", "enabled": true }
  ]
}"#)?;
let mut host = Host::new(spec)?;
host.apply(&Patch::Set {
    path: "/widgets/0/value".to_owned(),
    value: json!("Ada"),
})?;
host.run()?;
```

`HostSpec::schema()` returns the JSON Schema generated from the Rust types; the sidecar
prints it with `--schema`.

## Runtime

| Method | Purpose |
|---|---|
| `Host::new(spec)` | Validate and prepare a spec |
| `Host::load(path)` | Read a spec file and remember it for hot reload |
| `Host::apply(patch)` | Apply a JSON-pointer patch and validate the result |
| `Host::poll_events()` | Drain `Command`, `ValueChanged`, `TabChanged`, `TreeSelected`, `Custom` |
| `Host::set_dataset(name, dataset)` | Register a typed dataset for tables and plots |
| `Host::register_custom(kind, widget)` | Register a native Rust widget |
| `Host::reload_if_changed()` | Hot reload the source file |
| `Host::run()` | Own the native event loop |

Specs are re-rendered from the current tree; patches change values, append children, and
replace subtrees without rebuilding a host-side UI description every frame.

## Protocol

The protocol is versioned JSON with a four-byte big-endian frame length:

```text
{"type":"hello","version":1,"capabilities":["patches","data"]}
{"type":"request","id":7,"request":{"op":"get_spec"}}
{"type":"response","id":7,"response":{"result":"spec","spec":{ ... }}}
{"type":"event","event":{"event":"command","id":"file.save"}}
```

`Message::encode`/`decode`, `write_frame`, and `read_frame` work on any `Read`/`Write`;
stdio, sockets, and WebSocket binary messages carry the same frames. A newer protocol
version is rejected.

## Sidecar

```bash
# Open a spec (hot reload while it runs)
egui-workbench --spec app.json

# Headless protocol on stdin/stdout
egui-workbench --spec app.json --stdio

# Print the schema
egui-workbench --schema
```

Any language that can spawn a process and write a length-prefixed JSON frame can drive a
native window. `Host::handle` answers `ApplyPatch`, `GetSpec`, `RunCommand`, `SetData`,
and `Reload`.

## Custom widgets

`custom::CustomWidget` is the native escape hatch: a Rust extension registers a kind and
receives `(ui, props)`, returning `HostEvent`s. The spec uses
`{"kind":"custom","widget":"my.kind","props":{...}}`.

## Conformance

`conformance/basic.json` is a language-neutral fixture: spec + patches + expected tree.
The Rust host applies it in `tests/host.rs`; the default spec renders through a headless
wgpu harness in `tests/render.rs`. A binding should produce the same event sequence and
the same schema-valid tree.
