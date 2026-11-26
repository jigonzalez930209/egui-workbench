---
title: Changelog
description: What exists and what changed in egui-workbench.
---

# Changelog

The repository's [`CHANGELOG.md`](https://github.com/jigonzalez930209/egui-workbench/blob/main/CHANGELOG.md)
is the source of truth; this page mirrors it.

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html): while it is 0.x, a minor
release may change the API, and each entry says how to migrate.

## [Unreleased]

## [1.1.0] - 2026-11-25

### Added

- Plot measurement tools. The toolbar now exposes a ruler and an area tool:
  - **Ruler** (`PlotTool::MeasureDistance`): drag between two points, snapping to the
    nearest visible point within 14 pt. Reports Δx, Δy, Euclidean distance, slope, and
    angle.
  - **Area** (`PlotTool::MeasureArea`) has two modes, selected in the chart settings:
    - **Rectangle**: drag a rectangle. Reports its area and, for every visible series,
      the minimum and maximum (with the slope at each extremum), the mean, the
      trapezoidal integral, and the endpoint slope.
    - **Curve vs chord**: drag between two points on the same curve. Reports the signed
      and absolute area between the curve and its chord, the arc length, the chord
      distance/slope/angle, and the largest deviation.
- `MeasurementConfig` in `PlotConfig` controls the mode, the signed/absolute label,
  the snap radius, and whether labels draw. Builder methods are
  `measurement_mode`, `measurement_area_style`, `measurement_snap_radius`, and
  `measurement_labels`.
- One analysis icon opens a menu with every filter and transform. Active items colour
  the icon, and the same toggles are available in the chart settings:
  - Filters: Savitzky–Golay, moving average, Gaussian, median, and exponential low-pass,
    with configurable windows, order, sigma, and alpha.
  - Transforms: FFT magnitude, inverse FFT, power spectral density, autocorrelation,
    cross-correlation of two series, Hilbert envelope, and cepstrum.
- A clear (eraser) icon appears only when the chart has measurements, annotations,
  selection, or analysis; clicking it removes every painted overlay.
- A pin icon opens a draggable chart-settings panel on hover; clicking the pin keeps it
  open. The panel hosts the measurement mode, area style, snap radius, labels, filter
  toggles and parameters, and transform selection. The existing toolbar icons are
  unchanged.
- Measurements live in `PlotState` in data coordinates, so they survive zoom, pan, fit,
  and serde round-trips. `measurements`, `add_measurement`, `remove_measurement`,
  `pop_measurement`, and `clear_measurements` manage the list.
- `PlotMeasurement`, `DistanceSummary`, `CurveAreaSummary`, `Extremum`,
  `SeriesMeasurement`, `region_measurements`, and `curve_area_summary` expose the same
  values the labels show.
- `examples/measurements`: a QA workbench with a damped line, a cubic scatter, two
  sensor traces, and a 50,000-point stream, plus a live panel with the computed
  summaries and the chart settings.

### Changed

- Measurement drags start from the pointer press origin instead of the position where
  egui first reports the drag, so snapping still sees the point the user pressed.

### Tests

- 26 measurement tests across the plot unit and integration suites: distance components,
  horizontal/vertical/negative directions, area bounds, extrema and slopes at extrema,
  integral, mean, endpoint slope, reference filtering, unsorted input, rectangle and
  curve-vs-chord areas, snapping, toolbar tools, analysis icon, clear icon, pinned
  settings panel, list helpers, zoom persistence, and serde round-trips.
- 16 analysis tests: five filters, seven transforms (including FFT/IFFT round-trip and
  spectral peaks), and four pipeline toggles.

## [1.0.4] - 2026-11-25

### Security

- Extension sandbox: `host_read_file` canonicalizes the requested path and every
  `filesystem:read` grant, so `..`, sibling prefixes, and symlinks cannot escape the
  grant (X1). A file larger than the output cap is rejected from its metadata before it
  is read (X3).
