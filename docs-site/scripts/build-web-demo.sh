#!/usr/bin/env bash
# Build the component catalog for /guide/live-demo.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

rustup target add wasm32-unknown-unknown >/dev/null

cd "$ROOT/web"
wasm-pack build --target web --release \
  --out-dir "$ROOT/docs-site/public/demo/pkg" \
  --out-name egui_workbench_web

# wasm-pack writes a `*` gitignore in the output; these artifacts are part of the site.
rm -f "$ROOT/docs-site/public/demo/pkg/.gitignore"
