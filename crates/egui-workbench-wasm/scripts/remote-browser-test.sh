#!/usr/bin/env bash
# Build the wasm host, start the remote server, and prove that a real browser renders
# the spec, applies a remote patch, and reports the patched tree.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CHROME="${CHROME:-google-chrome}"
WASM="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$(mktemp /tmp/egui-workbench-remote-XXXX.json)"
trap 'kill ${REMOTE_PID:-} ${HTTP_PID:-} 2>/dev/null || true; rm -f "$SPEC"' EXIT

(cd "$ROOT" && cargo build -p egui-workbench-host -p egui-workbench-remote)
(cd "$WASM" && wasm-pack build --target web --release --out-dir pkg --out-name egui_workbench_wasm >/dev/null)
(cd "$WASM" && node scripts/clean-pkg.mjs)

cat > "$SPEC" <<'JSON'
{"version":1,"widgets":[{"kind":"label","text":"before","size":13.0,"tone":"default","strong":false}]}
JSON
"$ROOT/target/debug/egui-workbench-remote" --spec "$SPEC" --addr 127.0.0.1:31339 >/tmp/egui-workbench-remote.log 2>&1 &
REMOTE_PID=$!
(cd "$WASM" && python3 -m http.server 8011 >/tmp/egui-workbench-http.log 2>&1) &
HTTP_PID=$!
sleep 1

"$CHROME" --headless=new --no-sandbox --disable-gpu --enable-unsafe-swiftshader \
  --dump-dom --virtual-time-budget=8000 http://127.0.0.1:8011/tests/remote.html >/tmp/egui-workbench-dom.html 2>/tmp/egui-workbench-chrome.log
grep -q 'data-ready="yes"' /tmp/egui-workbench-dom.html
grep -q 'browser + remote' /tmp/egui-workbench-dom.html
echo "remote browser ok"
