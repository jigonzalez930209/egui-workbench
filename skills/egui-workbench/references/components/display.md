# Display

> Labels, badges, banners, cards, separators, status bars, and icon previews. Live page: <https://jigonzalez930209.github.io/egui-workbench/components/display.html>

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

| Item | Type | Default | Description |
|---|---|---|---|
| `Card::new()` | `Card` | fills width, 16 pt padding | An empty card. |
| `.title(text)` | `&str` | None | 15 pt title. |
| `.description(text)` | `&str` | None | Small muted line under the title. |
| `.tone(tone)` | `Tone` | Neutral | Title tone. |
| `.width(pt)` | `f32` | available width | Fixed width. |
| `.fill(color)` | `Color32` | window_fill | Override the background. |
| `.stroke(stroke)` | `Stroke` | 1 pt border | Override the border; Stroke::NONE removes it. |
| `.radius(r)` | `u8` | Style container radius | Override the corners. |
| `.padding(pt)` | `f32` | 16.0 | Inner padding. |

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
