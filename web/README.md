# Web demo

The component catalog can run in the browser with `eframe`'s WebRunner and WebGL. It uses
the same `egui-workbench` components and the same theme tokens as the desktop catalog, so
the layout and rendering match the native app.

## Build

```bash
rustup target add wasm32-unknown-unknown
wasm-pack build web --target web --release \
  --out-dir docs-site/public/demo/pkg --out-name egui_workbench_web
```

The generated `pkg/` directory is served by the docs site at `/demo/`. The page loads
`pkg/egui_workbench_web.js`, initializes the wasm module, and calls `WebHandle::start`
with the canvas.

## Run

```bash
cd docs-site
npm run dev
# open http://localhost:5173/demo/
```

The docs page at `/guide/live-demo` embeds the same URL.

## Notes

- The demo depends on the component catalog (`examples/catalog`) through a `#[path]`
  module, so there is one source of truth.
- The web crate disables the Material and Lucide asset packs; the catalog only uses the
  bundled UI glyphs. Enable the features in `web/Cargo.toml` if your demo needs them.
- `wasm-pack` must use a `wasm-bindgen` CLI matching the resolved `wasm-bindgen` crate.
  wasm-pack downloads the matching version automatically.
- Regenerate the artifacts after changing the catalog or any component used by it.
