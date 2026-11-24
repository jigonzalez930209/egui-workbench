//! WebAssembly extensions for egui-workbench.
//!
//! An extension is a directory with a versioned `extension.json` manifest and a `.wasm`
//! module. The manifest declares the host API range and the capabilities it asks for.
//! The runtime is enabled by the default `wasm` feature; an application that never loads
//! an extension can build with `default-features = false` and pay for none of it.

pub mod manifest;
#[cfg(feature = "wasm")]
pub mod runtime;
pub mod ui;

pub use manifest::{Capability, ExtensionManifest, ManifestError};
#[cfg(feature = "wasm")]
pub use runtime::{
    CommandContribution, DatasetView, ExtensionError, ExtensionInfo, ExtensionManager, PanelContribution, PluginLog,
    ViewerContribution,
};
pub use ui::{ExtensionManagerAction, ExtensionManagerState, show_extension_manager};

/// The host API version implemented by this crate.
pub const HOST_API_VERSION: &str = "0.15";
