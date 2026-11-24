//! The append-only command journal and macros recorded from it.
//!
//! Every dispatched command can be appended with its arguments and a timestamp. A
//! recorded range becomes a named macro; replaying the same `(command, args)` sequence
//! is deterministic, so a recorded session doubles as a regression test.

use std::fmt;
use std::path::Path;

use serde_json::Value;

use crate::commands::{Command, CommandId, CommandRegistry};

/// One recorded command.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct JournalEntry {
    /// Monotonic sequence number, starting at 1.
    pub sequence: u64,
    /// Seconds on the application clock (or Unix time).
    pub timestamp: f64,
    /// The command that ran.
    pub command: CommandId,
    /// Arguments, if the command has any.
    pub args: Option<Value>,
    /// A human label for the journal view.
    pub label: Option<String>,
}

impl JournalEntry {
    /// The command id and its arguments.
    pub fn command(&self) -> &CommandId {
        &self.command
    }

    /// The argument value.
    pub fn args(&self) -> Option<&Value> {
        self.args.as_ref()
    }
}

/// A versioned, append-only journal.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandJournal {
    /// File format version. Currently 1.
    pub version: u32,
    /// The entries, oldest first.
    pub entries: Vec<JournalEntry>,
    next_sequence: u64,
}

impl Default for CommandJournal {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandJournal {
    /// The current file version.
    pub const VERSION: u32 = 1;

    /// An empty journal.
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            entries: Vec::new(),
            next_sequence: 1,
        }
    }

    /// Append a command with no arguments.
    pub fn append(&mut self, command: impl Into<CommandId>) -> &JournalEntry {
        self.append_at(0.0, command, None, None)
    }

    /// Append a command with arguments.
    pub fn append_with(
        &mut self,
        command: impl Into<CommandId>,
        args: Value,
        label: impl Into<String>,
    ) -> &JournalEntry {
        self.append_at(0.0, command, Some(args), Some(label.into()))
    }

    /// Append a command with an explicit timestamp.
    pub fn append_at(
        &mut self,
        timestamp: f64,
        command: impl Into<CommandId>,
        args: Option<Value>,
        label: Option<String>,
    ) -> &JournalEntry {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        self.entries.push(JournalEntry {
            sequence,
            timestamp,
            command: command.into(),
            args,
            label,
        });
        &self.entries[self.entries.len() - 1]
    }

    /// The entries.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the journal is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The newest entry.
    pub fn last(&self) -> Option<&JournalEntry> {
        self.entries.last()
    }

    /// Entries with a sequence greater than `sequence`.
    pub fn since(&self, sequence: u64) -> &[JournalEntry] {
        let start = self.entries.partition_point(|entry| entry.sequence <= sequence);
        &self.entries[start..]
    }

    /// Remove every entry and reset the sequence.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_sequence = 1;
    }

    /// Serialize the journal as pretty JSON.
    ///
    /// # Errors
    /// When serialization fails (it does not for these types).
    pub fn to_json(&self) -> Result<String, JournalError> {
        serde_json::to_string_pretty(self).map_err(|error| JournalError::Json(error.to_string()))
    }

    /// Parse a journal, rejecting a newer version.
    ///
    /// # Errors
    /// When the JSON is malformed or the version is newer than [`Self::VERSION`].
    pub fn from_json(json: &str) -> Result<Self, JournalError> {
        let journal: Self = serde_json::from_str(json).map_err(|error| JournalError::Json(error.to_string()))?;
        if journal.version > Self::VERSION {
            return Err(JournalError::UnsupportedVersion(journal.version));
        }
        Ok(journal)
    }

    /// Write the journal atomically.
    ///
    /// # Errors
    /// When serialization or the write fails.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), JournalError> {
        let json = self.to_json()?;
        crate::app::session::atomic_write(path.as_ref(), json.as_bytes()).map_err(JournalError::Io)
    }

    /// Read a journal from disk.
    ///
    /// # Errors
    /// When the file cannot be read or parsed.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        let text = std::fs::read_to_string(path).map_err(|error| JournalError::Io(error.to_string()))?;
        Self::from_json(&text)
    }
}

/// A named sequence of journal entries.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Macro {
    /// Display name.
    pub name: String,
    /// The registry id a macro is registered under.
    pub command: CommandId,
    /// The steps, in order.
    pub steps: Vec<JournalEntry>,
}

impl Macro {
    /// Replay the steps through `run`. The function receives each step's command and
    /// arguments in the recorded order; no window or clock is involved.
    pub fn replay(&self, mut run: impl FnMut(&JournalEntry)) {
        for step in &self.steps {
            run(step);
        }
    }