- Extensions run under a fuel budget and a 64 MiB linear-memory limit; `loop {}` traps
  instead of freezing the UI thread (X2).
- Manifest `entry` accepts only normal path components on every platform, rejecting
  `C:\x.wasm`, `\\server\share\x.wasm`, `a/../b.wasm`, and `/x.wasm` (X4).
- The remote WebSocket server accepts only a request with no `Origin` (native clients)
  or one explicitly allowed with `RemoteServer::allow_origin`; headers are capped at
  8 KiB and 64 lines, and at most 32 connections are served at once (X5, X6).
- An MCP `--read-write` session now requires `EGUI_WORKBENCH_CONFIRM` for destructive
  commands; the confirmation token is no longer read from argv (X7, X8).
- Every C ABI entry point catches Rust panics and returns `-1`/`NULL` with the panic in
  `wb_last_error`, and rejects invalid UTF-8 instead of substituting U+FFFD (Q3, Q4).
- Replaced our direct use of unmaintained `chrono` and `serde_yaml` with `jiff` and
  `serde_norway`. `chrono` remains a transitive dependency of `arrow`/`parquet` for
  temporal arrays and stays named in `deny.toml` (X9).

### Fixed

- `wb_host_version()` reports the crate version instead of a hard-coded `0.13.0` (Q2).
- The starter template and install guide use `version = "1"`, so `cargo generate` builds
  against the current release instead of 0.4 (S1, S2, D6).
- `ExtensionManager::enable` no longer calls `expect` after a fallible rebuild (Q5).
- `web` carries the workspace version and the old wasm `.tgz` files are gone (S4, S6).
- The C ABI ships only as a GitHub release archive; it is not published on crates.io
  (S5).

### Changed

- `cargo xtask check` now runs the template check, the docs examples, the C ABI client,
  and the Python, Go, and wasm conformance suites when their prerequisites are
  installed, plus `cargo deny`, `cargo-semver-checks`, and the 1.95 MSRV check (T2, T3,
  T4, D2, D5).
- Added `cargo xtask budgets` and documented the ignored frame budgets (T1).
- The release gate runs clippy, validates the npm, PyPI, and C ABI versions, and no
  longer ignores npm or Pages publish failures (D7-D10).
- Split the extension and host runtimes so every file stays under 500 lines; the
  line-limit guard now scans `crates/` too (Q1, T5).
- Documented the standalone wasm, web, and probe crates and recorded the decision that
  the egui-free core crate lands at 2.0 (S3, S7).
- Added the manual `C ABI` workflow and the pre-push lint hook (D11, Q6).
- Fuzz targets are committed and documented in `CONTRIBUTING.md` (T7).
- Documentation no longer claims checks that CI does not run (D1-D5).

## [1.0.2] - 2026-11-25

### Changed

- Probe crate separated from dev-dependencies, hosted headless rendering, byte-for-byte
  pixel verification, three-runner groups, and the wgpu renderer feature.

## [1.0.1] - 2026-11-25

### Fixed

- Windows undecorated frame reassertion for pointer alignment.

### Added

- Manual platform pixel workflow and probe, plus the support boundary.

## [1.0.0] - 2026-11-24

### Stable

SemVer promise, egui/MSRV policy, independent spec/protocol/extension versions, and the
versioning/support guides.

## [0.17.0] - 2026-11-24

### Changed

- Feature freeze, deprecated radius removal, migration guide, security policy, and the
  single tag release flow.

### Migration

The extension ABI and host spec remain compatible.

## [0.16.0] - 2026-11-24

### Added

- Theme Studio, VS Code import/export, high contrast, localization, fonts, and the
  accessibility report.
- Theme Studio and global guide.

### Migration

Additive.

## [0.15.0] - 2026-11-24

### Added

- Sandboxed WebAssembly extensions, versioned manifest, capabilities, manager view, hot
  reload, and C/Rust examples.
- Extensions guide.

### Migration

Additive; the extension runtime is feature-gated.

