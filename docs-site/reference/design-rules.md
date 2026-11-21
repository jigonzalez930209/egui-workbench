---
title: Design rules
description: The principles, interaction rules, accessibility contract, and new-screen checklist.
---

# Design rules

These are the rules the shell is built on. They come from `WORKBENCH.md`; follow them when
you add a component or a screen so the workbench does not drift.

## Principles

1. **The shell owns presentation only.** Components take translated strings and state
   snapshots, and return events. They never do file I/O, change settings, or decide
   lifecycle questions.
2. **One `Command` enum, one handler.** Menus, the palette, the keyboard menu, shortcuts,
   and context menus produce the same command. Every action is reachable by pointer,
   keyboard, and search.
3. **No literal colors.** Every color comes from the theme tokens or the egui visuals
   derived from them. The two exceptions are the close button's red hover `#C42B1C` with
   a white glyph and the white tint for monochrome icon textures.
4. **One size table, one style.** Every chrome size is a named `metrics` constant. Never
   type a raw `34.0` in a component. Corner radii, border widths, and font sizes come
   from `style::get(ctx)` (`control()`, `inner()`, `border(color)`, `font(13.0)`…);
   `cargo xtask style-lint` rejects literal radii and font sizes.
5. **Undecorated window, custom chrome.** The app draws its title bar, window controls,
   drag region, and resize edges.
6. **No idle repaint.** Repaint on input, animation, or an explicit worker wake. Never use
   a periodic `request_repaint_after` "just in case".
7. **Accessible by construction.** Contrast follows WCAG 2.2 AA. Every pointer action has
   a keyboard path. Reduced motion sets every animation to 0 s.
8. **Small files.** Keep each source file under 500 lines and split by component.

```rust
// One enum, one handler: menus, the palette, and the keymap all produce a Command.
let commands = show_workbench(ui, self);
for command in commands {
    self.handle(ui.ctx(), command);
}
```

## Interaction

| Shortcut | Command |
|---|---|
| `Ctrl+N` / `Ctrl+O` / `Ctrl+Shift+O` | New / Open / Open Folder |
| `Ctrl+S` / `Ctrl+Shift+S` | Save / Save As |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Next / previous tab |
| `Alt+←` / `Alt+→` | Back / forward |
| `Ctrl+P` / `Ctrl+Shift+P` | Quick Open / command palette |
| `Ctrl+F` / `Ctrl+Shift+F` / `Ctrl+H` / `Ctrl+G` | Find / search workspace / replace / go to line |
| `Ctrl+E` / `Ctrl+Shift+E` | Toggle edit/view / split |
| `Ctrl+B` | Toggle sidebar (Bold inside a focused Markdown editor) |
| `Ctrl+Alt+B` | Toggle inspector |
| `Ctrl+J` | Toggle output |
| `Ctrl+,` | Settings |
| `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | Zoom in / out / reset (80–200 %) |
| `F2` | Rename |
| `F10`, `Alt+F/E/V/G/H` | Keyboard menu |
| `Ctrl+Q` | Quit through the close request |

Rules:

- `Ctrl` means Cmd on macOS. Use `Modifiers::COMMAND` or craft-style chords.
- Check the longer chord first: `Ctrl+Shift+P` before `Ctrl+P`.
- Consume handled keys so widgets never see them.
- Skip all shortcuts while a modal is open.
- `Esc` closes the topmost overlay. `Enter` runs the primary action or the highlighted
  row.
- Hover feedback is a fill, never a border. Only hovered inputs show the focus border.
- The pointing hand appears only on menu rows and segmented cells.
- Tooltips go on every icon-only button.

## Animation

| Duration | Use |
|---|---|
| 0.10 s | Hover fades and the egui default |
| 0.12 s | Switch knob and tab close button |
| 0.16 s | Activating a tab |

Reduced motion sets all of them to 0. `theme::set_reduced_motion` sets egui's animation
time to zero; components do not keep their own timers.

## Text and density

- `ctx.set_zoom_factor(0.8..=2.0)`; persist the choice.
- `Size::{Sm, Md, Lg}` for controls, `metrics` for chrome.
- `Density::{Compact, Default, Comfortable}` scales heights, not text.
- Long labels truncate with `…`; dialog labels keep the ellipsis character.

## Repaint

- `request_repaint` only when state changes.
- Workers and file watchers get a waker that calls `ctx.request_repaint()`.
- Polling fallbacks repaint at most once a second, and only while active.
- Toasts wake only for the next timeout or animation step.

## Accessibility

- Buttons and window controls are named by their tooltip.
- Segments are radio buttons; switches are check boxes; tree rows are tree items.
- Tabs and settings rail entries are tabs; palette rows are list options; menu rows are
  menu items.
- Fields are named by their visible label (`labelled_by`); `combo` takes a name.
- `egui_kittest` fails a test when a widget has no accessible name.
- Contrast: 4.5:1 for text, 3:1 for the focus ring. `Theme::accessibility_issues` checks
  the pairs.

## Checklist for a new screen

- [ ] Colors come from tokens or visuals only; it looks right in all nine themes.
- [ ] Sizes come from `metrics` or `style`; any new size is added there with a doc comment.
- [ ] Every action is a `Command`, reachable from a menu, the palette, and the keyboard.
- [ ] New controls use `egui_workbench::components` builders, not raw widgets.
- [ ] Icon-only buttons have tooltips; disabled actions are faded, not hidden.
- [ ] `Esc` closes it; `Enter` runs the primary action; focus starts in the first field.
- [ ] Text is translated at the call site and truncates with `…`.
- [ ] No repaint while idle; animations respect reduced motion.
- [ ] Files stay under 500 lines; clippy passes with `-D warnings`.
- [ ] A light and dark snapshot exists, or an E2E capture covers it.

## Checklist for a new component

1. Put it in the layer it belongs to (`foundation`, `widgets`, `chrome`, `components`).
2. Take sizes from `metrics`/`style` and colors from `theme` or `ui.visuals()`.
3. Return an event or `Option<Command>`; do not mutate application state.
4. Declare an AccessKit role and name.
5. Add a test and a snapshot if visible.
6. Re-export it and add it to the [API map](/reference/api-map).
7. Add a catalog section and a row to the component guide.
