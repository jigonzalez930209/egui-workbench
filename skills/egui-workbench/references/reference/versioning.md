# Versioning and support

> SemVer for every crate and package, the egui and MSRV policies, and the independent protocol, spec, and extension versions. Live page: <https://jigonzalez930209.github.io/egui-workbench/reference/versioning.html>

Every published crate and package follows Semantic Versioning. `1.0.0` is the first
stable promise; the 0.x series could break between minors.

| Surface | Version | Compatibility |
|---|---|---|
| Crates and packages | `1.0.x` | Major/minor/patch per SemVer |
| Host spec | `1` | Newer spec is rejected |
| Host protocol | `1` | Newer protocol is rejected |
| Extension ABI | `0.15` | Manifest declares the supported range |
| Python wheel | abi3 3.9+ | Stable across CPython 3.x |
| npm package | ESM | Types generated from the schema |

## egui and Rust

1.0 supports egui/eframe 0.37 and one following egui minor; egui patch releases are not
pinned. The MSRV is Rust 1.95, checked before each tag by `cargo xtask check`; raising it
requires a minor release and a changelog note.

## Release flow

A `vX.Y.Z` tag runs one cached workflow: basic tests on one OS, docs, npm, crates, Linux
x86_64/aarch64 wheels, macOS universal2, Windows amd64, PyPI, npm, and the GitHub
release. Strict checks run locally before the tag. CI is manual and one OS.

See `VERSIONING.md` and `SUPPORT.md` in the repository for the full policy.
