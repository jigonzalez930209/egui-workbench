//! Versioned workspace persistence: documents, active tab, explorer, panels, theme,
//! and plot views in one atomically written file.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ThemeId;

use super::layout::Layout;

/// One open document in the workspace.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WorkspaceDocument {
    /// Stable application id.
    pub id: String,
    /// Tab title.
    pub title: String,
    /// Unsaved changes.
    pub dirty: bool,
    /// Path on disk, when the document has one.
    pub path: Option<PathBuf>,
    /// Application-specific payload (cursor, scroll offset, model data).
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl WorkspaceDocument {
    /// A document with no path and no payload.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            dirty: false,
            path: None,
            payload: serde_json::Value::Null,
        }
    }
}

/// Which panels are visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WorkspacePanels {
    /// Explorer.
    pub sidebar: bool,
    /// Right inspector.
    pub inspector: bool,
    /// Bottom output.
    pub output: bool,
    /// Status bar.
    pub status_bar: bool,
}

impl Default for WorkspacePanels {
    fn default() -> Self {
        Self {
            sidebar: true,
            inspector: false,
            output: false,
            status_bar: true,
        }
    }
}

/// A serializable plot view: the visible x and y ranges plus hidden series.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WorkspacePlotView {
    /// Visible x range.
    pub x: [f64; 2],
    /// Visible y range.
    pub y: [f64; 2],
    /// Hidden series names.
    #[serde(default)]
    pub hidden: Vec<String>,
}

impl WorkspacePlotView {
    /// A view from two ranges.
    pub fn new(x: [f64; 2], y: [f64; 2]) -> Self {
        Self {
            x,
            y,
            hidden: Vec::new(),
        }
    }
}

/// The persisted workspace.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    /// File format version.
    pub version: u32,
    /// Requested theme.
    pub theme: ThemeId,
    /// Follow the desktop light/dark preference.
    pub follow_system: bool,
    /// Reduced motion.
    pub reduced_motion: bool,
    /// Open documents, in tab order.
    pub documents: Vec<WorkspaceDocument>,
    /// Active document index (clamped on load).
    pub active: usize,
    /// Expanded explorer keys.
    pub explorer: Vec<String>,
    /// Visible panels.
    pub panels: WorkspacePanels,
    /// Left panel width.
    pub sidebar_width: f32,
    /// Right panel width.
    pub right_width: f32,
    /// Bottom panel height.
    pub bottom_height: f32,
    /// Plot views by plot id.
    pub plot_views: BTreeMap<String, WorkspacePlotView>,
    /// The current editor layout.
    pub layout: Layout,
    /// Named layouts ("Analysis", "Review") that can be switched from the palette.
    pub named_layouts: BTreeMap<String, Layout>,
    /// The name of the active named layout.
    pub active_layout: String,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            theme: ThemeId::GithubLight,
            follow_system: false,
            reduced_motion: false,
            documents: Vec::new(),
            active: 0,
            explorer: Vec::new(),
            panels: WorkspacePanels::default(),
            sidebar_width: 240.0,
            right_width: 280.0,
            bottom_height: 180.0,
            plot_views: BTreeMap::new(),
            layout: Layout::default(),
            named_layouts: BTreeMap::new(),
            active_layout: "Default".to_owned(),
        }
    }
}

impl Workspace {
    /// The current file version.
    pub const VERSION: u32 = 1;

    /// A workspace with the current version and default panel sizes.
    pub fn new() -> Self {
        Self::default()
    }

    /// A workspace with documents.
    pub fn with_documents(documents: Vec<WorkspaceDocument>) -> Self {
        Self {
            documents,
            ..Self::default()
        }
    }

    /// Add a document and return its index.
    pub fn add_document(&mut self, document: WorkspaceDocument) -> usize {
        self.documents.push(document);
        self.documents.len() - 1
    }

    /// The active document.
    pub fn active_document(&self) -> Option<&WorkspaceDocument> {
        self.documents.get(self.active)
    }

