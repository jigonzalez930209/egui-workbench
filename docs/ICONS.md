# Icons

The shell ships three SVG sources behind one [`Icons`](../src/foundation/icons.rs)
cache. Components only take a name, so the same `IconButton`, `TabIcon`,
`RowIcon`, and `MenuItem` work with any of them.

## Resolution order

`Icons::texture(name)` looks up:

1. a custom SVG registered with `Icons::register(name, bytes)`;
2. a bundled workbench UI glyph in `assets/ui-icons`;
3. a bundled Lucide icon in `assets/lucide-icons`.

Prefixing a name with `lucide-` skips the first two steps, which matters when both sets
have the same name (`save`, `search`, `settings`, `plus`, …):

```rust
use egui_workbench::components::prelude::*;

IconButton::new("lucide-heart", "Favorite").show(ui, &mut icons);
IconButton::new("save", "Save").show(ui, &mut icons); // workbench glyph
MenuItem::new("lucide-trash-2", "Delete").show(ui, &mut icons);
TabItem::new("notes.md", TabIcon::Glyph("lucide-file-text"));
TreeNode::leaf("src/main.rs", "main.rs", RowIcon::Glyph("lucide-file-text".into()));
```

`Icons::lucide_names()` returns every available Lucide name, sorted. Use it to build a
picker or to validate a name in a test. `Icons::lucide_search(query)` ranks matches by
exact substring first, subsequence similarity second, then token edit distance, so `trsh`
and `hreat` still find `trash-2` and `heart`.
The starter's **Icons** tab uses this to provide search, 16/24/32/48 pt sizes, and a
4/6/8-column grid or a list.

## Lucide

- Source: `lucide-static` **1.77.0**, all 2,410 SVG icons copied into
  `assets/lucide-icons/`.
- License: **ISC**. The full text is in `assets/lucide-icons/LICENSE`.
- Style: 24×24, 2 pt strokes, `currentColor`. The loader replaces `currentColor` with
  white, then the normal tint path colors the texture with `text`, `text_muted`, the
  selection text color, or a custom token.
- The Lucide icons are **not** Material file icons. Material (`material-icons`, MIT) is
  still the file/folder resolver for `RowIcon::MaterialFile` and
  `TabIcon::Texture`; Lucide is the general-purpose icon set for buttons, menus, and
  custom rows.

Enable only Lucide (no Material) in a generated app:

```toml
egui-workbench = { path = "…", default-features = false, features = ["lucide-icons"] }
```

## Custom product mark

The product mark is a full-color SVG registered as `logo`:

```rust
let mut icons = Icons::new();
icons.register("logo", include_bytes!("../assets/logo.svg").to_vec());
```

Full-color textures are drawn untinted; monochrome UI and Lucide glyphs are tinted.
Draw custom monochrome glyphs in white so tinting works.

## Updating Lucide

Lucide is vendored rather than downloaded at build time, so builds are reproducible.

```bash
curl -L https://registry.npmjs.org/lucide-static/-/lucide-static-1.77.0.tgz -o /tmp/lucide.tgz
tar -xzf /tmp/lucide.tgz -C /tmp/lucide
cp /tmp/lucide/package/icons/*.svg assets/lucide-icons/
cp /tmp/lucide/package/LICENSE assets/lucide-icons/LICENSE
cargo test --lib foundation::icons
```

Update the version here, in `README.md`, and in `CHANGELOG.md` when changing the set.
