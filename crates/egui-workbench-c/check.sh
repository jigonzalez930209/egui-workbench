#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cargo build -p egui-workbench-c
cc "$ROOT/crates/egui-workbench-c/tests/c_abi.c" \
  -I "$ROOT/crates/egui-workbench-c/include" \
  -L "$ROOT/target/debug" -legui_workbench_c \
  -Wl,-rpath,"$ROOT/target/debug" \
  -o /tmp/egui-workbench-c-abi
/tmp/egui-workbench-c-abi
