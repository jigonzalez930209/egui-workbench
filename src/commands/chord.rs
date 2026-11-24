//! Parsing, formatting, and matching of one chord or a chord sequence.

use std::fmt;
use std::str::FromStr;

use egui::{Key, Modifiers};

/// One key press with its modifiers, such as `Ctrl+Shift+P`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyChord {
    /// The modifiers held with the key.
    pub modifiers: Modifiers,
    /// The logical key.
    pub key: Key,
}

impl KeyChord {
    /// A chord from a modifier set and a key.
    pub const fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// Parse one chord, such as `Ctrl+S` or `F2`.
    ///
    /// `Ctrl`/`Control` maps to [`Modifiers::CTRL`], `Cmd`/`Command`/`Meta`/`Super` to
    /// [`Modifiers::COMMAND`], `Alt`/`Option`, and `Shift`. The key is the last token.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let mut modifiers = Modifiers::NONE;
        let mut key = None;
        for (index, token) in text.split('+').map(str::trim).enumerate() {
            if token.is_empty() {
                return None;
            }
            if index == 0 && text.split('+').count() == 1 {
                key = parse_key(token);
                break;
            }
            match token.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= Modifiers::COMMAND,
                "cmd" | "command" | "cmdorctrl" | "commandorcontrol" => modifiers |= Modifiers::COMMAND,
                "meta" | "super" | "win" | "windows" => modifiers |= Modifiers::MAC_CMD,
                "alt" | "option" | "opt" => modifiers |= Modifiers::ALT,
                "shift" => modifiers |= Modifiers::SHIFT,
                other => {
                    if key.is_some() {
                        return None;
                    }
                    key = parse_key(other);
                }
            }
        }
        Some(Self { modifiers, key: key? })
    }

    /// Whether a key event with `modifiers` and `key` runs this chord.
    ///
    /// Uses egui's logical matching, so `Ctrl+S` also accepts the platform command key.
    pub fn matches(self, modifiers: Modifiers, key: Key) -> bool {
        self.key == key && modifiers.matches_logically(self.modifiers)
    }
}

impl fmt::Display for KeyChord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        let mut push = |formatter: &mut fmt::Formatter<'_>, name: &str| {
            if !first {
                formatter.write_str("+")?;
            }
            first = false;
            formatter.write_str(name)
        };
        if self.modifiers.ctrl || self.modifiers.command {
            push(formatter, "Ctrl")?;
        }
        if self.modifiers.mac_cmd && !self.modifiers.command {
            push(formatter, "Cmd")?;
        }
        if self.modifiers.alt {
            push(formatter, "Alt")?;
        }
        if self.modifiers.shift {
            push(formatter, "Shift")?;
        }
        if !first {
            formatter.write_str("+")?;
        }
        formatter.write_str(self.key.name())
    }
}

impl FromStr for KeyChord {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text).ok_or(())
    }
}

fn parse_key(text: &str) -> Option<Key> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(key) = Key::from_name(trimmed) {
        return Some(key);
    }
    let upper = trimmed.to_ascii_uppercase();
    if upper.len() == 1 {
        let character = upper.chars().next()?;
        if let Some(key) = Key::from_name(&character.to_string()) {
            return Some(key);
        }
    }
    Key::from_name(&upper)
}

/// A sequence of chords, such as `Ctrl+K Ctrl+S`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct KeySequence {
    chords: Vec<KeyChord>,
}

impl KeySequence {
    /// An empty sequence.
    pub fn new() -> Self {
        Self::default()
    }

    /// A single-chord sequence.
    pub fn chord(chord: KeyChord) -> Self {
        Self { chords: vec![chord] }
    }

    /// A sequence from chords.
    pub fn from_chords(chords: impl IntoIterator<Item = KeyChord>) -> Self {
        Self {
            chords: chords.into_iter().collect(),
        }
    }

    /// Parse a space-separated sequence. `Ctrl+K Ctrl+S` has two chords.
    pub fn parse(text: &str) -> Option<Self> {
        let mut chords = Vec::new();
        for part in text.split_whitespace() {
            chords.push(KeyChord::parse(part)?);
        }
        if chords.is_empty() {
            return None;
        }
        Some(Self { chords })
    }

    /// The chords, in order.
    pub fn chords(&self) -> &[KeyChord] {
        &self.chords
    }

    /// Whether the sequence has no chords.
    pub fn is_empty(&self) -> bool {
        self.chords.is_empty()
    }

    /// Number of chords.
    pub fn len(&self) -> usize {
        self.chords.len()
    }

    /// A display string.
    pub fn to_key_string(&self) -> String {
        self.chords
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Whether `prefix` starts this sequence.
    pub fn starts_with(&self, prefix: &[KeyChord]) -> bool {
        self.chords.starts_with(prefix)
    }

    /// Whether this sequence starts `prefix`.
    pub fn is_prefix_of(&self, other: &Self) -> bool {
        other.starts_with(&self.chords)
    }
}

impl fmt::Display for KeySequence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_key_string())
    }
}

impl FromStr for KeySequence {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text).ok_or(())
    }
}

impl From<KeyChord> for KeySequence {
    fn from(chord: KeyChord) -> Self {
        Self::chord(chord)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::{KeyChord, KeySequence};

    impl serde::Serialize for KeyChord {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_string())
        }
    }

    impl<'de> Deserialize<'de> for KeyChord {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let text = String::deserialize(deserializer)?;
            Self::parse(&text).ok_or_else(|| serde::de::Error::custom(format!("invalid chord `{text}`")))
        }
    }

    impl serde::Serialize for KeySequence {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_key_string())
        }
    }

    impl<'de> Deserialize<'de> for KeySequence {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let text = String::deserialize(deserializer)?;
            Self::parse(&text).ok_or_else(|| serde::de::Error::custom(format!("invalid key sequence `{text}`")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_single_chords() {
        let save = KeyChord::parse("Ctrl+S").expect("save");
        assert_eq!(save.key, Key::S);
        assert!(save.modifiers.command);
        assert_eq!(save.to_string(), "Ctrl+S");
        assert_eq!(KeyChord::parse("F2").map(|chord| chord.key), Some(Key::F2));
        assert_eq!(KeyChord::parse("ctrl+shift+p").map(|chord| chord.key), Some(Key::P));
        assert!(KeyChord::parse("").is_none());
        assert!(KeyChord::parse("Ctrl+").is_none());
    }

    #[test]
    fn parses_chord_sequences() {
        let sequence = KeySequence::parse("Ctrl+K Ctrl+S").expect("sequence");
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.to_string(), "Ctrl+K Ctrl+S");
        assert!(sequence.is_prefix_of(&KeySequence::parse("Ctrl+K Ctrl+S Ctrl+O").expect("longer")));
        assert!(!sequence.is_prefix_of(&KeySequence::parse("Ctrl+K Ctrl+O").expect("other")));
    }

    #[test]
    fn logical_matching_accepts_the_platform_command_key() {
        let save = KeyChord::parse("Ctrl+S").expect("save");
        assert!(save.matches(Modifiers::COMMAND, Key::S));
        assert!(!save.matches(Modifiers::COMMAND, Key::A));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn chords_round_trip_through_json() {
        let sequence = KeySequence::parse("Ctrl+K Ctrl+S").expect("sequence");
        let json = serde_json::to_string(&sequence).expect("serialize");
        assert_eq!(json, "\"Ctrl+K Ctrl+S\"");
        let back: KeySequence = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, sequence);
    }
}
