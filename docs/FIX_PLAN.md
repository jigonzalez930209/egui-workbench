# egui-workbench — Structural analysis and fix plan

As of 2026-11-25, against `main` at 1.0.2.

## Summary

egui-workbench 1.0.2 builds, passes its 439 Rust tests, survives three fuzz targets and is clippy-clean, but it has 41 problems. Four of them should block the next release: the extension sandbox reads files outside its grant (X1), extensions can hang the UI (X2), any web page can drive the remote server (X5), and `cargo generate` builds new apps against 0.4 (S1).

| Area | Problems | Most serious |
| --- | --- | --- |
| Security | 9 | X1 sandbox path escape (critical) |
| Docs and release | 11 | D9 npm publish can fail silently |
| Tests and CI | 8 | T2 conformance clients never run |
| Workspace structure | 7 | S1 template pinned to 0.4 |
| Code quality | 6 | Q3 a panic aborts any C host |

Scope: every crate in the workspace, the release and CI workflows, the template, and the docs, checked against the code on `main` at 1.0.2. Method: reading the code, running `cargo clippy`, `cargo test --workspace`, `cargo +1.95 check`, `cargo deny`, and 60 s of fuzzing per reader.

## Workspace and crate structure

The workspace is consistent at 1.0.2 across crates.io, npm and PyPI, but the starter template and the install guide still point new users at 0.4.

| ID | Problem | Where | Effect |
| --- | --- | --- | --- |
| S1 | The cargo-generate template depends on `version = "0.4"` when it is not generated from a checkout | `templates/app/Cargo.toml:17`, `xtask/src/main.rs:126` | `cargo generate` from git builds a new app against 0.4.x, which does not have half the API the template uses. The roadmap's 0.3.x promise of `cargo generate` without a checkout is broken. |
| S2 | The install guide shows `egui-workbench = { version = "0.4", … }` | `docs-site/guide/installation.md:107` (and the skill copy) | Anyone copying the snippet gets 0.4. |
| S3 | Three crates sit outside the workspace with their own `[workspace]` and `Cargo.lock`: `egui-workbench-wasm`, `web`, `egui-workbench-platform-probe` | `crates/egui-workbench-wasm`, `web`, `crates/egui-workbench-platform-probe` | Three lockfiles drift apart. The probe already has a 2.9 GB `target/` of its own, outside the shared `target/`. |
| S4 | `web/Cargo.toml` is still version `0.1.0` | `web/Cargo.toml:3` | Not published, but misleading next to 1.0.2 everywhere else. |
| S5 | `egui-workbench-c` and `egui-workbench-python` are `publish = false`, yet the C ABI is part of the 1.0 promise | `crates/egui-workbench-c/Cargo.toml` | Not a bug, but the release text should say the C ABI ships only as a GitHub release archive. |
| S6 | Old npm tarballs (0.13 to 0.16) lie in the wasm crate | `crates/egui-workbench-wasm/*.tgz` | Ignored by git, but they confuse `dist/*.tgz` globs run locally. |
| S7 | No egui-free core crate (roadmap Decision 1) | workspace | Every egui minor is a breaking major for every crate, including host, remote and agent, whose protocol types do not need egui. |

## Code quality

The library code is clean: `cargo clippy --workspace --all-targets --all-features` reports nothing, and production code has almost no `unwrap`/`expect`/`panic!`. The problems are in rules that are stated but not enforced, and in the C ABI.

