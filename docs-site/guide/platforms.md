---
title: Desktop platforms
description: Where each check runs, the repeatable IME and HiDPI pass, and the egui upgrade policy.
---

# Desktop platforms

The 0.3.x hardening release makes the crate solid on the three desktops before the 0.4
work builds on it.

## Where each check runs

| Check | Where | What runs |
|---|---|---|
| `cargo xtask check` | Locally, before every tag | Format, style lint, clippy, rustdoc, the full test suite (including pixel snapshots and `ime_hidpi`), template check |
| `CI` workflow | Ubuntu, manual (`workflow_dispatch`) | Format, library tests, `cargo audit`, `cargo deny check`, agent skill |
| `Release` workflow | Ubuntu, macOS, Windows, on a `v*` tag | Release gate on Ubuntu; macOS and Windows build the Python wheels |
| `Platform pixels` | Windows and macOS runners, manual | Native rendering and hit-testing probe with screenshots |

CI does not run the full test suite and has no automated macOS or Windows test job. Pixel
snapshots stay pinned to the Linux reference. The macOS and Windows IME/HiDPI pass is
still open; see "Known gaps at 1.0" in the [roadmap](/reference/roadmap).

## IME and HiDPI

The manual pass is written down in the repository at
[`docs/PLATFORMS.md`](https://github.com/jigonzalez930209/egui-workbench/blob/main/docs/PLATFORMS.md).
It covers dead-key accents, CJK composition and cancellation, mixed text, and a 200 %
scale factor, plus resizing the window across displays.

The automated half is `tests/ime_hidpi.rs`, which runs locally on Linux before each tag:

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
