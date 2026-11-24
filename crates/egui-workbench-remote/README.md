# egui-workbench-remote

Run a `Host` on a server and render it in a browser. The browser connects over
WebSocket and exchanges the 0.11 protocol messages in binary frames. The server binds to
`127.0.0.1` by default; authentication and TLS are the embedding application's job.

```rust
use egui_workbench_host::Host;
use egui_workbench_remote::RemoteServer;

let host = Host::load("app.json")?;
RemoteServer::new(host, "127.0.0.1:31337")?
    .on_command(|host, id| {
        if id == "increment" {
            let _ = host.apply(&egui_workbench_host::Patch::Set {
                path: "/widgets/0/text".to_owned(),
                value: serde_json::json!("clicked"),
            });
        }
    })
    .serve()?;
```

The npm package's `connectRemote` is the reference browser client.
