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

## [1.3.0] - 2026-11-26

### Added

- **Chemistry 2D editing, ChemDraw/MolView workflow.** The `chem2d` bond tool now draws
  the first bond when the canvas is clicked, sprouts the next bond at 120° when an atom is
  clicked, and follows an explicit direction when dragged (30° snapping, ghost endpoint).
  Clicking a single bond with the single tool makes it double. A click or drag creates all
  the atoms it needs as one undo step. `EditorTool::Charge` adds the MolView `e+`/`e−`
  tools, and `clear_document` is the trash tool. The editor types element symbols onto a
  selection or hovered atom (including two-letter symbols), adjusts formal charge with
  `+`/`-`, sets the order of a selected or hovered bond with `1`–`4`, sets wedge/hash with
  `W`/`H`, nudges with the arrow keys, selects all with `Ctrl+A`, opens atom
  element/charge/label fields on double-click, and inserts clipboard SMILES with `Ctrl+V`.
  `Chem2dEditor::execute` and the `set_selected_*` helpers expose the same edits to a
  toolbar.
- **Attached fragments.** `Command::AttachRing` and `Command::AttachSmiles`,
  `Chem2dEditor::attach_ring`, `attach_smiles`, and `EditorTool::AttachFragment` attach a
  ring or a whole SMILES fragment to a clicked atom as one undo step. The fragment is
  oriented outward so it does not grow over the existing molecule. Ring tools now attach
  when clicked on an atom, and `Ctrl+V` over an atom attaches clipboard SMILES.
- **Vector icons and chemistry libraries.** The `editor` example uses custom
  `egui::Painter` icons for every tool, including bonds, stereo, rings, chains, charges,
  amino acids, sugars, and functional groups; no icon font or asset registration is
  required. The new `library` module groups amino acids and their side chains, sugars, nucleobases, rings
  and heterocycles, functional groups, solvents, ligands/chelators, and lipids. Every entry
  carries SMILES; clicking one arms `EditorTool::AttachFragment`, then an atom click
  attaches it and an empty-canvas click places it.
- **MolView-style example.** The `editor` example has a utility bar, bond/ring/chain
  palette on the left, atom palette plus a periodic table on the right, and a status bar.

### Changed

- **Skeletal 2D rendering.** Carbon vertices are unlabelled, heteroatoms keep their CPK
  colour, and the SVG export is skeletal instead of drawing a coloured circle on every
  atom. Aromatic rings use one circle instead of a circle plus dashed parallel bonds.
- **2D geometry optimizer.** `layout::optimize_layout` (used by `Command::CleanLayout`)
  builds a deterministic canonical layout: 1.5 Å bonds, 120° chain angles, ring-specific
  interior angles, fused-ring placement, branch clearance, and packed components. It
  preserves the drawing centroid and orientation, and a second call is a no-op.
- **Selection box.** A non-empty selection draws a dashed bounding box; dragging inside
  it translates the whole selected part, and the round handle above it rotates around the
  selection centroid. Both are single undo steps, and clicking inside the box no longer
  clears the selection.
- **Visual detail.** Bonds stop at heteroatom labels, labels and stroke widths scale with
  camera zoom, double bonds have a real gap and put the inner line inside a ring, the grid
  has minor/major/origin lines, and aromatic circles use the bond colour. Aromatic ring
  bonds are drawn as single lines when their circle is shown, so Kekulé structures no
  longer display circles and double bonds together; SVG/PNG export follows the same rule.

### Fixed

- **PNG labels.** PNG export rasterized from an empty `usvg` font database, so every
  heteroatom label disappeared. The `resvg` text and system-font features are enabled and
  the platform font database is loaded once and cached.
- **Double bonds.** A chain double bond used to draw three nearly overlapping lines, and a
  ring double bond put its second line only 2.5 points from the edge. They are now two
  lines with a clear gap, and the second line of a ring bond goes inside the ring.
- **Chemistry 2D labels.** A custom atom label (`Ph`, `COOH`, a residue name) is now the
  whole label in the editor and in SVG/PNG export; it used to be ignored in favor of the
  element symbol and implicit hydrogens.

