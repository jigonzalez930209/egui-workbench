---
title: FAQ
description: "Common errors and their fixes: egui versions, fonts, wrapping, repaint, snapshots, and docs."
---

# FAQ

## Mismatched `egui` types (two copies of egui)

Your application depends on a different `egui` or `eframe` minor version than
egui-workbench (0.37). Use the re-exports (`egui_workbench::egui`,
`egui_workbench::eframe`) or align the versions, then `cargo update -p egui`. See
[Installation](/guide/installation#one-copy-of-egui).

## An icon is missing

Resolution order is custom → workbench UI glyph → Lucide. If the name exists in both UI
and Lucide, prefix it with `lucide-`. Check the exact name with
`Icons::lucide_names()` or the icon browser. A missing glyph paints nothing and does not
panic.

## A glyph shows as a box

The default fonts do not include every symbol. Avoid `→`, `←`, emoji, and unusual arrows
in labels; use an icon or plain text. The plot's `²` and `³` are chosen because the fonts
have them. If you need a symbol, load a font that covers it.

## Buttons do not wrap

`ui.scope` and child `Ui`s break `horizontal_wrapped` because they allocate the whole row
at once. Use a plain `horizontal_wrapped` and let the buttons allocate themselves. The
`Button` builder sets padding on the parent `Ui` for this reason.

## The window repaints continuously

Find the `request_repaint_after` or `request_repaint` call. A periodic "just in case"
timer is against the design rules. Request repaint only on input, animation, a worker
result, or an active stream. Toasts and the plot already wake only when needed.

## Sliders look wrong on a panel

egui sliders use the widget visuals, which follow the theme. If a custom surface changes
the fill without changing the visuals, the slider track and handle may blend in. Use the
theme tokens or wrap the area in a frame whose fill matches the surface; do not paint a
raw color behind a slider.

## A text field has no accessible name

`TextField` names itself from `.label(...)`; `Select` uses its first argument. If you use
the raw `text_field` function, call `.labelled_by(label.id)` yourself. kittest fails
when a widget has no name.

## `Esc` does not close my dialog

Use `Dialog`, `ConfirmDialog`, or `NameDialog`; they own Escape and backdrop handling. A
raw `egui::Window` does not. If you build a custom modal, handle `Key::Escape` and close on
the backdrop response.

## The close prompt never appears

Check that:

- `CloseGuard::should_prompt` is called in `App::logic`;
- `any_dirty()` is actually true;
- you did not call `allow_close` earlier and forget `reset`;
- `WindowAction::Close` is used instead of `std::process::exit`.

## The app does not fit at 640×480

- The shell keeps 480 pt for content and shrinks the explorer/right panel. If your content
  still overflows, it is using fixed widths.
- Use `ui.available_width()` and `horizontal_wrapped`.
- Use `split_panes`, which stacks below 620 pt.
- Hide non-essential status cells or move secondary actions into a menu.

## Snapshots changed after a dependency update

Font rasterization and egui layout can shift by a pixel. Check whether the change is
intended. If it is, refresh only the affected test binary:

```bash
UPDATE_SNAPSHOTS=1 cargo test --test snapshots_components
```

Review every PNG before committing. A snapshot-only change should not include logic.

## How do I generate the Rust API docs?

```bash
cargo doc --workspace --no-deps --open
# CI-style, warnings denied:
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

`cargo xtask check` includes the docs build.

## How do I build this site?

```bash
cd docs-site
npm install
npm run dev      # local preview
npm run build    # static output in .vitepress/dist
npm run preview  # serve the build
```

The examples marked `<!-- compile -->` can be checked with `npm run check-examples`.

## Why is `templates/app` not in the docs build?

It is a template copied by `cargo xtask new`, not a workspace member of the library's
docs. `cargo xtask template-check` regenerates it and runs its own fmt, clippy, and tests.

## Can I use the crate without the runtime?

Yes. Call the panels and components directly and own the order. The runtime is optional;
`WorkbenchApp` and `show_workbench` only save repeated plumbing.
