---
title: Use with AI agents
description: Install the egui-workbench agent skill so Claude, Codex, Cursor, Gemini, Copilot, or any coding agent builds apps with this crate correctly.
---

# Use with AI agents

The repository ships an [agent skill](https://agentskills.io): a folder with a
`SKILL.md` and reference pages that teaches a coding agent how to use egui-workbench. It
covers setup and the single copy of egui, the `WorkbenchApp` and `Command` model, a
minimal program that compiles, the design rules, and this whole guide as plain Markdown.
The agent loads it when a task involves a Rust desktop app, so you can simply ask "build
me a log viewer in Rust with tabs and a chart".

```text
skills/egui-workbench/
├── SKILL.md          # when to use it, setup, mental model, minimal app, rules
└── references/       # this guide, one Markdown file per page
```

## Install

### Claude Code

The repository is also a plugin marketplace:

```text
/plugin marketplace add jigonzalez930209/egui-workbench
/plugin install egui-workbench@egui-workbench
```

### Codex, Cursor, Gemini CLI, Copilot, and other agents

Agents that read the Agent Skills format can install it with the
[`skills`](https://www.npmjs.com/package/skills) command, which asks which agents to
install it for:

```bash
npx skills add jigonzalez930209/egui-workbench
```

Or copy the folder by hand into the agent's skills directory (for Claude Code,
`~/.claude/skills/` for every project or `.claude/skills/` for one):

```bash
git clone --depth 1 https://github.com/jigonzalez930209/egui-workbench
cp -r egui-workbench/skills/egui-workbench ~/.claude/skills/
```

### Claude.ai and the Claude desktop app

Download `egui-workbench-skill.zip` from the
[latest release](https://github.com/jigonzalez930209/egui-workbench/releases/latest)
and upload it in **Settings → Capabilities → Skills**.

### Agents without skill support

Point the agent at the raw file and ask it to follow it:

```text
https://raw.githubusercontent.com/jigonzalez930209/egui-workbench/main/skills/egui-workbench/SKILL.md
```

## How it stays correct

- `references/` is generated from this site by `docs-site/scripts/build-skill.py`. CI
  fails when a page changed and the skill was not regenerated.
- The program in `SKILL.md` and the complete programs in this guide are compiled in CI
  against the current crate (`npm run check-examples`).
- Each release attaches the skill zip of that version.

After editing a docs page, regenerate the skill:

```bash
python3 docs-site/scripts/build-skill.py
```
