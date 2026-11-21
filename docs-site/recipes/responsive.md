---
title: Responsive content
description: "Patterns that survive 640×480: wrapping rows, two-column thresholds, flexible widths, and stacking splits."
---

# Responsive content

The shell is responsive by design: the explorer and right panel yield to the content, the
bottom panel takes at most 60 % of the height, and `split_panes` stacks below 620 pt. Your
content must follow the same rules.

## Use the available width

```rust
TextField::new(&mut self.name)
    .label("Name")
    .width(ui.available_width())
    .show(ui);
```

A fixed 400 pt field in a 320 pt panel overflows. Prefer `available_width()` with a
min/max range:

```rust
let width = ui.available_width().clamp(160.0, 420.0);
```

## Let rows wrap

```rust
ui.horizontal_wrapped(|ui| {
    for action in self.actions() {
        if Button::new(action.label).size(Size::Sm).show(ui).clicked() {
            commands.push(action.command);
        }
    }
});
```

`horizontal_wrapped` moves buttons to the next line instead of clipping them. The
`Button` builder keeps its padding in a wrapped row; do not use a child `scope` for the
row, because that breaks wrapping.

## Two columns only when they fit

```rust
let two_columns = ui.available_width() >= 620.0;
if two_columns {
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| form(ui));
        ui.add_space(16.0);
        ui.vertical(|ui| preview(ui));
    });
} else {
    form(ui);
    ui.add_space(12.0);
    preview(ui);
}
```

The threshold is the same one `split_panes` uses (`STACK_BELOW = 620.0`). Pick one
breakpoint per screen and document it.

## Split panes

```rust
let mut ratio = self.split;
split_panes(ui, &mut ratio, |ui| editor(ui), |ui| preview(ui));
self.split = ratio;
```

Above 620 pt the panes are columns; below, they stack with the second pane below. The ratio
is clamped to 0.2–0.8 and each pane keeps its minimum when the space allows. Do not
implement a second responsive split.

## Cards and forms

```rust
Card::new()
    .title("Account")
    .description("Shown in the title bar.")
    .width(ui.available_width().min(420.0))
    .show(ui, |ui| {
        TextField::new(&mut self.name).label("Name").width(ui.available_width()).show(ui);
        TextField::new(&mut self.email).label("Email").width(ui.available_width()).show(ui);
    });
```

Without `.width`, a card fills the width and wraps long titles. With a fixed width, it
overflows on narrow windows.

## Empty and narrow states

- `show_empty_state` centers a heading, button, and hint; it works at any width.
- Hide non-essential status cells below a threshold rather than letting them clip.
- Collapse a toolbar into a `MenuIconButton` when it no longer fits.
- Keep the primary action visible at 640×480; move secondary actions into a menu.

## Test sizes

The crate's snapshot and layout tests use these sizes:

| Size | Purpose |
|---|---|
| 640×480 | Minimum native window |
| 760×560 | Component board |
| 980×720 | Default window |
| 1920×1080 | Wide layout |

Run `cargo run --example showcase` and resize through all four. Check that:

- the content never goes below 480 pt;
- the explorer collapses before the content is squeezed;
- the right panel yields before the editor;
- the bottom panel never takes more than 60 % of the height;
- long labels truncate with `…` instead of clipping;
- buttons wrap rather than overflow.

## Avoid

- Fixed pixel widths on fields that must fit a panel.
- Nested horizontal `ScrollArea`s inside a vertical one.
- A two-column layout that assumes 1000 pt.
- Absolute positions; use the layout system.
- Hiding the primary action on narrow windows.
- Scaling text and controls independently; use `Density`.

## Checklist

- [ ] `ui.available_width()` for flexible fields.
- [ ] `horizontal_wrapped` for action rows.
- [ ] One documented breakpoint (620 pt) for column/stacked layouts.
- [ ] `split_panes` for editor/preview.
- [ ] No horizontal overflow at 640×480.
- [ ] Long labels truncate or wrap.
- [ ] Primary action remains reachable at every size.