## [0.14.0] - 2026-11-24

### Added

- MCP server, permissions, accessibility tree, automation replay, agent panel, and the
  expanded DevTools.
- Agent threat model and guide.

### Migration

Additive; DevTools state gained defaults.

## [0.13.0] - 2026-11-24

### Added

- WebAssembly host and npm package with ESM and schema-generated TypeScript types.
- Remote UI server over WebSocket with the 0.11 protocol.
- C ABI, header, and Go/C conformance clients.
- Browser and remote tests plus the web guide.

### Migration

Additive; new crates and the npm package use the 0.13 spec.

## [0.12.0] - 2026-11-24

### Added

- `pip install egui-workbench`: PyO3/maturin abi3 wheels, Pythonic `Workbench` and
  widgets, event handlers, `app.post`, asyncio, NumPy/pandas/Polars/PyArrow data, and
  headless `app.step`.
- Python guide and CI wheel workflow.

### Migration

Additive; Python is a separate package on the same 0.12 host spec.

## [0.11.0] - 2026-11-24

### Added

- `egui-workbench-host` crate: versioned spec and schema, host runtime, patches, data
  buffers, protocol/frames, sidecar binary, custom widgets, and conformance fixtures.
- Native hot reload and a headless render test.

### Migration

Additive; the host is a separate crate published from the same tag.

## [0.10.0] - 2026-11-24

### Added

- Histogram/density, box, violin, hexbin, contour, polar, and Smith families.
- Analysis tools, FFT/spectrum, fits with errors, and selection statistics.
- Colormaps, colorbars, annotations/regions, math text, vector PDF, export presets.
- GPU filled surfaces for `Plot3D` and 2D/3D headless GPU tests.

### Migration

Additive; CPU remains the default renderer.

## [0.9.0] - 2026-11-24

### Added

- Optional `gpu` renderer for line/scatter data layers through `egui_wgpu`, with
  `Renderer::Gpu` and `set_output_format`.
- `MinMaxPyramid` for lazy min/max tiles over paged sources.
- `StreamingSeries` with live/frozen modes, rolling windows, and triggers.
- `Scale::SymmetricLog`, `Scale::Time`, and `Scale::Category`.
- The Plot engine guide.

### Migration

0.9 is additive; the CPU renderer remains the default.

## [0.8.1] - 2026-11-24

### Fixed

- `Dataset::rows_in_bounds` is gated behind `plot`, restoring the minimal feature builds.

## [0.8.0] - 2026-11-24

### Added

- `LinkHub` selection bus with Highlight/Filter/Isolate modes, predicates, and revisions.
- Dataset-bound plots (`Plot::from_dataset(...).x(...).y(...).color_by(...)`), brush
  publishing, and `DataGrid::link`.
- `PlotGrid` with a shared state and `ViewHistory` back/forward zoom.
- `examples/linked` and the Linked views guide.

### Migration

0.8 is additive; the linked methods are opt-in.

## [0.7.0] - 2026-11-24

### Added

- Typed `Dataset`/`Column` model with validity, units, metadata, slices, statistics, and
  `xy_pairs`.
- Paged `DataSource` and CSV, workbook, Arrow IPC, Parquet, SQLite, ring, and channel
  connectors behind features.
- Safe query language with positioned errors and bound column indices.
- `DataGrid` with sort, filter, search, hidden columns, selection, statistics, and
  TSV/Markdown/JSON copy.
- `ViewerRegistry` for tables, JSON/YAML/TOML, PNG/JPEG, SVG, Markdown, logs, code, hex,
  and text.
- The one-million-row CSV budget probe.

### Migration

0.7 is additive. Enable `viewer` for the registry and the connector feature you use;
`arrow`, `parquet`, and `sqlite` are opt-in.

## [0.6.0] - 2026-11-24

### Added

- `Layout`/`LayoutNode` editor-group tree, splits, tabs, pinned/preview tabs, and the
  closed-tab stack.
