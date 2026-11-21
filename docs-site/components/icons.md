---
title: Icons
description: The icon cache, Lucide names, custom SVGs, Material file icons, and the responsive icon grid.
---

# Icons

The shell ships three SVG sources behind one `Icons` cache. Components only take a name,
so the same `IconButton`, `TabIcon`, `RowIcon`, and `MenuItem` work with all of them.

## Resolution order

`Icons::texture(name)` looks up:

1. a custom SVG registered with `Icons::register(name, bytes)`;
2. a bundled workbench UI glyph in `assets/ui-icons`;
3. a bundled Lucide icon in `assets/lucide-icons`.

Prefixing a name with `lucide-` skips the first two steps. This matters when both sets
have the same name (`save`, `search`, `settings`, `plus`, …):

```rust
IconButton::new("lucide-heart", "Favorite").show(ui, &mut icons);
IconButton::new("save", "Save").show(ui, &mut icons);       // workbench glyph
MenuItem::new("lucide-trash-2", "Delete").show(ui, &mut icons);
```

## Custom glyphs

```rust
let mut icons = Icons::new();
icons.register("logo", include_bytes!("../assets/logo.svg").to_vec());
```

Full-color SVGs are drawn untinted. Draw custom monochrome glyphs in white so tinting
works. The product mark is drawn at 20 pt in the title bar.

## Search

```rust
let all = Icons::lucide_names();            // 2,410 names, sorted
let matches = Icons::lucide_search("trsh"); // ranked: trash-2 first
```

The search ranks exact substring matches first, subsequence similarity second, then token
edit distance. The starter's Icons tab uses it for name/similarity search, sizes, and
grid/list layouts.

## IconGrid

`IconGrid` computes the column count from the available width (as many columns as fit the
minimum cell) and then stretches every cell so the row fills the width exactly. It never
overflows the right edge and reflows live as the window resizes. With `IconSize::Auto`
the glyph scales with the cell (20–56 pt). Only rows inside the viewport are laid out, so
all 2,410 icons stay cheap.

```rust
use egui_workbench::components::{IconGrid, IconLayout, IconSize};

let names = Icons::lucide_search(&self.query);
if let Some(index) = IconGrid::new("icon-browser", &names)
    .prefix("lucide-")
    .size(IconSize::Auto)
    .layout(IconLayout::Grid)
    .labels(true)
    .selected(self.selected.as_deref())
    .show(ui, &mut self.icons)
{
    self.selected = Some(names[index].clone());
}
```

<ApiTable :rows="[
  { name: 'IconGrid::new(id, names)', type: 'IconGrid', default: 'Auto, Grid, labels on', description: 'A virtualized grid over names.' },
  { name: '.prefix(text)', type: '&str', default: '&quot;&quot;', description: 'Prefix added when resolving, usually lucide-.' },
  { name: '.size(s)', type: 'IconSize', default: 'Auto', description: 'Auto scales 20–56 pt; Fixed(pt) pins the glyph.' },
  { name: '.layout(l)', type: 'IconLayout', default: 'Grid', description: 'Grid cells or one row per icon.' },
  { name: '.labels(bool)', type: 'bool', default: 'true', description: 'Show names under grid cells.' },
  { name: '.selected(name)', type: 'Option&lt;&str&gt;', default: 'None', description: 'Outline the matching cell.' },
  { name: '.gap(pt)', type: 'f32', default: '6.0', description: 'Space between cells.' },
  { name: '.show(ui, icons)', type: 'Option&lt;usize&gt;', default: '—', description: 'Clicked index, or None.' }
]" />

The grid owns its vertical scroll area and fills the remaining height of the `Ui`, so
place it directly in the content panel.

## Material file icons

With the `material-icons` feature, `Icons::material` resolves file and folder names to
the VS Code Material Icon Theme:

```rust
let id = icons.material.file_icon(path, light);
let texture = icons.material.texture(ui.ctx(), &id, light);
```

`file_icon(path, light)` returns a stable identifier; `texture` and `texture_sized`
rasterize it. `folder_icon(name, expanded, root, light)` picks the open/closed and
root/child variant. Use `TabIcon::Texture(texture.id())` for tabs and
`RowIcon::MaterialFile(path)` / `RowIcon::MaterialFolder(name)` for tree rows.

## HiDPI and previews

Glyphs at 16 pt or below share the 48 px texture. Above 16 pt, or on HiDPI screens,
`Icons::texture_sized` rasterizes at 96, 192, or 384 px, so large previews stay sharp.

## Bundles and licenses

| Set | Count | License | Feature |
|---|---|---|---|
| Workbench UI glyphs | 40+ | MIT/Apache-2.0 (this crate) | always |
| Lucide | 2,410 | ISC | `lucide-icons` |
| Material Icon Theme | files and folders | MIT | `material-icons` |

Lucide is vendored from `lucide-static` 1.77.0. It is deliberately not downloaded at
build time, so builds are reproducible. See [theming/icons](/theming/icons) for update
and registration details.
