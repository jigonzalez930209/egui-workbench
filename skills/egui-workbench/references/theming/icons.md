# Icons

> The Icons and MaterialIcons caches, custom SVGs, Lucide search, and sharp HiDPI textures. Live page: <https://jigonzalez930209.github.io/egui-workbench/theming/icons.html>

The workbench uses one cache for three SVG sources. Components take a name, so the same
`IconButton`, `MenuIcon`, `TabItem`, and `Tree` row work with every source.

## Icons

```rust
use egui_workbench::Icons;

let mut icons = Icons::new();
icons.register("logo", include_bytes!("../assets/logo.svg").to_vec());

if IconButton::new("lucide-heart", "Favorite").show(ui, &mut icons).clicked() {
    // ...
}
```

| Method | Purpose |
|---|---|
| `new()` | An empty cache with the bundled sets available |
| `register(name, svg)` | Add or replace a custom SVG |
| `bundled()` | Names of the bundled workbench UI glyphs |
| `lucide_names()` | All 2,410 Lucide names, sorted |
| `lucide_search(query)` | Ranked Lucide matches |
| `texture(ctx, name)` | A cached tinted texture |
| `texture_sized(ctx, name, size)` | 48 px base, then 96/192/384 px for large or HiDPI glyphs |
| `paint(ui, name, center, size, color)` | Draw into an existing `Ui` |
| `put(ui, name, center, size, color)` | Place and allocate |
| `show(ui, name, size)` | Draw at the current text color |
| `show_tinted(ui, name, size, color)` | Draw with an explicit tint |

Resolution order: custom SVG → bundled UI glyph → Lucide. A `lucide-` prefix skips the
first two, which matters for names that exist in both sets (`save`, `search`, `settings`,
`plus`, …).

## Custom glyphs

Draw monochrome glyphs in white; the cache tints them. Full-color SVGs are drawn
untinted, which is how the product mark keeps its colors.

```rust
icons.register("brand-mark", include_bytes!("../assets/brand.svg").to_vec());
```

The title bar draws the `logo` name at 20 pt. Register it in `App::new`; if it is missing,
the title bar simply leaves the slot empty.

## Lucide

- 2,410 icons from `lucide-static` 1.77.0, embedded at build time.
- 24×24, 2 pt strokes, `currentColor` replaced by white and then tinted.
- Name search: exact substring first, subsequence similarity second, token edit distance
  third. `trsh` finds `trash-2`; `hreat` finds `heart`.
- The set is not Material file icons. Material remains the file/folder resolver.

Disable the ~5 MB Material set when the app never shows files:

```toml
egui-workbench = { path = "…", default-features = false, features = ["lucide-icons"] }
```

## MaterialIcons

```rust
let id = icons.material.file_icon(path, light);
let texture = icons.material.texture(ctx, &id, light);
```

| Method | Purpose |
|---|---|
| `file_icon(path, light)` | Stable icon id from a file path and its extension |
| `folder_icon(name, expanded, root, light)` | Folder variant, open/closed, root/child |
| `texture(ctx, id, light)` | Cached texture |
| `texture_sized(ctx, id, light, size)` | Rasterized for large targets |
| `draw(ui, id, light, size)` | Draw into a `Ui` |

Use `TabIcon::Texture(texture.id())` for tab icons and `RowIcon::MaterialFile(path)` /
`RowIcon::MaterialFolder(name)` in trees. The `light` flag picks the light or dark
variant of the file icon; pass `theme_state.is_light()`.

## HiDPI

`texture_sized` shares a 48 px texture for small glyphs and rasterizes at 96, 192, or
384 px depending on the requested point size and the screen's pixels-per-point. `IconGrid` uses it for cells above 16 pt, so previews
stay sharp on retina displays.

## Updating Lucide

Lucide is vendored, not downloaded at build time:

```bash
curl -L https://registry.npmjs.org/lucide-static/-/lucide-static-1.77.0.tgz -o /tmp/lucide.tgz
tar -xzf /tmp/lucide.tgz -C /tmp/lucide
cp /tmp/lucide/package/icons/*.svg assets/lucide-icons/
cp /tmp/lucide/package/LICENSE assets/lucide-icons/LICENSE
cargo test --lib foundation::icons
```

Update the version in `README.md`, `docs/ICONS.md`, and `CHANGELOG.md` when changing the
set. The bundled-glyph tests iterate every asset and fail if one does not rasterize.
