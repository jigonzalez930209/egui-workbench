# egui-workbench-extensions

Sandboxed WebAssembly extensions. A directory contains `extension.json` and the module:

```json
{
  "id": "com.example.hello",
  "version": "0.1.0",
  "name": "Hello",
  "api": ">=0.15,<0.16",
  "entry": "hello.wasm",
  "checksum": "sha256:...",
  "capabilities": ["commands", "panels", "viewers", "filesystem:read:notes"]
}
```

The host API is versioned (`HOST_API_VERSION`). Extensions import `workbench` functions
for logging, command/panel/viewer registration, dataset metadata, and a granted
filesystem prefix; they export `memory`, `init`, and `on_command`. The runtime is a
sandbox: no WASI, no filesystem, and no network unless a capability grants a path.

```rust
use egui_workbench_extensions::ExtensionManager;

let mut manager = ExtensionManager::new("extensions")?;
manager.install("examples/hello")?;
manager.grant("com.example.hello", "commands")?;
manager.enable("com.example.hello")?;
for info in manager.list() {
    println!("{} {} {}", info.id, info.version, info.enabled);
}
```

`ExtensionManager::reload` re-reads a module after it changes, which is the hot-reload
loop for extension development.
