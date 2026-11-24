//! Hot exit and crash recovery: dirty buffers stashed on a timer or on close, restored
//! with the workspace and journal after a crash.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::journal::CommandJournal;
use super::session::Workspace;

/// One dirty buffer in the stash.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StashDocument {
    /// Matches [`super::session::WorkspaceDocument::id`].
    pub id: String,
    /// Title at the time of the stash.
    pub title: String,
    /// The unsaved text.
    pub text: String,
}

impl StashDocument {
    /// A stashed buffer.
    pub fn new(id: impl Into<String>, title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            text: text.into(),
        }
    }
}

/// The versioned dirty-buffer stash.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stash {
    /// File format version.
    pub version: u32,
    /// The application clock when the stash was written.
    pub saved_at: f64,
    /// Dirty buffers.
    pub documents: Vec<StashDocument>,
}

impl Default for Stash {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            saved_at: 0.0,
            documents: Vec::new(),
        }
    }
}

impl Stash {
    /// The current file version.
    pub const VERSION: u32 = 1;

    /// An empty stash.
    pub fn new() -> Self {
        Self::default()
    }

    /// Capture dirty buffers. Buffers that are not dirty are skipped by the caller; this
    /// constructor keeps whatever it is given.
    pub fn capture(saved_at: f64, documents: impl IntoIterator<Item = StashDocument>) -> Self {
        Self {
            version: Self::VERSION,
            saved_at,
            documents: documents.into_iter().collect(),
        }
    }

    /// Whether there is anything to recover.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Serialize as pretty JSON.
    ///
    /// # Errors
    /// When serialization fails (it does not for this type).
    pub fn to_json(&self) -> Result<String, RecoveryError> {
        serde_json::to_string_pretty(self).map_err(|error| RecoveryError::Json(error.to_string()))
    }

    /// Parse a stash, rejecting a newer version.
    ///
    /// # Errors
    /// When the JSON is malformed or the version is newer.
    pub fn from_json(json: &str) -> Result<Self, RecoveryError> {
        let stash: Self = serde_json::from_str(json).map_err(|error| RecoveryError::Json(error.to_string()))?;
        if stash.version > Self::VERSION {
            return Err(RecoveryError::UnsupportedVersion(stash.version));
        }
        Ok(stash)
    }

    /// Write atomically.
    ///
    /// # Errors
    /// When serialization or the write fails.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), RecoveryError> {
        let json = self.to_json()?;
        super::session::atomic_write(path.as_ref(), json.as_bytes()).map_err(RecoveryError::Io)
    }

    /// Read from disk.
    ///
    /// # Errors
    /// When the file cannot be read or parsed.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, RecoveryError> {
        let text = std::fs::read_to_string(path).map_err(|error| RecoveryError::Io(error.to_string()))?;
        Self::from_json(&text)
    }
}

/// What was found after a crash.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct Recovery {
    /// The workspace, when the file exists and parses.
    pub workspace: Option<Workspace>,
    /// The dirty-buffer stash.
    pub stash: Option<Stash>,
    /// The journal.
    pub journal: Option<CommandJournal>,
    /// Files that were present but could not be read.
    pub warnings: Vec<String>,
}

impl Recovery {
    /// A one-line description to show the user, or `None` when nothing was recovered.
    pub fn notice(&self) -> Option<String> {
        if self.workspace.is_none() && self.stash.is_none() && self.journal.is_none() {
            return None;
        }
        let mut parts = Vec::new();
        if let Some(workspace) = &self.workspace {
            parts.push(format!("{} tab(s)", workspace.documents.len()));
        }
        if let Some(stash) = &self.stash {
            parts.push(format!("{} unsaved buffer(s)", stash.documents.len()));
        }
        if let Some(journal) = &self.journal {
            parts.push(format!("{} journal entries", journal.len()));
        }
        Some(format!("Recovered {}", parts.join(", ")))
    }
}

/// Read the workspace, stash, and journal from a session directory.
///
/// A missing file is not a warning; a present file that fails to parse is, so a single
/// corrupt file does not block the rest of the recovery.
pub fn recover(directory: impl AsRef<Path>) -> Recovery {
    let directory = directory.as_ref();
    let mut recovery = Recovery::default();

    let workspace_path = directory.join("workspace.json");
    if workspace_path.is_file() {
        match Workspace::load(&workspace_path) {
            Ok(workspace) => recovery.workspace = Some(workspace),
            Err(error) => recovery.warnings.push(format!("{}: {error}", workspace_path.display())),
        }
    }

    let stash_path = directory.join("stash.json");
    if stash_path.is_file() {
        match Stash::load(&stash_path) {
            Ok(stash) => recovery.stash = Some(stash),
            Err(error) => recovery.warnings.push(format!("{}: {error}", stash_path.display())),
        }
    }

    let journal_path = directory.join("journal.json");
    if journal_path.is_file() {
        match CommandJournal::load(&journal_path) {
            Ok(journal) => recovery.journal = Some(journal),
            Err(error) => recovery.warnings.push(format!("{}: {error}", journal_path.display())),
        }
    }

    recovery
}