    /// A registry id from a macro name, such as `macro.align-and-fit`.
    pub fn command_id(name: &str) -> CommandId {
        let slug: String = name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();
        let slug = slug.trim_matches('-');
        CommandId::new(format!("macro.{}", if slug.is_empty() { "untitled" } else { slug }))
    }
}

/// The named macros, versioned for the workspace file.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MacroLibrary {
    /// File format version.
    pub version: u32,
    /// The macros.
    pub macros: Vec<Macro>,
}

impl MacroLibrary {
    /// The current file version.
    pub const VERSION: u32 = 1;

    /// An empty library.
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            macros: Vec::new(),
        }
    }

    /// Record the entries after `after_sequence` as a new macro and return it.
    pub fn record_from(&mut self, journal: &CommandJournal, after_sequence: u64, name: impl Into<String>) -> &Macro {
        let name = name.into();
        let steps = journal.since(after_sequence).to_vec();
        let command = Macro::command_id(&name);
        self.macros.push(Macro { name, command, steps });
        &self.macros[self.macros.len() - 1]
    }

    /// Add an existing macro.
    pub fn add(&mut self, macro_: Macro) {
        self.macros.push(macro_);
    }

    /// The macros.
    pub fn iter(&self) -> impl Iterator<Item = &Macro> {
        self.macros.iter()
    }

    /// A macro by name.
    pub fn get(&self, name: &str) -> Option<&Macro> {
        self.macros.iter().find(|macro_| macro_.name == name)
    }

    /// A macro by its command id.
    pub fn by_command(&self, command: &CommandId) -> Option<&Macro> {
        self.macros.iter().find(|macro_| &macro_.command == command)
    }

    /// Register every macro as a command in `registry`.
    pub fn register(&self, registry: &mut CommandRegistry) {
        for macro_ in &self.macros {
            registry.register(
                Command::new(macro_.command.clone(), format!("Run macro: {}", macro_.name))
                    .category("Macros")
                    .description(format!("{} recorded steps", macro_.steps.len())),
            );
        }
    }
}

/// Why a journal file could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum JournalError {
    /// The JSON is malformed.
    Json(String),
    /// The file version is newer than this crate understands.
    UnsupportedVersion(u32),
    /// The file could not be read or written.
    Io(String),
}

impl fmt::Display for JournalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid journal JSON: {error}"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported journal version {version}"),
            Self::Io(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for JournalError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_append_only_and_sequenced() {
        let mut journal = CommandJournal::new();
        journal.append_at(1.0, "file.new", None, Some("New".to_owned()));
        journal.append_with("file.save", Value::from(1), "Save");
        assert_eq!(journal.len(), 2);
        assert_eq!(journal.last().map(|entry| entry.sequence), Some(2));
        assert_eq!(journal.since(1).len(), 1);
        assert_eq!(journal.since(0).len(), 2);
        assert!(journal.last().and_then(JournalEntry::args).is_some());
    }

    #[test]
    fn a_macro_replays_the_recorded_commands() {
        let mut journal = CommandJournal::new();
        journal.append("file.new");
        journal.append("edit.type");
        let mut library = MacroLibrary::new();
        let macro_ = library.record_from(&journal, 1, "New and type").clone();
        assert_eq!(macro_.steps.len(), 1);
        let mut replayed = Vec::new();
        macro_.replay(|step| replayed.push(step.command.to_string()));
        assert_eq!(replayed, ["edit.type"]);
        assert_eq!(macro_.command.to_string(), "macro.new-and-type");
    }

    #[test]
    fn registered_macros_are_commands() {
        let mut journal = CommandJournal::new();
        journal.append("file.new");
        let mut library = MacroLibrary::new();
        library.record_from(&journal, 0, "New");
        let mut registry = CommandRegistry::new();
        library.register(&mut registry);
        assert!(registry.get(&Macro::command_id("New")).is_some());
    }

    #[test]
    fn the_journal_round_trips_and_rejects_a_newer_version() {
        let mut journal = CommandJournal::new();
        journal.append_with("file.save", Value::from("a.rs"), "Save");
        let json = journal.to_json().expect("serialize");
        let back = CommandJournal::from_json(&json).expect("parse");
        assert_eq!(back, journal);
        let newer = json.replace("\"version\": 1", "\"version\": 9");
        assert!(matches!(
            CommandJournal::from_json(&newer),
            Err(JournalError::UnsupportedVersion(9))
        ));
    }

    #[test]
    fn macros_round_trip_through_json() {
        let mut journal = CommandJournal::new();
        journal.append("file.new");
        let mut library = MacroLibrary::new();
        library.record_from(&journal, 0, "New");
        let json = serde_json::to_string(&library).expect("serialize");
        let back: MacroLibrary = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, library);
    }
}
