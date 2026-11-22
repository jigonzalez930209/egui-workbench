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

### Added

- An agent skill, `skills/egui-workbench`, so AI coding agents (Claude, Codex, Cursor,
  Gemini, Copilot) build apps with the crate: setup, the `WorkbenchApp`/`Command` model,
  a compiling minimal app, the design rules, and the guide as Markdown. Installable as a
  Claude Code plugin marketplace, with `npx skills add`, or from the release zip.

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

[Unreleased]: https://github.com/jigonzalez930209/egui-workbench/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jigonzalez930209/egui-workbench/releases/tag/v0.1.0
