# egui-workbench (Python)

Native, IDE-grade workbench windows from a Python script, with no Rust toolchain.

```bash
pip install egui-workbench
```

```python
from egui_workbench import Workbench, Command, Label, Button

app = Workbench(title="My App", theme="github-dark")
app.add(Label("Hello from Python"), Button("greet", "Greet"))
app.menu("File", [Command("save", "Save", shortcut="Ctrl+S")])

@app.on("greet")
def greet(ctx):
    ctx.toast.success("Hello")

@app.on("save")
def save(ctx):
    ctx.toast.success("Saved")

app.run()
```

- `Workbench` builds the versioned host spec and opens the native window through PyO3.
- Widgets are dataclasses: `Label`, `Button`, `TextField`, `Checkbox`, `Switch`,
  `Select`, `Slider`, `Row`, `Column`, `Card`, `Tabs`, `Tree`, `DataGrid`, `Plot`, and
  `Custom`.
- `@app.on("command")` handlers run on the UI thread. `ctx.set`, `ctx.append`,
  `ctx.toast`, and `ctx.quit` are safe to call there.
- `app.post(callback)` runs work on the UI thread from any thread; `async` handlers run
  on an asyncio loop while the window is open; `await app.run_async()` is available in
  notebooks.
- `app.dataset(name, data)` accepts NumPy, pandas, Polars, PyArrow, mappings, and rows.
- `app.step(frames=1)` renders headless for tests and conformance.
- `app.schema()` is the JSON Schema generated from the Rust host types.

The wheel is built with maturin and abi3 for CPython 3.9+.
