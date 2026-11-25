//! Values exchanged with extensions.
//!
//! Kept apart from the manager so each file stays under the 500-line rule.

use serde::{Deserialize, Serialize};

/// A command contributed by an extension.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandContribution {
    /// Command id.
    pub id: String,
    /// Visible title.
    pub title: String,
}

/// A panel contributed by an extension.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PanelContribution {
    /// Panel id.
    pub id: String,
    /// Visible title.
    pub title: String,
}

/// A viewer contributed by an extension.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewerContribution {
    /// Viewer id.
    pub id: String,
    /// File extensions, without a dot.
    pub extensions: Vec<String>,
}

/// A dataset name and row count exposed to an extension.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetView {
    /// Dataset name.
    pub name: String,
    /// Row count.
    pub rows: usize,
}

/// One log line from an extension.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginLog {
    /// Extension id.
    pub extension: String,
    /// Message.
    pub message: String,
}

/// A snapshot shown by the extension manager.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionInfo {
    /// Manifest id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Extension version.
    pub version: String,
    /// Whether the runtime is active.
    pub enabled: bool,
    /// Requested capabilities.
    pub requested: Vec<String>,
    /// Granted capabilities.
    pub granted: Vec<String>,
    /// Contributed commands.
    pub commands: Vec<CommandContribution>,
    /// Contributed panels.
    pub panels: Vec<PanelContribution>,
    /// Contributed viewers.
    pub viewers: Vec<ViewerContribution>,
}

/// Dataset rows as a vector, for the extension manager view.
pub fn dataset_views(datasets: &std::collections::BTreeMap<String, usize>) -> Vec<DatasetView> {
    datasets
        .iter()
        .map(|(name, rows)| DatasetView {
            name: name.clone(),
            rows: *rows,
        })
        .collect()
}
