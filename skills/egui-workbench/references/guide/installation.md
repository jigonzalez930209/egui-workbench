# Installation

> Add egui-workbench to a new or existing Rust application from crates.io or git, choose features, and keep one copy of egui. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/installation.html>

## Requirements

- Rust **1.95 or newer** (edition 2024). This repository pins `1.97.1` in
  `rust-toolchain.toml`; use that toolchain when you also run the crate's tests.
- Linux needs the usual egui/winit development packages (`libxkbcommon`, `libwayland`,
  `libx11`, `libxi`, `libxcursor`, `libxrandr`, `libgl1-mesa`, and Vulkan drivers for the
  test harness).
- A GPU or software renderer supported by `eframe`'s Glow backend.

## Add the dependency

`Cargo.toml`:

```toml
[dependencies]
egui-workbench = "1.0"
```

## Python

```bash
pip install egui-workbench
```

The wheel includes the native host; no Rust toolchain is needed. See the
[Python guide](../guide/python.md) for widgets, events, threads, asyncio, and data.

## Theme Studio and global

The theme editor, VS Code import/export, Fluent localization, font loading, and the
accessibility report are described in the
[Theme Studio and global guide](../guide/theme-and-global.md).

## Extensions

```toml
[dependencies]
egui-workbench-extensions = "1.0"
```

The sandboxed WebAssembly runtime, manifest, capabilities, manager view, and examples are
in the [Extensions guide](../guide/extensions.md).

## Agents

```bash
cargo install egui-workbench-agent
```

The MCP server, permission model, automation driver, and DevTools are in the
[Agents and DevTools guide](../guide/agents.md).

## Web and remote

```bash
npm install egui-workbench
```

The npm package is the WebAssembly host, the `createWorkbench` ESM wrapper, and the
TypeScript types generated from the schema. The remote server and C ABI are in the
[Web and remote guide](../guide/web.md).

For the declarative host (spec, patches, protocol, sidecar), add the separate crate:

`Cargo.toml`:

```toml
[dependencies]
egui-workbench-host = "1.0"
```

Or follow the repository's `main` branch:

`Cargo.toml`:

```toml
[dependencies]
egui-workbench = { git = "https://github.com/jigonzalez930209/egui-workbench" }
```

### Features

| Feature | Default | What it embeds |
|---|---|---|
| `lucide-icons` | yes | 2,410 Lucide SVGs (ISC), about 1 MB; every icon name is available to buttons, menus, tabs, and tree rows. |
| `material-icons` | yes | The VS Code Material Icon Theme (MIT), about 5 MB; resolves file and folder names to full-color icons. |
| `plot` | yes | `Plot`, its chart families, legend animations, and SVG/PNG export. |
| `plot3d` | yes | `Plot3D` and `Series3D`. Implies `plot`. |
| `viewer` | yes | The file viewer, `ViewerRegistry`, JSON/YAML/TOML trees, Markdown, and PNG/JPEG. |
| `csv` | yes | The CSV/TSV data connector. |
| `workbook` | yes | The spreadsheet connector (`calamine`; implies `viewer`). |
| `ring` | yes | The in-memory ring-buffer connector. |
| `channel` | yes | The channel/socket/serial snapshot connector. |
| `arrow` | no | Arrow IPC connector. |
| `parquet` | no | Parquet connector (implies `arrow`). |
| `sqlite` | no | SQLite connector (`rusqlite`). |
| `gpu` | no | GPU line/scatter data layer through `egui_wgpu`. Implies `plot`. |
| `serde` | yes | Stable event names and `Serialize`/`Deserialize` for component state and events. |
| `svg` | yes | The `resvg`/`usvg` rasterizer. Icon packs and `plot` enable it; a minimal build with neither pays for no rasterizer. |

Disable Material when the application never shows files, or build a smaller binary with
only the pieces it uses:

`Cargo.toml`:

```toml
[dependencies]
egui-workbench = { version = "1", default-features = false, features = ["lucide-icons", "serde"] }
```

## One copy of egui

The crate builds on `egui` and `eframe` **0.37** from crates.io, with no patches or forks.
It re-exports both, so the simplest setup depends on nothing else:

```rust
use egui_workbench::{eframe, egui};
```

If your application also lists `egui` or `eframe` directly, keep them on the same 0.37
minor version; Cargo then resolves a single copy. Two different minors compile, but the
types do not match (`egui::Ui` from one is not the other's).

## Recommended lints

The library forbids `unsafe` and denies `missing_docs`, `unwrap_used`, `expect_used`,
`panic`, `unimplemented`, `todo`, and `unreachable`. The generated application copies
the same policy:

`Cargo.toml`:

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "deny"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
unimplemented = "deny"
todo = "deny"
unreachable = "deny"
```

## Start from the template

The fastest install is the generator, which already contains the dependency, the
lints, and a compiling application.

With [`cargo-generate`](https://cargo-generate.github.io/cargo-generate/) installed,
no checkout is needed:

```bash
cargo install cargo-generate
cargo generate --git https://github.com/jigonzalez930209/egui-workbench \
  --subfolder templates/app --name my-app
# prompts for the title; leave the workbench path empty to use crates.io
cd my-app
cargo run
```

From a checkout of this repository, `cargo xtask new` does the same without the extra
tool and points the dependency at the checkout:

```bash
git clone https://github.com/jigonzalez930209/egui-workbench
cd egui-workbench
cargo xtask new ../my-app --title "My App"
cd ../my-app
cargo run
```

Continue with [Quick start](../guide/quick-start.md). To wire the library manually, continue
with [Build your first app](../guide/first-app.md).