## [1.2.0] - 2026-11-26

This release publishes the chemistry crates as plugins of the library and hardens the
whole workspace. Upgrade notes are at the end of this section.

### Added

- **Chemistry plugins.** `egui-workbench-chem-core`, `egui-workbench-chem2d`, and
  `egui-workbench-mol3d` are published to crates.io at `1.2.0`, versioned with the library
  (they were unpublished `0.1.0` workspace members). `chem-core` is the egui-free document
  model, selection bus, events, and background jobs; `chem2d` is the 2D structure editor
  with SMILES, molfile, SDF, RXN, and CDXML, properties, and SVG/PNG export; `mol3d` is the
  3D molecular and crystal viewer with an offscreen renderer and an optional wgpu path. The
  library does not depend on them. See the new
  [Chemistry plugins guide](https://jigonzalez930209.github.io/egui-workbench/guide/chemistry.html).
- `ExtensionManager::with_signature_verifier`, `ExtensionError::Signature`,
  `FrameError::Closed`, `Host::peek_events`, `data::read_limited`, and `data::MAX_FILE_BYTES`.
- Extension modules may export `alloc(len) -> ptr` to receive the command id.
- A `cache-warm.yml` workflow and a `workflow_dispatch` rehearsal for the release workflow.

### Security

- **Extensions.** The runtime checks a module's memory range before allocating, so a
  module can no longer make the host reserve up to 2 GiB with one `host_*` call.
  `host_read_file` reads only regular files (a FIFO or device node no longer hangs it) and
  reads at most `out_cap + 1` bytes. The checksum is verified on the same bytes that are
  loaded. Logs, denials, and contributions are bounded, and the module's instances,
  memories, and tables are limited. Ids are plain reverse-DNS names, `api` bounds must
  parse, and an empty `filesystem:read:` prefix is rejected.
- **Extension signatures.** A manifest with `signature` is refused unless the application
  installs a verifier with `ExtensionManager::with_signature_verifier`. Before, the field
  was accepted and never checked.
- **Remote server.** Handshake and write timeouts stop idle sockets from holding the 32
  connection slots. Frame payloads grow with the bytes that arrive instead of reserving
  the announced length (up to 64 MiB per connection). Unmasked client frames and control
  frames over 125 bytes are rejected, as RFC 6455 requires.
- **MCP agent.** An empty `EGUI_WORKBENCH_CONFIRM` token no longer matches an empty
  `confirmation`, and tokens compare in constant time. Without an allow-list,
  `run_command` only accepts commands the spec declares. The audit log keeps at most 240
  characters per entry and 10,000 entries; successful calls used to store their whole
  result, including screenshots.
- **Root crate: data connectors.** The SQLite connector opens the database read-only
  and refuses statements that write, so a query can no longer modify a file or create one
  at a mistyped path; a query stops at 5,000,000 rows. CSV, Arrow, Parquet, and the
  viewer refuse files over 512 MiB (`read_limited`, `MAX_FILE_BYTES`). The image viewer
  decodes with a 16,384-pixel and 512 MiB limit, and SVGs no longer load files named by
  an `href`. A PNG export is capped at 100 million pixels.
- **Root crate: parsers.** The data query and `When` expression parsers stop at 64 levels
  of nesting instead of overflowing the stack. `Localizer::load` accepts only plain
  locale names, so `../x` cannot choose the file read, and `{ $name }` values are no
  longer expanded a second time by a later argument. A workspace `version` above
  `u32::MAX` is rejected instead of wrapping to a small number.
- **Root crate: session files.** Workspace, stash, and journal writes go through a
  hidden, uniquely named temporary file created exclusively, flushed to disk, and
  renamed over the target. The old code removed the target first, so a crash between
  the two steps left no file, and its predictable temporary name followed symlinks.
- **Root crate: workbooks and plots.** A workbook over 512 MiB, or a sheet over
  50,000,000 cells, is refused. A plot tick label wider than the plot used to panic in
  `clamp`; it is now centred. Cells of a crafted `.xlsx` are still expanded by `calamine`
  before the cell limit can apply, so a hostile workbook can use a lot of memory; open
  untrusted workbooks in a separate process.
- **Chemistry crates.** The 2D superatom library had four wrong structures: `Boc` had an
  O=O bond, `OTs` had no aryl ring (it was a mesylate), `bpy` and `phen` contained
  three-membered rings, and `EDTA` bonded its acetate arms to carbons instead of nitrogens.
  They are rebuilt (tolyl ring, 2,2'-bipyridine, 1,10-phenanthroline, and four
  CH2-COOH arms), and a test now checks every entry for valence, connectivity, and ring size.
- **Dihedral angles were 180 degrees off.** `dihedral` and the 3D `Dihedral` measurement
  read an anti arrangement as 0 and a syn one as 180; they follow the IUPAC sign convention
  now (anti 180, syn 0, +90 for a clockwise quarter turn).
- **π–π stacking was never reported.** The ring search discarded every ring it found. Contacts
  and hydrogen bonds also paired atoms within three bonds (every bond angle read as a clash);
  they are skipped. Both searches no longer scan all atom pairs or look atoms up by a linear
  scan, which was cubic on a protein.
- **2D properties.** The Hill formula puts carbon-free compounds in alphabetical order
  (`ClH`, not `HCl`). Fused aromatic carbons (4.5 bond orders) no longer report a valence
  error, and carbocations, carbanions, and `BH4-` get the hydrogens their group implies. The
  label and the formula share one hydrogen count. Alternating-bond rings of four or eight atoms
  are no longer aromatic, and Kekulé furan, pyrrole, and thiophene are. The pyrrole template is
  drawn in Kekulé form so its N-H shows. `format_chemical_text` no longer turns the bond in
  `CH_3-CH_3` into a subscript minus.
- **2D editing.** `add_chain` spaced atoms unevenly (alternate bonds were 1.5 and 2.5 long); every
  bond is 1.5 now with a 120 degree angle. A command that fails halfway is rolled back. Deleting
  atoms prunes expanded superatom groups. Deltas reject non-finite or absurd coordinates and
  stop at 100,000 atoms; chains stop at 1,000 atoms; `relax` no longer freezes on a few hundred
  atoms; PNG export caps its size and dpi.
- **3D limits.** A supercell is limited to 32 copies per axis and 2,000,000 atoms, `pore_points`
  to 1,000,000 samples, a coordination polyhedron to its 16 nearest neighbours, an orbital grid
  to 160 points per edge, and a CPU render to 8192 pixels per edge and 36 megapixels. The surface
  grid is built per atom instead of per grid point and atom pair. A zero-size or oversized GPU
  offscreen target returns nothing instead of failing wgpu validation.
- **Chemistry core.** Removing an atom freed its slot but not its bonds' slots; they stay free
  now. An `IdPool` read from a document with a dangling free slot no longer panics.
- **Host.** A patch cannot nest the spec deeper than 128 levels. The pending event queue
  holds at most 10,000 events, and `read_frame` grows with the bytes received.
- **C ABI.** An interior NUL in an error message no longer erases the message.
- **File readers.** A `.cube` header cannot request a grid larger than the file.
  Fixed-width molfile and mol columns, short mmCIF rows, a CIF loop with only `_fract_x`,
  a zero cube dimension, and a bare `M  CHG` line panicked; they now return an
  error or are skipped. A SMILES bracket atom accepts at most 8 explicit hydrogens.

### Fixed

- **Charts.** A plot tick label wider than the plot used to panic in `clamp`; it is centered.
  PNG export refuses more than 100 million pixels with `the image is too large`. SVG rasterizing
  no longer follows an `href` to a file, in plots and in the viewer.
- CSV files with a byte-order mark kept it in the first column name, and a blank line
  became a row with a null value.
- SQLite and workbook columns that mix types showed `Int(5)` instead of `5`. A workbook's
  first sheet was the alphabetically first, not the workbook's first.
- Arrow timestamps in seconds, milliseconds, or microseconds failed to load; they are
  converted to nanoseconds.
- `MinMaxPyramid::tile` with a level of 64 or more panicked or reserved memory for the
  empty levels.
- `Keybindings::import_vscode` imported VS Code's `-command` removal entries as bindings
  to a command that does not exist; they are skipped. Binding an empty key sequence is
  an error.
- `show_choice_window` answered Esc even when the window was closed. The stepper
  control panicked on an inverted range, `ellipsize_start(text, 0)` underflowed, and
  Left on a collapsed leaf in a tree emitted a toggle.
- Edge resizing started on a maximized or fullscreen window. A host spec's
  `window.resizable` was never applied.
- `RingBuffer::read_range` copied the whole buffer for every page; it copies the page.
- `egui-workbench --stdio` exited with an error at a normal end of input. `read_frame`
  now returns `FrameError::Closed` for a clean close.
- `Host::reload_if_changed` dropped the shared action queue, registered custom widgets,
  and unpolled events. It keeps them.
- Extension logs were reported again on every `invoke`.
- A patch reset every input widget to its spec value. Widgets the patch does not touch
  keep the user's value.
- mmCIF used the first `loop_` instead of `_atom_site`; CIF rejected coordinates with an
  uncertainty such as `0.1234(5)`; CDXML rejected carbon nodes without `Element` and
  matched `B="` inside `LabelB="`.
- Molfile: atom number `0` or a malformed number bonded atom 1, a truncated atom or bond
  block was accepted, `M  CHG` lines held more than 8 entries, and V3000 wrote the charge
  in the atom-map field. V3000 now uses `CHG=`.
- SMILES `[Fe++]` gave +1 instead of +2.
- Python: a NumPy buffer whose size is not a multiple of the dtype width lost its tail.
- A host dialog's window close button emits `<id>.cancel`.
- The MCP server returns `-32601` for an unknown method, and `get_state` no longer drains
  pending events.
- A panicking chem job reports `Failed("the job panicked: …")` and wakes the UI.
- `infer_bonds` uses a spatial grid instead of comparing all atom pairs, and PDB/CIF
  loading no longer scans the atom list per atom.

### Changed

- `FrameError` has a new variant `Closed`.
- `ExtensionError` has a new variant `Signature`.
- Extension modules may export `alloc(len) -> ptr`; the host copies the command id there.
  Without it the id is written at offset 1024, which can overwrite module data.
- `Host::peek_events` reads pending events without draining them.
- A signed manifest no longer installs without a verifier.
- An MCP session without `--allow` can no longer run a command id the spec does not
  declare.

### Upgrade notes

- **macOS wheels.** PyPI now receives one wheel per architecture (`macosx_arm64` and
  `macosx_x86_64`) instead of a `universal2` wheel; pip picks the right one. Linux aarch64
  wheels are built natively.
- **Dihedral angles change sign convention.** A dihedral you stored from 1.1 is 180 degrees
  off from the IUPAC value that 1.2 reports. Recompute stored values.
- **Signed extensions.** A manifest with `signature` is refused unless the application installs
  a verifier. Remove the field or add `with_signature_verifier`.
- **MCP.** Without `--allow`, `run_command` only runs commands the spec declares.
- **Chains and patches.** `add_chain` draws uniform 1.5 bonds at 120 degrees; a patch keeps the
  values the user entered in widgets it does not touch; a `.cube` or supercell request beyond
  the documented limits is refused or clamped.
- **Charts.** PNG export is capped at 100 million pixels, and `scale` stays within 0.25 to 8.
  Tick labels wider than the plot are centered instead of panicking.
- **Builds.** The workspace profiles use `debug = "line-tables-only"` and strip release
  binaries; a full debug build is about 9 GB instead of more than 150 GB. See
  `CONTRIBUTING.md`.

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
- The tool ribbon is controlled by the pin at its right edge: hovering the pin unrolls
  the existing icons with a tape-measure animation, clicking the pin keeps the ribbon
  open, and dragging the pin moves the ribbon anywhere in the plot. `PlotState` exposes
  `pin_toolbar` and `collapse_toolbar`.
- The settings gear opens a chart-settings dialog (not a dropdown) with the measurement
  mode, area style, snap radius, labels, filter toggles and parameters, and transform
  selection. It still emits `settings_requested` for applications with their own dialog.
  The existing toolbar icons are unchanged.
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

- 29 measurement tests across the plot unit and integration suites: distance components,
  horizontal/vertical/negative directions, area bounds, extrema and slopes at extrema,
  integral, mean, endpoint slope, reference filtering, unsorted input, rectangle and
  curve-vs-chord areas, snapping, toolbar tools, analysis icon, clear icon, pin hover
  unroll, pin drag, pinned settings panel, list helpers, zoom persistence, and serde
  round-trips.
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

- The platform probe is a standalone crate: it no longer inherits the root
  `egui_kittest`/`criterion`/wgpu test dependencies, cutting hosted Windows compile time
  from hours to about a minute.
- Hosted Windows and macOS run the headless wgpu probe; a verify job fails when the three
  Windows, three macOS, or two X11 images differ byte-for-byte.
- Platform groups run at most three runners and are capped at 20 minutes.
- New `wgpu-renderer` feature for eframe's D3D12/Metal/Vulkan renderer.

## [1.0.1] - 2026-11-25

### Fixed

- Windows: reassert `ViewportCommand::Decorations(false)` once per viewport, removing a
  native frame above the custom title bar that shifted pointer hit tests on some Windows
  builds.

### Added

- `platform_probe` example and the manual `Platform pixels` workflow: native screenshots
  and pointer reports on hosted Windows/macOS/X11/Wayland runners and self-hosted legacy
  desktops and Linux compositors.
- Platform support boundary documentation, including the Windows 7/8.1 and old macOS
  toolchain limitation.

## [1.0.0] - 2026-11-24

### Stable

No new features. The release states the promise: Semantic Versioning for every crate and
package, egui 0.37 plus one following minor, MSRV 1.95, and independently versioned spec
(1), protocol (1), and extension ABI (0.15).

- `VERSIONING.md`, `SUPPORT.md`, and the versioning guide.
- The single tag release flow publishes crates, docs, npm, PyPI wheels, the C ABI, and the
  GitHub release from one cached run.
- The accessibility report and migration guide are part of the documentation.

## [0.17.0] - 2026-11-24

### Changed

- Feature freeze: no new user-facing features.
- Removed the deprecated `Theme::radius()`; radii come from the style tokens.
- Added the migration guide from 0.3, `SECURITY.md`, and the accessiblity report as
  release documentation.
- The release workflow is a single tag-driven flow: basic checks on one OS, docs, npm,
  crates, C artifact, Linux x86_64/aarch64 wheels, macOS universal2, Windows amd64, PyPI,
  npm, and the GitHub release, all from one run with shared caches. CI is manual only.

### Migration

See the [migration guide](/reference/migration). The extension ABI stays at `0.15` and
the host spec stays at version 1.

## [0.16.0] - 2026-11-24

### Added

- Theme Studio: VS Code color-theme import with automatic WCAG AA adjustment, JSON and
  CSS variable export, and a high-contrast theme derivation.
- Fluent-shaped localization with built-in English and Spanish dictionaries, plural
  rules, placeholders, runtime locale switching, `.ftl` overrides, and RTL detection.
- A font loading API with named CJK, emoji, and math fallback families.
- The accessibility report with automated contrast, focus, and spec-audit results plus
  the manual screen-reader checklist and known gaps.

### Migration

Additive. Localization and fonts are new foundation modules; Theme Studio is additive to
the 0.14 DevTools theme tab.

## [0.15.0] - 2026-11-24

### Added

- `egui-workbench-extensions`: a sandboxed WebAssembly runtime behind the default `wasm`
  feature, a versioned manifest, capability grants, checksum verification, and hot
  reload.
- A host API for logging, command/panel/viewer contributions, dataset row metadata,
  selection, and prefix-scoped filesystem reads; no WASI, network, or ambient filesystem.
- An extension manager view that lists versions, grants, contributions, and enable,
  disable, and reload actions.
- C and no_std Rust example extensions that register commands and panels through the same
  ABI, with install/grant/enable/invoke/reload tests.
- The Extensions guide.

### Migration

Additive. The extension crate is new; applications that do not use extensions can build
it with `default-features = false`.

## [0.14.0] - 2026-11-24

### Added

- `egui-workbench-agent`: an MCP stdio server with tools for the spec, state, commands,
  accessibility tree, patches, activity, and command execution.
- A permission model: read-only by default, command allow-lists, confirmation tokens for
  destructive commands, and an audited activity log.
- An accessibility tree generated from the spec (roles and names) and headless automation
  replay for commands and patches.
- The agent panel component: streaming Markdown, tool-call cards, and diff approval.
- DevTools tabs for the widget inspector, live theme tokens with WCAG checks, frame-time
  and repaint diagnostics, and an accessibility audit.
- The agent threat model and the Agents and DevTools guide.

### Migration

Additive. DevTools gains state fields and labels; the default constructor keeps working.

## [0.13.0] - 2026-11-24

### Added

- npm package `egui-workbench`: a WebAssembly host mounted on a canvas, an ESM wrapper
  (`createWorkbench`, `connectRemote`), and TypeScript declarations generated from the
  host JSON Schema.
- `egui-workbench-remote`: a WebSocket server that carries the 0.11 messages, binds
  localhost by default, and exposes an `on_command` hook.
- The C ABI `libegui_workbench_c` with `include/egui_workbench.h` for C, C++, Go, Zig,
  and C#.
- Reference clients in Go (sidecar protocol) and C (C ABI) checked against the shared
  conformance fixture.
- Browser tests: wasm host tests in headless Chrome and a full canvas/remote test that
  renders the spec and applies a patch.
- Web, remote, C ABI, and reference-client documentation.

### Migration

Additive. The wasm, remote, and C crates are new; the npm package and the Python package
share the 0.13 spec.

## [0.12.0] - 2026-11-24

### Added

- PyPI package `egui-workbench`: a PyO3/maturin extension built for abi3 on CPython 3.9+
  with wheels for Linux x86_64/aarch64, macOS universal2, and Windows, plus an sdist.
- The Pythonic layer: `Workbench`, `Command`, dataclass widgets, `@app.on(...)`,
  `app.run()`, `app.emit()`, `app.save()`, `app.schema()`, and `app.step()` for headless
  tests.
- Event handlers on the UI thread with a `Context` for JSON-pointer patches, toasts,
  title changes, quit, and local command dispatch.
- Thread-safe `app.post(callback)` and asyncio integration, including `async` handlers
  and `await app.run_async()`.
- Data ingestion from NumPy, pandas, Polars, PyArrow tables/record batches/arrays through
  the Arrow C data interface, plus mappings and row dictionaries.
- `py.typed`, `_native.pyi`, and the host JSON Schema exposed to Python.
- A Python guide, CI wheel builds, and a PyPI wheel workflow with Trusted Publishing.

### Migration

Additive. The Python package is published separately from the Rust crates; it pins the
same 0.12 host spec.

## [0.11.0] - 2026-11-24

### Added

- New crate `egui-workbench-host`: a versioned `HostSpec` with window, theme, commands,
  menus, explorer, editor groups, panels, status bar, widget tree, dialogs, and toasts;
  JSON Schema generated from the Rust types.
- `Host` runtime: `new`, `load`, `apply`, `poll_events`, `set_dataset`, `register_custom`,
  `reload_if_changed`, and `run`, with native hot reload.
- JSON-pointer patches (`set`, `replace`, `remove`, `append`, `batch`) validated before
  they replace the spec.
- A transport-agnostic protocol with `Hello`, `Request`, `Response`, `Event`, `Patch`,
  `Data`, and `Error` messages, plus four-byte length-prefixed frames for stdio, sockets,
  and WebSocket binary messages.
- The `egui-workbench` sidecar binary: run a spec, speak `--stdio`, print `--schema`, and
  hot reload.
- Native `CustomWidget` escape hatch and data buffers that feed `Dataset` without a spec
  copy.
- Language-neutral `conformance/basic.json` fixture, runtime tests, and a headless wgpu
  render test.

### Migration

0.11 is additive for `egui-workbench`. The host is a new crate; the release workflow
publishes both crates from the same tag.

## [0.10.0] - 2026-11-24

### Added

- Chart families: histogram and density histogram, box, violin, hexbin/2D density,
  contour, polar, and Smith charts; `Plot::series_all` for multi-series families.
- Analysis tools: peak detection, rolling baseline removal, Savitzky–Golay smoothing and
  derivative, derivative/integral, radix-2 FFT and spectrum, resampling, Gauss–Newton
  fits with standard errors and RMSE, and selection statistics.
- `Colormap` with perceptually uniform, diverging, and color-blind-safe categorical
  palettes; heatmap `colormap` and a matching colorbar; standalone `show_colorbar`.
- `PlotAnnotation` text, arrows, x/y spans, rectangles, ellipses, and polygons in
  `PlotState`, serialized and included in exports.
- Math text: a LaTeX subset (Greek, scripts, fractions, roots, units) in titles, axis
  labels, legend names, and annotations.
- Vector PDF export (`Plot::to_pdf`, `ExportFormat::Pdf`) from the same scene, plus
  `ExportPreset` sizes (`Screen`, `HalfPage`, `Letter`, `Journal`).
- `Plot3DConfig::renderer`/`Plot3D::renderer`: GPU filled surfaces, bars, and primitives
  through the `egui_wgpu` triangle pipeline; axes, lines, text, legend, and hover stay on
  the CPU reference.
- 2D and 3D GPU headless tests.

### Migration

0.10 is additive. `Renderer::Cpu` remains the default, and existing titles/annotations are
plain text until they use the math subset.

## [0.9.0] - 2026-11-24

### Added

- Feature `gpu`: a line/scatter renderer through an `egui_wgpu` paint callback, selected
  per plot with `Renderer::Gpu`. `set_output_format` configures the surface format;
  `tests/gpu_plot.rs` renders through a real headless wgpu device. Charts, axes, legend,
  hover, selection, and export remain on the CPU reference painter.
- `data::MinMaxPyramid`: lazily built min/max tiles over a paged `DataSource`, a tile
  cache, level selection for a query budget, `range()` and `polyline()`.
- `data::StreamingSeries`: live/frozen modes, capacity, a rolling x window, revisions, and
  rising/falling `Trigger`s.
- `Scale::SymmetricLog`, `Scale::Time` (calendar ticks in UTC), and `Scale::Category` with
  `AxisConfig::categories`.
- The Plot engine guide and the `gpu` feature build in CI.

### Migration

0.9 is additive. `PlotConfig::renderer` defaults to `Renderer::Cpu`, so existing plots and
snapshots are unchanged. A host that selects `Renderer::Gpu` enables the `gpu` feature and
calls `set_output_format` once at startup.

## [0.8.1] - 2026-11-24

### Fixed

- `Dataset::rows_in_bounds` is behind the `plot` feature, so
  `cargo check --no-default-features` and the other feature builds compile again.

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

[Unreleased]: https://github.com/jigonzalez930209/egui-workbench/compare/v1.3.0...HEAD
[1.3.0]: https://github.com/jigonzalez930209/egui-workbench/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/jigonzalez930209/egui-workbench/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/jigonzalez930209/egui-workbench/compare/v1.0.4...v1.1.0
[1.0.4]: https://github.com/jigonzalez930209/egui-workbench/compare/v1.0.2...v1.0.4
[0.3.1]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/jigonzalez930209/egui-workbench/releases/tag/v0.1.0
