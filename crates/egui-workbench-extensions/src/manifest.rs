//! The versioned extension manifest.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The API range this extension targets.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiRange {
    /// Minimum host API version, such as `0.15`.
    pub min: String,
    /// Maximum host API version, exclusive.
    pub max: String,
}

impl ApiRange {
    /// A range for one minor line.
    pub fn line(version: &str) -> Self {
        let mut parts = version.split('.');
        let major = parts.next().unwrap_or("0");
        let minor = parts.next().unwrap_or("0").parse::<u64>().unwrap_or(0);
        Self {
            min: format!("{major}.{minor}"),
            max: format!("{major}.{}", minor + 1),
        }
    }

    /// Whether this host (`HOST_API_VERSION`) is inside the range. Comparison is major
    /// and minor only; the host API is not patch-sensitive.
    pub fn allows(&self, version: &str) -> bool {
        // An unparsable bound used to read as `0.0`, which made `min: "abc"` accept any host.
        [version, &self.min, &self.max].iter().all(|text| well_formed(text))
            && compare(version, &self.min) >= 0
            && compare(version, &self.max) < 0
    }
}

/// `major.minor` with optional further numeric parts.
fn well_formed(version: &str) -> bool {
    let mut parts = version.split('.');
    parts.clone().count() >= 2 && parts.all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn compare(left: &str, right: &str) -> i32 {
    let mut left = left.split('.').map(|part| part.parse::<u64>().unwrap_or(0));
    let mut right = right.split('.').map(|part| part.parse::<u64>().unwrap_or(0));
    for _ in 0..2 {
        let a = left.next().unwrap_or(0);
        let b = right.next().unwrap_or(0);
        if a != b {
            return if a < b { -1 } else { 1 };
        }
    }
    0
}

/// A module path that stays inside the extension directory on every platform.
///
/// `Path::components` does not treat a Windows drive or UNC prefix as a root on Unix, so
/// the separators and drive colon are rejected explicitly before the component walk.
fn entry_is_safe(entry: &str) -> bool {
    if entry.is_empty() || entry.contains('\\') || entry.contains(':') {
        return false;
    }
    Path::new(entry)
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
}

/// A capability requested by an extension.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Capability {
    /// Contribute commands.
    Commands,
    /// Contribute panels.
    Panels,
    /// Contribute viewers.
    Viewers,
    /// Contribute themes.
    Themes,
    /// Contribute keybindings.
    Keybindings,
    /// Store settings.
    Settings,
    /// Contribute plot layers.
    ChartLayers,
    /// Read files under a path prefix.
    FilesystemRead {
        /// The allowed prefix.
        prefix: String,
    },
    /// Read dataset metadata.
    Data,
    /// Read and write the selection bus.
    Selection,
}

impl Capability {
    /// Parse the manifest spelling (`commands`, `filesystem:read:notes`).
    ///
    /// # Errors
    /// When the spelling is not known.
    pub fn parse(text: &str) -> Result<Self, ManifestError> {
        let value = match text {
            "commands" => Self::Commands,
            "panels" => Self::Panels,
            "viewers" => Self::Viewers,
            "themes" => Self::Themes,
            "keybindings" => Self::Keybindings,
            "settings" => Self::Settings,
            "chart-layers" => Self::ChartLayers,
            "data" => Self::Data,
            "selection" => Self::Selection,
            other => {
                if let Some(prefix) = other.strip_prefix("filesystem:read:") {
                    // An empty prefix names nothing; reject it instead of granting a
                    // capability that can never match (or, resolved later, the working dir).
                    if prefix.is_empty() {
                        return Err(ManifestError::Capability(other.to_owned()));
                    }
                    Self::FilesystemRead {
                        prefix: prefix.to_owned(),
                    }
                } else {
                    return Err(ManifestError::Capability(other.to_owned()));
                }
            }
        };
        Ok(value)
    }
}

