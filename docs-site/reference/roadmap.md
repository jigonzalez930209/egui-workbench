---
title: Roadmap
description: From 0.3 to 1.0.0 in fourteen releases - commands and state, a docking workspace, one data model, a GPU plot engine, a declarative host with Python and web bindings, agents, and extensions.
---

# Roadmap

The repository's [`ROADMAP.md`](https://github.com/jigonzalez930209/egui-workbench/blob/main/ROADMAP.md)
is the source of truth; this page mirrors it.

Where egui-workbench is going, from 0.3 to 1.0.0. Discussion and proposals: GitHub issues.

## How to read this document

- **Order follows dependencies, not dates.** Each release builds on the ones before it;
  dates follow the work. Sizes are rough: **S** is days, **M** one or two weeks, **L** a
  few weeks, **XL** more than that.
- **One headline per release.** Every 0.x release has a single sentence a user can
  repeat, a scope, and an exit criterion that a test, a benchmark, or a running example
  can prove. A release ships only when its exit criterion holds.
- **Breaking changes are front-loaded.** While the crate is 0.x a minor release may break
  the API, and each changelog entry says how to migrate. The big API review is 0.4; after
  it, breaks stay small and are announced one release ahead with `#[deprecated]`.
- **Every release also ships** the items in [Definition of done](#definition-of-done-for-every-release).
- There are more 0.x releases than a typical crate has on purpose: each one is small
  enough to finish, test, and document properly before the next starts.

## Where we are: 0.3.1

egui-workbench is a desktop shell plus content components for egui 0.37: nine themes, an
undecorated window with title bar and menus, tabs, explorer tree, dialogs, command
palette, toasts, a shadcn-style component layer, 2,410 Lucide icons, the Material file
icons, `Plot` (many chart families, legend animations, SVG/PNG export), `Plot3D`, a
read-only `FileViewer`, the optional runtime (`ThemeState`, `Documents`, `Keymap`,
`CloseGuard`, `show_workbench`), a starter template, an agent skill, and a documentation
site whose demos run the real library in WebAssembly. About 230 tests and pixel
snapshots protect it.

What stands between this and a 1.0 that people can build products on, and the release
that closes each gap:

| Gap found in the code | Closed by |
|---|---|
| No component state or event is serializable (only the Material icon index uses `serde`) | 0.4, 0.5 |
| `Keymap` is a bare shortcut list: no command metadata, rebinding, contexts, or one registry shared by menus and palette | 0.4 |
| Two overlapping APIs (`widgets` functions and `components` builders), a crate root with dozens of re-exports, no `#[non_exhaustive]` | 0.4, 0.17 |
| No feature flags for plots, 3D, or the viewer; `calamine` and `resvg` are always built | 0.4 |
| No undo, no command journal, no session restore or crash recovery | 0.5 |
| `show_workbench` is a fixed five-region layout; `Documents<T>` is one flat list; no docking or detached windows | 0.6 |
| The viewer is read-only, stores every cell as a `String`, and cannot sort, filter, search, or select | 0.7 |
| Plots and tables share no selection; there are no subplots or linked axes | 0.8 |
| Plots paint on the CPU from `&[[f64; 2]]` pairs; `Plot3D` depth-sorts with the painter | 0.9, 0.10 |
| Few chart families beyond the basics (no histogram, box, contour, polar), no analysis tools, no math text, no PDF | 0.10 |
| No way to drive the shell from data, or from another language | 0.11 to 0.13 |
| CI runs on Linux only, although the window chrome and IME are platform-sensitive | 0.3.x |

## The plan at a glance

| Release | Name | Headline | Size |
|---|---|---|---|
| 0.3.x | Hardening | CI on three desktops, benchmarks, a manual IME and HiDPI pass | M |
| 0.4.0 | Commands and State | Every action is a named command, and every state is serializable | XL |
| 0.5.0 | History and Session | Undo tree, macros, and a session that survives a crash | L |
| 0.6.0 | Workspace | An IDE layout: editor groups, drag and drop, detached windows | XL |
| 0.7.0 | Data Fabric | One typed data model under tables, viewers, and plots | XL |
| 0.8.0 | Linked Views | Select in a plot and the table follows, and the other way round | L |
| 0.9.0 | Plot Engine | GPU rendering and level-of-detail for data that does not fit in memory | XL |
| 0.10.0 | Analysis and Publication | New chart families, analysis tools, math text, vector PDF | XL |
| 0.11.0 | Host | An application described as data, with a stable protocol | XL |
| 0.12.0 | Python | `pip install` a native workbench, no Rust toolchain | L |
| 0.13.0 | Web and Everywhere | npm and WebAssembly, remote UI, C ABI, more clients | L |
| 0.14.0 | Agents and DevTools | The UI is drivable by AI agents and inspectable by developers | XL |
| 0.15.0 | Extensions | Users extend an application without recompiling it | XL |
| 0.16.0 | Theme Studio and Global | VS Code themes, localization, accessibility to a measured bar | L |
| 0.17.0 | Freeze | API audit, deprecation removal, security review | M |
| 1.0.0 | Stable | A promise, not a feature | S |

How the releases depend on each other:

```text
0.3.x hardening
   │
   ▼
0.4 commands + state ──► 0.5 history + session ──► 0.6 workspace
   │                                                    │
   ├──► 0.7 data fabric ──► 0.8 linked views ──► 0.9 plot engine ──► 0.10 analysis
   │          │
   └──────────┴──► 0.11 host ──► 0.12 python
                        │
                        ├──► 0.13 web + everywhere
                        ├──► 0.14 agents + devtools   (needs 0.5 journal, 0.11 protocol)
                        └──► 0.15 extensions          (needs 0.4 registry, 0.7 viewers, 0.11 host)
                                         │
                                         ▼
                              0.16 theme studio + global ──► 0.17 freeze ──► 1.0.0
```

The data line (0.7 to 0.10) and the application line (0.4 to 0.6) are independent after
0.4, so they can be worked in either order. The host (0.11) needs both, because a spec
describes commands, layouts, and data.

---

## 0.3.x: hardening

**Headline:** the 0.3 crate is solid on every desktop before anything new is built on it.

Patch releases only; no API changes.

- **CI on Linux, macOS, and Windows.** The window chrome (undecorated window, resize
  edges, window controls) and IME text input are the platform-sensitive parts. Pixel
  snapshots stay pinned to one reference platform; the other two run the behavior tests.
- **A manual pass** of IME composition (accents, CJK) and HiDPI on each platform, written
  up in `docs/` so it can be repeated before 1.0.
- **Benchmarks as a baseline** (`criterion`): plot decimation, a large tree, opening and
  scrolling a one-million-row CSV, and palette filtering. Later releases state their
  performance goals against these numbers.
- **`cargo-semver-checks`** in CI, so every public change is visible in a review.
- **A policy for egui upgrades** (see [Decisions](#decisions-to-settle-early)).
- **`cargo generate`** for the starter template, so a new application does not need a
  checkout of this repository (`cargo xtask new` stays for contributors).
- Fix what early users report.

**Exit:** green CI on three desktop platforms, a recorded benchmark baseline, and two
external applications running on all three without local patches.

---

# Phase 1: the application core (0.4 to 0.6)

The pieces every serious application repeats, and the ones every later release depends
on: actions, state, history, and layout.

## 0.4.0: Commands and State

**Headline:** every action is a named command, and every piece of state can be saved,
sent, and restored.

**Why it matters.** Most egui applications wire each button, menu item, and shortcut by
hand, so the same action exists in three places and nothing can be remapped, listed,
recorded, or driven from outside. A registry fixes that once, and everything after this
release (macros, sessions, hosts, bindings, agents, extensions) is built on it.

**Commands**

- `CommandRegistry`: each command has a stable namespaced id (`file.save`), a title,
  category, icon, default shortcut, enabled and checked state, and an optional argument
  schema. Menus, the palette, the keymap, the title bar, and context menus are all views
  of the same registry.
- `when` contexts: a small set of context keys (`editorFocus`, `hasSelection`,
  `dialogOpen`) and a `when` expression on each command, so shortcuts and menu items
  apply only where they make sense.
- Rebindable keys: conflict detection, chord sequences (`Ctrl+K Ctrl+S`), a keybindings
  file (JSON), a keybindings editor built from the registry, and import of VS Code's
  `keybindings.json`.
- **Palette v2:** fuzzy ranking, recently used commands, prefix modes (`>` commands, `:`
  go to a line, `@` symbols), commands that ask for arguments, and results that arrive
  from a background provider without blocking the frame.

**State and events**

- Feature `serde`: one event model with stable names for what components return
  (`TabEvent`, `TreeEvent`, `PromptAction`, plot interactions, dialog results).
- Serializable state for every built-in component: `ThemeState`, `Documents`,
  `TreeState`, `PaletteState`, `PlotState`, `PlotConfig`, `Plot3DState`, and the rest, so
  a host can set them without closures.

**API review, the breaking window**

- Review names and defaults that would be awkward in Python or TypeScript and break them
  here, while it is cheap.
- `#[non_exhaustive]` on public enums and option structs, so later releases can add fields
  without breaking anyone.
- The duplicated `widgets` functions are deprecated in favor of the `components`
  builders; the crate root keeps only what applications import every day.
- Cargo features `plot`, `plot3d`, and `viewer` (plus the existing icon packs), so an
  application pays only for what it uses and the WebAssembly build shrinks.
- Decision before starting: split the repository into a workspace of crates
  (see [Decisions](#decisions-to-settle-early)).

**Not in scope:** undo, sessions, layouts, anything on disk.

**Exit:** the starter application's chrome (menus, palette, shortcuts, keymap file) is
rebuilt from one registry with no closure-based `ui` function for it; every built-in
state and event round-trips through JSON in a test; a migration guide covers every break.

## 0.5.0: History and Session

**Headline:** undo anything, replay anything, and never lose work to a crash.

**Why it matters.** Undo, journals, and recovery are what make an application feel
trustworthy, and they are tedious to retrofit. With commands from 0.4, they become one
mechanism instead of one per screen.

- **`History<S>`:** transactional undo and redo. Commands that change state describe the
  change (an inverse or a snapshot); typing and drags coalesce into one step; groups
  undo together.
- **An undo tree, not a stack.** Undoing and then editing keeps the abandoned branch. A
  History panel shows the tree and lets the user jump to any point ("time travel").
- **Command journal:** an append-only, serializable log of every command with its
  arguments and a timestamp.
- **Macros:** record a sequence from the journal, name it, and run it as a command. The
  same replay engine powers deterministic tests: a recorded session becomes a
  regression test.
- **`Workspace` persistence:** open documents, active tab, explorer expansion, panel
  sizes, theme, and plot views in one versioned file, written atomically, with
  migrations between versions.
- **Hot exit and crash recovery:** dirty buffers are stashed on a timer and on close;
  after a crash the journal and the stash restore the session, and the user is told what
  was recovered.
- **DevTools, first slice** (`F12`): a journal viewer and a state inspector that shows any
  registered state as a tree. The full DevTools arrive in 0.14.

**Not in scope:** merging concurrent edits from two users.

**Exit:** kill the process mid-edit and restart, and the same tabs, views, and unsaved
buffers return; a recorded macro replays to the same final state in a test; the undo tree
survives a restart.

## 0.6.0: Workspace

**Headline:** an IDE layout inside egui, with tabs you can drag into new groups and out
into their own windows.

**Why it matters.** `show_workbench` draws five fixed regions and `Documents<T>` is one
flat list, which is enough for a single-document tool and not for an IDE or an analysis
workbench. Docking is the feature users compare first.

- **Editor groups:** split in any direction (horizontal, vertical, grid); drag tabs
  between groups with drop-zone previews; pinned and preview tabs; reopen the last
  closed tab; an overflow menu.
- **Side bars and an activity bar:** views contributed to the left, right, and bottom
  containers; drag a view from one container to another; badges; a collapsed rail.
- **Floating and detached:** float a panel; tear a tab or a whole group off into a native
  window (egui multi-viewport) and dock it back. `CloseGuard` covers every window. In the
  browser, detached panels float inside the canvas.
- **Layouts as data:** a serializable `Layout` tree, named layouts ("Analysis", "Review"),
  switchable from the palette, saved with the workspace from 0.5, and migrated between
  versions.
- **Focus tools:** maximize a panel, Zen mode, breadcrumbs, and commands (from 0.4) to
  move between groups.
- **Keyboard and accessibility:** every drag has a keyboard equivalent; groups and panels
  have AccessKit roles.
- `show_workbench` and `WorkbenchApp` keep working as the one-group case; the new layout
  API is additive.

**Risks:** multi-viewport differs across Wayland, X11, macOS, and Windows, and this is
the first release that needs the 0.3.x platform CI to pass. See the decision on building
the dock ourselves or adopting `egui_dock`.

**Exit:** the showcase is rebuilt with three editor groups, a floating panel, and a
detached window; the layout survives a restart; the interaction tests drive every drag
with the keyboard.

---

# Phase 2: data and charts (0.7 to 0.10)

The crate's content side grows from "a viewer and some charts" into one data model that
tables and plots share.

## 0.7.0: Data Fabric

**Headline:** one typed, lazy data model under every table, viewer, and plot.

**Why it matters.** Today the table stores every cell as text and plots take borrowed
pairs, so a million rows cost far too much memory and a plot and a table cannot refer to
the same data. A shared model is what makes linking, GPU plots, Python buffers, and
agents practical.

- **`Dataset`:** columnar and typed (floats, integers, booleans, strings, date and time,
  categories), with a validity mask, units and metadata per column, derived columns, and
  zero-copy slices.
- **`DataSource`:** a lazy, paged trait (schema, row count, `read_range`, change
  notification, cancellation) with no async runtime dependency, so an application can
  back it with a file, a database, or a thread.
- **Connectors, each behind a feature:** CSV and TSV (streaming), the existing workbook
  support (`calamine`), Parquet, Arrow IPC, SQLite, an in-memory ring buffer, and an
  adapter for channels, sockets, and serial streams.
- **`DataGrid`** (extends the current table viewer): typed cells instead of `String`s;
  multi-column sort, filter, quick search, grouping and aggregates; pinned, resizable,
  reorderable, and hideable columns; row, column, and range selection; copy as TSV,
  Markdown, or JSON; optional editing with validation and undo (from 0.5); a column
  header with statistics and a sparkline.
- **A query language:** a small, safe expression language (`voltage > 0.3 && cycle in
  2..5`) with good error messages, used by the filter bar, derived columns, and (later)
  plot filters.
- **`ViewerRegistry`:** a `Viewer` trait selected by extension and content sniffing.
  Built in: text, table, JSON, YAML and TOML as a tree, images (PNG, JPEG, SVG), a hex
  view, rendered Markdown, structured logs with level filters, and highlighted code
  (feature).
- `Series` can read from a `Dataset` column; the borrowed-pairs API stays.

**Not in scope:** a SQL engine, joins across sources, writing back to databases.

**Exit:** a one-million-row, twenty-column CSV opens, scrolls, sorts, and filters inside
the frame budget, using less memory than the 0.3.x string-cell baseline; every connector
has a fixture test; the viewer registry opens each built-in format from one `show` call.

## 0.8.0: Linked Views

**Headline:** select in a plot and the table follows, and the other way round.

**Why it matters.** Brushing and linking is the feature that makes data tools feel
intelligent, and it is rare outside desktop analytics suites. With one data model (0.7)
it is a small amount of code with a large effect.

- **Selection bus:** a `LinkHub` carries a selection (rows, ranges, or a predicate) per
  dataset. Views publish and subscribe; modes are highlight, filter, and isolate.
- **Plot and grid brushing:** area-select in a plot highlights rows; selecting rows
  highlights points; hovering a row marks its point, and the reverse.
- **Plots bound to columns:** `Plot::from_dataset(&data).x("t").y("v").color_by("cycle")`,
  with size and label channels.
- **`PlotGrid`:** subplots in rows and columns, shared x, y, or both, a shared legend,
  per-subplot configuration, and small multiples faceted by a column.
- **Linked axes and cursors** across independent plots: pan and zoom together, a shared
  crosshair with a value readout, and the 3D camera can be linked too.
- **View history:** back and forward through zoom levels.

**Exit:** a demo scene with a scatter plot, a line plot, and a grid over one dataset where
a selection made in any of the three updates the other two; the interaction is covered by
snapshot and kittest tests.

## 0.9.0: Plot Engine

**Headline:** GPU-rendered plots with level-of-detail, for data that does not fit in
memory.

**Why it matters.** CPU painting with per-frame decimation is excellent up to millions of
points (0.2.1 measured 450,000 points in 13 ms) and then runs out. Instrument and
simulation data reach hundreds of millions of points.

- **GPU renderer** through `egui_wgpu` paint callbacks: instanced anti-aliased lines,
  markers, bars, and heatmap textures. The current painter stays as the fallback and the
  reference for snapshots; feature `gpu`, chosen at runtime. WebGPU in the browser, with
  WebGL2 as a fallback where it works.
- **Level of detail:** a lazily built min/max pyramid over a `DataSource`, with a tile
  cache and range queries, so a plot asks for what the screen can show. Memory-mapped,
  out-of-core files work the same way.
- **Streaming:** ring buffers, append-only series with incremental updates, a live and a
  frozen mode, rolling windows, and oscilloscope-style triggers.
- **Axes:** a real time axis (calendar ticks, time zones), categorical axes, and symmetric
  log scales.
- **Performance targets**, measured by the 0.3.x benchmarks on named reference hardware
  and enforced in CI as budgets: interactive pan and zoom over 100 million points on an
  integrated GPU, and sustained streaming of one million points per second.

**Risks:** GPU backends differ per platform and per driver; the CPU fallback and the CI
GPU matrix exist for this.

**Exit:** every existing documentation scene renders identically on both renderers within
a stated tolerance; the performance budgets are green in CI.

## 0.10.0: Analysis and Publication

**Headline:** the chart families and tools to analyze data in the application, and to
export a figure ready for a paper.

- **New families:** histogram (with density estimate), box and violin, 2D density and
  hexbin, contour and filled contour, polar, and Smith charts. Domain presets such as
  Nyquist and Bode layouts ship as recipes built from `PlotGrid`, not as core types.
- **Color:** perceptually uniform colormaps, diverging maps, color-blind-safe categorical
  palettes, colorbars, and symmetric-log color scales.
- **Annotations and regions of interest:** text, arrows, spans, and draggable
  rectangle, ellipse, and polygon regions, all serializable and included in exports.
- **Analysis tools:** peak detection, baseline correction, Savitzky-Golay smoothing,
  derivative and integral, FFT and spectrum, resampling, nonlinear least-squares fits
  with a user model and confidence bands, and a statistics panel for the selection.
- **Math text:** a LaTeX subset (sub and superscripts, Greek, fractions, units) in
  titles, axis labels, legends, and annotations, drawn identically on screen and in
  exports. This also removes the "a glyph shows as a box" FAQ entry.
- **Export:** vector PDF, better SVG, PNG at a chosen DPI, journal-size presets, copy as
  image, and optional embedded source data.
- **`Plot3D` on the GPU:** real meshes, a depth buffer, simple lighting, and point clouds
  of millions of points, instead of painter depth sorting.

**Exit:** a gallery that reproduces one figure of each family, with snapshots on both
renderers; the exported PDF opens identically in two independent viewers.

---

# Phase 3: the host and every language (0.11 to 0.14)

An application described as data, rendered by Rust, drivable from anywhere. Everything
after the host is a thin binding or a transport.

## 0.11.0: Host

**Headline:** describe an application as data, run it, and send changes as patches.

New crate in this repository: `egui-workbench-host`.

- **Spec:** window, theme and style, commands and menus, explorer tree, tabs and the
  workspace layout, panels, status bar, and a widget tree for content (layout
  containers, the component set, `DataGrid`, plots, toasts, dialogs). Versioned, with a
  JSON Schema generated from the Rust types.
- **Runtime:** `Host::new(spec)`, `host.apply(patch)`, `host.poll_events()`, and
  `host.run()`, which owns the event loop (macOS requires the main thread).
- **Patches, not re-renders:** a host sends diffs (set a value, append plot points,
  replace a subtree), so a binding does not rebuild the UI every frame.
- **Data without copies:** large series and tables arrive as contiguous buffers (`f64`
  columns, Arrow arrays) that feed a `Dataset` directly.
- **A transport-agnostic protocol** (JSON-RPC style, with a binary channel for buffers),
  versioned, rejecting what it does not know. In-process, stdio, sockets, and
  WebSocket all carry the same messages.
- **Sidecar mode:** a small `egui-workbench` binary that reads the spec and patches and
  writes events. Any language that can spawn a process can drive a native window.
- **Hot reload** of specs during development.
- **Escape hatch:** a `custom` node that a Rust extension crate registers, for products
  that need one native widget.
- **Conformance suite:** language-neutral fixtures (spec and patches in, rendered
  snapshot and events out) that every binding must pass.

**Exit:** the documentation scenes written as specs match the Rust versions pixel for
pixel; the starter application can be rebuilt as a spec.

## 0.12.0: Python

**Headline:** `pip install egui-workbench` gives a Python script a native, IDE-grade
window with no Rust toolchain.

- PyO3 and maturin; `abi3` wheels for CPython 3.9 and newer on Linux (manylinux, x86_64
  and aarch64), macOS (universal2), and Windows, published from CI with PyPI Trusted
  Publishing.
- A Pythonic layer over the spec: decorators for commands, dataclass-like widgets,
  `app.run()`, and type stubs (`.pyi`) generated from the schema.
- Event handlers run on the main thread with the GIL; long work goes to threads and
  reports back through `app.post(...)`, which is safe from any thread. `asyncio`
  integration.
- **Zero-copy data:** NumPy arrays, and pandas, Polars, and PyArrow tables through the
  Arrow C data interface, straight into a `Dataset`; plots and grids read the same
  buffers.
- A Python section in the guide with the same live demos.

```python
from egui_workbench import Workbench, Command

app = Workbench(title="My App", theme="github-dark")
app.menu("File", [Command("save", "Save", shortcut="Ctrl+S")])

@app.on("save")
def save(ctx):
    ctx.toast.success("Saved")

app.run()
```

**Exit:** the starter application rewritten in Python, installed from PyPI on a machine
with no Rust toolchain, passing the conformance suite on all three platforms.

## 0.13.0: Web and Everywhere

**Headline:** the same spec runs in a browser, over a network, and from any language.

- **npm package:** the host compiled to WebAssembly (`wasm-bindgen`), mounted on a
  `<canvas>`, with an ESM build for bundlers and TypeScript types generated from the
  schema. The documentation's live demos already prove the rendering.
- **Remote UI:** the application runs on a server and the browser renders it. Spec and
  events travel over WebSocket with the 0.11 protocol; authentication is the host
  application's responsibility, and the default binds to localhost.
- **C ABI:** a `cdylib` with a generated header for C, C++, Go, Zig, and C#.
- **Optional:** a Node and Deno addon (napi-rs) for desktop applications written in
  JavaScript.
- **Reference clients** in two languages beyond Python and JavaScript (for example Go
  and C#), written from the sidecar protocol and checked by the conformance suite.

**Exit:** a TypeScript application runs the same spec in the browser as the Python
application on the desktop; two further reference clients pass the conformance suite.

## 0.14.0: Agents and DevTools

**Headline:** AI agents can drive the UI, and developers can look inside it.

**Why it matters.** The crate already ships an agent skill so agents write applications
with it. This closes the loop: agents can also *use* those applications, through the
same commands and state a person uses.

- **MCP server** over the 0.11 protocol: lists commands (from the 0.4 registry), reads
  state snapshots, exposes the AccessKit tree and screenshots, runs commands, and
  applies patches.
- **A permission model:** read-only by default, an allow-list of commands per session,
  confirmation for destructive commands, and a visible activity log. Local transports
  (stdio, localhost) only by default.
- **Agent panel component:** a chat view with streaming Markdown, tool-call cards, and
  approval of proposed changes shown as a diff.
- **UI automation driver:** find widgets by accessible name and role, record flows from
  the journal (0.5), replay them in CI on all three platforms, and compare screenshots.
  This is the end-to-end layer above `egui_kittest`.
- **DevTools, complete** (`F12`): a widget inspector, a live editor for theme and style
  tokens, a performance overlay (frame time, repaint reasons, texture memory), an
  accessibility audit, and the journal timeline from 0.5.
- The agent skill and the plugin are updated with the agent-facing protocol.

**Exit:** a demonstration in which an agent builds and edits a dashboard using only the
MCP interface; automation tests run in CI on three platforms; a written threat model
for the agent interface.

---

# Phase 4: extensibility and polish (0.15 to 0.17)

## 0.15.0: Extensions

**Headline:** users extend an application without recompiling it, in any language that
targets WebAssembly.

- **WebAssembly plugins** (component model, WIT interfaces, `wasmtime` on desktop; the
  browser's own engine on the web) that contribute commands, views and panels, viewers,
  themes, keybindings, settings, and chart layers.
- **A manifest** with an id, version, the host API range it supports, and the
  capabilities it asks for. Plugins are sandboxed: no file system or network unless
  granted, and the user sees the grant.
- **A host API** for datasets (handles, not copies), the command registry, the
  selection bus, and the layout.
- **An extension manager** view: install, update, enable, disable, and see permissions;
  checksums for packages, signing as an option.
- **Hot reload** while developing an extension, and example extensions in Rust and in at
  least one other language.
- Native Rust extension crates (the 0.11 `custom` node) remain the way to add a
  GPU-heavy widget.

**Risks:** the `wasmtime` dependency is heavy, so it sits behind a feature and never
reaches an application that does not use it; the component model is still evolving, so
the interface is versioned from the first release.

**Exit:** an extension written in a language other than Rust adds a command, a panel,
and a viewer to the showcase, is installed from the extension manager, and cannot read
a file it was not granted.

## 0.16.0: Theme Studio and Global

**Headline:** any VS Code theme, any language, and accessibility measured instead of
promised.

- **Theme Studio:** a visual editor for theme tokens with live WCAG AA checks, import of
  VS Code color themes (and their syntax colors), export of tokens as JSON and CSS
  variables, and theme packs installable as extensions.
- **Localization:** Fluent (`.ftl`) with plurals and runtime locale switching, the
  built-in strings of every component translated (English and Spanish first, others
  from contributors), and checks for right-to-left layout.
- **Fonts:** a loading API with CJK and emoji fallbacks, and a bundled math font for
  0.10's math text.
- **Accessibility:** high-contrast themes, AccessKit on the web build, a screen-reader
  walkthrough of the showcase on all three platforms (NVDA or Narrator, VoiceOver,
  Orca), a focus-order audit, and reduced motion everywhere. The DevTools audit from 0.14
  runs in CI.

**Exit:** a published accessibility report; a VS Code theme imported with no manual
edits passes AA after the automatic adjustment; the showcase runs fully in Spanish.

## 0.17.0: Freeze

**Headline:** nothing new, so that 1.0 can be a promise.

- **Feature freeze.** Only fixes, documentation, and removal.
- **API audit:** every public item is reviewed for name, default, error type, and
  `#[non_exhaustive]`; everything deprecated since 0.4 is removed; `cargo-semver-checks`
  has no unexplained diff.
- **Documentation complete:** every public item documented, every guide page checked by
  `check-examples`, a migration guide from 0.3, and the specification (`WORKBENCH.md`)
  updated.
- **Security review:** `cargo audit` and `cargo deny` in CI, the plugin sandbox, the
  agent permission model, the remote transport, and the file parsers (fuzzing the CSV,
  workbook, and spec readers).
- **Performance budgets** of 0.7 and 0.9 are green and recorded.
- One or more release candidates (`1.0.0-rc.N`) on crates.io for external applications to
  try.

**Exit:** the 1.0 checklist below, with no open blocker.

---

## 1.0.0: stable

No new features. The release states the promise:

- **Semantic versioning** for every crate and package in the repository, with the
  [egui policy](#decisions-to-settle-early) written down.
- **A supported Rust version policy** (the current minimum is 1.95) and a stated number
  of egui releases supported.
- **The protocol, the spec schema, and the extension interface** are versioned
  separately and stay compatible within a major version.
- Entry criteria:
  - CI green on Linux, macOS, Windows, and WebAssembly, with the platform checks of 0.3.x
    repeated by hand.
  - At least two external applications in production on 1.0 release candidates.
  - Every performance budget green; every accessibility check green.
  - Wheels on PyPI, the package on npm, the crates on crates.io, all from tagged CI.
  - Documentation, live demos, the starter, and the agent skill match the release.

---

## Cross-cutting rules

These apply to every release.

- **One source of truth.** Rust types generate the JSON Schema, which generates the
  Python stubs and the TypeScript types. Nothing is written twice.
- **Protocol versioning.** The spec and the protocol carry a version; a host rejects what
  it does not know and says why.
- **Accessibility parity.** Every node keeps its AccessKit role and name, whatever
  language built it.
- **Size budgets.** Icon packs, plots, 3D, the viewer, connectors, `wasmtime`, and the GPU
  renderer are features. A default `egui-workbench` build stays small, and CI records
  the WebAssembly size.
- **Performance is tested.** Budgets live in benchmarks; a regression fails CI.
- **The egui train.** egui releases roughly every three months and breaks API; the
  upgrade is its own small release with its own migration note.
- **No idle repaint** and **files under 500 lines** stay as the design rules in
  `WORKBENCH.md`.

### Definition of done for every release

A release is done when all of these are true:

1. The exit criterion above passes in CI.
2. `CHANGELOG.md` has the entry, with a **Migration** section for any break.
3. Every new public item is documented, and the guide has a page and a live demo scene.
4. Snapshots cover every new visual, and the new behavior has kittest interaction tests.
5. The starter template, the showcase, and the agent skill are updated and compile.
6. `cargo xtask check`, the MSRV job, and the platform matrix pass.
7. Benchmarks show no regression against the previous release.

---

## Decisions to settle early

1. **One crate or a workspace.** egui types appear in our public API, so every egui
   breaking release is a breaking release for us, and that conflicts with a 1.0 promise.
   Recommended: split before 0.4 into an egui-free core (`egui-workbench-core`:
   commands, contexts, history, workspace and layout schema, dataset, query, protocol
   types), the egui-dependent crates (`egui-workbench` for the shell and components,
   `egui-workbench-plot`), and the bindings. `egui-workbench` keeps re-exporting
   everything, so applications do not change. The core can reach a true 1.0; the
   egui-dependent crates track egui.
2. **The egui policy at 1.0.** Recommended: a 1.x line supports one egui minor; a new egui
   minor ships as a new major of the egui-dependent crates only when egui's own breaking
   changes reach our signatures, and the core and the protocol do not change.
3. **Docking: build or adopt.** Recommended: prototype 0.6 on `egui_dock`, and replace it
   only if it blocks layouts as data, detached windows, or keyboard operation.
4. **How much of the component layer the spec exposes first.** The full set from the
   start is expensive; recommended: the shell, a core of inputs, `DataGrid`, and plots in
   0.11, then the rest by demand.
5. **An immediate-mode API for Python** (in the style of Dear PyGui) next to the
   declarative one. Recommended: declarative only until 1.0; revisit with user demand.
6. **Which data engines to depend on.** Arrow and Parquet are large; recommended: each
   connector behind its own feature, and the default build has none of them.
7. **Whether the sidecar is the first-class path for languages without bindings**, or a
   stepping stone to the C ABI. Recommended: the sidecar is first-class; the C ABI is for
   in-process needs.

## Ideas parked until after 1.0

- Real-time collaboration on a workspace (CRDTs).
- Notebook-style documents mixing text, data, and live plots.
- A visual designer that edits specs.
- Mobile targets (Android and iOS).
- Package registry hosting for extensions.

## How this changes

This is a plan, and plans change when users and measurements disagree with it. Open an
issue to propose a change, to argue for moving something earlier, or to say what you
would build on top of the next release. The scope of a release may move; its exit
criterion is the part to keep.
