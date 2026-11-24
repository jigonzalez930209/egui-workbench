# Desktop platform pass: IME and HiDPI

This is the repeatable manual pass for the two platform-sensitive paths in the 0.3.x
hardening release. The automated companion is `tests/ime_hidpi.rs`, which runs on Linux,
macOS, and Windows in CI. The manual pass covers what a headless test cannot: the
system IME window, the compositor's scale factor, and the native window chrome.

Run it on each desktop before a 0.3.x patch release and again before 1.0. Record the
result, the machine, and the scale factor. A failure blocks the release.

## What the automated test already covers

| Check | Test |
|---|---|
| IME pre-edit followed by a commit produces the composed character | `ime_preedit_then_commit_inserts_the_composed_text` |
| A 2× pixels-per-point frame keeps the text field usable | `two_x_pixels_per_point_keeps_text_and_layout_usable` |
| Window controls, resize edges, and tab geometry | `tests/theme_window.rs`, `tests/title_bar.rs`, `tests/tabs.rs` |

## IME checklist

Use a text field in the starter application (`Ctrl+N`, then the name field) or the
catalog's Inputs section (`cargo run --example catalog`).

1. **Accents (dead keys).** On a keyboard layout with dead keys, type `´` then `e`.
   The pre-edit is underlined, then the field contains `é`. Backspace during the
   pre-edit cancels it and leaves the previous text unchanged.
2. **CJK composition.** Choose a Japanese or Chinese IME. Type a romaji/pinyin word,
   move through the candidate window with the arrow keys, and commit with Space/Enter.
   The candidate window sits next to the insertion point, not at the window origin.
3. **CJK cancellation.** Start a composition, press Esc, and confirm the field is
   unchanged.
4. **Mixed text.** Put the caret in the middle of existing text and commit a composed
   character. The text around the caret does not move or lose its selection.
5. **Password-style fields.** If the application uses one, confirm the pre-edit is not
   painted as plain text.

## HiDPI checklist

1. Set the desktop scale factor to 200 % (or move the window to a 2× display) and
   restart the application.
2. The title bar, tab strip, and status bar keep their logical heights; no row is
   clipped and no glyph is blurry. Compare a screenshot at 100 % and 200 %: the layout
   is the same in points.
3. Drag the window between a 100 % and a 200 % display. The window chrome and text
   re-rasterize without a restart; the cursor and hit targets stay aligned.
4. Use the resize edges at 200 %. All eight edges and corners still resize, and the
   cursor changes shape on each.
5. Open a plot, a workbook, and the icon grid. Icons larger than 16 pt use the sharp
   texture; the plot's strokes are not half-pixel shifted.

## Recording a pass

| Date | Platform | Version | Scale | IME | HiDPI | Notes |
|---|---|---|---|---|---|---|
| 2026-11-24 | Linux (X11, Xvfb) | 0.3.1 | 100 %, 200 % | automated test | automated test | `cargo test --test ime_hidpi` |
|  | macOS |  |  |  |  |  |
|  | Windows |  |  |  |  |  |

The Linux row is the CI reference. macOS and Windows rows are filled by the person
running the pass; CI runs the behavioral half on both.
