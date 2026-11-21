# Changelog

## Unreleased

- **egui from crates.io, no fork.** The crate now depends on the official `egui`/`eframe`
  0.37 (and `egui_kittest` 0.37 for tests); the `[patch.crates-io]` block on the
  storytold forks is gone from the crate, the template, and the docs. Applications add
  one dependency line. All tests and pixel snapshots pass unchanged.
- Publication: crates.io metadata (repository, homepage, docs.rs, keywords,
  categories), an `include` list that ships only `src/`, `assets/`, the README,
  changelog, and licenses (3 MB, 640 KB compressed); CI runs the style lint, rustdoc,
  and `cargo package`; a `Docs` workflow builds the wasm scenes and the site and deploys
  them to GitHub Pages (`DOCS_BASE` sets the site's base path).
- Live documentation: every visual docs page embeds `<LiveDemo>`, which runs the real
  library in WebAssembly. Demos start by themselves near the viewport (queued, one wasm
  compile at a time), follow the site's light/dark switch without losing state, and
  leave the mouse wheel to the page until clicked. 22 scenes in `examples/scenes`:
  shell, catalog, components, buttons, inputs, display, toasts, menus, overlays,
  settings, icons, style and accent playgrounds, and nine charts (signal, selection, log,
  bars, live data, every series kind, fits over a selection, two million points, and the
  starter's full Charts gallery with its settings dialog). Chart export downloads the
  SVG/PNG in the browser. `cargo run --example scenes -- scene=<name>` runs one natively.
- Every Charts page has a live demo (configuration, series, fits, live data, and
  performance were missing).
- `tests/doc_scenes.rs`: renders every scene at phone and desktop widths, validates every
  `<LiveDemo>` tag, and compares each embedded demo's first frame with
  `tests/snapshots/doc-scenes/` (the static screenshot copies are gone).
- Starter: the Charts gallery refers to `chart_settings` with `super::` paths so the docs
  include it unchanged, and `ChartsDemo::collected` hands exports to a host instead of
  the export folder (the web demo turns them into downloads).
- The web bundle enables Lucide (toasts and icons in the browser) and takes the scene
  from the page's query string; `WebHandle::set_theme` switches themes live.
- Fixed: `TabStrip` in the catalog's Data section painted over the content above it
  (it paints its whole `max_rect`; it now gets a strip of its own).
- Style reaches the whole library: every component, widget, the tab strip, the title
  bar, and the plot chrome read radii, borders, and fonts from `WorkbenchStyle` (about
  60 literal radii, strokes, and font sizes replaced), so `sharp()` is square
  everywhere, including checkboxes, switches, menus, the tree, and the plot toolbar.
- `WorkbenchStyle::inner_radius` (menu and tree rows, checkboxes, segments) with the
  `inner`, `tight`, and `mark` helpers; `pill_radius` clamps to half the height (default
  12, full capsules in `rounded()`); `border(color)` for borders of the style's width.
- Text scale: `WorkbenchStyle::text_scale` (0.75–1.5) multiplies the type scale
  (`style::TYPE_SCALE`), builder labels, and chrome text; `font`, `mono`, `font_size`,
  and `text` helpers for custom widgets.
- `ThemeState::set_accent` / `accent`: a brand color applied on top of the effective
  built-in theme and re-derived on theme changes and desktop light/dark switches. The
  starter no longer re-implements system sync for its accent.
- `cargo xtask style-lint` (part of `cargo xtask check`): rejects literal radii and font
  sizes under `src/`, also in multi-line calls; `// style: fixed` marks an exception.
- Borderless styles stay usable: an unchecked checkbox gets a fill when
  `border_width` is 0, and `Outline` buttons keep a 1 pt line.
- Bordered buttons measure `content + padding + 2 × stroke` like egui's frame (they were
  2 pt narrow, squeezing the border).
- `Theme::radius` is deprecated in favor of `style::get(ctx).control()`.
- Starter: Settings → Shape → Text size (small, default, large; persisted).
- Snapshots: large text + borderless board; compact/sharp and comfortable/rounded
  updated for the wider style coverage.
- Documentation site (`docs-site/`): complete VitePress guide, component and chart
  reference, theming and runtime chapters, practical recipes, an API map, light/dark
  screenshots, and `npm run check-examples` for the complete Rust programs.
- Docs snapshots: icon browser in GitHub Light and GitHub Dark, and the settings dialog
  in GitHub Dark.
- Web demo: `web/` compiles `examples/catalog` to WebAssembly with eframe's WebRunner;
  the docs page `/guide/live-demo` embeds the running catalog, the same UI as desktop.
- `native_options`/`native_options_sized` are native-only; wasm entry points use
  `eframe::WebRunner` and `WebOptions`.
- `tests/line_limit.rs`: the test suite fails when any tracked Rust source under
  `src`, `tests`, `examples`, `templates`, `xtask`, or `web/src` exceeds 500 lines.
- Customization: custom themes (`theme::apply_tokens`, `Theme::with_accent`,
  `Theme::accessibility_issues`, `theme::mix`, `ThemeState::set_custom`); a global
  `style::WorkbenchStyle` (density, radii, border width; `sharp`/`rounded` presets)
  read by buttons, fields, steppers, menus, palette, tree, dialogs, popovers, cards,
  badges, and toasts; per-instance overrides on `Button`, `IconButton`, `Card`, and
  `Badge`; optional right and bottom panels in `WorkbenchApp`. Defaults are
  pixel-identical (snapshot-tested), with new snapshots for compact/sharp,
  comfortable/rounded, and a custom accent.
- Starter: Settings → Shape (density, corners, brand accent, persisted), View →
  Inspector (Ctrl+Alt+B) and Output (Ctrl+J).
- `components::plot`: interactive 2D plots (lines, areas, steps, scatter, bars, error
  bars, functions, reference lines; linear/log axes; zoom, pan, box zoom, area
  selection, hover popups, legend toggles, toolbar; least-squares fits with R²; live
  windows; per-pixel decimation for millions of points). Starter gallery: Charts.
- Plot configuration (`PlotConfig`): texts, font sizes, fixed axis limits, major step,
  minor divisions, label decimals, grid per axis, line weights and opacities, color
  overrides, and per-series styles; SVG/PNG export (`to_svg`, `to_png`) drawn from the
  same scene as the screen; toolbar settings and export buttons. The starter's charts
  have a settings dialog and export to a folder.
- Legend swatches center the marker on the line; legend names no longer elide early.
- `Button` applies its size's padding (small and ghost buttons were tight); ghost
  buttons show their frame only on hover. The padding is set on the parent `Ui`, so
  rows of buttons keep wrapping in `horizontal_wrapped` (regression test included).
- Button labels are centered in any parent layout (in top-aligned rows such as toast
  actions they sat 3 pt high); buttons take whole-point widths for crisp edges.
- The plot legend keeps clear of the toolbar: it elides names, or moves below it.
- `Toasts` / `Toast`: sonner-style notifications (info, success, warning, error,
  loading → update in place), corner positions, actions, hover-to-pause, queueing.
- `IconGrid`: responsive, virtualized icon grid/list whose cells always fill the
  width; `IconSize::Auto` scales glyphs with the window. The starter Icons tab uses it
  and shows all matches instead of the first 256.
- `Icons::texture_sized`: large and HiDPI glyphs are rasterized at 96–384 px, so
  previews are no longer blurry.
- `theme::current(ctx)` and `theme::motion_enabled(ctx)` expose the applied tokens.
- Responsive fixes found by rendering the starter at 640×480 → 1920×1080:
  `split_panes` no longer panics below 240 pt (and never overflows its space);
  settings and the new dialogs fit small windows with a 16 pt gutter; the explorer
  yields so the content keeps 480 pt (`CONTENT_MIN_WIDTH`, `SIDEBAR_MIN_WIDTH`).
- `show_choice_window` and `show_progress` use the shared dialog frame, header, and
  right-aligned footer (primary on the right) instead of egui's default window.
- `close_glyph` draws a centered vector cross instead of the "×" character; icon
  buttons of different sizes align on one row; the palette filter spans the row width
  and shortcuts are right-aligned and muted.
- `error_fg_color` / `warn_fg_color` use the theme's `danger` / `warning` tokens;
  destructive buttons pick readable text (`theme::readable_on`).
- Design: banners span the full width with a radius and tinted border; cards without
  `.width` fill the width and wrap long titles; neutral badges use the border token.

## 0.1.0

Initial reusable base.

- shadcn-style `components` layer: builders, shared variants/tones, state reactivity,
  accessibility, and composite blocks; `docs/COMPONENTS.md` and `examples/catalog`.
- 2,410 Lucide icons embedded and resolved by name (`docs/ICONS.md`).
- Starter gallery is split into fixed navigator sections and closable editor tabs;
  the Icons tab searches 2,410 Lucide icons by name/similarity with sizes and grid/list
  layouts. Tab strip has a left inset.
- Nine themes, size tokens, and the icon caches (`foundation`).
- Undecorated window, title bar and menus, tab strip, status bar, split view (`chrome`).
- Buttons, controls, dialogs, palette, settings, keyboard menu, explorer tree (`widgets`).
- Application runtime: `ThemeState`, `Documents`, `Keymap`, `CloseGuard`, and the fixed
  panel order in `show_workbench` (`app`).
- `templates/app` starter and `cargo xtask new`, with settings persistence.
- Component, runtime, and snapshot tests; `WORKBENCH.md` specification;
  `docs/ARCHITECTURE.md` and `docs/NEW_PROJECT.md`.
