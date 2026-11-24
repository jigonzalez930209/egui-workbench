# egui-workbench C ABI

`libegui_workbench_c` exposes the host to C, C++, Go (cgo), Zig, C#, and any language
with a C FFI. `include/egui_workbench.h` is the hand-written header.

```c
#include "egui_workbench.h"

WbHost *host = wb_host_new(spec_json);
if (!host) { fprintf(stderr, "%s\n", wb_last_error()); return 1; }
if (wb_host_apply_patch(host, patch_json) != 0) { /* wb_last_error() */ }
char *spec = wb_host_spec(host);
/* ... */
wb_string_free(spec);
wb_host_free(host);
```

Build with `cargo build -p egui-workbench-c --release`; link `-legui_workbench_c`. The
host window runs on the calling thread in `wb_host_run`; use `wb_host_frame` for headless
tests and `wb_host_poll_events` for the JSON event array.
