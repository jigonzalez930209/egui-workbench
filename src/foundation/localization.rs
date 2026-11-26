//! Fluent-shaped localization with plurals and runtime locale switching.
//!
//! The parser implements the subset of Fluent used by the built-in strings: messages,
//! `{ $name }` placeholders, and `{ $count -> [one] … *[other] … }` plurals. A missing
//! key falls back to English, then to the key itself.

use std::collections::BTreeMap;
use std::path::Path;

const ENGLISH: &str = include_str!("../../locales/en.ftl");
const SPANISH: &str = include_str!("../../locales/es.ftl");

/// A locale and its messages.
#[derive(Clone, Debug)]
pub struct Localizer {
    locale: String,
    messages: BTreeMap<String, String>,
    fallback: BTreeMap<String, String>,
}

impl Default for Localizer {
    fn default() -> Self {
        Self::builtin("en")
    }
}

impl Localizer {
    /// The built-in dictionary for `en` or `es`; any other locale gets English.
    pub fn builtin(locale: &str) -> Self {
        let messages = match locale {
            "es" => parse(SPANISH),
            _ => parse(ENGLISH),
        };
        Self {
            locale: locale.to_owned(),
            messages,
            fallback: parse(ENGLISH),
        }
    }

    /// Load `<locale>.ftl` from a directory, falling back to the built-ins.
    ///
    /// # Errors
    /// When the file exists but cannot be read.
    pub fn load(directory: impl AsRef<Path>, locale: &str) -> Result<Self, LocalizationError> {
        // The locale often comes from settings or the environment; `../../x` must not
        // choose which file is read.
        let plain = !locale.is_empty()
            && locale.len() <= 32
            && locale
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
        if !plain {
            return Ok(Self::builtin("en"));
        }
        let path = directory.as_ref().join(format!("{locale}.ftl"));
        if path.is_file() {
            let text = std::fs::read_to_string(&path).map_err(|error| LocalizationError::Io(error.to_string()))?;
            let mut messages = Self::builtin(locale).messages;
            messages.extend(parse(&text));
            Ok(Self {
                locale: locale.to_owned(),
                messages,
                fallback: parse(ENGLISH),
            })
        } else {
            Ok(Self::builtin(locale))
        }
    }

    /// The current locale.
    #[allow(dead_code)]
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Whether the locale is right-to-left.
    pub fn is_rtl(&self) -> bool {
        matches!(
            self.locale.split(['-', '_']).next().unwrap_or(""),
            "ar" | "he" | "fa" | "ur"
        )
    }

    /// Translate a key with no arguments.
    pub fn text(&self, key: &str) -> String {
        self.translate(key, &[])
    }

    /// Translate a key, replacing `{ $name }` placeholders.
    pub fn translate(&self, key: &str, args: &[(&str, &str)]) -> String {
        let raw = self
            .messages
            .get(key)
            .or_else(|| self.fallback.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_owned());
        interpolate(&raw, args)
    }

    /// A count, using the locale's plural rule for `one` vs `other`.
    pub fn plural(&self, key: &str, count: i64) -> String {
        let raw = self
            .messages
            .get(key)
            .or_else(|| self.fallback.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_owned());
        let category = if count == 1 { "one" } else { "other" };
        let selected = select_plural(&raw, category).unwrap_or(raw);
        interpolate(&selected, &[("count", &count.to_string())])
    }
}

/// Runtime locale state.
#[derive(Clone, Debug)]
pub struct LocalizationState {
    /// The active localizer.
    pub localizer: Localizer,
}

impl Default for LocalizationState {
    fn default() -> Self {
        Self::new("en")
    }
}

impl LocalizationState {
    /// State for a built-in locale.
    pub fn new(locale: &str) -> Self {
        Self {
            localizer: Localizer::builtin(locale),
        }
    }

    /// Switch locale at runtime.
    pub fn set_locale(&mut self, locale: &str) {
        self.localizer = Localizer::builtin(locale);
    }

    /// Translate a key.
    pub fn t(&self, key: &str) -> String {
        self.localizer.text(key)
    }

    /// The current locale.
    #[allow(dead_code)]
    pub fn locale(&self) -> &str {
        self.localizer.locale()
    }