/// A timer for the hot-exit stash.
///
/// Call [`Self::due`] every frame; when it returns true, write the workspace, the dirty
/// buffers, and the journal with [`Self::save`]. The close path calls `save` once more
/// regardless of the timer.
#[derive(Clone, Debug, PartialEq)]
pub struct HotExit {
    /// Seconds between automatic stashes.
    pub interval: f64,
    /// The clock of the last stash.
    pub last_saved: f64,
}

impl HotExit {
    /// A timer with a 10 s interval.
    pub fn new() -> Self {
        Self {
            interval: 10.0,
            last_saved: 0.0,
        }
    }

    /// A timer with a custom interval, clamped to at least one second.
    pub fn with_interval(interval: f64) -> Self {
        Self {
            interval: interval.max(1.0),
            last_saved: 0.0,
        }
    }

    /// Whether a stash is due at `now`.
    pub fn due(&self, now: f64) -> bool {
        now - self.last_saved >= self.interval
    }

    /// Write all three files and move the timer.
    ///
    /// # Errors
    /// When a file cannot be serialized or written.
    pub fn save(
        &mut self,
        now: f64,
        paths: &SessionPaths,
        workspace: &Workspace,
        stash: &Stash,
        journal: &CommandJournal,
    ) -> Result<(), RecoveryError> {
        workspace
            .save(paths.workspace())
            .map_err(|error| RecoveryError::Io(error.to_string()))?;
        stash.save(paths.stash())?;
        journal
            .save(paths.journal())
            .map_err(|error| RecoveryError::Io(error.to_string()))?;
        self.last_saved = now;
        Ok(())
    }
}

impl Default for HotExit {
    fn default() -> Self {
        Self::new()
    }
}

/// The files a session keeps in its directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionPaths {
    /// The session directory.
    pub directory: PathBuf,
}

impl SessionPaths {
    /// Paths under `directory`.
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// The workspace file.
    pub fn workspace(&self) -> PathBuf {
        self.directory.join("workspace.json")
    }

    /// The dirty-buffer stash.
    pub fn stash(&self) -> PathBuf {
        self.directory.join("stash.json")
    }

    /// The journal.
    pub fn journal(&self) -> PathBuf {
        self.directory.join("journal.json")
    }
}

/// Why recovery data could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecoveryError {
    /// Malformed JSON.
    Json(String),
    /// A newer file version.
    UnsupportedVersion(u32),
    /// A read or write failure.
    Io(String),
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid recovery JSON: {error}"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported recovery version {version}"),
            Self::Io(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for RecoveryError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_directory(prefix: &str) -> std::path::PathBuf {
        let thread: String = std::thread::current()
            .name()
            .unwrap_or("test")
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        std::env::temp_dir().join(format!("{prefix}-{}-{thread}", std::process::id()))
    }

    use crate::app::session::WorkspaceDocument;

    fn directory() -> PathBuf {
        test_directory("egui-workbench-recovery")
    }

    #[test]
    fn recovery_reads_workspace_stash_and_journal() {
        let directory = directory();
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("create");
        let paths = SessionPaths::new(&directory);
        let mut workspace = Workspace::new();
        workspace.add_document(WorkspaceDocument::new("a", "A"));
        workspace.save(paths.workspace()).expect("workspace");
        Stash::capture(1.0, [StashDocument::new("a", "A", "unsaved")])
            .save(paths.stash())
            .expect("stash");
        let mut journal = CommandJournal::new();
        journal.append("file.new");
        journal.save(paths.journal()).expect("journal");

        let recovery = recover(&directory);
        assert_eq!(recovery.workspace.as_ref().map(|ws| ws.documents.len()), Some(1));
        assert_eq!(recovery.stash.as_ref().map(|stash| stash.documents.len()), Some(1));
        assert_eq!(recovery.journal.as_ref().map(CommandJournal::len), Some(1));
        assert!(recovery.warnings.is_empty());
        assert!(recovery.notice().is_some());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn a_corrupt_file_is_a_warning_not_a_failure() {
        let directory = directory();
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("create");
        let paths = SessionPaths::new(&directory);
        std::fs::write(paths.stash(), b"not json").expect("write");
        let recovery = recover(&directory);
        assert!(recovery.stash.is_none());
        assert_eq!(recovery.warnings.len(), 1);
        assert!(recovery.notice().is_none());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn the_hot_exit_timer_fires_and_writes_all_files() {
        let directory = directory();
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("create");
        let paths = SessionPaths::new(&directory);
        let mut timer = HotExit::with_interval(5.0);
        assert!(timer.due(5.0));
        let workspace = Workspace::new();
        let stash = Stash::new();
        let journal = CommandJournal::new();
        timer.save(5.0, &paths, &workspace, &stash, &journal).expect("save");
        assert!(!timer.due(9.0));
        assert!(timer.due(10.0));
        assert!(paths.workspace().is_file());
        assert!(paths.stash().is_file());
        assert!(paths.journal().is_file());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn a_stash_round_trips() {
        let stash = Stash::capture(2.0, [StashDocument::new("a", "A", "text")]);
        let json = stash.to_json().expect("serialize");
        let back = Stash::from_json(&json).expect("parse");
        assert_eq!(back, stash);
        assert!(!back.is_empty());
    }
}
