# Desktop platforms

> The Linux, macOS, and Windows CI matrix, the repeatable IME and HiDPI pass, and the egui upgrade policy. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/platforms.html>

The 0.3.x hardening release makes the crate solid on the three desktops before the 0.4
work builds on it.

## CI matrix

| Job | Runner | What runs |
|---|---|---|
| `check` | Ubuntu | Format, style lint, clippy, rustdoc, the full test suite (including pixel snapshots), benchmarks compile, template check, package, agent skill |
| `desktop` | macOS, Windows | Unit tests and every behavior test, including `ime_hidpi`, the chrome, tabs, controls, dialogs, tree, viewer, and all plot families |
| `msrv` | Ubuntu | `cargo +1.95 check --lib --all-features --locked` |
| `semver` | Ubuntu | `cargo-semver-checks` against the previous release tag |

Pixel snapshots stay pinned to the Linux reference. The other two desktops run behavior
and interaction tests, so a platform-specific chrome or input regression fails before a
release, without re-baselining fonts on three systems.

## IME and HiDPI

The manual pass is written down in the repository at
[`docs/PLATFORMS.md`](https://github.com/jigonzalez930209/egui-workbench/blob/main/docs/PLATFORMS.md).
It covers dead-key accents, CJK composition and cancellation, mixed text, and a 200 %
scale factor, plus resizing the window across displays.

The automated half is `tests/ime_hidpi.rs`, which runs on all three desktops:

- IME pre-edit followed by a commit inserts the composed character.
- A 2× pixels-per-point frame keeps the text field usable and its accessible name
  intact.

## egui upgrades

egui types appear in the public API, so every egui minor is a workbench release. The
policy is written in
[`docs/EGUI_UPGRADE.md`](https://github.com/jigonzalez930209/egui-workbench/blob/main/docs/EGUI_UPGRADE.md):

- one egui minor per workbench minor while 0.x;
- the upgrade is its own release, with a Migration section and the full platform matrix;
- at 1.0, a 1.x line supports one egui minor, and a new egui major only changes the
  egui-dependent crates, not the core or the protocol.

## Benchmarks

The recorded baseline lives in
[`docs/BENCHMARKS.md`](https://github.com/jigonzalez930209/egui-workbench/blob/main/docs/BENCHMARKS.md).
The weekly `Benchmarks` workflow runs the same four suites and uploads the criterion
artifact.