    /// Select an existing document.
    pub fn set_active(&mut self, index: usize) -> bool {
        if index < self.documents.len() {
            self.active = index;
            true
        } else {
            false
        }
    }

    /// Clamp the active index and the panel sizes after a load.
    pub fn normalize(&mut self) {
        if self.documents.is_empty() {
            self.active = 0;
        } else {
            self.active = self.active.min(self.documents.len() - 1);
        }
        self.sidebar_width = self.sidebar_width.clamp(120.0, 800.0);
        self.right_width = self.right_width.clamp(120.0, 800.0);
        self.bottom_height = self.bottom_height.clamp(80.0, 600.0);
        self.layout.normalize();
        if self.active_layout.is_empty() {
            self.active_layout = "Default".to_owned();
        }
    }

    /// Save the current layout under a name and return the name.
    pub fn save_layout(&mut self, name: impl Into<String>) -> String {
        let name = name.into();
        self.named_layouts.insert(name.clone(), self.layout.clone());
        self.active_layout = name.clone();
        name
    }

    /// Switch to a named layout. Returns whether it existed.
    pub fn switch_layout(&mut self, name: &str) -> bool {
        let Some(layout) = self.named_layouts.get(name).cloned() else {
            return false;
        };
        self.layout = layout;
        self.active_layout = name.to_owned();
        true
    }

    /// Serialize as pretty JSON.
    ///
    /// # Errors
    /// When serialization fails (it does not for this type).
    pub fn to_json(&self) -> Result<String, SessionError> {
        serde_json::to_string_pretty(self).map_err(|error| SessionError::Json(error.to_string()))
    }

    /// Parse a workspace, migrating an older version and rejecting a newer one.
    ///
    /// # Errors
    /// When the JSON is malformed or the version is newer than [`Self::VERSION`].
    pub fn from_json(json: &str) -> Result<Self, SessionError> {
        let mut value: serde_json::Value =
            serde_json::from_str(json).map_err(|error| SessionError::Json(error.to_string()))?;
        // `as u32` wrapped 2^32 + 1 to 1 and let a far newer file through.
        let raw = value.get("version").and_then(serde_json::Value::as_u64).unwrap_or(0);
        let version = u32::try_from(raw).unwrap_or(u32::MAX);
        if version > Self::VERSION {
            return Err(SessionError::UnsupportedVersion(version));
        }
        if version < 1 {
            // Version 0 had no `version` field and no plot views. `serde(default)` fills
            // every field added since; write the current version before deserializing.
            if let Some(object) = value.as_object_mut() {
                object.insert("version".to_owned(), serde_json::Value::from(Self::VERSION));
            }
        }
        let mut workspace: Self =
            serde_json::from_value(value).map_err(|error| SessionError::Json(error.to_string()))?;
        workspace.normalize();
        Ok(workspace)
    }

    /// Write the workspace atomically.
    ///
    /// # Errors
    /// When serialization or the write fails.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), SessionError> {
        let json = self.to_json()?;
        atomic_write(path.as_ref(), json.as_bytes()).map_err(SessionError::Io)
    }

    /// Read a workspace from disk.
    ///
    /// # Errors
    /// When the file cannot be read or parsed.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, SessionError> {
        let text = std::fs::read_to_string(path).map_err(|error| SessionError::Io(error.to_string()))?;
        Self::from_json(&text)
    }
}

