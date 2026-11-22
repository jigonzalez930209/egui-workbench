# Roadmap

Where egui-workbench is going, in order. Each phase has an exit criterion; dates follow
the work, not the other way around. Discussion and proposals: GitHub issues.

## Now: 0.1.x, the Rust crate

The first release on crates.io, then hardening from real use.

- Publish 0.1.0 (tag-driven: the `Release` workflow publishes with the
  `CARGO_REGISTRY_TOKEN` secret and creates the GitHub release).
- CI on Linux, **macOS, and Windows**: the window chrome (undecorated window, resize
  edges, window controls) and IME text input are the platform-sensitive parts.
- Manual pass of IME composition (accents, CJK) and HiDPI on each platform.
- A `cargo generate` entry for the starter template, so a new app does not need a
  checkout of this repository (`cargo xtask new` stays for contributors).
- Fix what early users report; patch releases only.

**Exit:** two external applications run on all three desktop platforms without local
patches.

## 0.2: a host-ready Rust API

Make the shell drivable from data before anything crosses a language boundary.

- One serializable event model: commands, widget changes, tab and tree events, plot
  interactions, dialog results, all as `serde` values with stable names.
- Serializable state for every built-in widget (values, selection, open dialogs), so a
  host can set it without closures.
- `Command` metadata (id, label, shortcut, enabled) shared by menus, the palette, and the
  keymap.
- Review the public API for names and defaults that would be awkward in Python or
  TypeScript; break them here, while it is cheap.

**Exit:** the starter application can be rebuilt without writing a single closure-based
`ui` function for its chrome.

## Phase A: `egui-workbench-host`, the declarative core

A new crate in this repository. An application is described as data; Rust renders it
with egui and reports events. Everything after this phase is a thin binding.

- **Spec:** window, theme and style, menus and commands, explorer tree, tabs, panels,
  status bar, and a widget tree for content (layout containers, the component set,
  plots, toasts, dialogs). Versioned, with a JSON Schema generated from the Rust types.
- **Runtime:** `Host::new(spec)`, `host.apply(patch)` for state changes,
  `host.poll_events()`, and `host.run()` that owns the event loop (macOS requires the
  main thread).
- **Patches, not re-renders:** hosts send diffs (set a value, append plot points,
  replace a subtree), so a language binding does not rebuild the UI every frame.
- **Plots without copies:** large series arrive as contiguous `f64` buffers.
- **Escape hatch:** a `custom` node that a Rust extension crate can register, for
  products that need one native widget.
- Tests: every spec example renders to a snapshot, like the docs scenes today.

**Exit:** the docs scenes can be written as specs, and the snapshots match the Rust
versions pixel for pixel.

## Phase B: Python (`pip install egui-workbench`)

- PyO3 + maturin; `abi3` wheels for CPython ≥ 3.9 on Linux (manylinux, x86_64 and
  aarch64), macOS (universal2), and Windows; published from CI with PyPI Trusted
  Publishing.
- A Pythonic layer over the spec: decorators for commands, dataclass-like widgets,
  `app.run()`; type stubs (`.pyi`) generated from the schema.
- Event handlers run on the main thread with the GIL; long work goes to threads and
  reports back through `app.post(...)`, which is safe from any thread.
- NumPy arrays into plots through the buffer protocol (no copies).
- Docs: a Python section in the guide with the same live demos.

```python
from egui_workbench import Workbench, Command

app = Workbench(title="My App", theme="github-dark")
app.menu("File", [Command("save", "Save", shortcut="Ctrl+S")])

@app.on("save")
def save(ctx):
    ctx.toast.success("Saved")

app.run()
```

**Exit:** the starter application rewritten in Python, installed from PyPI with no Rust
toolchain.

## Phase C: JavaScript and the web (`npm i egui-workbench`)

- The host compiled to WebAssembly (`wasm-bindgen`), mounted on a `<canvas>`; the docs'
  live demos already prove the rendering.
- TypeScript types generated from the schema; an ESM package for bundlers.
- Optional later: a native Node/Deno addon (napi-rs) for desktop apps from JavaScript.

**Exit:** a TypeScript app runs the same spec in the browser as the Python app on the
desktop.

## Phase D: any language

- **Sidecar mode:** a small `egui-workbench` binary that reads the spec and patches as
  JSON over stdin and writes events to stdout. Any language that can spawn a process can
  drive a native desktop window.
- **C ABI:** a `cdylib` with a generated header, for languages with an FFI (C, C++, Go,
  Zig, C#).

**Exit:** reference clients in two languages beyond Python and JavaScript.

## Across all phases

- One source of truth: the Rust types generate the JSON Schema, which generates the
  Python stubs and TypeScript types.
- Protocol versioning: the spec carries a version; hosts reject what they do not know.
- Accessibility parity: every node keeps its AccessKit role and name, whatever language
  built it.
- Size budgets: icon packs stay optional features in every package.

## Open questions

- How much of the component layer to expose in the spec first (the full set or the
  shell plus a core of inputs and plots).
- Whether the Python package should also offer an immediate-mode API for scripting (in
  the style of Dear PyGui) alongside the declarative one.
- Hot reload of specs during development.
