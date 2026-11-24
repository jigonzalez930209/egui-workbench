---
title: Migration from 0.3
description: What changed between the 0.3 shell and 1.0, and how to move an application without surprises.
---

# Migration from 0.3 to 1.0

0.3 was a presentation shell. 1.0 is the same shell plus commands, state, layout, data,
hosts, Python, WebAssembly, agents, and extensions. The presentation API stayed the
source of truth; this page collects the moves.

## Crate and features

- `egui-workbench` keeps its name. The host is a separate crate, `egui-workbench-host`;
  Python, remote, agent, and extension crates are separate packages on the same tag.
- Features are additive and default to the desktop set. A minimal build:

```toml
egui-workbench = { version = "1", default-features = false, features = ["lucide-icons", "serde"] }
```

- Themes, icons, plots, 3D, viewer, connectors, GPU, and extensions are features. The
  `material-icons` pack is opt-in.

## State and commands

- 0.4 added `CommandRegistry`, `When`, rebinding, and the palette feed. If 0.3 dispatched
  a hand-written enum, register the same ids and route the palette through the registry.
- 0.5 added the undo tree, journal, macros, `Workspace`, stash, hot exit, and recovery.
  Use `CommandJournal` as the source for automation replay.
- 0.6 added the `Layout` tree and `LayoutAction`; named layouts persist through
  `Workspace`.

## Content

- 0.7 added typed `Dataset`/`Column`, paged data sources, connectors, the query language,
  `DataGrid`, and `ViewerRegistry`.
- 0.8 added `LinkHub` and linked views; 0.9 added the plot engine, GPU renderer, and
  streaming series; 0.10 added the analysis charts, tools, colormaps, annotations, math
  text, and vector export.
- 0.11 moved the spec, patches, protocol, and sidecar into `egui-workbench-host`. Native
  custom widgets remain the escape hatch.

## New clients

- 0.12: `pip install egui-workbench`; `Workbench`, dataclass widgets, events, asyncio,
  NumPy/pandas/Polars/PyArrow data.
- 0.13: npm/wasm host, WebSocket remote server, C ABI, Go and C conformance clients.
- 0.14: MCP server with permissions/activity log, accessibility tree, automation replay,
  agent panel, and the six-tab DevTools.
- 0.15: sandboxed WebAssembly extensions with manifests, capabilities, manager, hot
  reload, and C/Rust examples.
- 0.16: Theme Studio (VS Code import/export, high contrast), Fluent-shaped localization,
  fonts, and the accessibility report.

## Breaking changes to expect

- 0.x minor releases could break APIs; 0.17 is the freeze. The deprecated
  `Theme::radius()` was removed in 0.17; use `style::get(ctx).control()` for radii.
- `HostSpec`/protocol versions reject newer versions with an error; keep the spec version
  and the host on the same minor.
- Python wheels are abi3 for CPython 3.9+; the npm package is ESM.
- The extension ABI is versioned separately (`HOST_API_VERSION`, currently `0.15`); a
  manifest names its range.

## Checklist

1. Move any file loading out of components; components return events.
2. Register commands and keybindings; delete application-specific enums from the shell.
3. Replace literal colors and radii with theme/style tokens.
4. Add `ThemeState`, `Documents`, `Workspace`, and the journal if the application has
   undo, sessions, or layout persistence.
5. If the app needs other languages, use the host/Python/wasm path rather than binding
   Rust UI directly.
