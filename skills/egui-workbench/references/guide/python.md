# Python

> Install egui-workbench from PyPI, build a native window from a Python script, and feed NumPy, pandas, Polars, and Arrow data into grids and plots. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/python.html>

`pip install egui-workbench` gives a Python script the same native window, widgets, data
grid, and plots as a Rust application, with no Rust toolchain. The wheel is built with
PyO3 and maturin, uses the CPython abi3 ABI, and supports CPython 3.9 and newer.

```bash
pip install egui-workbench

# Optional data sources
pip install numpy pandas pyarrow polars
```

## First window

```python
from egui_workbench import Workbench, Command, Label, Button

app = Workbench(title="My App", theme="github-dark")
app.add(Label("Hello from Python"), Button("greet", "Greet"))
app.menu("File", [Command("save", "Save", shortcut="Ctrl+S")])

@app.on("greet")
def greet(ctx):
    ctx.toast.success("Hello", "The window is native.")

@app.on("save")
def save(ctx):
    ctx.toast.success("Saved")

app.run()
```

`Workbench` builds the same versioned `HostSpec` the Rust host uses, then opens an
eframe window. `app.spec()` returns the dictionary and `app.save("app.json")` writes it
for the sidecar (`egui-workbench --spec app.json`).

## Widgets

Widgets are dataclasses, so editors and type checkers see every field:

```python
from egui_workbench import (
    Button, Card, Checkbox, Column, DataGrid, Label, Plot, Row, Select, Separator,
    Series, Slider, Spacer, Switch, Tabs, TextField, Tree, TreeNode,
)

app.add(
    Card("Filters", [
        Row([
            TextField("query", "Search", hint="name or id"),
            Select("kind", "Kind", ["all", "event", "metric"]),
            Switch("live", "Live", on=True),
        ]),
        Slider("threshold", "Threshold", min=0, max=100, value=42, step=1),
        Checkbox("exact", "Exact match"),
    ]),
    Separator(),
    Tabs("tabs", ["Table", "Plot"], active=0, children=[
        [DataGrid("grid", "trace", height=320)],
        [Plot("plot", "Signal", [Series("x", dataset="trace", x="t", y="x")], height=320)],
    ]),
)
```

| Widget | Purpose |
|---|---|
| `Label`, `Separator`, `Spacer` | Text and spacing |
| `Button` | Emits the command with its `id` |
| `TextField`, `Checkbox`, `Switch`, `Select`, `Slider` | Inputs; changes emit `value_changed` |
| `Row`, `Column`, `Card`, `Tabs` | Layout |
| `Tree`, `TreeNode` | Explorer-style navigation |
| `DataGrid` | Typed grid over a registered dataset |
| `Plot`, `Series` | Line or scatter plot over inline points or a dataset |
| `Custom` | A native Rust widget registered by an extension |

## Events and state

```python
@app.on("save")
def save(ctx):
    ctx.set("/widgets/0/text", "Saved")
    ctx.append("/widgets", {"kind": "label", "text": "done"})

@app.on("value:threshold")
def threshold(ctx):
    ctx.toast.info(f"threshold = {ctx.value}")

@app.on("tree:explorer")
def selected(ctx):
    ctx.title(f"Selected {ctx.key}")
```

`@app.on(key)` accepts a command id (`save`), an event name (`value_changed`,
`tab_changed`, `tree_selected`, `custom`), or a specific key (`value:threshold`,
`tree:explorer`). Handlers run on the UI thread. `ctx` exposes:

- `ctx.set`, `ctx.replace`, `ctx.remove`, `ctx.append`, `ctx.batch` — queue JSON-pointer
  patches for the next frame.
- `ctx.toast.info/success/warning/error/loading(title, description=None)`.
- `ctx.title(text)`, `ctx.quit()`, `ctx.wake()`, `ctx.command(id)`.
- `ctx.event`, `ctx.kind`, `ctx.key`, `ctx.value`.

`app.emit("save")` dispatches a command from Python, which is useful in tests and in
code that reacts to a file change.

## Threads and `app.post`

A long task must not block the UI thread:

```python
import threading

def export():
    report = build_report()          # slow, no UI access
    app.post(lambda: app.toast.success("Exported", report.name))

threading.Thread(target=export).start()
```

`app.post(callback)` is safe from any thread, wakes the window, and runs `callback` on
the UI thread. Without `post`, a worker should only touch plain Python state.

## asyncio

Async handlers are awaited on an asyncio loop started with the window:

```python
import asyncio

@app.on("refresh")
async def refresh(ctx):
    rows = await fetch_rows()
    ctx.set("/widgets/0/text", f"{len(rows)} rows")
    ctx.toast.success("Refreshed")
```

`await app.run_async()` runs the native window from an asyncio program or a notebook.
The window still owns a thread; async handlers keep their loop while it is open.

## Data

`app.dataset(name, data)` accepts:

- NumPy arrays and mappings of arrays. Numeric dtypes go through a dtype-aware byte
  path, so no per-element Python objects are created; strings and objects become text
  columns.
- pandas DataFrames. Numeric columns go through NumPy; pandas 3's
  `__arrow_c_stream__` is used when PyArrow is installed.
- Polars DataFrames and Series, and PyArrow tables, record batches, and arrays through
  the Arrow C data interface (`__arrow_c_stream__` / `__arrow_c_array__`).
- Mappings of column names to lists, and lists of row dictionaries.

```python
import numpy as np
import pandas as pd
import pyarrow as pa

app.dataset("numpy", {"t": np.arange(1_000_000.0), "x": np.random.randn(1_000_000)})
app.dataset("pandas", pd.DataFrame({"t": [0.0, 1.0], "x": [1.0, 2.0]}))
app.dataset("arrow", pa.table({"t": [0.0, 1.0], "x": [1.0, 2.0]}))

app.add(DataGrid("grid", "arrow"), Plot("plot", series=[Series("x", dataset="arrow", x="t", y="x")]))
```

The grid and the plot read the same `Dataset`; no per-cell Python objects cross the FFI.
Arrow tables import their buffers through the C data interface, and the dataset keeps the
typed columns the grid and plot read.

## Testing without a window

`app.step(frames=1)` renders headless and dispatches the events that a window would
produce. It needs no display server, so it runs in CI:

```python
app = Workbench(title="Test")
app.add(Label("before"), Button("go", "Go"))

@app.on("go")
def go(ctx):
    ctx.set("/widgets/0/text", "after")

app.step(1)
app.emit("go")
app.step(1)
assert app.current_spec()["widgets"][0]["text"] == "after"
```

The `egui-workbench-host` conformance fixture is language neutral; the Python test suite
applies `conformance/basic.json` through `_native.NativeWorkbench` and checks the same
tree as the Rust test.

## Schema and types

`app.schema()` returns the JSON Schema generated from the Rust `HostSpec` types. The
package ships `py.typed` and `_native.pyi`, so mypy and Pyright see the native methods;
the widget classes carry inline types.

## Sidecar

The same spec can run without Python:

```bash
egui-workbench --spec app.json            # native window, hot reload
egui-workbench --spec app.json --stdio    # framed protocol on stdin/stdout
egui-workbench --schema                   # HostSpec JSON Schema
```

Driving a Python application from another language is the 0.11 host protocol: send
`apply_patch`, `get_spec`, `run_command`, and `set_data` requests over length-prefixed
frames. The 0.13 release adds the browser and C clients.
