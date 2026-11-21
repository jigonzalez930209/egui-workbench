---
title: Live demo
description: Run the component catalog in the browser with the same egui-workbench rendering as the desktop app.
---

# Live demo

The component catalog runs in the browser through `eframe`'s WebRunner and WebGL. It is
the same catalog from `examples/catalog`, using the same `egui-workbench` components,
themes, sizes, and style tokens. The rendering is not a screenshot: it is the desktop UI
compiled to WebAssembly.

Every other visual page of this site embeds a smaller live scene the same way; it starts
by itself as you scroll to it. Click a demo to give it the keyboard and the mouse wheel;
until then the wheel scrolls the page. The
scenes live in `examples/scenes`; run any of them natively with
`cargo run --example scenes -- scene=<name>`.

<LiveDemo
  scene="catalog"
  height="680"
  alt="egui-workbench component catalog running in the browser"
  caption="The full catalog: pick a section on the left, switch themes, open dialogs and the palette."
/>

If the demo stays blank, use **Open in a tab** below it, or check that the wasm artifacts
were built (`npm run build:demo`).

## Build it

```bash
cd docs-site
npm run build:demo
```

The web crate lives in `web/`:

- `web/src/lib.rs` exposes `WebHandle` (`new`, `start(canvas, query)`, `set_theme`,
  `destroy`) and includes `examples/scenes/scenes.rs` through a `#[path]` module. The
  query selects the scene (`?scene=plot&theme=github-light&style=compact-sharp`); without
  one, the catalog runs.
- `docs-site/public/demo/index.html` owns the canvas, loading state, and error display,
  and forwards the site's light/dark switch to `set_theme`.
- Exported charts become browser downloads (a `Blob` and an `<a download>`).
- The page posts `egui-workbench:ready` (or `egui-workbench:error`) to the docs page, which
  fades the demo in; embedded demos start unfocused so the wheel keeps scrolling the page.
- `web/Cargo.toml` enables Lucide (toasts and the icon browser need it) and skips the
  ~5 MB Material file-icon pack.

## Why it looks identical

The browser build uses the same:

- `Theme` tokens and `theme::apply_tokens` path;
- `WorkbenchStyle` density and radii;
- `show_workbench` panel order and `metrics` sizes;
- component builders and AccessKit metadata.

Only the platform layer differs: winit/Glow on desktop, wasm-bindgen/WebGL in the browser.
The catalog's own state and layout code are shared, not reimplemented for the web.

## Constraints

- WebGL is required (any current browser).
- The web build skips Material file icons; enable `material-icons` in `web/Cargo.toml`
  if your demo needs them, at the cost of a larger `.wasm`.
- Browser clipboard and file dialogs use the web implementations from eframe; native
  filesystem APIs are not available.
- The iframe keeps the site's navigation usable; use the standalone tab for a larger
  canvas.
