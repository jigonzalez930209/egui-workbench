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

## Standalone crates

Three crates declare their own `[workspace]` and keep a separate `Cargo.lock`:
`crates/egui-workbench-wasm`, `web`, and `crates/egui-workbench-platform-probe`. This is
intentional:

- `egui-workbench-wasm` is built by `wasm-pack` for `wasm32-unknown-unknown`; keeping it
  apart lets it pin the wasm-bindgen version that matches the npm toolchain without
  changing the native build.
- `web` is the documentation demo and is built only for the docs site.
- `egui-workbench-platform-probe` must not inherit the root `egui_kittest`, `criterion`,
  and wgpu dev-dependencies; that inheritance added hours to the hosted Windows probe.

Each lockfile is refreshed with `cargo update` inside its own directory. The native
workspace does not build them.

## Fuzzing

`fuzz/` holds libFuzzer targets for the CSV reader, the workbook reader, and the host spec
reader. Run each target for at least 60 seconds before changing a reader:

```bash
cargo +nightly fuzz run csv -- -max_total_time=60
cargo +nightly fuzz run workbook -- -max_total_time=60
cargo +nightly fuzz run spec -- -max_total_time=60
```

The corpus and seeds are committed; keep a crashing input in `fuzz/artifacts/` in the fix
commit so the regression is reproducible.

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

## Pre-push hook

`rustfmt` and `clippy` run in `cargo xtask check` and the release gate. Enable the
included hook so a release tag never fails on lint:

```bash
git config core.hooksPath .githooks
```

## Releasing

Releases are continuous delivery from tags: the `Release` workflow runs
`cargo xtask check`, publishes to crates.io with the `CARGO_REGISTRY_TOKEN` repository
secret, and creates the GitHub release with that version's changelog section.

1. Move the *Unreleased* entries of `CHANGELOG.md` under a new `## [X.Y.Z] - date`
   heading (and update the compare links at the bottom); set `version` in `Cargo.toml`
   and in `.claude-plugin/plugin.json`.
2. Locally: `cargo xtask check`, `cargo xtask template-check`, and
   `cargo publish --dry-run`.
3. Commit and push, then tag: `git tag vX.Y.Z && git push origin vX.Y.Z`.

The workflow stops before publishing if the tag does not match `Cargo.toml` or
`plugin.json`, the changelog has no section for the version, the agent skill is stale,
the secret is missing, or a check fails. The release carries `egui-workbench-skill.zip`.

One-time setup: create a crates.io API token (Account Settings → API Tokens) with the
`publish-new` and `publish-update` scopes, then store it as a repository secret:
`gh secret set CARGO_REGISTRY_TOKEN`.

## Agent skill

`skills/egui-workbench/` teaches AI agents to use the crate. `SKILL.md` is written by
hand; `references/` and the index at the end of `SKILL.md` are generated from
`docs-site/`. After changing a docs page or the public API:

```bash
python3 docs-site/scripts/build-skill.py   # regenerate references/
bash docs-site/scripts/check-examples.sh    # the SKILL.md program and docs programs compile
claude plugin validate .                    # optional: the plugin and marketplace manifests
```

`cargo xtask check` runs the first two and fails on a stale skill.

## Commits

Small, focused commits. State the behavior change and the reason. Do not mix a
snapshot-only change with a logic change.

## Build size and speed

The workspace profiles in `Cargo.toml` keep `target/` small. A default debug build of
every crate with every feature wrote more than 150 GB, because each test binary links the
whole dependency tree with full debug info; with the profiles it is about 8 GB and a clean
`cargo test --all-features --workspace --no-run` takes about 90 seconds on 16 cores.

- `[profile.dev] debug = "line-tables-only"` keeps file and line numbers in backtraces and
  panics but drops local-variable info. If you need a debugger with variables, build one
  crate with `CARGO_PROFILE_DEV_DEBUG=full cargo test -p <crate>` into a separate
  `CARGO_TARGET_DIR`.
- Dependencies build without debug info. The rendering and decoding crates (`tiny-skia`,
  `resvg`, `usvg`, `png`, `miniz_oxide`, `fdeflate`) and `wasmi` run at `opt-level = 1`
  so the snapshot and extension tests do not crawl.
- `.cargo/config.toml` sets `build.jobs = 6`; a full-parallel build has been killed by the
  out-of-memory killer while linking tests. Raise it with `CARGO_BUILD_JOBS` if you have
  the memory.
- Mixing `--all-features`, `--workspace`, `check`, `clippy`, and `test` writes a separate
  set of artifacts for each. Pick one combination for day-to-day work and run
  `cargo clean` now and then.

### Release builds

A clean `cargo build --release` of the C library, host, remote, and agent crates takes about
100 s with the default `build.jobs = 6` and about 73 s with `CARGO_BUILD_JOBS=12` (16 cores,
27 GB of memory). The limit of six protects the test links; a release build is lighter, so
raise it for release work. `[profile.release] strip = "symbols"` shrinks the shipped binaries
from 22 MB to 16 MB without changing build time; the `wb_*` C symbols stay exported.
`opt-level = 2` built no faster than 3 in these measurements, so it was not adopted.

### Linker

`mold` 2.40.4 was measured against the `lld` that `rustc` 1.97 already uses on Linux, and it
was not adopted: a clean release build took 69 s with `lld` and 70 s with `mold`; a clean
test build (`cargo test --all-features --workspace --no-run`) took 97 s with either, and
after editing the root crate the rebuild took 34 s with `lld` and 46 s with `mold`. The
default stays.

### Release workflow

`release.yml` runs its independent jobs in parallel (version gate, checks, docs and npm,
crates, wheels per OS and architecture) and `publish` waits for all of them. Linux aarch64
and macOS x86_64 wheels build on native runners, which are free for this public repository,
instead of cross-compiling or building `universal2` in series. The last release before this
change took 26 min, with `macos universal2` (26 min), the Linux wheels (17 min), and the
Windows wheel (16 min) as the longest jobs.

`cache-warm.yml` calls the release workflow on `main` every Monday so a tag restores a
warm Rust cache; only `main` saves it. Run **Release** with `workflow_dispatch` to rehearse
every build without publishing.
