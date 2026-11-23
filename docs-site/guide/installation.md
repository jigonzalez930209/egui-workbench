---
title: Installation
description: Add egui-workbench to a new or existing Rust application from crates.io or git, choose features, and keep one copy of egui.
---

# Installation

## Requirements

- Rust **1.95 or newer** (edition 2024). This repository pins `1.97.1` in
  `rust-toolchain.toml`; use that toolchain when you also run the crate's tests.
- Linux needs the usual egui/winit development packages (`libxkbcommon`, `libwayland`,
  `libx11`, `libxi`, `libxcursor`, `libxrandr`, `libgl1-mesa`, and Vulkan drivers for the
  test harness).
- A GPU or software renderer supported by `eframe`'s Glow backend.

## Add the dependency

```toml [Cargo.toml]
[dependencies]
egui-workbench = "0.3"
```

Or follow the repository's `main` branch:

```toml [Cargo.toml]
[dependencies]
egui-workbench = { git = "https://github.com/jigonzalez930209/egui-workbench" }
```

### Features

| Feature | Default | What it embeds |
|---|---|---|
| `lucide-icons` | yes | 2,410 Lucide SVGs (ISC), about 1 MB; every icon name is available to buttons, menus, tabs, and tree rows. |
| `material-icons` | yes | The VS Code Material Icon Theme (MIT), about 5 MB; resolves file and folder names to full-color icons. |

Disable Material when the application never shows files:

```toml [Cargo.toml]
[dependencies]
egui-workbench = { version = "0.2", default-features = false, features = ["lucide-icons"] }
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

```toml [Cargo.toml]
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
lints, and a compiling application:

```bash
git clone https://github.com/jigonzalez930209/egui-workbench
cd egui-workbench
cargo xtask new ../my-app --title "My App"
cd ../my-app
cargo run
```

Continue with [Quick start](/guide/quick-start). To wire the library manually, continue
with [Build your first app](/guide/first-app).