/// The manifest on disk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionManifest {
    /// Stable, reverse-DNS id.
    pub id: String,
    /// Extension version.
    pub version: String,
    /// Display name.
    pub name: String,
    /// Host API range.
    pub api: ApiRange,
    /// Module path, relative to the manifest.
    pub entry: String,
    /// Optional `sha256:` checksum of the module.
    pub checksum: Option<String>,
    /// Optional signature over the module. A manager refuses a signed manifest unless the
    /// application installed a verifier with `ExtensionManager::with_signature_verifier`.
    pub signature: Option<String>,
    /// Capabilities, in manifest spelling.
    #[serde(default)]
    pub capabilities: BTreeSet<String>,
    /// Optional author.
    pub author: Option<String>,
    /// Optional description.
    pub description: Option<String>,
}

impl ExtensionManifest {
    /// Read and validate `extension.json` in `directory`.
    ///
    /// # Errors
    /// When the file is missing, malformed, or the id/version/entry is invalid.
    pub fn load(directory: impl Into<PathBuf>) -> Result<(Self, PathBuf), ManifestError> {
        let directory = directory.into();
        let path = directory.join("extension.json");
        let text = std::fs::read_to_string(&path).map_err(|error| ManifestError::Io(error.to_string()))?;
        let manifest: Self = serde_json::from_str(&text).map_err(|error| ManifestError::Json(error.to_string()))?;
        manifest.validate()?;
        Ok((manifest, directory))
    }

    /// Validate the id, version, entry, API range, and capabilities.
    ///
    /// # Errors
    /// When a field is invalid.
    pub fn validate(&self) -> Result<(), ManifestError> {
        // The id names directories and log lines: keep it to a plain reverse-DNS spelling.
        if self.id.is_empty()
            || self.id.len() > 128
            || self.id.starts_with('.')
            || !self
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        {
            return Err(ManifestError::Id(self.id.clone()));
        }
        if self.version.split('.').count() < 2 || self.version.split('.').any(|part| part.parse::<u64>().is_err()) {
            return Err(ManifestError::Version(self.version.clone()));
        }
        if !entry_is_safe(&self.entry) {
            return Err(ManifestError::Entry(self.entry.clone()));
        }
        if !self.api.allows(crate::HOST_API_VERSION) {
            return Err(ManifestError::Api {
                min: self.api.min.clone(),
                max: self.api.max.clone(),
            });
        }
        for capability in &self.capabilities {
            Capability::parse(capability)?;
        }
        Ok(())
    }

    /// Capabilities as parsed values.
    ///
    /// # Errors
    /// When a spelling is invalid.
    pub fn parsed_capabilities(&self) -> Result<BTreeSet<Capability>, ManifestError> {
        self.capabilities.iter().map(|text| Capability::parse(text)).collect()
    }

    /// The module path.
    pub fn module_path(&self, directory: &std::path::Path) -> PathBuf {
        directory.join(&self.entry)
    }
}

/// Why a manifest is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestError {
    /// I/O failed.
    Io(String),
    /// JSON failed.
    Json(String),
    /// The id is invalid.
    Id(String),
    /// The version is invalid.
    Version(String),
    /// The entry path escapes the directory.
    Entry(String),
    /// The host API range does not include this host.
    Api { min: String, max: String },
    /// A capability spelling is unknown.
    Capability(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => formatter.write_str(error),
            Self::Json(error) => formatter.write_str(error),
            Self::Id(id) => write!(formatter, "invalid extension id `{id}`"),
            Self::Version(version) => write!(formatter, "invalid extension version `{version}`"),
            Self::Entry(entry) => write!(formatter, "invalid entry `{entry}`"),
            Self::Api { min, max } => write!(
                formatter,
                "host API {} is outside {min}..{max}",
                crate::HOST_API_VERSION
            ),
            Self::Capability(capability) => write!(formatter, "unknown capability `{capability}`"),
        }
    }
}

impl std::error::Error for ManifestError {}
