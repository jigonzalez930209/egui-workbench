# Theme Studio and global

> Import VS Code themes, edit tokens with WCAG AA feedback, switch languages with Fluent plural rules, load fonts, and read the accessibility report. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/theme-and-global.html>

0.16 makes the workbench themable, translatable, and measured. The theme editor from
0.14 now imports VS Code themes and exports JSON or CSS; localization has Fluent plural
rules and runtime switching; font loading covers CJK, emoji, and math fallbacks.

## Theme Studio

```rust
use egui_workbench::app::{ThemeStudioState, show_theme_studio};

let mut studio = ThemeStudioState::default();
if show_theme_studio(ui, &mut studio) {
    egui_workbench::theme::apply_tokens(ui.ctx(), &studio.theme);
}
```

The left column imports a VS Code theme and edits its tokens with live WCAG AA feedback;
the right column exports JSON tokens or `--wb-*` CSS custom properties. Import maps the
VS Code color keys (`editor.background`, `foreground`, `focusBorder`,
`list.activeSelectionBackground`, and the rest), then `ensure_aa` adjusts text, muted
text, surfaces, links, and the accent until the text pairs pass AA. The high-contrast
button derives a pure black/white variant with stronger borders and a high-contrast focus
ring.

```rust
let theme = egui_workbench::theme::Theme::from_vscode_json(json)?;
let css = theme.to_css_variables();
let tokens = theme.to_json();
let accessible = theme.ensure_aa();
let high = theme.high_contrast();
```

Theme packs are directories with a manifest and a JSON token file; install them through
the 0.15 extension manager as a `themes` contribution.

## Localization

```rust
use egui_workbench::localization::LocalizationState;

let mut i18n = LocalizationState::new("en");
assert_eq!(i18n.t("app.save"), "Save");
i18n.set_locale("es");
assert_eq!(i18n.t("app.save"), "Guardar");
assert_eq!(i18n.localizer.plural("items", 3), "3 elementos");
assert!(!i18n.is_rtl());
```

`LocalizationState` loads the built-in English and Spanish dictionaries; `Localizer::load`
reads a `<locale>.ftl` file and merges it over the built-in dictionary, falling back to
English for missing keys. The parser supports messages, `{ $name }` placeholders, and
`{ $count -> [one] … *[other] … }` plurals. `is_rtl` covers Arabic, Hebrew, Persian, and
Urdu; layout code reverses rows when it is true.

The built-in strings cover the shell, tabs, tree, palette, DevTools, extensions, and the
agent panel. Applications can pass their own `&str` labels to the component label structs,
so a checked-in `.ftl` dictionary drives the whole window.

## Fonts

```rust
use egui_workbench::fonts::{CJK_FAMILY, FontLibrary};

let mut fonts = FontLibrary::new();
fonts.add_font("noto-sans-cjk", std::fs::read("NotoSansCJK-Regular.ttc")?);
fonts.add_fallback(CJK_FAMILY, "noto-sans-cjk");
fonts.add_fallback("math", "stix-two-math");
fonts.install(ctx);
```

Nothing large is bundled. The application supplies licensed font bytes and names the
fallback family; `CJK_FAMILY`, `EMOJI_FAMILY`, and `MATH_FAMILY` are the conventional
names.

## Accessibility

- Every built-in theme passes the WCAG AA pairs in `Theme::accessibility_issues`; the
  tests cover text/background, text/surface, muted text, links, accent text, danger, and
  the 3:1 focus ring.
- `Theme::high_contrast()` gives a maximum-contrast variant.
- DevTools → Accessibility audits the spec for missing labels, empty labels, and
  untitled plots/grids; the 0.14 tests run it in `cargo test`.
- Reduced motion is respected by `ThemeState` and the switch/tab animations.
- AccessKit roles and names come from egui and the spec's accessibility tree.

The published [accessibility report](../reference/accessibility-report.md) records the
automated results and the manual screen-reader checklist that still needs hardware.
