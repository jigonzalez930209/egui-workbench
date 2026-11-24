#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cargo build -p egui-workbench-host
cd "$ROOT/clients/go"
${GO:-go} run .
