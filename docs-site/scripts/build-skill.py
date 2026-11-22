#!/usr/bin/env python3
"""Generate the agent skill's reference pages from the documentation site.

The skill in `skills/egui-workbench/` ships the guide as plain Markdown so an AI agent can
read it offline. This script converts each VitePress page (frontmatter, Vue components,
containers, site links) into portable Markdown under `references/`, and rewrites the
reference index inside `SKILL.md`.

    python3 docs-site/scripts/build-skill.py           # regenerate
    python3 docs-site/scripts/build-skill.py --check   # fail if the skill is stale (CI)
"""

from __future__ import annotations

import html
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SITE = ROOT / "docs-site"
SKILL = ROOT / "skills" / "egui-workbench"
REFS = SKILL / "references"
SITE_URL = "https://jigonzalez930209.github.io/egui-workbench"

# Sections in reading order, and pages that only make sense on the website or for
# contributors to this repository.
SECTIONS = ["guide", "app", "components", "charts", "theming", "recipes", "reference"]
SKIP = {
    "guide/live-demo",  # how the website embeds wasm demos
    "guide/ai-agents",  # how to install this skill
    "reference/changelog",
    "reference/roadmap",
    "reference/xtask",  # repository maintenance commands
}
TITLES = {
    "guide": "Guide",
    "app": "Application runtime",
    "components": "Components",
    "charts": "Charts",
    "theming": "Theming",
    "recipes": "Recipes",
    "reference": "Reference",
}

START = "<!-- references:start -->"
END = "<!-- references:end -->"


def pages() -> list[tuple[str, Path]]:
    out = []
    for section in SECTIONS:
        files = sorted((SITE / section).glob("*.md"))
        # The section index first, then the rest alphabetically.
        files.sort(key=lambda p: (p.stem not in ("index", "overview", "introduction"), p.stem))
        for path in files:
            slug = f"{section}/{path.stem}"
            if slug not in SKIP:
                out.append((slug, path))
    return out