| ID | Problem | Where | Effect |
| --- | --- | --- | --- |
| Q1 | Two files break the 500-line rule, and the guard test does not look at them | `crates/egui-workbench-extensions/src/runtime.rs` (711 lines), `crates/egui-workbench-host/src/runtime.rs` (596); `tests/line_limit.rs` only scans `src`, `tests`, `examples`, `templates`, `xtask`, `web/src` | `WORKBENCH.md` and the roadmap promise "files under 500 lines", and `crates/` silently escapes it. |
| Q2 | `wb_host_version()` returns a hard-coded `"0.13.0"` | `crates/egui-workbench-c/src/lib.rs:221` | A C client checking the ABI version at 1.0.2 reads 0.13.0. |
| Q3 | No `std::panic::catch_unwind` in any `extern "C"` function | `crates/egui-workbench-c/src/lib.rs` (every `wb_*`) | A panic inside the host (a bad patch, a renderer error) aborts the whole C, Go or Python-via-C process instead of returning `-1` and setting `wb_last_error`. |
| Q4 | Input strings go through `to_string_lossy` | `crates/egui-workbench-c/src/lib.rs:44`, `:88` | Invalid UTF-8 is silently replaced by U+FFFD instead of being rejected with an error. |
| Q5 | An `expect("extension exists")` sits after a call that can remove the extension on error paths | `crates/egui-workbench-extensions/src/runtime.rs:227` | Low risk today; replace it with `ok_or(ExtensionError::Unknown)` so a future change cannot panic the host. |
| Q6 | `clippy` and `rustfmt` run only in `cargo xtask check` and the release gate | `xtask/src/main.rs`, `.github/workflows/ci.yml` | Fine by the CI decision, but worth a pre-push hook so a release tag never fails on lint. |

## Tests and CI

The Rust suite is green: `cargo test --workspace` passed 439 tests with 0 failures on 2026-11-25, and the library builds on Rust 1.95 with `--all-features --locked`. The gaps are the tests that nothing runs at all, not tests that fail.

Fuzzing, 60 s per target on nightly: `csv` 1.03 M runs, `workbook` 430 k runs, `spec` finished, with no crash in any of them. The targets live in `fuzz/` and are not committed yet.

| ID | Problem | Where | Effect |
| --- | --- | --- | --- |
| T1 | Both performance-budget tests are `#[ignore]` and nothing runs them with `--ignored --release` | `tests/data_budget.rs:37`, `tests/stream_budget.rs:12` | The 0.7 promise (a 1 M row CSV inside the frame budget) and the 0.9 streaming budget are never checked, not even locally. |
| T2 | `cargo xtask check` does not run the non-Rust tests | `xtask/src/main.rs:27-32` | `crates/egui-workbench-python/tests/test_workbench.py`, `crates/egui-workbench-c/tests/c_abi.c`, `clients/go/check.sh` and the wasm `wasm_bindgen_test` suite only run when someone remembers them. These are the conformance clients behind the 0.12 and 0.13 exit criteria. |
| T3 | `xtask check` does not run `template-check`, `check-examples.sh`, `cargo deny`, or `cargo semver-checks` | `xtask/src/main.rs` | S1 (template on 0.4) slipped through for exactly this reason. |
| T4 | No MSRV check anywhere | — | It builds on 1.95 today, but the next dependency bump can break it silently. Add `cargo +1.95 check --lib --all-features --locked` to `xtask check`. |
| T5 | The line-limit guard skips `crates/` | `tests/line_limit.rs:16` | See Q1. |
| T6 | No tests for the security paths | `crates/egui-workbench-extensions/tests`, `crates/egui-workbench-remote` | No test tries `..`, a sibling prefix, or a symlink against `host_read_file` (X1), an infinite loop (X2), or a cross-origin handshake (X5). |
| T7 | The fuzz targets are not committed and not documented | `fuzz/` | Commit them and add `cargo +nightly fuzz run <target> -- -max_total_time=60` to `CONTRIBUTING.md`. |
| T8 | Snapshots are pinned to Linux; macOS and Windows have only the manual `Platform pixels` probe | `.github/workflows/platform-pixels.yml` | Accepted by the CI decision; listed so it stays a conscious choice. |

## Security

The extension sandbox can read files outside its grant, which breaks the 0.15 exit criterion "cannot read a file it was not granted". This is the most serious problem in the project.

