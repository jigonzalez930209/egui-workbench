---
title: Display
description: Labels, badges, banners, cards, separators, status bars, and icon previews.
---

# Display

<LiveDemo
  scene="display"
  height="360"
  alt="Badges, a card with a title and body, and a separator"
  caption="Badges, a banner with actions, labels, tabs, a card, and a tree; click them."
/>

## Label

A text label with the shared size and tone scale.

```rust
Label::new("Workspace").strong().show(ui);
Label::new("Unsaved changes").tone(Tone::Warning).show(ui);
Label::new("Secondary").size(Size::Sm).tone(Tone::Muted).show(ui);
```

Tones: `Neutral` (primary text), `Muted`, `Accent`, `Warning`, `Danger`, or
`Custom(Color32)`. Use `Custom(theme.success)` for success, because egui's `Visuals` does
not carry that token.

## Badge

A 20 pt status pill with an optional 5 pt dot.

```rust
Badge::new("Saved").show(ui);
Badge::new("Accent").tone(Tone::Accent).show(ui);
Badge::new("Conflicts").tone(Tone::Danger).dot(true).show(ui);
```

The neutral border uses `window_stroke`; colored tones derive a border from their text
color. Override the radius with `.radius(u8)`.

## Banner

A full-width inline message with actions. The default tone is `Warning`.

```rust
Banner::new("The file changed on disk.").tone(Tone::Warning).show(ui, |ui| {
    if Button::new("Reload").size(Size::Sm).show(ui).clicked() {
        reload();
    }
});
```

Use `.color(theme.success)` for a success banner. Banners span the available width with a
tinted border and the container radius.

## Card

A raised surface with an optional title, description, and body. Without `.width`, it
fills the available width and wraps long titles.

```rust
Card::new()
    .title("Appearance")
    .description("Theme and motion.")
    .width(420.0)
    .show(ui, |ui| {
        TextField::new(&mut self.name).label("Name").show(ui);
        Separator::new().show(ui);
        if Button::new("Save").show(ui).clicked() {
            self.save();
        }
    });
```

`show` returns the body value, so a card can wrap a computation or a button result.

<ApiTable :rows="[
  { name: 'Card::new()', type: 'Card', default: 'fills width, 16 pt padding', description: 'An empty card.' },
  { name: '.title(text)', type: '&str', default: 'None', description: '15 pt title.' },
  { name: '.description(text)', type: '&str', default: 'None', description: 'Small muted line under the title.' },
  { name: '.tone(tone)', type: 'Tone', default: 'Neutral', description: 'Title tone.' },
  { name: '.width(pt)', type: 'f32', default: 'available width', description: 'Fixed width.' },
  { name: '.fill(color)', type: 'Color32', default: 'window_fill', description: 'Override the background.' },
  { name: '.stroke(stroke)', type: 'Stroke', default: '1 pt border', description: 'Override the border; Stroke::NONE removes it.' },
  { name: '.radius(r)', type: 'u8', default: 'Style container radius', description: 'Override the corners.' },
  { name: '.padding(pt)', type: 'f32', default: '16.0', description: 'Inner padding.' }
]" />

## Separator

```rust
Separator::new().show(ui);             // horizontal, 6 pt spacing
Separator::vertical().show(ui);        // vertical, fills the available height
Separator::new().spacing(12.0).show(ui);
```

A separator paints the divider token and allocates its spacing on the cross axis.

## StatusBar

The status bar is a row of left and right cells. It is normally drawn by
`show_workbench` in a 24 pt bottom panel.

```rust
use egui_workbench::StatusItem;

StatusBar::new()
    .left(StatusItem::new("3 tabs"))
    .left(StatusItem::new("Modified"))
    .right(StatusItem::new("UTF-8"))
    .right(StatusItem::new("LF"))
    .show(ui);
```

`StatusItem` supports `.tooltip(text)` and `.warning()` to paint the text in the warning
color. `left_items(Vec<_>)` and `right_items(Vec<_>)` replace the whole side.

## IconPreview

An icon at an arbitrary size for galleries and pickers.

```rust
if IconPreview::new("lucide-heart").size(32.0).show(ui, &mut icons).clicked() {
    self.selected = Some("heart".to_owned());
}
```

Default size: 24 pt. The clickable area adds 6 pt around the glyph. For large or HiDPI
icons, the cache rasterizes at up to 384 px so previews stay sharp.

## Composition rules

- Use `Card` for grouped controls, not a raw frame.
- Use `Banner` for inline, dismissible or actionable messages; use a `Toast` for
  transient feedback.
- Use `Badge` for status, not for actions.
- Do not hard-code colors: `Tone` and theme tokens keep all nine themes legible.
