# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html): while it is 0.x, a minor
release may change the API, and each entry says how to migrate.

## [Unreleased]

## [0.8.0] - 2026-11-24

### Added

- `data::LinkHub`, `LinkSelection`, and `LinkMode` (Highlight, Filter, Isolate): one
  selection bus per dataset with a revision, a publisher name, and an optional predicate.
- `Plot::from_dataset(&data).x("t").y("v").color_by("state")`, with `scatter`, `title`,
  and `height`; a text/category `color_by` becomes one colored series per value.
- `DatasetPlot::show_linked`, which maps a plot brush back to dataset rows and publishes
  them; Filter/Isolate modes hide the points outside the selection.
- `PlotGrid`: rows × columns of plots sharing one `PlotState` for linked pan, zoom,
  hidden series, and selection.
- `DataGrid::link(&hub)`: table filtering/isolating and highlight from a linked
  selection.
- `ViewHistory`: bounded back/forward zoom levels and `PlotState::view()`.
- `examples/linked`: a colored scatter, a line/scatter grid, and a linked table over one
  dataset.

### Migration

0.8 is additive. `Plot` and `DataGrid` keep their existing constructors; the linked
methods are opt-in.

## [0.7.0] - 2026-11-24

### Added

- `data::Dataset` and `Column`: typed columns (float, int, bool, text, date, time,
  category), validity masks, units, metadata, zero-copy-backed `Arc` storage, slices,
  `take`, statistics, and `xy_pairs`.
- `DataSource`, a synchronous paged trait with `schema`, `row_count`, `read_range`,
  `revision`, and `cancel`.
- Connectors behind features: CSV/TSV (`csv`), spreadsheet sheets (`workbook`), Arrow IPC
  (`arrow`), Parquet (`parquet`, Arrow-backed), SQLite (`sqlite`), the in-memory ring
  buffer (`ring`), and channel snapshots (`channel`). `csv`, `workbook`, `ring`, and
  `channel` are in the default set; `arrow`, `parquet`, and `sqlite` are opt-in.
- The safe query language: comparisons, `&&`, `||`, `!`, parentheses, `in 2..5`,
  `in [1, 4]`, strings, booleans, numbers, and null, with positioned errors and
  once-per-query column binding.
- `components::DataGrid`: typed virtualized rows, sort, query filter, quick search,
  hidden columns, row/cell selection, header statistics, and TSV/Markdown/JSON copy.
- `ViewerRegistry`: extension and content detection for tables, JSON, YAML, TOML,
  PNG/JPEG, SVG, Markdown, logs with level filters, code, hex, and text, all through one
  `show` call.
- `Series::from_dataset(&data, "t", "v")`.
- `tests/data_budget.rs`, the ignored release probe for the one-million-row CSV.

### Changed

- The `viewer` feature now includes `viewer-formats` (JSON/YAML/TOML trees, Markdown, and
  PNG/JPEG); the image and tree dependencies are optional behind it.
- The default build adds the dependency-free CSV, ring, and channel connectors and the
  workbook connector.

### Migration

0.7 is additive. A `default-features = false` build that wants the viewer must enable
`viewer`; a build that wants a data connector enables its feature. The 0.6 `FileViewer`
stays available, and `DataGrid` is the typed successor for new tables.

## [0.6.0] - 2026-11-24

### Added

- `Layout` and `LayoutNode`: a serializable tree of editor groups and horizontal or
  vertical splits, with tabs, pinned and preview tabs, active indices, and a 20-tab
  closed stack.
- Group and tab operations: split, close and promote the sibling, move a tab between
  groups, close and reopen, pin, maximize, and Zen mode.
- `LayoutAction`: keyboard equivalents for every split, focus, tab-move, close, maximize,
  Zen, and reopen operation.
- Named layouts stored in `Workspace` (`layout`, `named_layouts`, `active_layout`) and
  switched from the palette; a 0.5 workspace migrates to a single empty group.
- `ActivityBar` with icons, an active marker, badges, tooltips, and AccessKit `Tab` roles.
- `examples/workspace`: three editor groups from one layout, a floating panel, and a
  native detached viewport.
- The Workspace guide page.

### Changed

- `Workspace` gained the layout fields; `serde(default)` keeps older files loading.
- `show_workbench` and `WorkbenchApp` keep their one-group behavior; the layout API is
  additive.

### Migration

No removed or renamed items. A 0.5 `workspace.json` loads with one empty editor group and
the new fields defaulted.

## [0.5.0] - 2026-11-24

### Added

- `History<S>`: transactional undo/redo over a tree, not a stack. Undoing and then
  editing keeps the abandoned branch; `begin_group` coalesces typing and drags; `jump_to`
  time-travels. The History panel renders the tree.
- `CommandJournal`: an append-only, versioned, serializable log of every command with its
  arguments and timestamp, with atomic save/load.
- Macros: record a journal range as a named `macro.<name>` command, register it in a
  registry, and replay the recorded `(command, args)` sequence deterministically.