| ID | Severity | Problem | Where | Effect |
| --- | --- | --- | --- | --- |
| X1 | Critical | `host_read_file` checks the grant with a string `path.starts_with(prefix)`, on a path that is neither normalized nor canonicalized | `crates/egui-workbench-extensions/src/runtime.rs:545` | With grant `/data/`, the path `/data/../home/user/.ssh/id_rsa` passes. With grant `/data`, `/data-private/x` passes too. A symlink inside the granted directory also escapes. |
| X2 | High | No CPU or memory limit on extensions: no fuel, no epoch interruption, no `StoreLimits` | `crates/egui-workbench-extensions/src/runtime.rs` (`Engine`/`Store` setup around `:440`) | An extension with `loop {}` in `on_command` freezes the UI thread forever. `memory.grow` can reach 4 GiB. |
| X3 | Medium | `host_read_file` reads the whole file before it checks `out_cap` | `runtime.rs:549-553` | A granted extension can make the host load a multi-GB file into RAM. Check `metadata().len()` first. |
| X4 | Medium | The manifest `entry` check only rejects `..` and a leading `/` | `crates/egui-workbench-extensions/src/manifest.rs:161` | On Windows, `C:\x.wasm` and `\\server\share\x.wasm` pass. Use `Path::components()` and reject anything that is not `Component::Normal`. |
| X5 | High | The WebSocket server does not check the `Origin` header | `crates/egui-workbench-remote/src/lib.rs` `handshake` (`:254`) | Any web page open in the user's browser can connect to `ws://127.0.0.1:<port>` and drive the host (cross-site WebSocket hijacking). Binding to localhost does not prevent this. |
| X6 | Medium | Handshake headers are read with an unbounded `read_line`, and every connection gets its own thread with no limit | `remote/src/lib.rs:257`, `:77` | One client sending a header without a newline, or many open sockets, exhausts memory or threads. |
| X7 | Medium | With `--read-write` and no `--confirm`, `destructive` commands run with no confirmation | `crates/egui-workbench-agent/src/bin/*.rs:52` | The threat model assumes destructive commands are gated; the CLI default leaves them open. |
| X8 | Low | The confirmation token is passed as a command-line argument | same binary, `--confirm <token>` | Visible to other local users through `ps`. Read it from an environment variable or a file instead. |
| X9 | Low | `chrono` and `serde_yaml`/`unsafe-libyaml` are unmaintained (ignored in `deny.toml`) | `Cargo.toml:78`, `:90` | No known vulnerability, but no fixes will come either. |

The frame readers are fine: `read_frame` and the WebSocket reader both cap a frame at 64 MiB before they allocate.

## Docs and release

Seven documentation pages still promise CI checks that do not exist, and the release gate is weaker than `cargo xtask check`.

| ID | Problem | Where | Fix |
| --- | --- | --- | --- |
| D1 | "MSRV 1.95 … checked in CI" | `VERSIONING.md:27`, `docs-site/reference/versioning.md:23` | Add an MSRV step (see T4) or say "checked before each tag". |
| D2 | "compiled in CI" for the `SKILL.md` program and the guide programs | `docs-site/guide/ai-agents.md:68`, `CONTRIBUTING.md:136` | CI runs only `build-skill.py --check`, not `check-examples.sh`. Add it to `xtask check` and fix the text. |
| D3 | "the 0.14 tests run it in CI" (accessibility audit) | `docs-site/guide/theme-and-global.md:88` | It runs locally in `cargo test`. Change the text. |
| D4 | "the `gpu` feature is exercised in CI" | `docs-site/reference/benchmarks.md:66` | Same: local only. |
| D5 | "`cargo-semver-checks` against the previous release" | `docs/EGUI_UPGRADE.md:16` | It runs nowhere. Add `cargo semver-checks` to `xtask check` or drop the claim. |
| D6 | The install guide and the template point to 0.4 | see S1, S2 | Use `"1"`. |
| D7 | The release gate runs only `cargo test --workspace --lib`, with no clippy, no rustdoc, and no integration tests | `.github/workflows/release.yml:66-71` | A tag can publish code that `cargo xtask check` would reject if the local run was skipped. At least add `clippy -D warnings`; it is cheap. |
| D8 | The release checks the version of the Rust crates and `plugin.json`, but not `crates/egui-workbench-wasm/package.json`, `pyproject.toml`, or `wb_host_version()` | `release.yml:55-64` | Add them to the loop; Q2 is exactly the drift this misses. |
| D9 | npm publish has `continue-on-error: true` | `release.yml` publish job | A failed npm publish leaves a GitHub release that claims an npm version that does not exist (a failed "Repair registry publication" run already exists in the history). Fail loudly instead, and make the step idempotent with `npm view`. |
| D10 | Pages deployment has `continue-on-error: true` | `release.yml` docs job | The docs can silently lag the release. |
| D11 | The new C ABI jobs for macOS and Windows have never run | `release.yml` | Run them once by hand (`workflow_dispatch` on a branch) before the next tag. |

