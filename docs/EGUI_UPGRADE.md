# egui upgrade policy

egui releases roughly every three months and breaks its API. egui types appear in this
crate's public API, so an egui release is a release of `egui-workbench` too. This is the
policy we follow from 0.3 onward and write down again at 1.0.

## While the crate is 0.x

- `egui-workbench` names one egui minor in `Cargo.toml` (`egui = "0.37"`). A patch
  release of the workbench may move to a later patch of the same egui minor.
- A new egui minor is a new workbench minor: 0.3 uses egui 0.37, 0.4 uses egui 0.38 (for
  example). The workbench never forces an egui minor on applications silently.
- The upgrade is its own release, with its own changelog entry and a **Migration**
  section. It does not share a release with a feature that changes the workbench API.
- The upgrade release runs the full desktop matrix, the MSRV job, the benchmark
  baseline, and `cargo xtask check`. A regression in either the CPU or GPU path blocks
  it.
- `cargo xtask snapshots` is run once on the reference Linux machine. A pixel change is
  reviewed like an API change; font or anti-aliasing noise is called out separately.

## At 1.0

- A 1.x line of the egui-dependent crates supports exactly one egui minor.
- A new egui minor ships as a new **major** of the egui-dependent crates only when egui's
  breaking changes reach our public signatures. The egui-free core
  (`egui-workbench-core`) and the protocol/schema do not change for that reason.
- The core and the protocol version independently of egui. A workbench 1.4 may use a
  newer egui while the protocol stays at 1.x.
- The supported range is stated in the README and in the release notes: for example
  "egui-workbench 1.4 supports egui 0.42; 1.5 supports egui 0.43".

## Checklist for an upgrade

1. Read egui's changelog and migration notes; list every type that reaches our public
   API.
2. Bump `egui`, `eframe`, `egui_kittest`, and the web crate in the same change; run
   `cargo update -p egui` and commit `Cargo.lock`.
3. `cargo xtask check`, `cargo test --workspace --all-features`, and the three-desktop
   matrix.
4. Run the IME and HiDPI pass in [`PLATFORMS.md`](PLATFORMS.md) on the platforms whose
   input or window code changed.
5. Run the benchmark baseline and compare with the previous release.
6. Add the changelog entry with the egui versions before and after and a Migration
   section for every changed signature.
7. Tag the release. The release workflow publishes it and creates the GitHub release.