- `Workspace` persistence: open documents, active tab, explorer expansion, panel sizes and
  visibility, theme preferences, and plot views in one versioned file. Writes are atomic;
  version 0 migrates through `serde(default)` and newer versions are rejected.
- Hot exit and crash recovery: `Stash` captures unsaved buffers, `HotExit` writes the
  workspace, stash, and journal on a timer and on close, and `recover` restores what
  parses and reports corrupt files as warnings.
- DevTools, first slice: F12 opens the journal viewer and a JSON state inspector.
- `examples/session`: a runnable history/journal/workspace/DevTools application.
- The starter application journals every command and opens DevTools with F12.

### Changed

- The web build enables `plot`, `plot3d`, `viewer`, and `serde` explicitly, so the
  documentation scenes keep rendering with the new feature gates.

### Migration

0.5 is additive: existing 0.4 calls keep working. Put `workspace.json`, `stash.json`, and
`journal.json` in one session directory (or keep your own paths) and use the new
`app::Workspace`, `app::Stash`, and `app::CommandJournal` types.

## [0.4.0] - 2026-11-24

### Added

- `commands::CommandRegistry`: stable namespaced ids, title, category, icon, default
  shortcut, enabled and checked state, an optional argument schema, and one source for
  menus, the palette, the title bar, the keyboard menu, and context menus.
- `commands::When` / `WhenContext`: the `editorFocus`, `hasSelection`, `dialogOpen`,
  `paletteOpen`, `readOnly`, and `dirty` keys with `!`, `&&`, `||`, and parentheses.
- Rebindable keys: conflict detection, chord sequences (`Ctrl+K Ctrl+S`), the versioned
  keybindings JSON file, VS Code `keybindings.json` import, and a keybindings editor
  built from the registry.
- Palette v2: fuzzy ranking, recent commands, the `>`/`:`/`@` prefix modes, and
  `PaletteFeed` for background results that never block the frame.
- Feature `serde`: stable snake_case event names and `Serialize`/`Deserialize` for every
  built-in state, including `ThemeState`, `Documents`, `TreeState`, `PaletteState`,
  `PlotState`, `PlotConfig`, `Plot3DState`, the viewer data, and the event types.
- Cargo features `plot`, `plot3d`, `viewer`, and `svg`, with `calamine` and
  `resvg`/`usvg` optional. The default set matches 0.3.
- `show_command_menu`, `command_menu_groups`, `command_palette_items`, and
  `show_command_palette` as registry views.
- `docs/MIGRATION_0.4.md` and the command/keybinding guide.

### Changed

- The starter application's chrome reads one `CommandRegistry`; menus, the palette, the
  keyboard menu, and shortcuts no longer each carry their own table.
- Public enums are `#[non_exhaustive]`; exhaustive matches outside the crate need a
  wildcard arm.
- `PaletteState` has a `recent` list, and `show_palette` ranks matches and records the
  chosen row.
- The starter template enables `plot`, `plot3d`, `viewer`, and `serde` explicitly.

### Migration

- Add the features a `default-features = false` build uses (`plot`, `plot3d`, `viewer`,
  `serde`); all are on by default.
- Replace `Keymap<C>` with `CommandRegistry` and map the returned `CommandId` to your
  handler. `Keymap` remains available for simple applications.
- Construct `PaletteState` with `..PaletteState::default()`; it gained `recent`.
- Add `_` arms to exhaustive matches on the public enums.
- See `docs/MIGRATION_0.4.md` for the full list.

## [0.3.2] - 2026-11-24

### Added

- CI on three desktops: the full suite (including pixel snapshots) stays on Linux, and
  macOS and Windows run the unit and behavior tests, including `tests/ime_hidpi.rs`.
- `cargo-semver-checks` in CI against the previous release tag, so every public change
  is visible in review.
- A criterion baseline for plot decimation (1M points), a 10k-row explorer tree, a
  1M-row × 20-column CSV (open and first frame), and palette filtering. The weekly
  `Benchmarks` workflow uploads the samples; the recorded numbers are in
  `docs/BENCHMARKS.md`.
- `tests/ime_hidpi.rs`: IME pre-edit/commit composition and a 2× pixels-per-point frame,
  run on Linux, macOS, and Windows.
- `docs/PLATFORMS.md`: the repeatable IME and HiDPI pass for each desktop, with the
  automated coverage listed next to each manual step.
- `docs/EGUI_UPGRADE.md`: the egui upgrade policy for 0.x and the 1.0 promise.
- `cargo generate` support: `templates/app` is a cargo-generate template with an
  optional local-checkout path, so a new application can start from the published
  template without cloning the repository. `cargo xtask new` keeps working for
  contributors.
- Documentation pages for the desktop matrix and the benchmark baseline.

### Changed

- The starter template now depends on `egui-workbench` 0.3 from crates.io when no
  checkout path is given, and on the checkout when `cargo xtask new` creates it.

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
