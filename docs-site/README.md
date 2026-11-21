# egui-workbench documentation site

The complete documentation for `egui-workbench`, built with VitePress.

## Run

```bash
cd docs-site
npm install
npm run dev        # http://localhost:5173
npm run build      # static output in .vitepress/dist
npm run preview    # serve the build
```

Node 18+ is required. The build fails on broken internal links (`ignoreDeadLinks` is left
at its default, `false`).

## Structure

```text
docs-site/
├── .vitepress/
│   ├── config.ts              # nav, sidebar, search, edit links, Shiki themes
│   └── theme/
│       ├── index.ts           # registers the Vue components
│       ├── custom.css         # GitHub Light/Dark alignment
│       └── components/
│           ├── LiveDemo.vue   # the real library in wasm, started automatically
│           ├── Preview.vue    # light/dark image switch (for static images)
│           ├── ApiTable.vue   # name, type, default, description
│           └── TokenSwatch.vue
├── public/
│   ├── logo.svg
│   ├── demo/                  # wasm scenes + catalog (index.html + generated pkg/)
│   └── workbench.png          # the home page hero (the shell scene's first frame)
├── guide/                     # introduction, install, quick start, first app, concepts
├── components/                # builders, inputs, display, feedback, overlays, navigation
├── charts/                    # series, axes, interaction, config, fits, export, live data
├── theming/                   # themes, tokens, metrics, icons, style, customization
├── app/                       # runtime, commands, documents, window, settings, i18n
├── recipes/                   # editor, live plot, icon browser, settings, tasks, brand
└── reference/                 # API map, rules, testing, xtask, changelog, FAQ
```

## Live demos

Every visual page embeds a `<LiveDemo>`: a scene from `examples/scenes` that runs the
real library in the browser through WebAssembly.

```md
<LiveDemo
  scene="plot-live"        # a name from SCENES in examples/scenes/scenes.rs
  look="compact-sharp"     # optional style preset
  theme="dracula"          # optional fixed theme; otherwise it follows the site
  accent="e06c2b"          # optional brand accent
  height="420"             # height in CSS pixels (capped at 78 % of the viewport)
  alt="…"
  caption="…"
/>
```

Demos start by themselves when they come near the viewport. `live.ts` queues them, so
one iframe compiles the wasm at a time and the rest reuse the cached download; each
fades in when its page posts `egui-workbench:ready`. The iframe loads
`/demo/index.html?scene=…` (spelled out: `vitepress dev` routes `/demo/` to the site's
404 page), and the site's light/dark switch is posted to it, so the scene changes theme
without losing state.

An embedded demo starts unfocused: the mouse wheel scrolls the docs page, and egui
ignores it, until the reader clicks the demo. Then the wheel zooms plots and scrolls the
scene's own lists.

```bash
# Run a scene natively, exactly as the site embeds it
cargo run --example scenes -- scene=plot-live theme=dracula

# Refresh the reference frames after an intended visual change (also checks every tag)
cd ..
UPDATE_SNAPSHOTS=1 cargo test --test doc_scenes

# Rebuild the wasm bundle after changing a scene or a component
cd docs-site
npm run build:demo
```

`cargo test --test doc_scenes` fails when a `<LiveDemo>` names an unknown scene, look, or
theme, when a scene panics at phone or desktop width, or when a scene's first frame no
longer matches its reference in `tests/snapshots/doc-scenes/`. Review every changed PNG.

### Adding a scene

1. Write a `Scene` in `examples/scenes/` (state + `fn ui(&mut self, ui, host)`; report
   actions with `host.report(…)`).
2. Add its name to `SCENES` and to `build()` in `scenes.rs`.
3. Embed it with `<LiveDemo scene="…">` and run the three commands above.

## Build the wasm bundle

One wasm bundle serves every live demo and the `/guide/live-demo` catalog:

```bash
npm run build:demo
```

`web/src/lib.rs` includes `examples/scenes/scenes.rs` (which includes
`examples/catalog/main.rs`), so native, web, and the poster tests run the same code.
Rebuild after changing a scene or a component. The script also removes the `*`
`.gitignore` that wasm-pack writes in the output. The bundle is about 12 MB (2.9 MB
gzipped); browsers cache it, so every demo after the first starts quickly.

## Compile-check the examples

Complete programs are marked with `<!-- compile -->` before the fence. The script extracts
them into a temporary crate that depends on the repository and runs `cargo check`:

```bash
npm run check-examples
```

Short snippets are not marked; they intentionally live inside a `fn ui(ui: &mut egui::Ui)`
or a model struct and are not standalone programs.

## Rust API docs

The site documents the shell; rustdoc documents every public item:

```bash
cd ..
cargo doc --workspace --no-deps --open
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

`cargo xtask check` builds the API docs with warnings denied.

## Writing rules

- Verify every type, method, field, and default against `src/` before writing it.
- Every page has at least one code example; visual pages have a `Preview` screenshot.
- Use `Preview` for light/dark pairs, `ApiTable` for builder APIs, and `TokenSwatch` for
  theme tokens.
- Link each type to its page or to `reference/api-map.md`.
- Keep terminology consistent: `Command`, token, builder, tone, frame, view.
- Do not invent API. If the source does not have it, document what it has.
