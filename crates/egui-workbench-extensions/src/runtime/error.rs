//! Extension failures.
//!
//! Kept apart from the manager so each file stays under the 500-line rule.

use crate::manifest::ManifestError;

/// Why an extension operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExtensionError {
    /// The manifest is invalid.
    Manifest(ManifestError),
    /// I/O failed.
    Io(String),
    /// The id is already installed.
    Duplicate(String),
    /// The extension is unknown.
    Unknown(String),
    /// The checksum does not match.
    Checksum {
        /// Manifest value.
        expected: String,
        /// Module value.
        actual: String,
    },
    /// The manifest carries a signature that no verifier accepted.
    Signature(String),
    /// Required capabilities were not granted.
    MissingCapabilities(Vec<String>),
    /// The module trapped or is malformed.
    Runtime(String),
    /// `init` returned a non-zero code.
    Init(i32),
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manifest(error) => write!(formatter, "{error}"),
            Self::Io(error) => formatter.write_str(error),
            Self::Duplicate(id) => write!(formatter, "`{id}` is already installed"),
            Self::Unknown(id) => write!(formatter, "`{id}` is not installed"),
            Self::Checksum { expected, actual } => write!(formatter, "checksum {actual} does not match {expected}"),
            Self::Signature(reason) => write!(formatter, "signature check failed: {reason}"),
            Self::MissingCapabilities(capabilities) => write!(formatter, "missing capabilities: {capabilities:?}"),
            Self::Runtime(error) => formatter.write_str(error),
            Self::Init(code) => write!(formatter, "init returned {code}"),
        }
    }
}

impl std::error::Error for ExtensionError {}