- `LayoutAction` keyboard equivalents, named layouts, and layout persistence in
  `Workspace`.
- `ActivityBar`, `examples/workspace`, and the Workspace guide.

### Migration

No removed items. A 0.5 workspace loads with a single empty group.

## [0.5.0] - 2026-11-24

### Added

- `History<S>` undo tree with coalescing groups and the History panel.
- `CommandJournal`, macros recorded from a journal range, and deterministic replay.
- `Workspace` persistence, atomic writes, version migration, and plot views.
- `Stash`, `HotExit`, and crash recovery with warnings for corrupt files.
- DevTools F12: journal viewer and JSON state inspector.
- `examples/session` and the starter's journal/DevTools integration.

### Migration

0.5 is additive; existing 0.4 calls keep working. See
[History, journal, and sessions](/app/history-and-session).

## [0.4.0] - 2026-11-24

### Added

- `CommandRegistry` with stable namespaced ids, titles, categories, icons, default
  shortcuts, enabled/checked state, an optional argument schema, and one source for
  menus, the palette, the title bar, the keyboard menu, and context menus.
- `When`/`WhenContext` with the standard keys and `!`, `&&`, `||`, parentheses.
- Rebindable keys, conflict detection, chords, the versioned keybindings JSON file, VS
  Code import, and a keybindings editor built from the registry.
- Palette v2 fuzzy ranking, recent commands, prefix modes, and background `PaletteFeed`.
- Feature `serde`: stable event names and JSON round-trips for every built-in state.
- Features `plot`, `plot3d`, `viewer`, and `svg`; `calamine` and `resvg`/`usvg` optional.
- Registry views for menus, the keyboard menu, and the command palette.

### Changed

- The starter application's chrome reads one registry.
- Public enums are `#[non_exhaustive]`.
- `PaletteState` gained `recent`; the palette ranks and records choices.

### Migration

- Add the features a `default-features = false` build uses; all are on by default.
- Prefer `CommandRegistry` over `Keymap`; construct `PaletteState` with
  `..PaletteState::default()`; add wildcard arms for non-exhaustive enums.

## [0.3.2] - 2026-11-24

### Added

- CI on three desktops: the full suite (including pixel snapshots) stays on Linux, and
  macOS and Windows run the unit and behavior tests, including `tests/ime_hidpi.rs`.
- `cargo-semver-checks` in CI against the previous release tag.
- A criterion baseline for plot decimation, a large tree, a one-million-row CSV, and
  palette filtering; the weekly `Benchmarks` workflow uploads the samples.
- `tests/ime_hidpi.rs`, `docs/PLATFORMS.md`, `docs/EGUI_UPGRADE.md`, and
  `docs/BENCHMARKS.md`.
- `cargo generate` support for the starter template, with an optional local-checkout
  path; `cargo xtask new` keeps working for contributors.

### Changed

- The starter depends on `egui-workbench` 0.3 from crates.io when no checkout path is
  given, and on the checkout when `cargo xtask new` creates it.

## [0.3.1] - 2026-11-24

### Fixed

- `Plot3D` rejects projected points more than eight frames outside the data area.
  A point just in front of the camera's near plane no longer paints a line to the edge
  of the chart; it is treated as behind the camera, so the line breaks instead. This
  protects 3D lines, bars, and surfaces from near-plane projection spikes.

## [0.3.0] - 2026-11-23

### Added

- `components::plot::Plot3D`: a native 3D chart with an orbit camera, perspective or
  orthographic projection, x/y/z axes with linear or log10 scales, back-wall grids, and
  the same themes, legend toggles, drag-and-drop legend, 13 hover animations, direction
  arrow, hover popup, and SVG/PNG export as `Plot`. Drag orbits, Shift-drag pans, the
  wheel zooms, and double-click fits the data.
- `Series3D`: `line`, `scatter`, 3D `bars` from a z base, regular `surface` grids, and
  `surface_fn` for sampled functions. Bars and surface cells are depth-sorted; surface
  cells are filled as quad meshes so concave projections stay correct. Per-series
  overrides live in `Plot3DConfig::series_style_mut`.