def frontmatter(text: str) -> tuple[dict[str, str], str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.index("\n---\n", 4)
    meta = {}
    for line in text[4:end].splitlines():
        key, _, value = line.partition(":")
        meta[key.strip()] = value.strip().strip('"')
    return meta, text[end + 5 :]


def api_table(match: re.Match[str]) -> str:
    rows = re.findall(
        r"\{\s*name:\s*'(.*?)',\s*type:\s*'(.*?)',\s*default:\s*'(.*?)',\s*description:\s*'(.*?)'\s*\}",
        match.group(1),
        re.DOTALL,
    )
    if not rows:
        raise SystemExit(f"unparsed ApiTable:\n{match.group(0)}")
    lines = ["| Item | Type | Default | Description |", "|---|---|---|---|"]
    for name, kind, default, description in rows:
        cells = [html.unescape(c).replace("|", "\\|") for c in (name, kind, default, description)]
        lines.append(f"| `{cells[0]}` | `{cells[1]}` | {cells[2]} | {cells[3]} |")
    return "\n".join(lines)


def container(match: re.Match[str]) -> str:
    kind, title, body = match.group(1), match.group(2).strip(), match.group(3).strip("\n")
    head = f"> **{title or kind.capitalize()}**"
    return "\n".join([head, ">"] + [f"> {line}".rstrip() for line in body.splitlines()])


def link(slug: str, available: set[str]):
    depth = slug.count("/")

    def replace(match: re.Match[str]) -> str:
        text, target = match.group(1), match.group(2)
        path, _, anchor = target.partition("#")
        path = path.strip("/").removesuffix(".html") or "index"
        anchor = f"#{anchor}" if anchor else ""
        if f"{path}/index" in available:
            path = f"{path}/index"
        if path in available:
            return f"[{text}]({'../' * depth}{path}.md{anchor})"
        return f"[{text}]({SITE_URL}/{path}{'.html' if path != 'index' else ''}{anchor})"

    return replace


def convert(slug: str, path: Path, available: set[str]) -> tuple[dict[str, str], str]:
    meta, body = frontmatter(path.read_text(encoding="utf-8"))
    body = re.sub(r"\n?<LiveDemo\b.*?/>\n?", "\n", body, flags=re.DOTALL)
    body = re.sub(r'<TokenSwatch[^>]*value="([^"]+)"[^>]*/>', r"`\1`", body)
    body = re.sub(r'<ApiTable :rows="\[(.*?)\]"\s*/>', api_table, body, flags=re.DOTALL)
    body = re.sub(r"^::: ?(tip|warning|info|danger|details)([^\n]*)\n(.*?)^:::$", container, body, flags=re.DOTALL | re.MULTILINE)
    body = re.sub(r"^<!--\s*compile\s*-->\n", "", body, flags=re.MULTILINE)
    body = re.sub(r"^```(\w+) \[([^\]]+)\]$", r"`\2`:\n\n```\1", body, flags=re.MULTILINE)
    body = re.sub(r"\[([^\]]+)\]\((/[^)\s]*)\)", link(slug, available), body)
    body = re.sub(r"\n{3,}", "\n\n", body).strip()
    prose = re.sub(r"```.*?```|`[^`\n]*`", "", body, flags=re.DOTALL)
    for leftover in re.findall(r"<(LiveDemo|ApiTable|TokenSwatch|Preview)\b", prose):
        raise SystemExit(f"{slug}: unconverted <{leftover}>")
    source = f"{SITE_URL}/{slug}.html"
    lines = body.splitlines()
    # Keep the page title first, then a pointer to the live page with demos.
    head, rest = (lines[0], lines[1:]) if lines and lines[0].startswith("# ") else (f"# {meta.get('title', slug)}", lines)
    text = "\n".join([head, "", f"> {meta.get('description', '')} Live page: <{source}>", *rest]) + "\n"
    return meta, text


def index(entries: list[tuple[str, dict[str, str]]]) -> str:
    out = [START, ""]
    section = None
    for slug, meta in entries:
        current = slug.split("/")[0]
        if current != section:
            if section is not None:
                out.append("")
            section = current
            out += [f"**{TITLES[current]}**", ""]
        out.append(f"- [`references/{slug}.md`](references/{slug}.md): {meta.get('description', meta.get('title', ''))}")
    out += ["", END]
    return "\n".join(out)


def main() -> int:
    check = "--check" in sys.argv
    listed = pages()
    available = {slug for slug, _ in listed}
    wanted: dict[Path, str] = {}
    entries = []
    for slug, path in listed:
        meta, text = convert(slug, path, available)
        wanted[REFS / f"{slug}.md"] = text
        entries.append((slug, meta))

    skill_md = SKILL / "SKILL.md"
    current = skill_md.read_text(encoding="utf-8")
    if START not in current or END not in current:
        raise SystemExit(f"{skill_md} needs {START} and {END} markers")
    head, _, tail = current.partition(START)
    _, _, tail = tail.partition(END)
    wanted[skill_md] = head + index(entries) + tail

    existing = set(REFS.rglob("*.md")) if REFS.exists() else set()
    stale = sorted(p for p in existing if p not in wanted)
    changed = sorted(p for p, text in wanted.items() if not p.exists() or p.read_text(encoding="utf-8") != text)

    if check:
        if changed or stale:
            for path in changed + stale:
                print(f"stale: {path.relative_to(ROOT)}")
            print("run: python3 docs-site/scripts/build-skill.py")
            return 1
        print(f"skill references up to date ({len(listed)} pages)")
        return 0

    for path in stale:
        path.unlink()
    for path in changed:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(wanted[path], encoding="utf-8")
    print(f"wrote {len(changed)} file(s), removed {len(stale)}; {len(listed)} pages")
    return 0


if __name__ == "__main__":
    sys.exit(main())
