# Inputs

> Text fields, text areas, switches, checkboxes, selects, steppers, and segmented controls. Live page: <https://jigonzalez930209.github.io/egui-workbench/components/inputs.html>

All inputs take `&mut` state and mutate it immediately. Most return `bool` for "changed";
the text fields return an `egui::Response` so the caller can inspect focus and edits.

## TextField

```rust
let response = TextField::new(&mut self.name)
    .label("Name")
    .hint("Untitled")
    .width(240.0)
    .show(ui);
if response.changed() {
    self.dirty = true;
}
```

The label is drawn small above the field and names the field for AccessKit. Default
width: 200 pt. Use `.enabled(false)` to disable editing.

| Item | Type | Default | Description |
|---|---|---|---|
| `TextField::new(&mut value)` | `TextField` | width 200 | A single-line field bound to the string. |
| `.label(text)` | `&str` | None | Visible small label and accessible name. |
| `.hint(text)` | `&str` | "" | Placeholder text. |
| `.width(pt)` | `f32` | 200.0 | Total width in points. |
| `.enabled(bool)` | `bool` | true | Disable editing. |
| `.show(ui)` | `Response` | — | Paint and return the response. |

## TextArea

```rust
let response = TextArea::new(&mut self.notes)
    .label("Notes")
    .rows(6)
    .width(ui.available_width())
    .show(ui);
self.dirty |= response.changed();
```

Default width: 320 pt; default rows: 4. The frame is the sunken surface with the control
radius.

## Switch

A 36×19 pt switch. The label is the AccessKit name and is drawn to the right when not
empty. Clicking the label toggles the value too.

```rust
if Switch::new(&mut self.wrap).label("Word wrap").show(ui) {
    self.mark_dirty();
}
```

| Item | Type | Default | Description |
|---|---|---|---|
| `Switch::new(&mut value)` | `Switch` | label "" | A switch bound to a bool. |
| `.label(text)` | `&str` | "" | Accessible name and visible label. |
| `.enabled(bool)` | `bool` | true | Disable interaction. |
| `.show(ui)` | `bool` | false | Whether the value changed. |

## Checkbox

An 18 pt checkbox whose whole row is clickable.

```rust
Checkbox::new(&mut self.show_hidden).label("Show hidden files").show(ui);
```

The API mirrors `Switch`. Use a checkbox for independent options and a switch for
settings that take effect immediately.

## Select

A typed combo box. The label is not drawn; it names the box for AccessKit and salts its
id. Pair it with a settings row or a `Label`.

```rust
let options = vec![
    (Scale::Linear, "Linear".to_owned()),
    (Scale::Log10, "Log10".to_owned()),
];
let mut scale = Scale::Linear;
if Select::new("Scale", &mut scale, &options).show(ui) {
    self.scale = scale;
}
```

The value type must be `PartialEq + Copy`. Default width: 170 pt.

## Stepper

A numeric field with stacked chevrons, clamped to a range.

```rust
if Stepper::new(&mut self.zoom, 0.8..=2.0, 0.1)
    .decimals(2)
    .suffix("×")
    .show(ui, &mut icons)
{
    self.apply_zoom();
}
```

Default width: 112 pt; decimals: 0. The stepper needs the icon cache for its chevrons.

## SegmentedControl

The View/Edit/Split icon control. `Segment` lives at the crate root.

```rust
use egui_workbench::Segment;

let segments = [
    Segment { icon: "eye", tooltip: "View", enabled: true },
    Segment { icon: "edit", tooltip: "Editor", enabled: true },
    Segment { icon: "split", tooltip: "Split", enabled: true },
];
if let Some(index) = SegmentedControl::new("mode", &segments, self.mode).show(ui, &mut icons) {
    self.mode = index;
}
```

`show` returns the newly selected index, or `None` when nothing changed. Disabled cells
are skipped by the keyboard.

## Accessibility and layout

- Fields are named by their visible label (`labelled_by`); `Select` by its `label`.
- Switch and checkbox rows are clickable end to end.
- Every input has a keyboard path and shows the focus ring from `theme.focus`.
- In narrow layouts, put fields in `horizontal_wrapped` or use `ui.available_width()`
  instead of fixed widths; see [Responsive content](../recipes/responsive.md).