- `Plot3DState` keeps the camera, hidden series, legend position, and hover state, with
  `reset_view`, `camera`, and `set_camera` for programmatic control.
- 3D bars draw their faces as quad meshes and cull near-edge-on faces, so no stray
  anti-aliasing lines appear in perspective or orthographic views.
- `examples/plot3d`: a native gallery with a title bar, theme toggle, chart/projection
  selectors, the 13 legend animations, easing, wireframe, direction arrow, camera reset,
  and SVG/PNG export.
- The documentation has a detailed 3D guide, four live demos (`plot-3d`,
  `plot-3d-surface`, `plot-3d-bars`, `plot-3d-helix`), and the runnable commands:
  `cargo run --example plot3d` and `cargo run --example scenes -- scene=plot-3d`.

## [0.2.3] - 2026-11-22

### Added

- Hovering a legend row now draws that series above every other series without moving
  the legend rows. A hovered line covers overlapping bars, a pie slice sits on top of its
  neighbors, and a radar polygon covers the other polygons; the legend order and hidden
  state stay exactly where they were.
- `examples/viewer`: a runnable native example of the data viewer with drag-and-drop,
  an optional command-line path, the embedded sample workbook, and a light/dark toggle.
- The documentation now has a live gallery of every new chart family (pie/donut,
  candlesticks, heatmap, radar, gauges, and waterfall) instead of a single default chart.

## [0.2.2] - 2026-11-22

### Added

- Legend hover animations: resting the pointer on a legend row can play an effect on
  that series. `PlotConfig::legend_hover` (`LegendHoverConfig`) has an on/off switch, the
  effect, easing, period, and a second color; `Plot::legend_animation` and
  `Plot::legend_hover` are the builder shorthands. The default is a strong `Blink` with `AnimationEasing::Linear`: in dark mode it
  pulses between white and the series color, and in light mode between black and the
  series color. The 13 effects are `Blink`, `Fade`, `Throb`,
  `DashMarch`, `DashToggle`, `Glow`, `Sweep`, `Wiggle`, `Reveal`, `MarkerBounce`,
  `MarkerPulse`, `Twinkle`, and `Wave`; they adapt to line, pie, radar, gauge, heatmap,
  candlestick, and waterfall series. Reduced motion disables them.
- Direction arrow for streaming data: `PlotConfig::direction`
  (`DirectionArrowConfig`), `Plot::direction_arrow(true)`, and
  `Plot::direction_points(n)`. It measures the last 5 samples by default (2–64) and draws
  with each line-like series' color; `SeriesStyle::direction_arrow` overrides it per
  series. It is part of the scene, so exports include it.
- New chart types on `Plot`, with their layouts in `PlotConfig`:
  - pie and donut (`Series::pie`, `PieConfig`);
  - candlesticks (`Series::candles`, `[x, open, high, low, close]`);
  - heatmaps (`Series::heatmap`, `[x, y, value]`, `HeatmapConfig`);
  - radar polygons (`Series::radar`, `RadarConfig::labels`);
  - gauges (`Series::gauge`, `0.0..=100.0`, `GaugeConfig`);
  - waterfalls (`Series::waterfall`, `WaterfallConfig`).
  `Plot::chart_mode()` reports the inferred family.
- `components::viewer::FileViewer`: native Excel workbooks with one tab per sheet
  (`.xlsx`, `.xlsm`, `.xls`, `.xlsb`, `.ods`), CSV and TSV (RFC 4180 quoting), and
  UTF-8 plain text. The table and text views are virtualized, keep header and line-number
  toggles, and never do I/O while painting. `FileViewer::{open, from_bytes, text}` and
  the `Workbook`, `Sheet`, `Table`, `Cell`, `CellKind`, `TextFile`, and `ViewerError`
  types are public. Workbook parsing uses `calamine` (MIT).
