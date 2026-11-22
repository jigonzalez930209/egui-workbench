# Workbench UI Guide — a reusable desktop shell

This guide specifies the visual core of the workbench: the window, title bar, menus, tabs,
explorer tree, dialogs, settings, palette, status bar, icons, themes, and the rules that
tie them together. It leaves out everything about content: file formats, document
renderers, the editor internals, media, and file loading.

The shell is packaged as a Rust crate, **`egui-workbench`** ([GitHub](https://github.com/jigonzalez930209/egui-workbench), [crates.io](https://crates.io/crates/egui-workbench)).
Use the crate to lay out a new application. Use this document as the specification: it
gives every size, color, order, and behavior. You need it to extend the shell without
drifting from the look, or to rebuild it on another toolkit.

- Crate: the library, `examples/showcase`, and `templates/app` in this repository.
- Origin: the shell of a production Markdown desktop editor, extracted and generalized;
  the numbers below are the ones that application ships with.
- Units: egui points (1 pt = 1 px at 100 % scale). Colors are sRGB hex.

---

## 1. Principles

1. **The shell owns presentation only.** Components take translated strings and state
   snapshots, and return events (`Option<Command>`, `TabEvent`, `TreeEvent`,
   `PromptAction`). They never do file I/O, change settings, or decide lifecycle questions.
   The application applies the events.
2. **One `Command` enum, one `handle(command)` function.** The title-bar menus, the command
   palette, the keyboard menu, shortcuts, and context menus all produce the same commands.
   Every action can be reached by pointer, by keyboard, and from the palette.
3. **No literal colors.** Every color comes from the theme tokens (§4) or from the egui
   visuals derived from them. There are only two exceptions: the close button's red hover
   `#C42B1C` with a white glyph, and the white tint for full-color icon textures.
4. **One size table.** Every chrome size is a named constant (§5). Never type a raw `34.0`
   in a component. The radii, border widths, and font sizes in this document are the
   defaults of `style::WorkbenchStyle`; components read them from `style::get(ctx)`, and
   `cargo xtask style-lint` rejects literals.
5. **Undecorated window, custom chrome.** The OS title bar is off. The app draws its own
   title bar, window controls, drag region, and resize edges (§6).
6. **No idle repaint.** The UI repaints only on input, on an animation, or when a
   background worker wakes it with `ctx.request_repaint()`. Never use a periodic
   `request_repaint_after` "just in case".
7. **Accessible by construction.** Contrast follows WCAG 2.2 AA (tested). Every pointer
   action has a keyboard path. Reduced motion sets every animation to 0 s.
   Every custom-painted widget declares an AccessKit role and name
   (`response.widget_info(...)`):
   - Buttons and window controls are named by their tooltip.
   - Segments are radio buttons; switches are check boxes; tree rows are tree items.
   - Tabs and settings rail entries are tabs; palette rows are list options; menu rows
     are menu items.
   - Fields are named by their visible label (`labelled_by`), and `combo` takes a name.

   Screen readers depend on these names, and so do the tests: `egui_kittest` fails a
   test when a widget has no accessible name.
8. **Small files.** Keep each source file under 500 lines. Split by component: the title
   bar and its menus, the tab strip, the tree and its keyboard handling, settings and each
   settings tab.

---

## 2. Stack and project setup

| Piece | Choice |
|---|---|
| Language | Rust 2024 edition, toolchain pinned (`rust-toolchain.toml`, 1.97.1) |
| UI | `egui = "=0.37"` and `eframe 0.37` (features `glow`, `x11`, `wayland`, `default_fonts`, `clipboard`) |
| Renderer | `eframe::Renderer::Glow` |
| SVG icons | `resvg`/`usvg` `=0.48.1`, embedded with `rust-embed 8` |
| File icons | VS Code Material Icon Theme (MIT), embedded (~5 MB, feature `material-icons`) |
| Lints | `unsafe_code = "forbid"`; clippy `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable` denied; `-D warnings` |
| Format | `rustfmt.toml`: `max_width = 120` |
| Scaffolding | `cargo xtask new <path>` copies `templates/app`; `cargo xtask check` runs fmt, clippy, tests |

Applications add one dependency; egui and eframe come from crates.io and are re-exported.

```toml
[dependencies]
egui-workbench = "0.2"
```

egui 0.37's API differs from older egui releases in a few places you will meet:
`egui::Panel::top/left/bottom(...).show(ui, …)` (panels nest inside the root `Ui`),
`App::logic(&mut self, ctx, frame)` before `App::ui(&mut self, ui, frame)`,
`Id::unique(…)` for global ids (`Id::new` is deprecated), `ui.scope_id()` (not
`ui.id()`), `ctx.content_rect()`, `ctx.all_styles_mut`, and `ui.close()` inside menus.

### 2.1 Recommended project tree

Start with `cargo xtask new ../my-app --title "My App"`. The generated application has
this shape:

```text
my-app/
├── Cargo.toml              # dependency and lints
├── rust-toolchain.toml
├── rustfmt.toml            # max_width = 120
├── clippy.toml             # allow unwrap/expect/panic in tests only
├── src/
│   ├── main.rs             # run_native
│   ├── app.rs              # App state, handle(Command), close guard, save_tab
│   ├── model.rs            # Tab + TabData: the per-tab domain model
│   ├── content.rs          # active-tab screen: the main product seam
│   ├── commands.rs         # Command, keymap, palette rows
│   ├── ui.rs               # WorkbenchApp panels and overlays
│   └── settings.rs         # persisted appearance preferences
└── assets/
    ├── logo.svg            # product mark, registered as the workbench logo
    ├── icons/              # app icon: 16…1024 png, .ico, .icns
    └── ui-icons/           # extra 24×24 white glyphs
```

Grow into a workspace only when a piece earns it: split the theme, the UI, and the
application into crates once the model, renderer, and workspace need their own test
cycles. The runtime in [`src/app`](src/app) and the theme module are the generic halves
of that split, already extracted.

---

## 3. Window anatomy

Default inner size **980 × 720**, minimum **640 × 420**, decorations **off**.

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│ [M] File Edit View Go Help  ‹ ›        Document.md •        ▥ 🔍 ⌘ ⚙ │ – ▢ ✕ │ 34  title bar
├──────────────────┬───────────────────────────────────────────────────────────────┤
│ WORKSPACE  ⊕ ⊞ ↻ ⌃│ ⓜ README.md  ⓜ Document.md ×         ◉│✎│◫  +  ⋯  ⟲ │ 34  tab strip
│ › 📁 assets       ├───────────────────────────────────────────────────────────────┤
│ ⌄ 📂 docs         │ (banner: conflict / missing file, optional)                    │
│     ⓜ guide.md    │                                                               │
│ ▍ⓜ Document.md   │                  central panel (editor / preview / split)      │
│   ⓘ README.md     │                                                               │
│ 260 (resizable)   │                                                               │
│                   ├───────────────────────────────────────────────────────────────┤
│                   │ …/docs/Document.md │ Saved │ Ln 1, Col 1   Markdown│Spaces: 4│LF│UTF-8 │ 24 status
└──────────────────┴───────────────────────────────────────────────────────────────┘
```

**Panel order in `ui()` is fixed.** egui lays panels out in the order they are shown:

1. `handle_window_edges(ctx)`: start an OS resize from a 4 pt band on the W, E, and S
   edges.
2. `Panel::top("app_header").exact_size(34)`: the title bar, across the full width.
3. `Panel::left("workspace_sidebar").resizable(true).default_size(260)`: the explorer,
   filled with `window_fill`, inner margin 0. It is shown only when a folder is open and
   the sidebar is toggled on. It **starts right under the title bar**, as in VS Code and
   Cursor.
4. `Panel::top("tab_bar").exact_size(34)`: the tab strip, **in the editor column only**.
5. `Panel::bottom("status_bar").exact_size(24)`: the status bar, also in the editor column.
6. `CentralPanel`: an optional banner, then the welcome screen, a viewer, or the editor.
7. Overlays, in this order: settings, special editors, palette, About, the close prompt,
   lightbox, search popover, keyboard menu, update dialog, loading dialog, diff window,
   then native file dialogs.

`clear_color` returns the theme's `background`, so no black flash appears while resizing.

---

## 4. Themes

### 4.1 Families and identifiers

There are nine stable ids. They are stored as strings in settings, and `light`/`dark`
are accepted as legacy values.

| Family | Light | Dark | Extra dark |
|---|---|---|---|
| GitHub | `github-light` | `github-dark` | `github-dark-dimmed` |
| Dracula | `alucard` | `dracula` | `dracula-soft` |
| One Dark Pro | `one-dark-pro-light` | `one-dark-pro` | `one-dark-pro-darker` |

**Follow system** keeps the family and picks its light or dark base from
`input.raw.system_theme` (`ThemeId::follow_system`). `logic()` checks the system
preference every frame while the option is on, and re-applies the theme only when it
changes. The default is GitHub Light. The showcase starts on GitHub Dark.

### 4.2 Tokens

| Token | Role |
|---|---|
| `background` | Window, editor, active tab, and title bar (`panel_fill`) |
| `surface` | Raised chrome: sidebar, tab strip, menus, dialogs (`window_fill`) |
| `surface_raised` | Cards |
| `sunken` | Text fields, steppers, segmented track, switch off (`extreme_bg_color`) |
| `text` / `text_muted` | Primary text; hints, descriptions, inactive tabs, chevrons |
| `accent` / `accent_text` | Brand accent (pressed widgets); text on it |
| `border` | Separators, field and dialog borders |
| `selection` | Selected rows, active segment, primary button, focus ring of fields |
| `hover` | Hover fill of rows and icon buttons (`faint_bg_color`, `hovered.weak_bg_fill`) |
| `danger` / `warning` / `success` / `done` | Status colors |
| `link` | Hyperlinks |
| `scrollbar` | Scroll handle |
| `focus` | Keyboard focus, hovered borders, active-row marker |

| Token | GitHub Light | GitHub Dark | GH Dimmed | Dracula | Dracula Soft | Alucard | One Dark Pro | ODP Darker | ODP Light |
|---|---|---|---|---|---|---|---|---|---|
| `background` | `#ffffff` | `#0d1117` | `#22272e` | `#282a36` | `#2b2d3a` | `#ffffff` | `#282c34` | `#1e2227` | `#fafafa` |
| `surface` | `#f6f8fa` | `#161b22` | `#2d333b` | `#343746` | `#363948` | `#f5f5f5` | `#21252b` | `#181b20` | `#f0f0f0` |
| `surface_raised` | `#ffffff` | `#21262d` | `#373e47` | `#3c4052` | `#404455` | `#ffffff` | `#2c313a` | `#252a31` | `#ffffff` |
| `sunken` | `#f6f8fa` | `#161b22` | `#2d333b` | `#21222c` | `#242631` | `#eeeeee` | `#1e2227` | `#15181c` | `#e5e5e6` |
| `text` | `#1f2328` | `#e6edf3` | `#adbac7` | `#f8f8f2` | `#f2f2ec` | `#1f1f1f` | `#abb2bf` | `#b0b7c4` | `#383a42` |
| `text_muted` | `#596069` | `#9fa7b2` | `#8b949e` | `#bfbfd0` | `#b8b8c8` | `#626262` | `#9aa2b1` | `#9aa2b1` | `#696c77` |
| `accent` | `#0969da` | `#1f6feb` | `#539bf5` | `#bd93f9` | `#b48cf0` | `#644ac9` | `#61afef` | `#5ca7e6` | `#2f6bd8` |
| `accent_text` | `#ffffff` | `#ffffff` | `#22272e` | `#282a36` | `#2b2d3a` | `#ffffff` | `#1e2227` | `#15181c` | `#ffffff` |
| `border` | `#d1d9e0` | `#30363d` | `#444c56` | `#44475a` | `#4a4d60` | `#cfcfcf` | `#3e4451` | `#353b45` | `#d3d3d6` |
| `selection` | `#b6d7ff` | `#264f78` | `#3b5075` | `#44475a` | `#4a4d60` | `#ddd2fb` | `#3e4451` | `#353b45` | `#d0e4ff` |
| `hover` | `#eaeef2` | `#21262d` | `#373e47` | `#3c4052` | `#404455` | `#f0f0f0` | `#2c313a` | `#252a31` | `#ececee` |
| `danger` | `#cf222e` | `#ff7b72` | `#f47067` | `#ff5555` | `#f07070` | `#cb3a2f` | `#f2838c` | `#ec7d86` | `#c02b20` |
| `warning` | `#9a6700` | `#e3b341` | `#daaa3d` | `#ffb86c` | `#edb87a` | `#846e15` | `#e5c07b` | `#dcb772` | `#986400` |
| `success` | `#1a7f37` | `#3fb950` | `#57ab5a` | `#50fa7b` | `#6ad98a` | `#147d30` | `#98c379` | `#91bc72` | `#227d3a` |
| `done` | `#8250df` | `#ab7df8` | `#986ee2` | `#bd93f9` | `#bd9bf2` | `#644ac9` | `#c678dd` | `#c678dd` | `#a626a4` |
| `link` | `#0969da` | `#58a6ff` | `#539bf5` | `#8be9fd` | `#9ad8e8` | `#644ac9` | `#61afef` | `#5ca7e6` | `#2f6bd8` |
| `scrollbar` | `#8c959f` | `#6e7681` | `#636e7b` | `#6272a4` | `#667090` | `#8a8a8a` | `#4b5264` | `#434958` | `#9a9aa0` |
| `focus` | `#0969da` | `#58a6ff` | `#539bf5` | `#bd93f9` | `#b48cf0` | `#644ac9` | `#61afef` | `#5ca7e6` | `#2f6bd8` |

The contrast tests require these minimums for every theme: `text` on `background` and
on `surface` ≥ 4.5:1, `text_muted`/`link`/`danger` on `background` ≥ 4.5:1,
`accent_text` on `accent` ≥ 4.5:1, and `focus` on `background` ≥ 3:1.

### 4.3 How tokens become egui visuals (`theme::apply_id`)

Start from `Visuals::light()` or `Visuals::dark()`, then set:

| Visual | Value |
|---|---|
| `panel_fill` | `background` |
| `window_fill` | `surface` |
| `faint_bg_color` | `hover` |
| `extreme_bg_color` | `sunken` |
| `hyperlink_color` | `link` |
| `selection.bg_fill` / `selection.stroke` | `selection` / 1 pt `text` |
| `window_stroke` | 1 pt `border` |
| `window_corner_radius` | 8 |
| `override_text_color` | `text` |
| `error_fg_color`, `warn_fg_color` | egui defaults (kept on purpose) |

The widget states share radius 6 and a foreground stroke of 1 pt `focus`:

| State | `bg_fill` = `weak_bg_fill` | `bg_stroke` | `fg_stroke` |
|---|---|---|---|
| noninteractive | `background` | 1 pt `border` | 1 pt `focus` |
| inactive | `surface` | 1 pt `border` | 1 pt `focus` |
| hovered | `hover` | 1 pt `focus` | 1 pt `focus` |
| active (pressed) | `accent` | 2 pt `focus` | 1 pt `text` |
| open | `surface` | (egui default) | 1 pt `focus` |

These go into **all styles** (`ctx.all_styles_mut`), so light and dark mode stay
identical:

| Spacing | Value |
|---|---|
| `item_spacing` | 8 × 4 |
| `button_padding` | 8 × 3 |
| `interact_size` | 26 × 24 |
| `menu_margin` | 4 |
| `scroll.bar_width` / `floating_width` / `floating_allocated_width` / `bar_inner_margin` | 6 / 3 / 4 / 2 |
| `animation_time` | 0.10 s (0 with reduced motion) |

| Text style | Font |
|---|---|
| Body | proportional 14 |
| Button | proportional 12.5 |
| Small | proportional 11 |
| Monospace | monospace 13 |
| Heading | proportional 20 |

The UI font is egui's bundled default (Ubuntu-Light based), so no system font is needed.
Applications may add a lowest-priority CJK fallback (found on a background thread) and
faces for their own content views. Those faces are content, not shell.

---

## 5. Size tokens (`egui_workbench::metrics`)

| Constant | pt | Used by |
|---|---|---|
| `TITLE_HEIGHT` | 34 | title bar panel |
| `TAB_HEIGHT` | 34 | tab strip panel and each tab |
| `TAB_STRIP_PADDING` | 8 | left inset of the first tab |
| `STATUS_HEIGHT` | 24 | status bar panel |
| `SIDEBAR_WIDTH` | 260 | explorer default width (persisted after resize) |
| `TREE_ROW_HEIGHT` | 21 | explorer rows |
| `TREE_INDENT` | 12 | per depth |
| `CHEVRON_SIZE` | 14 | expand slot (empty for leaves) |
| `ICON_SMALL` / `ICON_MEDIUM` / `ICON_LARGE` | 14 / 16 / 20 | menus and window glyphs / buttons, tabs, rows / product mark |
| `ACTION_BUTTON` | 28 | title-bar buttons (16 pt glyph) |
| `TOOL_BUTTON` | 26 | tab-strip buttons (18 pt glyph) |
| `SMALL_BUTTON` | 21 | explorer header buttons (16 pt glyph) |
| `WINDOW_BUTTON_WIDTH` × `HEIGHT` | 44 × 32 | minimize, maximize/restore, close |
| `CONTROL_HEIGHT` | 30 | text fields, dialog buttons, steppers |
| `MENU_ROW_HEIGHT` | 22 | context-menu rows |
| `LIST_ROW_HEIGHT` | 26 | palette rows (keyboard menu rows are 24) |
| `RESIZE_EDGE` | 4 | edge resize band |

**Compact button pitch (deliberate).** Title-bar buttons and tab-strip tool buttons place
their glyph with `ui.put`. That moves the layout cursor to the glyph's edge, so the next
button starts there and overlaps the previous cell. The rendered pitch is:

- window controls: 31 pt (cells 44 wide overlap by 13);
- title-bar actions: 24 pt (28 wide);
- tab-strip tools: 24 pt (26 wide).

Hover fills keep their full size. This is what makes the chrome look tight. To keep it,
use `Icons::put` in those buttons; use `Icons::paint` elsewhere.

---

## 6. Window and title bar

### 6.1 Window

- `egui_workbench::native_options("App")` builds a 980 × 720 window with a 640 × 420
  minimum, no decorations, and the glow renderer.
- Moving the window: pressing the center region of the title bar sends
  `ViewportCommand::StartDrag`, and double-clicking it toggles maximize.
- Resizing: a primary press within 4 pt of the W, E, or S edge sends `BeginResize`
  (SW and SE at the corners). The N edge belongs to the title bar.
- Closing: the close button sends `ViewportCommand::Close`. In `logic()`, if
  `close_requested()` and something is dirty, send `CancelClose` and show the
  unsaved-changes prompt (§10.4). After the user answers, close again with a flag that
  lets the request through.
- Keep the OS title in sync: `"name • — App"` style, through `set_window_title`.

### 6.2 Title bar layout (34 pt, `panel_fill`)

Left, left-to-right, vertically centered:

1. A 10 pt gap, then the **product mark**: a 20 pt full-color `logo` glyph. The bundled
   placeholder mark is a 24×24 rounded square (`rx 5`, `#4c8df6`) with a white "M".
2. An 8 pt gap, then the **text menus** `File Edit View Go Help`. They are flat buttons
   with 13 pt labels, 9 × 4 padding, and 3 pt spacing; transparent at rest, with no
   borders in any state, and the hover fill on hover.
3. A 6 pt gap, then **history**: `back` and `forward` action buttons. They are disabled
   (glyph at 45 % of `text_muted`) when history is empty.

Center:

- The **document title**: proportional 12.5, `text`, centered on the bar, with ` •`
  appended when dirty.
- The **drag region**: the 140 pt around the center (`center ± 70`, clamped 8 pt inside
  the bar). Covering the whole bar stole clicks from the buttons.

Right, right-to-left:

1. A 1 pt gap, then the window controls `close`, `maximize`/`restore`, `minimize`. Each is
   44 × 32 with no radius and a 14 pt glyph; transparent at rest, with the hover fill on
   hover. Close turns `#C42B1C` with a white glyph on hover.
2. A 6 pt gap, a vertical separator, then a 4 pt gap.
3. The **workbench toggles** (28 pt action buttons, 16 pt glyphs, radius 5), rightmost
   first: `settings`, `command` (palette), `search`, and `sidebar` (only when a folder is
   open). A toggled button (search open, sidebar visible) keeps the hover fill plus a
   2 pt underline in `selection`, inset 7 pt.
4. Optional **pill** "Open Folder", shown when no folder is open. It is 104 × 24, radius
   12, with the inactive fill and a 1 pt `border` stroke, a 14 pt `folder-open` glyph at
   x + 17, and a 12 pt label at x + 29. It is centered 58 pt left of the toggles.

```rust
let mut bar = TitleBar::new(&doc_name);
bar.dirty = dirty;
bar.maximized = egui_workbench::is_maximized(ui.ctx());
let output = egui_workbench::show_title_bar(ui, &mut icons, &bar,
    |ui, icons| { /* title_menu(...) ×5, action_button(back/forward) */ },
    |ui, icons| { /* action_button(settings), (command), (search), (sidebar) */ });
if let Some(action) = output.window { egui_workbench::apply_window_action(ui.ctx(), action); }
```

### 6.3 Menus

Every dropdown calls `compact_menu`: 12 pt rows, 1 pt vertical gaps, 8 × 2 padding,
4 pt margin. Clicking an item runs its command and calls `ui.close()`. Toggles are
`selectable_label` rows (`menu_toggle`), drawn selected when on. Unavailable items are
disabled, not hidden. Separators group the items.

| Menu | Min width | Items (a document editor, for example) |
|---|---|---|
| File | 230 | New · Open… · Open Folder… ─ Save · Save As… · Reopen with Encoding ▸ ─ Rename File… · Delete File… ─ Close Tab · Close Folder ─ Quit |
| Edit | 220 | Find · Search in Workspace… · Replace · Go to Line · (content inserts) ─ Command Palette… |
| View | 230 | Edit mode · Reading view · Split view (exclusive, disabled when unavailable) ─ Synchronize scrolling · Word wrap · Line numbers ─ Command Palette… · Quick Open… |
| Go | 200 | Back · Forward ─ Quick Open… · Next Tab · Previous Tab |
| Help | 210 | Settings… · Check for Updates… · About App |

Labels end in "…" when the item opens a dialog. Product-specific items go in the same
menus; do not add a sixth top-level menu without a strong reason.

---

## 7. Tab strip

The strip is 34 pt high and filled with `surface` (`window_fill`).

**Tabs** take the strip width minus 276 pt, which is reserved for the controls. The
first tab starts with an 8 pt inset inside the strip panel; the panel margin adds to it.

- Width is `label_width + 58`, clamped to **96–212 pt**. The label is measured with
  proportional 12.5.
- Shape: radius 4 on the top corners and 0 on the bottom.
- The **active** tab is filled with `background`, fading in over 0.16 s, so it merges with
  the document below. Inactive tabs have no fill and take the hover fill (0.10 s fade)
  when hovered.
- Content, left to right:
  - **Icon:** 16 pt at x + 15, center. It is a full-color Material file icon, untinted.
  - **Label:** at x + 28, proportional 12.5, truncated with "…" at `width − 54`.
  - **Close:** a 16 pt box centered at `right − 13`. Its glyph is drawn at 12 pt and
    tinted `text`. It is visible on the active tab and on hover, fading over 0.12 s. The
    box takes the hover fill (radius 3) while hovered.
- Label color:
  - `text` when active, `text_muted` otherwise;
  - `warn_fg_color` for a disk conflict or a missing file;
  - a missing file's label is also struck through with a 1 pt line.
- **Dirty marker:** an inactive, unhovered dirty tab shows a 3 pt dot in `text_muted` at
  `right − 12`, where the close button would be.
- Tabs are 2 pt apart. The strip scrolls horizontally with the raw mouse wheel (the
  smoothed delta overshot), at 20 pt per wheel line. A newly activated tab is scrolled
  into view with the minimal offset.
- Click selects. Clicking the close box or middle-clicking closes. Tabs report
  `Role::Tab` with the selected state for accessibility.

**Controls**, right to left, with 2 pt spacing:

1. A 2 pt gap, then `history` (Quick Open).
2. `more` (the tab actions menu). It stays lit while its popup is open; the popup is
   210 pt wide with Close Tab, Close Other Tabs ─ Move Tab Left, Move Tab Right ─ Save.
3. `plus` (new tab).
4. The **segmented mode switch**: a 108 × 24 track in `sunken` with radius 5 and three
   36 pt cells inset 1 pt. The selected cell is filled with `selection` (radius 4) and its
   glyph uses the selection text color; hovered cells take the hover fill. Disabled cells
   show the glyph at 45 % of `text_muted`, and the pointing-hand cursor appears on hover.
   The order is `eye` (View), `edit` (Editor), `split` (Split).

---

## 8. Explorer

The panel is filled with `surface`, with no inner margin, is resizable, and defaults to
260 pt.

**Section header:**
- A 3 pt gap above.
- The root name in **upper case**, 11.5 pt, bold, `text_muted`, 6 pt from the left.
- 21 pt `small_button`s, right to left: `collapse`, `refresh`, `folder-plus`, `file-plus`.
  They are transparent at rest, take the hover or active fill with radius 4 when hovered
  or pressed, and their glyph is 16 pt.
- A 4 pt gap below.
- Error text in `error_fg_color` under the header when an operation fails.

**Tree** (`show_tree`): a vertical scroll area filling the panel, with 2 pt padding at
the top and 12 pt at the bottom.

- **Row:**
  - Full width, 21 pt high (a 25 pt pitch with the 4 pt item spacing), radius 4.
    Content is inset 5 pt, then indented
    `depth × 12 + 2`, with 4 pt between items.
  - **Chevron:** 14 pt, `chevron-right` or `chevron-down`, in `text_muted`. Leaves keep
    the empty slot so files align with folders.
  - **Icon:** a 16 pt Material icon. Folders use the open variant when expanded, and some
    folder names (`src`, `docs`, `assets`, …) have their own icons.
  - **Label:** 12.5 pt, `text`, truncated, not selectable.
- **Row fills:**
  - selected: `selection`;
  - the active document's row: `selection` at 45 %, plus a **2 × (h − 8) pt bar** at
    x + 4, y + 4, radius 1, in the active stroke color (`focus`);
  - hovered: hover fill;
  - otherwise none. The cursor stays the default arrow.
- **Clicks:** on a folder, toggle; on a file, open. The selection then moves to the row
  and the tree takes keyboard focus.
- **Following the active document:** when the active document changes, the selection
  follows it, its ancestors expand, and the row scrolls minimally into view. Rows only
  follow a *change*, so a clicked folder is not overwritten on the next frame.
- **Keyboard** (while the tree has focus):
  - `↑`/`↓` move the selection;
  - `→` expands a folder or steps into its first child;
  - `←` collapses a folder or goes to its parent;
  - `Enter` toggles a folder or opens a file;
  - `F2` renames.

  Focus is locked for the arrow keys, so they never leak to the editor.
- **Context menu:** anchored at the row's bottom-left (+1 pt), 196–220 pt wide, with
  `compact_menu` rows made by `menu_item`:
  - Each row is 22 pt high with radius 4: a 14 pt glyph centered at x + 13, the label at
    x + 25 (12 pt), and the shortcut right-aligned at `right − 6` (10.5 pt, `text_muted`).
  - Destructive rows use `error_fg_color`, and the cursor is a pointing hand.
  - Folder rows: New file · New folder · Reveal ─ Rename… `F2` · Delete… (danger).
  - File rows: Open `Enter` · Reveal ─ Rename… `F2` · Delete… (danger).
- **External files:** documents open outside the folder are listed after a separator, in
  a synthetic "External files" folder. It is always expanded, has no chevron, and its
  files can only be renamed.

Create, rename, and delete go through the modal prompts (§10). The default names are
`untitled.md` and `folder`.

---

## 9. Status bar and central panel

**Status bar** (24 pt, `panel_fill`): small text (11 pt) cells, vertically centered and
separated by thin vertical rules.

- **Left:**
  - the path, shortened from the start to 48 characters with `…`, with the full path as
    tooltip;
  - `Saved`/`Modified` (or the viewer name);
  - `Ln x, Col y`;
  - `Sel n` while there is a selection.
- **Right**, from the edge inward: `UTF-8`, `LF`, `Spaces: 4`, the language, then flags
  such as `Wrap` and `Large file` (in `warn_fg_color`).

**Central panel:**

- **Banners** sit above the document. They are full width, filled with 12 % of the status
  color, with 8 × 4 padding: the message in that color, then plain buttons ("Reload",
  "Keep mine", "Compare" for a conflict; "Save As…", "Keep editing" for a missing file).
- **Empty state:** the heading (20 pt) at 24 % of the height (at least 24 pt), the
  subtitle (14 pt), a 16 pt gap, one default button, an 8 pt gap, and the hint.
- **Editors and long documents** use `editor_scroll_style`. The bars are floating and
  their handle stays visible at rest (55 % opacity; 75 % hovered; 100 % dragged; track 0
  / 30 % / 50 %), with a bar width of at least 10, a floating width of at least 6, and a
  minimum handle of 24. Solid bars change the viewport size and broke the split view's
  scroll sync, so do not use them. Wrap the editor in `with_editor_scrollbars`.
- **Split view** (`split_panes`): the editor first, the preview second.
  - At 620 pt or wider the panes are columns; narrower, they stack with the preview
    below.
  - The divider is a 6 pt drag handle drawn as a 1 pt line in the inactive border color,
    with a horizontal or vertical resize cursor on hover.
  - The ratio (the preview's share) is kept in 0.2–0.8 per tab, default 0.5. Panes keep
    at least 200 pt (columns) or 120 pt (rows).
  - Scroll synchronization is an app feature on top: the pane under the pointer leads.

---

## 10. Dialogs and overlays

### 10.1 Modal frame (`dialog_frame`)

- `window_fill`, a 1 pt `window_stroke`, radius **10**, padding **18**.
- Shadow: offset (0, 8), blur 24, spread 0, black at alpha 60.
- egui's `Modal` dims the backdrop. Clicking the backdrop or pressing `Esc` cancels;
  `Enter` submits where there is a primary action.
- **Header:** a bold title (17 pt for prompts, 18 for settings, 15 for the palette) and a
  borderless `×` on the right (14 pt; 18 in settings) with a "Close" tooltip, then an
  8 pt gap.
- **Footer:** right to left, the **primary** button, an 8 pt gap, then **Cancel**.

### 10.2 Buttons and fields

| Control | Geometry | Colors |
|---|---|---|
| `primary_button` | ≥ 92 × 30, radius 6, 13 pt label | `selection` fill, selection text color, no stroke; `danger` uses `error_fg_color` with a white label |
| `secondary_button` | ≥ 88 × 30, radius 6, 13 pt | inactive fill, 1 pt inactive border, `text` label |
| `text_field` | width × 30, radius 6, 8 pt side padding, text centered vertically (7 pt top/bottom) | `sunken` fill, 1 pt border; focused: 1.5 pt `selection` ring inside |
| `numeric_stepper` | width × 30, value at x + 9 (13 pt) | `sunken`; two 22 × 14 cells on the right with 14 pt `chevron-up`/`chevron-down` glyphs in `text_muted`, hover fill radius 3 |
| `switch(ui, value, label)` | 36 × 19, radius 10, knob r 6.5 moving 10 pt inside, 0.12 s; announced as a check box named `label` | on: `selection` + `text` knob; off: `sunken` + `text_muted` knob; 1 pt border |
| `combo` | 170 wide | egui ComboBox in theme; its `label` argument is the accessible name (not drawn) |

### 10.3 Prompts

- **Name prompt** (380 wide):
  - title, then an optional small weak description and a 10 pt gap;
  - the small label in `text_muted`, a 3 pt gap, then a 344 pt field that takes focus
    when nothing else has it;
  - an optional error (small, `error_fg_color`) after a 5 pt gap;
  - a 14 pt gap, then the footer.

  It is used for New file, New folder, and Rename. Validate in the app and pass the
  message back through `error`.
- **Confirm prompt** (340 wide): the title, the message, a 14 pt gap, then the footer.
  Set `danger` for destructive actions ("Delete").
- **Unsaved changes** (`show_choice_window`): a centered egui `Window`, 420 wide, not
  resizable or collapsible, titled "Save changes?". The message is "Do you want to save
  your changes before closing?" and the buttons are `Save` `Discard` `Cancel`. `Esc`
  means Cancel.
- **Progress** (`show_progress`): a centered 460 pt window with the path, a percentage
  bar, and Cancel. It shows while a background load runs, polled about every 50 ms.
- **About:** a 360 pt window with the heading, a one-line description, and the version.

### 10.4 Settings (`show_settings_dialog`)

- **Width:** `clamp(window_w − 48, 560, 620)`. **Body height:**
  `clamp(window_h − 150, 240, 352)`.
- **Title row:** 18 pt bold, then `×`.
- **Rail:** 128 pt wide with 2 pt gaps. Each tab is 28 pt high, 96–126 wide, radius 5,
  with a **right-aligned** label at `right − 14` (12.5 pt). The selected tab takes the
  `selection` fill, a 2 pt underline (inset 5) in the selection text color, and that text
  color; hovered tabs take the hover fill.
- A vertical separator and a 10 pt gap follow the rail, then the body, which scrolls
  vertically (width = dialog − rail − 36).
- **Section:** a 15 pt bold title, a small weak description, and a 10 pt gap.
- **Row:** 7 pt vertical padding. The left column is 260 pt: a 12.5 pt title over a small
  weak description. One control sits on the right: a switch, a 170 pt combo, a 112 pt
  stepper, or secondary buttons.
- **Footer:** after an 8 pt gap, a separator, and an 8 pt gap: the small weak
  "App x.y.z" on the left and the primary **Done** on the right.
- A document editor's tabs, for example:
  - **Appearance:** theme, follow system, language, text scale 80–200 %.
  - **Editor:** font, size, wrap, line numbers, sync scroll.
  - **Reading.**
  - **Explorer:** hidden files, `.gitignore`, open and close folder.
  - **Media.**
  - **About:** version, check for updates, log size, open the logs folder.
- Every change applies immediately (a `Command`) and is persisted with a debounced save.
  There is no Apply button.

### 10.5 Command palette and Quick Open (`show_palette`)

- A modal 560 pt wide with radius 10 and padding 14; the shadow is the dialog's.
- The title is 15 pt bold, followed by `×`. A 532 pt `text_field` (hint "Type to
  filter…") always has focus.
- Results: up to **12** rows, case-insensitive substring matches on the label or the
  detail. They sit in a 300 pt scroll area. Each row is 26 pt with radius 5, and its text
  is `label   detail` at x + 10 (12.5 pt).
- **Row 0 is always highlighted** with `selection` and runs on `Enter`; hovered rows take
  the hover fill, and a click runs any row. With no results, "No matches" shows in weak
  text.
- The command palette lists every `Command` with its shortcut as the detail. Quick Open
  lists workspace files with the parent folder as the detail.

### 10.6 Popovers: search and the keyboard menu

- `popover_frame`: `window_fill`, 1 pt `window_stroke`, radius 8, padding 10, and a shadow
  of offset (0, 6), blur 18, alpha 48. It sits on the `Foreground` layer, is not modal,
  and does not dim the backdrop.
- **Search** (`show_popover`): 560 pt (at most the window width − 24), centered
  horizontally, 40 pt below the window top. A 13 pt bold "Search" header with `×`, the
  query field, scope and options, then the results.
- **Keyboard menu:** a 330 pt popover at (12, 40) with radius 7 and padding 8.
  - **Top:** the group labels as `selectable_label`s.
  - **Body:** below a separator, 24 pt rows with radius 4. The selected row is filled with
    `selection`; hovered rows take the hover fill.
  - **Footer hint** (10 pt, weak): "Arrows navigate · Enter runs · Esc closes".
  - **Keys:** `F10` or `Alt+F/E/V/G/H` opens it. Alt+letter's stray text event is
    consumed so the editor never types the letter.

---

## 11. Icons

### 11.1 UI glyphs (`Icons`)

These are 24 × 24 SVGs drawn in **white** (`#fff`), with strokes of 1.7–2 pt (mostly
1.8), round caps and joins, and no fills except for dots. Each one is rasterized once
into a 48 px texture and **tinted** when drawn (`text`, `text_muted`, the faded 45 %,
the selection text color, white on red). The bundled set:

`back` `forward` `chevron-down` `chevron-right` `chevron-up` `close` `collapse`
`command` `edit` `eye` `split` `file-plus` `folder-plus` `folder-open` `files`
`history` `logo` `maximize` `minimize` `restore` `menu` `more` `plus` `refresh` `save`
`search` `settings` `sidebar`, plus the formatting set `fmt-bold` `fmt-italic`
`fmt-underline` `fmt-code` `fmt-link` `fmt-heading` `fmt-quote` `fmt-table`
`fmt-list-bullet` `fmt-list-numbered` `fmt-list-task` `fmt-equation` `fmt-equation-edit`.

To add or replace a glyph, use `icons.register("logo", include_bytes!("../assets/ui-icons/logo.svg").to_vec())`.
Draw it in white so tinting works; the logo is the only full-color glyph, and it is
drawn untinted.

**Lucide.** All 2,410 Lucide icons (ISC, `assets/lucide-icons`) are embedded too. Name
resolution is custom → workbench UI glyph → Lucide; prefix with `lucide-` to force the
Lucide version when both exist. Any component that takes an icon name (`IconButton`,
`MenuItem`, `TabIcon::Glyph`, `RowIcon::Glyph`) accepts a Lucide name. `Icons::lucide_names()`
lists them. See `docs/ICONS.md`.

### 11.2 File and folder icons (`MaterialIcons`, feature `material-icons`)

These are the VS Code Material Icon Theme's manifest and SVGs, embedded. Files resolve in
this order:

1. the exact file name (`Cargo.toml`, `README.md`, `.gitignore`);
2. compound extensions, longest first (`schema.json`, `ts.map`);
3. the extension;
4. the generic `file` icon.

Light variants come from the manifest's `light` overrides. Folders resolve by name, with
`folderNamesExpanded` used for open folders, and fall back to `folder`/`folder-open`; the
root uses `folder-root`.

Textures are cached per id, light/dark, and pixel size: `size × pixels_per_point`,
clamped to 16–64 px. They are drawn **untinted**. Tree rows pass
`RowIcon::MaterialFile(path)` or `RowIcon::MaterialFolder(name)`, and tabs pass
`TabIcon::Texture(icons.material.texture(...).id())`.

---

## 12. Interaction rules

| Shortcut | Command |
|---|---|
| `Ctrl+N` / `Ctrl+O` / `Ctrl+Shift+O` | New / Open / Open Folder |
| `Ctrl+S` / `Ctrl+Shift+S` | Save / Save As |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Next / previous tab |
| `Alt+←` / `Alt+→` | Back / forward |
| `Ctrl+P` / `Ctrl+Shift+P` | Quick Open / command palette |
| `Ctrl+F` / `Ctrl+Shift+F` / `Ctrl+H` / `Ctrl+G` | Find / search the workspace / replace / go to line |
| `Ctrl+E` / `Ctrl+Shift+E` | Toggle edit/view / split |
| `Ctrl+B` | Toggle the sidebar (Bold inside a focused Markdown editor) |
| `Ctrl+,` | Settings |
| `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | Zoom in / out / reset (80–200 %) |
| `F2` | Rename |
| `F10`, `Alt+F/E/V/G/H` | Keyboard menu |
| `Ctrl+Q` | Quit (through the close request) |

- `Ctrl` means Cmd on macOS. Use `Modifiers::COMMAND` or `craft-keymap` chords
  (`"Cmd+Shift+P"`).
- Check the longer chord first: `Ctrl+Shift+P` before `Ctrl+P`.
- Consume handled keys (`consume_key`/`consume_shortcut`) so widgets never see them.
- Skip all shortcuts while a modal is open.
- `Esc` closes the topmost overlay. `Enter` runs the primary action or the highlighted
  row.
- Hover feedback is a fill, never a border: borders stay `border`, and only hovered
  inputs show the `focus` border. The pointing hand appears only on menu rows and
  segmented cells. Tooltips go on every icon-only button.
- Animation times:
  - 0.10 s: hover fades and the egui default;
  - 0.12 s: the switch knob and the tab close button;
  - 0.16 s: activating a tab.

  Reduced motion sets all of them to 0.
- Text scale: `ctx.set_zoom_factor(0.8..=2.0)`, persisted.
- Repaint:
  - `request_repaint` only when state changes;
  - workers and file watchers get a waker that calls `ctx.request_repaint()`;
  - polling fallbacks repaint at most once a second, and only while they are active.

---

## 13. Application skeleton

Start from `templates/app`; this is its shape, reduced to the essential wiring.

```rust
use egui_workbench::app::{CloseGuard, Documents, ThemeState, WorkbenchApp, show_workbench};
use egui_workbench::{eframe, egui, ThemeId};

fn main() -> eframe::Result {
    eframe::run_native("App", egui_workbench::native_options("App"),
        Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx)))))
}