/// Why a workspace could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SessionError {
    /// The JSON is malformed.
    Json(String),
    /// The file version is newer than this crate understands.
    UnsupportedVersion(u32),
    /// The file could not be read or written.
    Io(String),
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid workspace JSON: {error}"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported workspace version {version}"),
            Self::Io(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for SessionError {}

/// Write a file through a temporary sibling and a rename, so a crash leaves either the
/// old file or the new one, never a partial write.
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "the path has no file name".to_owned())?;
    // A hidden sibling named after the whole file name: `with_extension` made `a.json`
    // and `a.txt` share a temporary, and two writers in one process shared it too.
    let temp = path.with_file_name(format!(
        ".{name}.tmp-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let written = (|| {
        // `create_new` refuses an existing file or symlink at the temporary path.
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| error.to_string())?;
        file.write_all(bytes).map_err(|error| error.to_string())?;
        // Flush to disk before the rename, or a power cut can leave an empty file.
        file.sync_all().map_err(|error| error.to_string())
    })();
    if let Err(error) = written {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    // POSIX `rename` replaces the target atomically. Removing the old file first opened a
    // window in which a crash left no file at all.
    if let Err(error) = std::fs::rename(&temp, path) {
        // Windows refuses to rename over an existing file; fall back to remove + rename,
        // and keep the complete temporary file if that also fails.
        if path.exists() && std::fs::remove_file(path).is_ok() && std::fs::rename(&temp, path).is_ok() {
            return Ok(());
        }
        return Err(error.to_string());
    }
    Ok(())
}

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

    #[test]
    fn documents_and_panels_round_trip() {
        let mut workspace = Workspace::new();
        workspace.add_document(WorkspaceDocument::new("one", "One"));
        workspace.add_document(WorkspaceDocument::new("two", "Two"));
        workspace.set_active(1);
        workspace.theme = ThemeId::Dracula;
        workspace.explorer = vec!["src".to_owned()];
        workspace.panels.inspector = true;
        workspace
            .plot_views
            .insert("main".to_owned(), WorkspacePlotView::new([0.0, 1.0], [2.0, 3.0]));
        workspace
            .layout
            .split(crate::app::EditorGroupId(0), crate::app::SplitDirection::Horizontal);
        workspace.save_layout("Review");
        let json = workspace.to_json().expect("serialize");
        let back = Workspace::from_json(&json).expect("parse");
        assert_eq!(back, workspace);
        assert_eq!(back.active_document().map(|doc| doc.title.as_str()), Some("Two"));
        assert_eq!(back.active_layout, "Review");
        assert!(back.named_layouts.contains_key("Review"));
    }

    #[test]
    fn missing_fields_and_version_zero_migrate() {
        let json = r#"{"documents":[{"id":"a","title":"A","dirty":true,"path":null}]}"#;
        let workspace = Workspace::from_json(json).expect("migrate");
        assert_eq!(workspace.version, Workspace::VERSION);
        assert_eq!(workspace.documents.len(), 1);
        assert!(workspace.panels.sidebar);
        assert_eq!(workspace.active, 0);
    }

    #[test]
    fn a_newer_version_is_rejected() {
        let error = Workspace::from_json(r#"{"version": 9}"#).expect_err("newer");
        assert!(matches!(error, SessionError::UnsupportedVersion(9)));
    }

    #[test]
    fn normalize_clamps_active_and_sizes() {
        let mut workspace = Workspace::new();
        workspace.add_document(WorkspaceDocument::new("a", "A"));
        workspace.active = 99;
        workspace.sidebar_width = 1.0;
        workspace.right_width = 9999.0;
        workspace.bottom_height = 1.0;
        workspace.normalize();
        assert_eq!(workspace.active, 0);
        assert_eq!(workspace.sidebar_width, 120.0);
        assert_eq!(workspace.right_width, 800.0);
        assert_eq!(workspace.bottom_height, 80.0);
    }

    #[test]
    fn save_and_load_are_atomic() {
        let directory = test_directory("egui-workbench-workspace");
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("create");
        let path = directory.join("workspace.json");
        let mut workspace = Workspace::new();
        workspace.add_document(WorkspaceDocument::new("a", "A"));
        workspace.save(&path).expect("save");
        let back = Workspace::load(&path).expect("load");
        assert_eq!(back, workspace);
        let leftovers = std::fs::read_dir(&directory).expect("dir").count();
        assert_eq!(leftovers, 1, "no temporary file stays behind");
        let _ = std::fs::remove_dir_all(directory);
    }
}