## Fix plan

Fix in three batches: P0 ships as 1.0.3 as soon as it is done, P1 in the same patch if it fits, P2 in 1.1. Every item names the check that proves it is fixed.

### P0 — security and broken installs (1.0.3)

- [x] **X1** In `host_read_file`, canonicalize both the path and each granted prefix with `std::fs::canonicalize`, then compare with `Path::starts_with` (component-wise, not string). Reject a path that does not exist or cannot be canonicalized. *Verify:* new tests in `crates/egui-workbench-extensions/tests` for `/grant/../secret`, `/grant-sibling/x`, and a symlink inside the grant pointing outside it; all three return `-3`.
- [x] **X3** Before reading, call `std::fs::metadata(&path)?.len()` and return `-5` when it exceeds `out_cap`. *Verify:* a test with `out_cap = 4` on a 1 MiB granted file returns `-5` without allocating.
- [x] **X2** Turn on `Config::epoch_interruption(true)`, set a deadline before each `init`/`on_command` call, and tick the epoch from a timer thread; add `StoreLimitsBuilder::new().memory_size(64 << 20)` through `Store::limiter`. *Verify:* a test extension with `loop {}` returns `ExtensionError::Runtime` within 1 s; one that grows memory past 64 MiB fails.
- [x] **X4** Validate `entry` with `Path::new(&entry).components().all(|c| matches!(c, Component::Normal(_)))`. *Verify:* unit tests for `C:\x.wasm`, `\\srv\x.wasm`, `a/../b.wasm` and `/x.wasm`.
- [x] **X5** In `handshake`, read `Origin` and accept only an absent Origin (native clients) or one in an allow-list given to `RemoteServer` (default empty). *Verify:* a test sending `Origin: https://evil.example` gets `403` and no host access.
- [x] **X6** Cap the handshake at 8 KiB and 64 header lines (`reader.take(8192)`), and cap concurrent connections (for example 32) with an `AtomicUsize`. *Verify:* a test sending 16 KiB without a newline gets an error, not growing memory.
- [x] **S1, S2, D6** Replace `version = "0.4"` with `version = "1"` in `templates/app/Cargo.toml:17`, `xtask/src/main.rs:126` and `docs-site/guide/installation.md:107`; regenerate the skill. *Verify:* `cargo generate --git <repo> templates/app` in a temp directory, then `cargo build` resolves 1.0.x.
- [x] **Q2, D8** Return `env!("CARGO_PKG_VERSION")` from `wb_host_version()` (a static `CStr` built with `concat!(env!("CARGO_PKG_VERSION"), "\0")`), and add `crates/egui-workbench-wasm/package.json` and `pyproject.toml` to the release version loop. *Verify:* `c_abi.c` asserts the version equals the crate version.

### P1 — robustness and honest gates

