# Contributing

## Setup

The toolchain is pinned in `rust-toolchain.toml` (Rust 1.97.1, rustfmt, clippy). egui
and eframe come from crates.io (0.37); there are no patches.

```bash
cargo xtask check     # fmt --check, style-lint, clippy -D warnings, docs, test
cargo xtask template-check
cargo run --example showcase -- .
```

The VitePress guide is in `docs-site/`:

```bash
cd docs-site
npm install
npm run dev
npm run build
npm run check-examples
```

Rustdoc is part of `cargo xtask check`; use
`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` to reproduce it alone.

Rebuild the live wasm catalog after changing it or the components it uses:

```bash
cd docs-site
npm run build:demo
```

## Rules

- Keep `src/foundation` free of panels and application state; keep `src/chrome` and
  `src/widgets` free of application state. `src/app` may coordinate, but performs no
  I/O and picks no storage format.
- Components return events or commands. State changes belong to the application's one
  `handle` function.
- New public controls belong in `src/components`: a builder with shared `Size`,
  `ButtonVariant`, or `Tone`, taking `&mut` state and returning the change. Keep the
  matching function in `widgets` when the shell still needs it.
- Colors come from `theme` tokens or `ui.visuals()`; sizes come from `metrics`. The two
  documented exceptions are the red close-button hover and white icon tint.
- Every icon-only button needs a tooltip and an AccessKit name. Every new input needs a
  keyboard path.
- Each source file stays under 500 lines. Split by component, not by line count.
  `tests/line_limit.rs` fails the suite if any tracked Rust source exceeds it.
- Public items need doc comments (`missing_docs = "deny"`). `unwrap`, `expect`,
  `panic`, `todo`, `unimplemented`, and `unreachable` are denied in library code.
- Write strings in English; translate at the call site. Keep `…` in dialog labels.

## Tests

- Unit tests live next to the code for pure logic (theme contrast, palette filtering,
  tree flattening, document index math).
- Component tests live in `tests/` and drive widgets through `egui_kittest` by
  AccessKit role and name. Assert both the returned event and the geometry.
- Snapshots live in `tests/snapshots/`. Refresh them only for intended visual changes:

```bash
UPDATE_SNAPSHOTS=1 cargo test --test snapshots
```

Review every changed PNG before committing. A snapshot update without a visual change
is a bug in the test.

## Adding or changing a component

1. Choose the layer (see `docs/ARCHITECTURE.md`).
2. Reuse `metrics`, `components::Size`/`Tone`, and `theme`; add a token or size instead of
   a literal. Take radii, borders, and fonts from `style::get(ui.ctx())` (`control()`,
   `inner()`, `container()`, `border(color)`, `font(points)`), never from numbers:
   `cargo xtask style-lint` fails otherwise. Mark a deliberately fixed value with
   `// style: fixed`.
3. Return an event, `bool`, `Option<T>`, or `Response`.
4. Add AccessKit metadata and a tooltip.
5. Add or update tests; add a snapshot if it is visible chrome.
6. Add a section to `examples/catalog` and a row to `docs/COMPONENTS.md`. If the docs
   page should show it live, add or extend a scene in `examples/scenes` and embed it with
   `<LiveDemo>` (see `docs-site/README.md`), then refresh the reference frames with
   `UPDATE_SNAPSHOTS=1 cargo test --test doc_scenes` and the bundle with
   `npm run build:demo`.
7. Re-export it from `src/components/mod.rs` and the prelude; update `WORKBENCH.md` §13.1.
8. Update `README.md` if the new component changes the starter application.

## Updating egui

egui and eframe are pinned to one minor version (`egui = "=0.37"`, `eframe = "0.37"`,
`egui_kittest = "0.37"`). A new egui minor is a breaking release of this crate: bump all
three, then

```bash
cargo update -p egui
cargo xtask check
cargo xtask template-check
```

Review every changed snapshot, and bump this crate's minor version in `Cargo.toml` and
`CHANGELOG.md`.

## Releasing

1. Update `CHANGELOG.md` (move *Unreleased* under the new version) and `version` in
   `Cargo.toml`.
2. `cargo xtask check`, `cargo xtask template-check`, and `cargo publish --dry-run`.
3. Commit, tag `vX.Y.Z`, push the tag, then `cargo publish`.

## Commits

Small, focused commits. State the behavior change and the reason. Do not mix a
snapshot-only change with a logic change.
