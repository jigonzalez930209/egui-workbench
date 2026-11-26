//! The keybindings file and the VS Code importer.

use std::fmt;

use super::{BindingError, CommandId, CommandRegistry, KeySequence};

/// One command binding in a keybindings file.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct KeybindingEntry {
    /// The command to bind.
    pub command: CommandId,
    /// A chord or a space-separated chord sequence.
    pub keys: String,
    /// An optional context expression. The registry applies command-level contexts;
    /// this field is preserved for round-tripping an imported file.
    pub when: Option<String>,
}

impl KeybindingEntry {
    /// A binding with no context.
    pub fn new(command: impl Into<CommandId>, keys: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            keys: keys.into(),
            when: None,
        }
    }

    /// Attach a context expression.
    pub fn when(mut self, when: impl Into<String>) -> Self {
        self.when = Some(when.into());
        self
    }
}

/// A versioned keybindings file.
///
/// ```json
/// { "version": 1, "bindings": [{ "command": "file.save", "keys": "Ctrl+S" }] }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Keybindings {
    /// File format version. Currently 1.
    pub version: u32,
    /// The bindings.
    pub bindings: Vec<KeybindingEntry>,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self::new()
    }
}

impl Keybindings {
    /// The current file version.
    pub const VERSION: u32 = 1;

    /// An empty file.
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            bindings: Vec::new(),
        }
    }

    /// Add a binding.
    pub fn with(mut self, entry: KeybindingEntry) -> Self {
        self.bindings.push(entry);
        self
    }

    /// Apply every entry to a registry and return the entries that failed.
    pub fn apply(&self, registry: &mut CommandRegistry) -> Vec<KeybindingError> {
        let mut errors = Vec::new();
        for entry in &self.bindings {
            let Some(sequence) = KeySequence::parse(&entry.keys) else {
                errors.push(KeybindingError::InvalidSequence {
                    command: entry.command.clone(),
                    keys: entry.keys.clone(),
                });
                continue;
            };
            if let Err(error) = registry.bind(&entry.command, sequence) {
                errors.push(KeybindingError::Binding {
                    command: entry.command.clone(),
                    error,
                });
            }
        }
        errors
    }

    /// The active bindings of a registry, ready to save.
    pub fn from_registry(registry: &CommandRegistry) -> Self {
        let mut file = Self::new();
        for (sequence, command) in registry.bindings() {
            file.bindings
                .push(KeybindingEntry::new(command, sequence.to_key_string()));
        }
        file
    }
}

/// Why a keybindings file entry could not be applied.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeybindingError {
    /// The shortcut text is invalid.
    InvalidSequence {
        /// The command.
        command: CommandId,
        /// The invalid text.
        keys: String,
    },
    /// The registry rejected the binding.
    Binding {
        /// The command.
        command: CommandId,
        /// The registry error.
        error: BindingError,
    },
}

impl fmt::Display for KeybindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSequence { command, keys } => {
                write!(formatter, "`{command}` has an invalid shortcut `{keys}`")
            }
            Self::Binding { command, error } => write!(formatter, "`{command}` was not bound: {error}"),
        }
    }
}

impl std::error::Error for KeybindingError {}

/// Why a keybindings file could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeybindingsError {
    /// The JSON is malformed.
    Json(String),
    /// The version is newer than this crate understands.
    UnsupportedVersion(u32),
    /// The VS Code file is not an array of bindings.
    InvalidVscode(String),
}

impl fmt::Display for KeybindingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid keybindings JSON: {error}"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported keybindings version {version}"),
            Self::InvalidVscode(error) => write!(formatter, "invalid VS Code keybindings: {error}"),
        }
    }
}

impl std::error::Error for KeybindingsError {}

#[cfg(feature = "serde")]
mod file {
    use super::*;

    #[derive(serde::Deserialize)]
    struct VscodeEntry {
        key: String,
        command: String,
        #[serde(default)]
        when: Option<String>,
    }

