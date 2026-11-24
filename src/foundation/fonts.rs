//! Font loading and fallbacks.
//!
//! egui ships a small Latin default. Applications that need CJK, emoji, or a math font
//! load their own licensed font files with [`FontLibrary`]; nothing large is bundled in
//! the crate.

use std::collections::BTreeMap;
use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily};

/// A font set under construction.
#[derive(Clone, Debug)]
pub struct FontLibrary {
    definitions: FontDefinitions,
    families: BTreeMap<String, Vec<String>>,
}

impl Default for FontLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl FontLibrary {
    /// The egui defaults.
    pub fn new() -> Self {
        Self {
            definitions: FontDefinitions::default(),
            families: BTreeMap::new(),
        }
    }

    /// Register a font from owned bytes.
    pub fn add_font(&mut self, name: impl Into<String>, bytes: Vec<u8>) -> &mut Self {
        self.definitions
            .font_data
            .insert(name.into(), Arc::new(FontData::from_owned(bytes)));
        self
    }

    /// Append a registered font to a named family, after the existing entries.
    pub fn add_fallback(&mut self, family: impl Into<String>, font: impl Into<String>) -> &mut Self {
        self.families.entry(family.into()).or_default().push(font.into());
        self
    }

    /// Install into the context. Call once at startup, before the first frame.
    pub fn install(self, ctx: &Context) {
        let mut definitions = self.definitions;
        for (family, fonts) in self.families {
            definitions
                .families
                .entry(FontFamily::Name(family.into()))
                .or_default()
                .extend(fonts);
        }
        ctx.set_fonts(definitions);
    }

    /// The fonts registered for a named family.
    pub fn family(&self, family: &str) -> &[String] {
        self.families.get(family).map(Vec::as_slice).unwrap_or(&[])
    }

    /// The default definitions, for an application that wants to tweak them directly.
    pub fn definitions(&self) -> &FontDefinitions {
        &self.definitions
    }
}

/// The conventional family name for CJK fallbacks.
pub const CJK_FAMILY: &str = "cjk";
/// The conventional family name for emoji fallbacks.
pub const EMOJI_FAMILY: &str = "emoji";
/// The conventional family name for the math font.
pub const MATH_FAMILY: &str = "math";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts_register_and_target_named_families() {
        let mut library = FontLibrary::new();
        library.add_font("noto", vec![0, 1, 2, 3]);
        library.add_fallback(CJK_FAMILY, "noto");
        assert_eq!(library.family(CJK_FAMILY), ["noto"]);
        assert!(library.definitions().font_data.contains_key("noto"));
        assert!(library.family("missing").is_empty());
        assert_eq!(EMOJI_FAMILY, "emoji");
        assert_eq!(MATH_FAMILY, "math");
        let ctx = Context::default();
        library.install(&ctx);
    }
}