- The starter's chart settings dialog has a **Motion** section for the legend animation
  and direction arrow, and a new `plot-types` documentation scene plus a `viewer` scene.

### Changed

- `PlotConfig` gains `legend_hover`, `direction`, `pie`, `radar`, `gauge`, `heatmap`, and
  `waterfall`; `SeriesStyle` gains `direction_arrow`.
- Legend rows are shaded in the plot scene instead of by the interaction pass, so the
  hovered row and the animated series are drawn in the same frame.

### Migration

- Code that builds `PlotConfig` or `SeriesStyle` with struct literals adds
  `..Default::default()` (or the new fields).

## [0.2.1] - 2026-11-22

### Changed

- Plots draw large unsorted series much faster. A hundred cyclic voltammograms
  (450 000 points) went from 165 ms to 13 ms per frame in a release build:
  - unsorted lines skip points within half a pixel of the last one drawn;
  - every line is simplified on screen (Ramer–Douglas–Peucker, 0.3 pt);
  - unsorted scatter draws one marker per screen cell (¾ of the radius), so retraced
    curves draw their markers once;
  - series extents for the automatic view are cached in `PlotState`;
  - hover on unsorted series searches the drawn points, refined to the exact one;
  - a frame's shapes reach egui in one batch, without copying the point lists.

## [0.2.0] - 2026-11-22

### Added

- Plots get a second y axis on the right: `Series::y_axis(YAxis::Right)` (or
  `SeriesStyle::y_axis`) measures a series on it, with its own range, scale, ticks, grid,
  and title (`PlotConfig::y2`, `Plot::y2_label`, `y2_scale`, `y2_limits`). With two axes,
  Alt + wheel zooms the left axis over the left half of the frame and the right axis
  over the right half. `PlotResponse::y2_view`, `screen_pos_on`, `PlotHover::axis`, and
  `PlotState::set_y2_view` expose it.
- The wheel over an axis's tick labels (outside the frame) zooms that axis alone.
- The legend can be dragged anywhere inside the frame; the position lives in
  `PlotState` (`legend_position`, `set_legend_position`) and is used by exports.
- Smooth lines: `Series::smooth(Smoothing::Spline | Monotone)`, `Series::spline`, and
  `SeriesStyle::smoothing`.
- Title and axis-title styling: `PlotConfig::title_color`, `title_align` (`TitleAlign`),
  and per-axis `AxisConfig::label_size`, `label_color`, and `tick_color`.
- The starter's Charts gallery adds a two-axis climate chart, and its settings dialog a
  Titles section, the right axis, tick colors, and per-series smoothing and axis.

### Changed

- Plot cursors: a crosshair over the data area with every tool, a closed hand while
  pressing or dragging to pan, resize arrows over an axis, and the ordinary pointer over
  the legend (whose hovered row is shaded) and the toolbar.
- `PlotState::reset_view` also resets the right axis.

### Migration

- `AxisConfig`, `PlotConfig`, `SeriesStyle`, and `PlotHover` have new public fields:
  code that builds them with struct literals adds `..Default::default()` (or the new
  `axis` field for `PlotHover`).

## [0.1.1] - 2026-11-21

### Added

- An agent skill, `skills/egui-workbench`, so AI coding agents (Claude, Codex, Cursor,
  Gemini, Copilot) build apps with the crate: setup, the `WorkbenchApp`/`Command` model,
  a compiling minimal app, the design rules, and the guide as Markdown. Installable as a
  Claude Code plugin marketplace, with `npx skills add`, or from the release zip.

### Changed

- The README (also the crates.io page) explains how to use the crate with AI agents.
- Releases attach `egui-workbench-skill.zip` and check that `.claude-plugin/plugin.json`
  matches the tag; CI fails when the skill is out of date with the docs.
- GitHub Actions moved to their Node 24 releases (checkout v7, setup-node v7, Pages v5/v6).

## [0.1.0] - 2026-11-21

