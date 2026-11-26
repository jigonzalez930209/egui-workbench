# Versioning and support

## Crates and packages

Every published crate and package follows Semantic Versioning:

- `egui-workbench`, `egui-workbench-host`, `egui-workbench-agent`,
  `egui-workbench-remote`, `egui-workbench-extensions`, the chemistry plugins
  (`egui-workbench-chem-core`, `egui-workbench-chem2d`, and `egui-workbench-mol3d`), and the
  Python and npm packages share the release version (`1.2.x`). The chemistry plugins were
  unpublished 0.1.0 workspace members before 1.2.0.
- A major version changes when a public Rust API, the host spec, the protocol, or the
  extension ABI breaks incompatibly.
- A minor version adds compatible API or features.
- A patch version fixes behavior without adding API.

The 0.x series could break between minors and said so in each changelog. From 1.0, the
guarantee above applies.

## egui policy

`egui-workbench` 1.0 supports egui/eframe 0.37 and one following egui minor after its
release, through a compatible `egui-workbench` patch or minor. We do not pin egui patch
releases; applications may choose any 0.37.x compatible patch. An egui API break that
cannot be absorbed compatibly requires a new `egui-workbench` minor.

## Rust version

The minimum supported Rust version is 1.95, stated in `rust-version` and checked before each
tag by `cargo xtask check`.
The policy is the current stable toolchain and 1.95 for the 1.x line; raising it requires
a minor release and a changelog note.

## Protocol, spec, and extension interface

These are versioned separately from the crate:

| Surface | Version | Rule |
|---|---|---|
| Host spec | `1` | A host rejects a newer spec with an error. |
| Host protocol | `1` | A host rejects a newer protocol version. |
| Extension ABI | `0.15` | A manifest names its supported range in `api`. |
| Python wheel | abi3 CPython 3.9+ | ABI stable across 3.x. |
| npm package | ESM | TypeScript declarations generated from the spec schema. |

A protocol or spec change that cannot be understood by an older host requires a major
surface version; the crate major can stay unchanged when both versions are carried.

## Releases

Pushing a tag `vX.Y.Z` runs one workflow: basic tests, docs, npm, crates, Linux x86_64 and
aarch64 wheels, macOS universal2, Windows amd64, PyPI, npm, and the GitHub release. Strict
checks run locally before the tag; the manual CI workflow is a one-OS smoke test.

## Support window

The latest 1.x minor receives fixes. Security fixes are reported through GitHub Security
Advisories; see `SECURITY.md`.
