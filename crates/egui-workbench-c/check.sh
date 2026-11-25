#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/crates/egui-workbench-c/Cargo.toml" | head -n 1)"
cargo build -p egui-workbench-c
cc "$ROOT/crates/egui-workbench-c/tests/c_abi.c" \
  -I "$ROOT/crates/egui-workbench-c/include" \
  -DWB_VERSION="\"$VERSION\"" \
  -L "$ROOT/target/debug" -legui_workbench_c \
  -Wl,-rpath,"$ROOT/target/debug" \
  -o /tmp/egui-workbench-c-abi
/tmp/egui-workbench-c-abi