    impl Keybindings {
        /// Parse a versioned keybindings file.
        ///
        /// # Errors
        /// When the JSON is malformed or the version is newer than [`Self::VERSION`].
        pub fn from_json(json: &str) -> Result<Self, KeybindingsError> {
            let file: Self = serde_json::from_str(json).map_err(|error| KeybindingsError::Json(error.to_string()))?;
            if file.version > Self::VERSION {
                return Err(KeybindingsError::UnsupportedVersion(file.version));
            }
            Ok(file)
        }

        /// Serialize the file as pretty JSON.
        ///
        /// # Errors
        /// When serialization fails (it does not for these types).
        pub fn to_json(&self) -> Result<String, KeybindingsError> {
            serde_json::to_string_pretty(self).map_err(|error| KeybindingsError::Json(error.to_string()))
        }

        /// Read a `keybindings.json` from VS Code.
        ///
        /// VS Code's file is an array of `{ "key", "command", "when" }` objects. Keys are
        /// normalized to this crate's chord syntax; command ids are kept as written.
        ///
        /// # Errors
        /// When the JSON is not an array of VS Code bindings.
        pub fn import_vscode(json: &str) -> Result<Self, KeybindingsError> {
            let entries: Vec<VscodeEntry> =
                serde_json::from_str(json).map_err(|error| KeybindingsError::InvalidVscode(error.to_string()))?;
            let mut file = Self::new();
            for entry in entries {
                // VS Code writes `-command` to remove a default binding; it is not a command.
                if entry.command.starts_with('-') {
                    continue;
                }
                let keys = normalize_vscode_keys(&entry.key);
                if KeySequence::parse(&keys).is_none() {
                    continue;
                }
                let mut binding = KeybindingEntry::new(entry.command, keys);
                binding.when = entry.when;
                file.bindings.push(binding);
            }
            Ok(file)
        }
    }
}

#[cfg(feature = "serde")]
fn normalize_vscode_keys(keys: &str) -> String {
    keys.split_whitespace()
        .map(|chord| {
            chord
                .split('+')
                .map(|part| match part.to_ascii_lowercase().as_str() {
                    "ctrl" | "control" => "Ctrl".to_owned(),
                    "cmd" | "command" | "cmdorctrl" => "Cmd".to_owned(),
                    "alt" | "option" => "Alt".to_owned(),
                    "shift" => "Shift".to_owned(),
                    other => other.to_owned(),
                })
                .collect::<Vec<_>>()
                .join("+")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Command;

    #[test]
    fn files_apply_and_report_errors() {
        let mut registry = CommandRegistry::new();
        registry.register(Command::new("file.save", "Save"));
        registry.register(Command::new("file.close", "Close"));
        let file = Keybindings::new()
            .with(KeybindingEntry::new("file.save", "Ctrl+S"))
            .with(KeybindingEntry::new("file.close", "Ctrl+W"))
            .with(KeybindingEntry::new("missing.command", "Ctrl+M"))
            .with(KeybindingEntry::new("file.save", "not a chord"));
        let errors = file.apply(&mut registry);
        assert_eq!(errors.len(), 2);
        assert_eq!(
            registry.binding(&CommandId::new("file.save")).map(ToString::to_string),
            Some("Ctrl+S".to_owned())
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn json_round_trips_and_imports_vscode() {
        let file = Keybindings::new().with(KeybindingEntry::new("file.save", "Ctrl+S"));
        let json = file.to_json().expect("serialize");
        let back = Keybindings::from_json(&json).expect("parse");
        assert_eq!(back, file);
        let vscode = r#"[{"key":"ctrl+shift+p","command":"workbench.palette","when":"editorFocus"}]"#;
        let imported = Keybindings::import_vscode(vscode).expect("vscode");
        assert_eq!(imported.bindings.len(), 1);
        assert_eq!(imported.bindings[0].keys, "Ctrl+Shift+p");
        assert_eq!(imported.bindings[0].when.as_deref(), Some("editorFocus"));
    }
}
