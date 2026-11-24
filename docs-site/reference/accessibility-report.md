---
title: Accessibility report
description: Automated contrast, focus, and spec-audit results for 0.16, plus the manual screen-reader checklist.
---

# Accessibility report — 0.16.0

This report is generated from the repository checks and records what was **not** verified
in this environment. It is updated with the release tag.

## Automated results

| Check | Command | Result |
|---|---|---|
| Theme contrast pairs | `cargo test -p egui-workbench --lib theme` | 19 passed |
| Localization, plurals, RTL | `cargo test -p egui-workbench --lib localization` | 3 passed |
| Font fallback API | `cargo test -p egui-workbench --lib fonts` | 1 passed |
| DevTools accessibility audit | `cargo test -p egui-workbench --lib devtools` | 4 passed |
| Spec/agent accessibility tree | `cargo test -p egui-workbench-agent` | 7 passed |
| Workspace gate | `cargo xtask check` | passed |

`Theme::accessibility_issues()` returns an empty list for all nine built-in themes. The
pairs checked are:

- primary text on background and on surface: 4.5:1,
- muted text on background: 4.5:1,
- links on background: 4.5:1,
- accent text on accent: 4.5:1,
- danger on background: 4.5:1,
- focus ring on background: 3:1.

`Theme::high_contrast()` derives pure black/white surfaces, maximum text contrast,
stronger borders, and a high-contrast focus ring; its test asserts the same AA pairs.

The DevTools audit scans a serialized spec for buttons/fields without a label or id,
empty labels, and plots/grids without a title. It is part of the test suite, not a manual
demo.

## Manual checklist

These require hardware or a person and are **not** claimed as verified here:

- [ ] NVDA on Windows, Narrator on Windows.
- [ ] VoiceOver on macOS.
- [ ] Orca on Linux.
- [ ] A full screen-reader walkthrough of the showcase: title bar, menus, tabs, explorer,
      data grid, plot, dialogs, DevTools, extension manager, agent panel.
- [ ] Focus order and visible focus on every interactive component.
- [ ] 200% and 400% zoom / HiDPI at 2×.
- [ ] Reduced motion through the operating-system setting.
- [ ] Web build with a browser accessibility tree (AccessKit on wasm).

## Known gaps

- The agent panel's Markdown renderer is text-first; links are not announced as links.
- The plot canvas exposes a role and title, not a data table; pair it with a `DataGrid`
  for screen-reader access to values.
- The extension manager is a list of buttons; it does not yet implement a roving tab stop.
- The manual checklist above is the blocker for a full 1.0 accessibility promise.

## Reproduce

```bash
cargo test -p egui-workbench --lib theme
cargo test -p egui-workbench --lib localization
cargo test -p egui-workbench --lib devtools
cargo test -p egui-workbench-agent
cargo xtask check
```
