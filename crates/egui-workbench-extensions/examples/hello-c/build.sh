#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LLD="${RUST_LLD:-$(find "$HOME/.rustup/toolchains" -path '*/bin/rust-lld' | head -1)}"
if [ -z "$LLD" ]; then echo "rust-lld not found; set RUST_LLD" >&2; exit 1; fi
clang --target=wasm32 -O2 -nostdlib -c "$DIR/hello.c" -o "$DIR/hello.o"
"$LLD" -flavor wasm --no-entry --export-memory --allow-undefined \
  --export=init --export=on_command "$DIR/hello.o" -o "$DIR/hello.wasm"
rm -f "$DIR/hello.o"