First release. egui-workbench builds on `egui` and `eframe` 0.37 from crates.io and
needs Rust 1.95 or newer.

### Shell

- Undecorated window with a custom title bar: product mark, text menus, history and
  workbench toggles, an optional pill, drag region, double-click maximize, resize edges,
  and native-looking window controls (`native_options`, `handle_window_edges`).
- Document tab strip with dirty and warning markers, close buttons, middle-click close,
  scrolling, and a controls area; explorer tree with keyboard navigation, context menus,
  and the active-document marker; status bar; split view; keyboard menu (F10/Alt).
- `show_workbench` / `WorkbenchApp`: the fixed panel order (title bar, explorer, tabs,
  optional right inspector and bottom output panels, status bar, central content,
  overlays) with responsive minimum widths.

### Components

- A shadcn-style builder layer: `Button`, `IconButton`, `TextField`, `TextArea`,
  `Switch`, `Checkbox`, `Select`, `Stepper`, `SegmentedControl`, `Badge`, `Banner`,
  `Card`, `Label`, `Separator`, `StatusBar`, `Menu`/`MenuItem`/`MenuToggle`, `TabStrip`,
  `Tree`, `CommandPalette`, `Dialog`, `ConfirmDialog`, `NameDialog`, `ProgressDialog`,
  `Popover`, `IconGrid`, and `IconPreview`, with shared variants, sizes, and tones.
- `Toasts`: sonner-style notifications (info, success, warning, error, loading updated in
  place), six positions, actions, hover-to-pause, and queueing.
- Settings dialog scaffold (`show_settings_dialog`, rows, sections, switches).
- Every interactive widget carries AccessKit roles and names; keyboard paths for every
  pointer action.

### Plots

- `components::plot::Plot`: lines, areas, steps, scatter, bars, error bars, functions,
  and reference lines; linear and log axes; zoom, pan, box zoom, area selection, hover
  popups, legend toggles, and a toolbar; live windows; per-pixel decimation for millions
  of points.
- Least-squares fits (linear, polynomial, exponential, power, logarithmic) with R², and
  moving averages; `points_in` for selections.
- `PlotConfig`: titles, font sizes, axis limits, ticks, grids, line weights, colors, and
  per-series styles; SVG and PNG export drawn from the same scene as the screen.

### Theming and style

- Nine themes (GitHub Light/Dark/Dark Dimmed, Dracula, Dracula Soft, Alucard, One Dark
  Pro/Darker/Light) with WCAG AA contrast tests, follow-system light/dark, and reduced
  motion.
- Custom themes and brand colors: `Theme::with_accent` (contrast kept AA),
  `Theme::accessibility_issues`, `ThemeState::set_accent` (re-derived on every theme
  change), and `ThemeState::set_custom`.
- `style::WorkbenchStyle`: density, corner radii, border width, and text scale for every
  component and the chrome text, with `sharp()`/`rounded()` presets. The default is
  pixel-identical to the reference shell.

### Runtime and icons

- `ThemeState`, `Documents` (tab order, active document, dirty query), `Keymap`
  (ordered shortcuts), and `CloseGuard` (unsaved-changes prompts).
- Icon cache with tinted SVG textures rasterized for HiDPI: workbench UI glyphs, 2,410
  Lucide icons (feature `lucide-icons`), and the VS Code Material file icons (feature
  `material-icons`).

### Tooling and documentation

- `cargo xtask new` generates a complete starter application (settings persistence,
  command palette, shortcuts, charts gallery, unsaved-changes prompts);
  `cargo xtask check` runs fmt, a style lint (no literal radii or font sizes), clippy,
  rustdoc, and the tests.
- Over 200 tests, including pixel snapshots of the shell, components, plots, and every
  live documentation scene.
- Guide with live demos: every visual page runs the real library in the browser through
  WebAssembly (<https://jigonzalez930209.github.io/egui-workbench/>).

[Unreleased]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/jigonzalez930209/egui-workbench/releases/tag/v0.1.0
