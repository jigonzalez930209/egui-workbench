# Icon browser

> Search 2,410 Lucide icons, show a responsive virtualized grid, and track the selection. Live page: <https://jigonzalez930209.github.io/egui-workbench/recipes/icon-browser.html>

## State

```rust
struct IconBrowser {
    icons: Icons,
    query: String,
    names: Vec<String>,
    selected: Option<String>,
    size: IconSize,
    layout: IconLayout,
}
```

Refresh `names` when the query or the data changes, not every frame:

```rust
fn refresh(&mut self) {
    self.names = if self.query.trim().is_empty() {
        Icons::lucide_names()
    } else {
        Icons::lucide_search(&self.query)
    };
    if self.selected.as_ref().is_some_and(|name| !self.names.contains(name)) {
        self.selected = None;
    }
}
```

## Search field

```rust
let response = TextField::new(&mut self.query)
    .hint("Search icons…")
    .width(ui.available_width())
    .show(ui);
if response.changed() {
    self.refresh();
}
```

`Icons::lucide_search` handles typos and partial names: `trsh` finds `trash-2`, `hreat`
finds `heart`. It ranks exact substring matches first, then subsequence similarity, then
token edit distance.

## Grid

```rust
if let Some(index) = IconGrid::new("icon-browser", &self.names)
    .prefix("lucide-")
    .size(self.size)
    .layout(self.layout)
    .labels(self.layout == IconLayout::Grid)
    .selected(self.selected.as_deref())
    .gap(6.0)
    .show(ui, &mut self.icons)
{
    self.selected = Some(self.names[index].clone());
}
```

| Control | Options |
|---|---|
| Size | `IconSize::Auto` (20–56 pt) or `IconSize::Fixed(pt)` |
| Layout | `IconLayout::Grid` or `IconLayout::List` |
| Labels | On for grid cells, optional for list rows |
| Prefix | `"lucide-"` skips the UI glyphs |
| Selection | `.selected(Some(name))` outlines the cell |

`IconGrid` owns its scroll area and fills the remaining height. Do not wrap it in another
vertical `ScrollArea`.

## Size toolbar

```rust
ui.horizontal_wrapped(|ui| {
    for (label, size) in [
        ("Auto", IconSize::Auto),
        ("16", IconSize::Fixed(16.0)),
        ("24", IconSize::Fixed(24.0)),
        ("32", IconSize::Fixed(32.0)),
        ("48", IconSize::Fixed(48.0)),
    ] {
        if Button::new(label)
            .size(Size::Sm)
            .variant(if self.size == size { ButtonVariant::Primary } else { ButtonVariant::Ghost })
            .show(ui)
            .clicked()
        {
            self.size = size;
        }
    }
    if IconButton::new("more", "Layout").show(ui, &mut self.icons).clicked() {
        self.layout = match self.layout {
            IconLayout::Grid => IconLayout::List,
            IconLayout::List => IconLayout::Grid,
        };
    }
});
```

## Detail panel

Show the selected name, size, and a copy button. The name is the value to pass to
`IconButton::new` or `TabIcon::Glyph`:

```rust
if let Some(name) = &self.selected {
    Card::new().title(name).show(ui, |ui| {
        Label::new(format!("lucide-{name}")).tone(Tone::Muted).show(ui);
        if Button::new("Copy name").size(Size::Sm).show(ui).clicked() {
            ui.ctx().copy_text(format!("lucide-{name}"));
        }
    });
}
```

## Performance

- 2,410 names are cheap to filter; the grid is virtualized.
- Do not call `lucide_names()` every frame; cache it or refresh on query change.
- `IconSize::Fixed(48)` rasterizes at a higher resolution, so first paint of a screen full
  of large icons is more expensive than the default.
- The grid reflows on resize; no manual column count is needed.

## Checklist

- [ ] Search refreshes only on change.
- [ ] `prefix("lucide-")` to avoid UI glyph collisions.
- [ ] Keep the selection valid after filtering.
- [ ] Use the grid's own scroll area.
- [ ] Copy the exact name the components expect.
