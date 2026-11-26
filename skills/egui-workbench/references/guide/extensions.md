# Extensions

> Install sandboxed WebAssembly extensions, grant capabilities, contribute commands and panels, and hot reload during development. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/extensions.html>

An extension is a directory with a versioned manifest and a WebAssembly module. The host
runs it in a sandbox with no WASI and no ambient filesystem or network; every host import
is checked against the capabilities the user granted.

## Install and enable

```rust
use egui_workbench_extensions::ExtensionManager;

let mut manager = ExtensionManager::new("extensions")?;
manager.install("extensions/hello")?;
manager.grant("com.example.hello", "commands")?;
manager.grant("com.example.hello", "panels")?;
let info = manager.enable("com.example.hello")?;
assert_eq!(info.commands[0].id, "hello.run");
manager.invoke("hello.run")?;
```

The runtime is enabled by the default `wasm` feature. An application that never loads an
extension can build with `default-features = false` and pay for none of it.

The manager view is a component in the same crate:

```rust
use egui_workbench_extensions::{
    ExtensionManagerAction, ExtensionManagerState, show_extension_manager,
};

let mut state = ExtensionManagerState::default();
if let Some(action) = show_extension_manager(ui, &mut state, &manager.list()) {
    match action {
        ExtensionManagerAction::Enable(id) => { manager.enable(&id)?; }
        ExtensionManagerAction::Disable(id) => manager.disable(&id)?,
        ExtensionManagerAction::Reload(id) => { manager.reload(&id)?; }
        ExtensionManagerAction::Grant { id, capability } => manager.grant(&id, &capability)?,
        _ => {}
    }
}
```

It lists installed extensions, their version, enabled state, requested/granted
capabilities, and the commands, panels, and viewers they contributed.

## Manifest

```json
{
  "id": "com.example.hello",
  "version": "0.1.0",
  "name": "Hello",
  "api": { "min": "0.15", "max": "0.16" },
  "entry": "hello.wasm",
  "checksum": "sha256:…",
  "signature": "optional",
  "capabilities": ["commands", "panels", "viewers", "filesystem:read:notes"]
}
```

`api` is the host range the extension supports; the host rejects a range that does not
include `HOST_API_VERSION` (`0.15`). `entry` cannot escape the extension directory.
`checksum` is verified on install and reload, against the same bytes that are loaded.
`id` is a plain reverse-DNS name: ASCII letters, digits, `.`, `-`, `_`, not starting with
a dot. Both `api` bounds must be `major.minor`; an unparsable bound rejects the manifest.
`filesystem:read:` needs a non-empty prefix.

`signature` is checked only when the application configures a verifier:

```rust
let manager = ExtensionManager::new(dir)?
    .with_signature_verifier(|module_bytes, manifest| my_policy.verify(module_bytes, manifest));
```

Without a verifier, a manifest that carries `signature` is refused with
`ExtensionError::Signature`. The crate ships no signature algorithm.

### Passing the command id

When the host runs a command it copies the id into the module's memory. Export
`alloc(len: i32) -> i32` and the host writes into the buffer you return:

```rust
static mut INBOX: [u8; 256] = [0; 256];

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: i32) -> i32 {
    if len < 0 || len as usize > 256 { return -1; }
    core::ptr::addr_of_mut!(INBOX) as i32
}
```

A module without `alloc` still works, but the host writes the id at offset 1024, which
can overwrite the module's data.

### Limits

Each call gets a fuel budget, and memory is capped at 64 MiB with one instance, one
memory, and four tables. `host_read_file` reads regular files only and returns an error
when the file exceeds the buffer. An extension keeps at most 1,024 contributions and
1,024 log lines between drains.

Capabilities:

| Capability | Grants |
|---|---|
| `commands` | Register commands |
| `panels` | Register panels |
| `viewers` | Register file viewers |
| `themes`, `keybindings`, `settings`, `chart-layers` | Declared contribution classes |
| `data` | Read dataset names and row counts (not rows) |
| `selection` | Read/write the selection bus |
| `filesystem:read:<prefix>` | Read paths under `<prefix>` only |

`enable` requires every requested capability to be granted. Granting a capability after
enable rebuilds the runtime. The manager records every denial and every log line.

## Host API

The 0.15 ABI is core WebAssembly v1 with a small, versioned import set. It deliberately
avoids WASI.

Imports from module `workbench`:

```text
host_register(kind: i32, json_ptr: i32, json_len: i32) -> i32
host_log(ptr: i32, len: i32)
host_read_file(path_ptr: i32, path_len: i32, out_ptr: i32, out_cap: i32) -> i32
host_dataset_rows(name_ptr: i32, name_len: i32) -> i32
host_selection(ptr: i32, len: i32)
```

`kind` is `0` command, `1` panel, `2` viewer. The JSON payloads are
`{"id","title"}` and `{"id","extensions"}`. Exports are `memory`, `init() -> i32`, and
`on_command(ptr, len) -> i32`; `init` returns `0` on success.

`host_read_file` returns the byte count, or a negative code when the capability is
missing, the path is outside the prefix, the file is missing, or the output buffer is too
small. `host_dataset_rows` returns the row count or a negative code. No import returns
dataset rows.

The component model and WIT are the next ABI revision; 0.15 keeps a core-module ABI so a
C or Rust guest can be built with the toolchain a developer already has. The manifest
and capability model are already versioned for that transition.

## Examples

Two extensions ship in the repository:

```bash
# C, no WASI, compiled with clang + rust-lld
bash crates/egui-workbench-extensions/examples/hello-c/build.sh

# Rust no_std, compiled to wasm32-unknown-unknown
bash crates/egui-workbench-extensions/examples/hello-rust/build.sh
```

The C extension registers a command, a panel, and a viewer; the Rust extension registers a command and a panel through the same imports. Tests install,
grant, enable, invoke, and reload the compiled modules; the C extension also proves the
sandbox rejects an ungranted capability.

## Hot reload

`manager.reload(id)` re-reads the module and rebuilds the instance while keeping the id
and grants. Use it after `build.sh`; an extension that fails to compile or instantiate
leaves the previous runtime untouched only if the caller chooses to ignore the error.
The manager view exposes the same action as **Reload**.
