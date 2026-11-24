# Go reference client

Start the sidecar on stdio and apply the shared conformance fixture:

```bash
cargo build -p egui-workbench-host
go run ./clients/go
```

`EGUI_WORKBENCH_SIDECAR` overrides the binary path. The client uses only the Go standard
library: four-byte length prefix, JSON messages, and the 0.11 request envelope.
