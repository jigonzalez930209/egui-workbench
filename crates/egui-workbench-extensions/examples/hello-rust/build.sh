#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
cd "$DIR"
RUSTFLAGS="-C link-arg=--allow-undefined" cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/hello_extension.wasm "$DIR/hello_extension.wasm"