    /// Whether the current locale is RTL; layout code should reverse rows.
    pub fn is_rtl(&self) -> bool {
        self.localizer.is_rtl()
    }
}

/// Why a locale file could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalizationError {
    /// I/O failed.
    Io(String),
}

impl std::fmt::Display for LocalizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for LocalizationError {}

fn parse(text: &str) -> BTreeMap<String, String> {
    let mut messages = BTreeMap::new();
    let mut key: Option<String> = None;
    let mut value = String::new();
    for line in text.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with(char::is_whitespace) && line.contains('=') {
            if let Some(key) = key.take() {
                messages.insert(key, value.trim().to_owned());
            }
            let (name, rest) = line.split_once('=').unwrap_or((line, ""));
            key = Some(name.trim().to_owned());
            value = rest.trim_start().to_owned();
        } else if key.is_some() {
            value.push('\n');
            value.push_str(line);
        }
    }
    if let Some(key) = key {
        messages.insert(key, value.trim().to_owned());
    }
    messages
}

fn interpolate(text: &str, args: &[(&str, &str)]) -> String {
    // One pass over the template: replacing argument by argument let a value that
    // contains `{ $other }` be expanded again by a later argument.
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("{ $") {
        result.push_str(&rest[..start]);
        let after = &rest[start + 3..];
        let replaced = after.find(" }").and_then(|end| {
            let name = &after[..end];
            args.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (end + 2, *value))
        });
        match replaced {
            Some((consumed, value)) => {
                result.push_str(value);
                rest = &after[consumed..];
            }
            None => {
                result.push_str("{ $");
                rest = after;
            }
        }
    }
    result.push_str(rest);
    result
}

fn select_plural(raw: &str, category: &str) -> Option<String> {
    let start = raw.find("->")?;
    let block = &raw[start + 2..];
    let mut current: Option<String> = None;
    let mut values: Vec<(String, String)> = Vec::new();
    for line in block.lines() {
        let trimmed = line.trim();
        let selector = trimmed
            .strip_prefix('[')
            .or_else(|| trimmed.strip_prefix("*["))
            .and_then(|rest| rest.split_once(']'));
        if let Some((name, rest)) = selector {
            let name = name.trim().to_owned();
            current = Some(name.clone());
            let text = rest.trim();
            if !text.is_empty() {
                values.push((name, text.to_owned()));
            }
        } else if let Some(name) = &current {
            let text = trimmed.trim_end_matches('}').trim();
            if text.is_empty() {
                continue;
            }
            if let Some(entry) = values.iter_mut().find(|(key, _)| key == name) {
                if !entry.1.is_empty() {
                    entry.1.push(' ');
                }
                entry.1.push_str(text);
            } else {
                values.push((name.clone(), text.to_owned()));
            }
        }
    }
    values
        .into_iter()
        .find(|(name, _)| name == category)
        .map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_and_spanish_strings_switch_at_runtime() {
        let mut state = LocalizationState::new("en");
        assert_eq!(state.t("app.save"), "Save");
        state.set_locale("es");
        assert_eq!(state.t("app.save"), "Guardar");
        assert_eq!(state.t("devtools.performance"), "Rendimiento");
        assert!(!state.is_rtl());
        assert_eq!(state.t("missing.key"), "missing.key");
    }

    #[test]
    fn plurals_use_one_and_other() {
        let english = Localizer::builtin("en");
        assert_eq!(english.plural("items", 1), "1 item");
        assert_eq!(english.plural("items", 3), "3 items");
        let spanish = Localizer::builtin("es");
        assert_eq!(spanish.plural("files", 1), "1 archivo");
        assert_eq!(spanish.plural("files", 2), "2 archivos");
        assert!(Localizer::builtin("ar").is_rtl());
    }

    #[test]
    fn an_ftl_file_overrides_a_builtin() {
        let directory = std::env::temp_dir().join(format!("egui-workbench-locale-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("dir");
        std::fs::write(directory.join("es.ftl"), "app.save = Guardar como\n").expect("write");
        let localizer = Localizer::load(&directory, "es").expect("load");
        assert_eq!(localizer.text("app.save"), "Guardar como");
        assert_eq!(localizer.text("app.cancel"), "Cancelar");
        let _ = std::fs::remove_dir_all(directory);
    }
}
