#!/usr/bin/env bash
# Compile-check the complete Rust examples marked with <!-- compile --> in the docs.
#
# Each marked block becomes a binary in a temporary crate that depends on this repository.
# Short snippets are intentionally not marked; they are not standalone programs.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SITE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/egui-workbench-docs.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

python3 - "$SITE" "$WORK" <<'PY'
import re
import sys
from pathlib import Path

site = Path(sys.argv[1])
work = Path(sys.argv[2])
out = work / "src" / "bin"
out.mkdir(parents=True)

pattern = re.compile(
    r"<!--\s*compile\s*-->\s*\n```rust[^\n]*\n(.*?)\n```",
    re.DOTALL,
)

count = 0
# The agent skill's SKILL.md carries its own program; its references are copies of these pages.
skill_pages = sorted((site.parent / "skills").glob("*/SKILL.md"))
for page in sorted(site.rglob("*.md")) + skill_pages:
    if "node_modules" in page.parts or ".vitepress" in page.parts:
        continue
    text = page.read_text(encoding="utf-8")
    for index, match in enumerate(pattern.finditer(text), start=1):
        count += 1
        stem = page.parent.name if page.name == "SKILL.md" else page.stem
        name = re.sub(r"[^a-z0-9]+", "_", stem.lower()).strip("_")
        path = out / f"{name}_{index}.rs"
        path.write_text(match.group(1) + "\n", encoding="utf-8")
        print(f"  {path.name} <- {page.relative_to(site.parent)}")

if count == 0:
    raise SystemExit("no <!-- compile --> blocks found")
print(f"extracted {count} example(s)")
PY

{
  printf '[package]\nname = "egui-workbench-docs-examples"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n\n'
  printf '[workspace]\n\n'
  printf '[dependencies]\negui-workbench = { path = "%s" }\n' "$ROOT"
} > "$WORK/Cargo.toml"

cd "$WORK"
CARGO_TARGET_DIR="$ROOT/target/docs-examples" cargo check --bins
printf 'doc examples compile\n'