- [x] **Q3** Wrap the body of each `wb_*` function in `std::panic::catch_unwind(AssertUnwindSafe(…))`, setting `wb_last_error` to the panic message and returning `-1`/`NULL`. *Verify:* a Rust test that triggers a panic through the ABI returns `-1`.
- [x] **Q4** Use `CStr::to_str()` and return an error on invalid UTF-8. *Verify:* `c_abi.c` passes `"\xff"` and gets `NULL` plus an error.
- [x] **X7, X8** Make `--read-write` without `--confirm` refuse `destructive` commands; read the token from `EGUI_WORKBENCH_CONFIRM` instead of argv. *Verify:* an MCP test calling a destructive command without confirmation gets `ConfirmationRequired`.
- [x] **Q5** Replace the `expect` at `runtime.rs:227` with `ok_or_else(… ExtensionError::Unknown …)`.
- [x] **D7** Add `cargo clippy --workspace --all-targets -- -D warnings` to the release gate.
- [x] **D9, D10** Remove `continue-on-error` from the npm and Pages steps; skip npm publish only when `npm view egui-workbench@$version` already exists.
- [x] **D11** Run the new macOS/Windows C ABI jobs once by hand before tagging 1.0.3.
- [x] **T2, T3, T4** Extend `cargo xtask check` with `template-check`, `bash docs-site/scripts/check-examples.sh`, `cargo deny check`, `cargo +1.95 check --lib --all-features --locked`, `pytest crates/egui-workbench-python/tests`, the C ABI test, and `clients/go/check.sh`; skip a step with a clear message when its tool is missing. *Verify:* `cargo xtask check` lists each step and passes.
- [x] **D1–D5** Rewrite the seven "in CI" sentences to "before each tag (`cargo xtask check`)", or drop the claim when nothing checks it (semver-checks).
- [x] **T7** Commit `fuzz/` and document it in `CONTRIBUTING.md`.

### P2 — structure (1.1)

- [x] **Q1, T5** Split `extensions/src/runtime.rs` (host functions into `runtime/host_api.rs`) and `host/src/runtime.rs` under 500 lines; add `crates` to the dirs in `tests/line_limit.rs`. *Verify:* the guard test passes with `crates` included.
- [x] **T1** Add `cargo xtask budgets` running `cargo test --release --test data_budget --test stream_budget -- --ignored`, and record the numbers in `docs/BENCHMARKS.md` each release.
- [x] **T6** The security tests from P0 cover this item.
- [x] **S3** Make `egui-workbench-wasm`, `web` and the probe members of the workspace (excluded from default members), or document why each keeps its own lockfile.
- [x] **S4, S6** Set `web` to `publish = false` with version `1.0.2` (or `0.0.0` like the probe); delete the old `.tgz` files.
- [x] **X9** Replace `serde_yaml` with `serde_norway` or `serde_yml`, and `chrono` with `time` or `jiff`; drop the ignores from `deny.toml`.
- [x] **S7** Decide on the egui-free core crate before 2.0, as the roadmap already says.

## Status — 1.0.4 (2026-11-25)

All P0, P1, and P2 items are implemented and verified. Evidence:

- `cargo xtask check` passed: fmt, style-lint, clippy, rustdoc, all workspace tests,
  template check, docs examples, C ABI client, `cargo deny`, and the 1.95 MSRV check.
  Python, Go, semver-checks, and wasm steps skip with a message when their prerequisite
  is not installed.
- `cargo xtask budgets` passed the 1M-row data budget (4.22 s) and the streaming budget
  (0.02 s).
- The `C ABI` workflow ran green on Linux, macOS, and Windows before the tag.
- The `Release` workflow for `v1.0.4` completed successfully and published crates.io,
  npm, PyPI, the docs, and all three C ABI archives.

X9 note: the direct `chrono` and `serde_yaml` dependencies are gone (`jiff` and
`serde_norway` replace them). `chrono` remains a transitive dependency of
`arrow`/`parquet` for temporal arrays, so one advisory ignore stays in `deny.toml` with
that reason.