struct App {
    theme: ThemeState,
    docs: Documents<Doc>,
    guard: CloseGuard,
    /* palette, settings, overlays, keymap, … */
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        Self {
            theme: ThemeState::new(ctx, ThemeId::GithubLight, false),
            docs: Documents::new(),
            guard: CloseGuard::new(),
        }
    }

    fn handle(&mut self, ctx: &egui::Context, command: Command) {
        // The only place state changes.
    }
}

impl WorkbenchApp for App {
    type Command = Command;

    fn sidebar_visible(&self) -> bool { true }
    fn tab_strip_visible(&self) -> bool { true }
    fn status_bar_visible(&self) -> bool { true }
    fn title_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> { Vec::new() }
    fn sidebar(&mut self, ui: &mut egui::Ui) -> Vec<Command> { Vec::new() }
    fn tab_strip(&mut self, ui: &mut egui::Ui) -> Vec<Command> { Vec::new() }
    fn central(&mut self, ui: &mut egui::Ui) -> Vec<Command> { Vec::new() }
    fn status_bar(&mut self, ui: &mut egui::Ui) -> Vec<Command> { Vec::new() }
    fn overlays(&mut self, ui: &mut egui::Ui) -> Vec<Command> { Vec::new() }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
        if self.guard.should_prompt(ctx, self.docs.any_dirty()) {
            // Show Save / Discard / Cancel; call guard.allow_close(ctx) on the first two.
        }
        // Then shortcuts and workers.
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let commands = show_workbench(ui, self);
        for command in commands {
            self.handle(&ctx, command);
        }
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}
```

`show_workbench` calls `handle_window_edges` and lays out the panels in the fixed order
from §3. Applications that need a different order can call the components and panels
directly.

`templates/app` is this skeleton made concrete: eight source files, appearance persisted
in the platform config directory, theme switching, the palette, tabs, a navigator, a
placeholder content screen, and close prompts for both the window and dirty tabs. It
ships no file I/O, format, or editor assumption; `TabData`, `content::show`, and
`App::save_tab` are the three seams, and `gallery.rs` is the removable demo. Run it with:

```bash
cargo xtask new ../my-app --title "My App"
cd ../my-app && cargo run
```

`examples/showcase` is the same runtime driving every component, including the explorer
tree, the mode switch, settings, Quick Open, search, the keyboard menu, prompts, and
snapshots. `examples/catalog` shows every builder variant and state in one live page;
`docs/COMPONENTS.md` is the API and authoring guide.

### 13.1 Crate API map

| Area | Items |
|---|---|
| Components | `components::{Button, IconButton, CloseButton, IconPreview, NavTab, TextField, TextArea, Switch, Checkbox, Select, Stepper, SegmentedControl, Badge, Banner, Label, Card, Separator, StatusBar, Menu, MenuItem, MenuToggle, MenuButton, MenuIconButton, Popover, TabStrip, CommandPalette, Tree, Dialog, ConfirmDialog, NameDialog, ProgressDialog, Size, Tone, ButtonVariant, prelude, blocks}` |
| Charts | `components::plot::{Plot, PlotState, PlotConfig, Series, ChartMode, LegendAnimation, AnimationEasing, LegendHoverConfig, DirectionArrowConfig, PieConfig, RadarConfig, GaugeConfig, HeatmapConfig, WaterfallConfig, Fit, ExportOptions, …}` |
| File viewer | `components::viewer::{FileViewer, FileFormat, Document, Workbook, Sheet, Table, Cell, CellKind, TextFile, ViewerError}` |
| Application runtime | `app::{ThemeState, Documents, Document, Keymap, CloseGuard, WorkbenchApp, show_workbench, TITLE_BAR_PANEL, SIDEBAR_PANEL, TAB_STRIP_PANEL, STATUS_BAR_PANEL}` |
| Theme | `Theme`, `ThemeId`, `ThemeMode`, `theme::{apply_id, apply_theme, effective_id, system_is_dark, set_reduced_motion, ANIMATION_TIME}`, `theme::{contrast_ratio, relative_luminance}` |
| Window | `native_options`, `native_options_sized`, `handle_window_edges`, `apply_window_action`, `is_maximized`, `set_window_title`, `WindowAction` |
| Title bar | `show_title_bar`, `TitleBar`, `TitleBarOutput`, `WindowLabels`, `Pill`, `title_menu` |
| Buttons | `action_button` + `IconState`, `tool_button`, `tool_button_response`, `menu_tool_button` (its menu closure gets `&mut Icons`), `small_button`, `segmented` + `Segment` |
| Controls | `text_field`, `primary_button`, `secondary_button`, `close_glyph`, `numeric_stepper`, `switch`, `combo`, `compact_menu`, `menu_item`, `menu_button`, `menu_toggle` |
| Tabs | `show_tab_strip(ui, icons, tabs, active, close_label, controls_width, controls)`, `TabItem`, `TabIcon`, `TabEvent` |
| Explorer | `section_header`, `show_tree`, `TreeNode`, `TreeState`, `TreeView`, `TreeEvent`, `RowIcon` |
| Dialogs | `dialog_frame`, `dialog_header`, `dialog_footer`, `show_name_prompt`/`NamePrompt`, `show_confirm_prompt`/`ConfirmPrompt`, `PromptAction`, `show_choice_window`, `show_progress`, `banner`, `popover_frame`, `show_popover` |
| Settings | `show_settings_dialog`/`SettingsDialog`, `settings_section`, `settings_row`, `settings_switch` |
| Palette | `show_palette`, `PaletteItem<T>`, `PaletteState`, `PaletteLabels`, `filter_palette` |
| Keyboard menu | `show_keyboard_menu`, `open_from_keyboard`, `KeyboardMenuState`, `MenuGroup<T>`, `MenuEntry<T>` |
| Split | `split_panes`, `STACK_BELOW` |
| Status | `show_status_bar`, `StatusItem`, `ellipsize_start`, `show_empty_state`, `editor_scroll_style`, `with_editor_scrollbars` |
| Icons | `Icons` (`register`, `texture`, `paint`, `put`, `show`, `show_tinted`, `lucide_names`, `lucide_search`), `MaterialIcons` (`file_icon`, `folder_icon`, `texture`, `texture_sized`, `draw`), `FULL_UV` |
| Sizes | `metrics::*` |

Components are generic over your command type (`PaletteItem<T>`, `MenuGroup<T>`,
`TreeNode<K>`), so the crate never depends on your `Command` enum.

---

## 14. Internationalization

- Write source strings in English and translate them at the call site:
  `t(lang, "Open Folder…")`. Components receive the translated `&str`.
- Keep the ellipsis character `…` (not three dots) in labels that open dialogs.
- Lay out for at least +40 % text length. The pseudo-locale (`[Ŏƥēň Ƒōľďēŕ ~~~]`) must
  not clip the menus, settings rows, or tabs, which truncate with "…".
- Language preference: `Auto` (system), English, Spanish, Pseudo. It switches live.

---

## 15. Testing the shell

The crate has over 200 tests (`cargo test`, including doctests). Each component is driven through `egui_kittest`
(0.37, features `wgpu` and `snapshot`). Tests find widgets by their AccessKit
role and name, click them, press keys, drag, and assert both the events returned and
the geometry.

| File | What it proves |
|---|---|
| `src/**` unit tests | Contrast (AA) for all nine themes; palette filtering; tree flattening; path ellipsis; every bundled glyph rasterizes; document index math; keymap matching; theme state; close guard |
| `tests/app_runtime.rs` | The fixed panel order, command collection order, and hidden panels |
| `tests/components.rs` | Every builder variant, disabled state, labels, typing, switch/checkbox, select/stepper, dialog Escape |
| `tests/theme_window.rs` | `apply_id` sets every visual, widget state, spacing, and text size in both egui styles; reduced motion; follow system (including the desktop preference fed through input); native options; a press on each window edge starts the right resize; window actions send the right viewport commands |
| `tests/title_bar.rs` | Close, maximize, restore, and minimize report their actions; the compact 31 pt / 24 pt pitch; double-clicking the center toggles maximize; dragging it starts an OS move but its edges do not; menus run items and close; disabled history ignores clicks; toggles are announced; the pill |
| `tests/controls.rs` | Icon buttons (28/26/21 pt) and the disabled state; the overflow menu and its 22 pt rows; menu toggles; segmented selection skipping disabled cells; the switch; the stepper clamping at 80–200 %; text field typing and 30 pt height; the combo; dialog button geometry |
| `tests/tabs.rs` | Select; close by button and middle click; 96–212 pt widths; the controls area; 20 tabs scroll, the active one is revealed, and the wheel scrolls back; dirty tabs keep their close button |
| `tests/tree.rs` | The capitalized header and its buttons; folder toggle and file open; 21 pt rows with a 25 pt pitch and 12 pt indentation; Up/Down/Left/Right/Enter/F2; the selection follows a new active document only; context menus open under the row and run items |
| `tests/dialogs.rs` | The name prompt focuses its field, submits with Enter, and cancels with Cancel, ×, and Esc; validation text; primary on the right; the confirm prompt; the choice window (Esc = last); progress; banners; popover position; settings rail, switch rows, Done/×/Esc, and width bounds |
| `tests/palette_menu.rs` | All rows with the first highlighted; filtering by label and detail; Enter and click run a row; "No matches"; at most 12 rows; F10 and Alt+letter open the keyboard menu; arrows wrap groups and clamp items; Enter, click, and Esc |
| `tests/layout_icons.rs` | Status bar cell order and baseline; the empty state; the split as columns (drag moves the ratio) and as rows below 620 pt (ratio clamped); editor scrollbars applied and restored; registering custom glyphs; Material file and folder icons and their texture sizes |
| `tests/plot_charts.rs` | The new chart families, the direction arrow in SVG, and every legend hover animation |
| `tests/viewer.rs` | Workbook sheets and cells, CSV/TSV, text lines, binary rejection, and rendering each viewer kind |
| `tests/snapshots.rs` | PNG references of the full workbench in GitHub Light, GitHub Dark, and Dracula; settings; the palette (One Dark Pro); the tree context menu with the pill |
| `tests/snapshots_components.rs` | PNG references of the component board in GitHub Light and GitHub Dark |

- **Harness clock:** kittest advances about 0.75 s per step by default, which is longer
  than egui's 0.3 s double-click window. Tests that need a double click use a harness
  with `with_step_dt(1/60)`.
- **Steps:** animations need more than the default 4 steps, so the shared harness
  allows 64.
- **Snapshots:** refresh them with `UPDATE_SNAPSHOTS=1 cargo test --test snapshots`, only
  for intended visual changes, and look at every changed PNG.
- **End to end:** run the real binary under Xvfb and drive it with XTest
  (`md-view-rs/scripts/e2e/x11.py`):
  - Start it with `env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 DISPLAY=:N`, or winit
    picks Wayland and opens the window on the real desktop.
  - Use a temporary `XDG_RUNTIME_DIR` and config dir, so the test never touches user
    settings or forwards to a running instance.
  - Focus the window (`x11.py focus <wid>`) before sending keys.
  - Drag slowly. Move to the start point, press, wait about 200 ms, then move in steps.
    If the press and the first moves arrive in one frame, egui hit-tests the moved
    position, and the wrong widget takes the drag.
  - Kill only the PIDs you started.
- **Visual parity:** compare captures with the snapshot references pixel by pixel at the same window size.
  The title-bar and tab-strip button centers must match (§5, compact pitch).

---

## 16. Checklist for a new screen

- [ ] Colors come from tokens or visuals only; it looks right in all nine themes.
- [ ] Sizes come from `metrics`; any new size is added there with a doc comment.
- [ ] Every action is a `Command`, reachable from a menu, the palette, and the keyboard.
- [ ] New controls use `egui_workbench::components` builders instead of raw widgets.
- [ ] Icon-only buttons have tooltips, and disabled actions are drawn faded, not hidden.
- [ ] `Esc` closes it and `Enter` runs the primary action; focus starts in the first field.
- [ ] Text is translated at the call site, and long strings truncate with "…".
- [ ] No repaint while idle; animations respect reduced motion.
- [ ] Files stay under 500 lines; clippy passes with `-D warnings`.
- [ ] A light and dark snapshot exists, or an E2E capture covers it.
