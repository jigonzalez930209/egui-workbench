# Web and remote

> Run the same host spec in a browser with WebAssembly, render a server over WebSocket, and drive it from C, Go, and other languages. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/web.html>

The 0.13 release makes the spec portable: a browser renders it with WebAssembly, a
server can own the application and stream the same 0.11 messages over WebSocket, and a C
ABI exposes the host to the languages that cannot use Rust or Python directly.

## npm package

```bash
npm install egui-workbench
```

```ts
import { createWorkbench } from "egui-workbench";
import type { HostSpec } from "egui-workbench/schema";

const spec: HostSpec = {
  version: 1,
  window: { title: "Browser", width: 900, height: 620 },
  widgets: [
    { kind: "label", text: "Hello from WebAssembly", size: 16, tone: "default", strong: true },
    { kind: "button", id: "go", label: "Go", variant: "primary", enabled: true },
  ],
};

const canvas = document.querySelector("canvas")!;
const app = await createWorkbench({ spec, canvas });

for (const event of app.pollEvents()) {
  if (event.event === "command") app.applyPatch({ op: "set", path: "/widgets/0/text", value: "Clicked" });
}
```

`createWorkbench` loads the wasm module, mounts eframe on the canvas, and returns the
handle: `applyPatch`, `setSpec`, `spec`, `pollEvents`, `setTheme`, `schema`, and
`destroy`. The package is ESM, includes the generated wasm and its `.d.ts`, and its
TypeScript declarations for `HostSpec`, `WidgetSpec`, and every component are generated
from the host JSON Schema.

### Build the wasm package

```bash
cargo install wasm-pack
cd crates/egui-workbench-wasm
npm run build          # wasm-pack + schema types + package cleanup
npm pack --dry-run
```

The wasm host has a headless browser test (`wasm-pack test --headless --chrome`) and a
full canvas test that loads the package in Chrome, renders the spec, and checks a patch.

## Remote UI

Run the host on a server and let the browser render it. The browser is the view; commands
and patches travel over WebSocket as the versioned length-prefixed messages.

```rust
use egui_workbench_host::{Host, Patch};
use egui_workbench_remote::RemoteServer;
use serde_json::json;

let host = Host::load("app.json")?;
RemoteServer::new(host, "127.0.0.1:31337")?
    .on_command(|host, id| {
        if id == "increment" {
            let count = host.json()["widgets"][0]["text"].as_str().unwrap_or("0");
            let next = count.parse::<u64>().unwrap_or(0) + 1;
            let _ = host.apply(&Patch::Set {
                path: "/widgets/0/text".to_owned(),
                value: json!(next.to_string()),
            });
        }
    })
    .serve()?;
```

```ts
import { connectRemote } from "egui-workbench";

const remote = await connectRemote({
  url: "ws://127.0.0.1:31337",
  canvas: document.querySelector("canvas")!,
  spec,                          // first paint while the socket connects
  onEvent: (event) => console.log(event),
});

remote.applyPatch({ op: "set", path: "/widgets/0/text", value: "clicked" });
remote.sendCommand("increment");
```

The server sends `Hello`, the spec, and `Event` messages; the browser sends `Request`
values (`get_spec`, `apply_patch`, `run_command`, `set_data`, `reload`). `RemoteServer`
binds `127.0.0.1` by default. Put authentication and TLS in front of it before exposing
it; the host does not invent either.

The binary `egui-workbench-remote --spec app.json --addr 127.0.0.1:31337` serves a spec
with no command handlers. Use the library when commands must change state.

## C ABI

`libegui_workbench_c` is a `cdylib` with `include/egui_workbench.h`:

```c
#include "egui_workbench.h"

WbHost *host = wb_host_new(spec_json);
if (!host) { fprintf(stderr, "%s\n", wb_last_error()); return 1; }
wb_host_apply_patch(host, "{\"op\":\"set\",\"path\":\"/widgets/0/text\",\"value\":\"hi\"}");
char *spec = wb_host_spec(host);
char *events = wb_host_frame(host);
wb_string_free(spec);
wb_string_free(events);
wb_host_free(host);
```

| Function | Purpose |
|---|---|
| `wb_host_new`, `wb_host_free` | Create and destroy a host |
| `wb_host_apply_patch` | JSON-pointer patch |
| `wb_host_spec` | Current spec |
| `wb_host_poll_events`, `wb_host_frame` | Events and one headless frame |
| `wb_host_run` | Native window on the calling thread |
| `wb_host_schema`, `wb_host_version` | Schema and ABI version |
| `wb_last_error`, `wb_string_free` | Error text and string ownership |

Each function is safe to call from one thread at a time; wrap the host in your language's
mutex for multi-threaded use. The header is plain C and works from C++, Go (cgo), Zig,
and C# P/Invoke.

## Reference clients

The conformance fixture is `crates/egui-workbench-host/conformance/basic.json`. Three
independent clients apply it:

- Rust: `crates/egui-workbench-host/tests/host.rs`
- Python: `crates/egui-workbench-python/tests/test_workbench.py`
- Go: `clients/go`, which spawns the sidecar on stdio and checks the patched spec.

The C client in `crates/egui-workbench-c/tests/c_abi.c` exercises the C ABI with the same
fixture values. Build and run it with:

```bash
bash crates/egui-workbench-c/check.sh   # c abi ok
bash clients/go/check.sh                # go client ok
```

The browser package is a fourth path: `crates/egui-workbench-wasm/scripts/remote-browser-test.sh`
starts the server, loads the package in headless Chrome, applies a remote patch, and checks
the rendered spec (`remote browser ok`).

## Protocol

Messages are JSON values with a four-byte big-endian length prefix, exactly as in the
0.11 host:

```text
{"type":"hello","version":1,"capabilities":["patches","data","schema"]}
{"type":"request","id":1,"request":{"op":"apply_patch","patch":{"op":"set","path":"/widgets/0/text","value":"hi"}}}
{"type":"response","id":1,"response":{"result":"spec","spec":{...}}}
{"type":"event","event":{"event":"command","id":"increment"}}
```

Inside WebSocket, each binary frame carries one such message. A browser client that
encodes the same four bytes can talk to the sidecar, the remote server, or any future
transport.
