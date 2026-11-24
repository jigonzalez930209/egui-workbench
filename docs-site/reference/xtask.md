---
title: xtask
description: Scaffold an application, run the full check, refresh snapshots, and verify the template.
---

# xtask

`xtask` is the repository's task runner. It is a workspace member, so `cargo xtask` works
from the repository root.

```bash
cargo xtask --help
```

## new

```bash
cargo xtask new ../my-app
cargo xtask new ../my-app --name my-app --title "My App"
```

Copies `templates/app` to the destination, replaces:

| Placeholder | Value |
|---|---|
| `{{project-name}}` | Crate name (`--name` or derived from the directory) |
| `{{app_title}}` | Window title (`--title` or derived from the name) |
| `{{workbench_path}}` | Canonical path to this crate |

The same template works with `cargo generate --git … --subfolder templates/app`; the
cargo-generate manifest asks for the title and an optional checkout path (empty means
`egui-workbench = "0.3"` from crates.io).

It also copies `LICENSE-APACHE` and `LICENSE-MIT`. The destination must not exist. The
generated app is a standalone workspace with a path dependency on this crate and the lints.

## check

```bash
cargo xtask check
```

Runs, in order:

1. `cargo fmt --all --check`
2. `cargo xtask style-lint`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings`
5. `cargo test --workspace`

This is the gate for a contribution. Fix the first failure before looking at the next.

## style-lint

```bash
cargo xtask style-lint
```

Scans `src/` for literal radii and font sizes, so a custom `WorkbenchStyle` reaches every
pixel. It reports `path:line` for a non-zero number literal as:

- the argument of `CornerRadius::same`, `.corner_radius`, `FontId::proportional`,
  `FontId::monospace`, or `FontId::new`;
- the radius (second argument) of `.rect`, `.rect_filled`, or `.rect_stroke`, also when
  the call spans several lines.

`src/foundation/style.rs` (which owns the values) and the plot's export scene (sizes come
from `PlotConfig`) are exempt. A line that really needs a fixed value ends with
`// style: fixed`. Replace a violation with the matching helper: `style.control()`,
`style.inner()`, `style.container()`, `style.font(13.0)`, and so on (see
[Style and density](/theming/style-and-density#radius-helpers)).

## snapshots

```bash
cargo xtask snapshots
```

Runs `UPDATE_SNAPSHOTS=1 cargo test --test snapshots`. It refreshes the workbench frame,
settings, palette, and context-menu references. Component and plot snapshots have their
own test binaries:

```bash
UPDATE_SNAPSHOTS=1 cargo test --test snapshots_components
UPDATE_SNAPSHOTS=1 cargo test --test snapshots_plot
```

Look at every changed PNG. A snapshot-only commit should not include logic changes.

## template-check

```bash
cargo xtask template-check
```

Removes and regenerates `target/template-check`, then runs `fmt --check`,
`clippy --all-targets -- -D warnings`, and `test` in the generated app. This catches
placeholder substitutions or template API drift before release.

## fmt

```bash
cargo xtask fmt
```

Runs `cargo fmt --all`. The repository uses `max_width = 120`.

## CI

The GitHub workflow runs:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
xvfb-run -a cargo test --workspace
cargo xtask template-check
```

on Ubuntu with the X11/Wayland/GL/Vulkan development packages and Rust 1.97.1.

## Docs

| What | Command |
|---|---|
| Rust API docs | `cargo doc --workspace --no-deps --open` |
| Docs with warnings denied | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` |
| VitePress dev server | `cd docs-site && npm run dev` |
| VitePress production build | `cd docs-site && npm run build` |
| Compile-check doc examples | `cd docs-site && npm run check-examples` |

The `check` task builds rustdoc; it does not build the VitePress site. Run the site build
separately when you change `docs-site/`.
