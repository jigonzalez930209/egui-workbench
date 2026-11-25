//! The sandboxed WebAssembly runtime and extension manager.
//!
//! Plugins are core WebAssembly modules with a deliberately small ABI:
//!
//! - import `workbench.host_register(kind, ptr, len)` with JSON
//! - import `workbench.host_log(ptr, len)`
//! - import `workbench.host_read_file(path_ptr, path_len, out_ptr, out_cap) -> i32`
//! - import `workbench.host_dataset_rows(name_ptr, name_len) -> i32`
//! - import `workbench.host_selection(ptr, len)`
//! - export `memory`, `init() -> i32`, and `on_command(ptr, len) -> i32`
//!
//! No WASI is linked. A module can only touch the filesystem through a granted
//! `filesystem:read:<prefix>` capability and only see a dataset row count.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use wasmi::{Config, Engine, Instance, Linker, Module, Store, StoreLimits, StoreLimitsBuilder, TypedFunc};

/// Fuel granted to one `init` or `on_command` call. A module that loops forever runs out
/// and traps instead of freezing the host thread.
const FUEL_PER_CALL: u64 = 1_000_000;

/// Linear memory an extension may grow to.
const MEMORY_LIMIT: usize = 64 << 20;

use crate::manifest::{Capability, ExtensionManifest};

mod error;
mod host_api;
mod types;

pub use error::ExtensionError;
pub use types::{
    CommandContribution, DatasetView, ExtensionInfo, PanelContribution, PluginLog, ViewerContribution, dataset_views,
};

#[derive(Default)]
struct HostState {
    extension: String,
    granted: BTreeSet<Capability>,
    commands: Vec<CommandContribution>,
    panels: Vec<PanelContribution>,
    viewers: Vec<ViewerContribution>,
    logs: Vec<PluginLog>,
    denied: Vec<String>,
    datasets: BTreeMap<String, usize>,
    selection: String,
    limiter: StoreLimits,
}

struct PluginRuntime {
    store: Store<HostState>,
    instance: Instance,
    init: TypedFunc<(), i32>,
    on_command: TypedFunc<(i32, i32), i32>,
}

struct Installed {
    manifest: ExtensionManifest,
    directory: PathBuf,
    granted: BTreeSet<Capability>,
    enabled: bool,
    runtime: Option<PluginRuntime>,
}

/// The extension manager and its installed set.
pub struct ExtensionManager {
    root: PathBuf,
    extensions: BTreeMap<String, Installed>,
    datasets: BTreeMap<String, usize>,
    selection: String,
    logs: Vec<PluginLog>,
    denied: Vec<String>,
}

impl ExtensionManager {
    /// A manager rooted at `directory` (created when missing).
    ///
    /// # Errors
    /// When the directory cannot be created.
    pub fn new(directory: impl Into<PathBuf>) -> Result<Self, ExtensionError> {
        let root = directory.into();
        std::fs::create_dir_all(&root).map_err(|error| ExtensionError::Io(error.to_string()))?;
        Ok(Self {
            root,
            extensions: BTreeMap::new(),
            datasets: BTreeMap::new(),
            selection: String::new(),
            logs: Vec::new(),
            denied: Vec::new(),
        })
    }

    /// Install a directory containing `extension.json` and the module.
    ///
    /// # Errors
    /// When the manifest, checksum, or module is invalid, or the id already exists.
    pub fn install(&mut self, directory: impl Into<PathBuf>) -> Result<ExtensionInfo, ExtensionError> {
        let (manifest, directory) = ExtensionManifest::load(directory).map_err(ExtensionError::Manifest)?;
        if self.extensions.contains_key(&manifest.id) {
            return Err(ExtensionError::Duplicate(manifest.id));
        }
        self.verify_checksum(&manifest, &directory)?;
        let info = ExtensionInfo {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            enabled: false,
            requested: manifest.capabilities.iter().cloned().collect(),
            granted: Vec::new(),
            commands: Vec::new(),
            panels: Vec::new(),
            viewers: Vec::new(),
        };
        self.extensions.insert(
            manifest.id.clone(),
            Installed {
                manifest,
                directory,
                granted: BTreeSet::new(),
                enabled: false,
                runtime: None,
            },
        );
        self.logs.push(PluginLog {
            extension: info.id.clone(),
            message: "installed".to_owned(),
        });
        Ok(info)
    }

    /// Grant a capability. The user interface should show this before enabling.
    ///
    /// # Errors
    /// When the extension is unknown or the capability spelling is invalid.
    pub fn grant(&mut self, id: &str, capability: &str) -> Result<(), ExtensionError> {
        let capability = Capability::parse(capability).map_err(ExtensionError::Manifest)?;
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::Unknown(id.to_owned()))?;
        extension.granted.insert(capability);
        if extension.enabled {
            self.rebuild(id)?;
        }
        Ok(())
    }

    /// Enable an installed extension and run its `init` export.
    ///
    /// # Errors
    /// When the extension is unknown, a required capability is not granted, or the module
    /// traps.
    pub fn enable(&mut self, id: &str) -> Result<ExtensionInfo, ExtensionError> {
        let extension = self
            .extensions
            .get(id)
            .ok_or_else(|| ExtensionError::Unknown(id.to_owned()))?;
        let required = extension
            .manifest
            .parsed_capabilities()
            .map_err(ExtensionError::Manifest)?;
        let missing = required
            .difference(&extension.granted)
            .cloned()
            .collect::<BTreeSet<_>>();
        if !missing.is_empty() {
            return Err(ExtensionError::MissingCapabilities(
                missing.iter().map(|capability| format!("{capability:?}")).collect(),
            ));
        }
        self.rebuild(id)?;
        self.extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::Unknown(id.to_owned()))?
            .enabled = true;
        self.info(id).ok_or_else(|| ExtensionError::Unknown(id.to_owned()))
    }

    /// Disable an extension and drop its runtime.
    ///
    /// # Errors
    /// When the extension is unknown.
    pub fn disable(&mut self, id: &str) -> Result<(), ExtensionError> {
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::Unknown(id.to_owned()))?;
        extension.enabled = false;
        extension.runtime = None;
        Ok(())
    }

    /// Remove an installed extension. The files stay on disk.
    ///
    /// # Errors
    /// When the extension is unknown.
    pub fn remove(&mut self, id: &str) -> Result<(), ExtensionError> {
        self.extensions
            .remove(id)
            .ok_or_else(|| ExtensionError::Unknown(id.to_owned()))?;
        Ok(())
    }

    /// Rebuild the runtime from the module on disk. Use this after a build.
    ///
    /// # Errors
    /// When the extension is unknown or the module fails.
    pub fn reload(&mut self, id: &str) -> Result<ExtensionInfo, ExtensionError> {
        let enabled = self
            .extensions
            .get(id)
            .map(|extension| extension.enabled)
            .unwrap_or(false);
        if enabled {
            self.rebuild(id)?;
        }
        self.info(id).ok_or_else(|| ExtensionError::Unknown(id.to_owned()))
    }

    /// A snapshot of every installed extension.
    pub fn list(&self) -> Vec<ExtensionInfo> {
        self.extensions.keys().filter_map(|id| self.info(id)).collect()
    }

    /// One extension snapshot.
    pub fn info(&self, id: &str) -> Option<ExtensionInfo> {
        let extension = self.extensions.get(id)?;
        let (commands, panels, viewers) = extension
            .runtime
            .as_ref()
            .map(|runtime| {
                (
                    runtime.store.data().commands.clone(),
                    runtime.store.data().panels.clone(),
                    runtime.store.data().viewers.clone(),
                )
            })
            .unwrap_or_default();
        Some(ExtensionInfo {
            id: extension.manifest.id.clone(),
            name: extension.manifest.name.clone(),
            version: extension.manifest.version.clone(),
            enabled: extension.enabled,
            requested: extension.manifest.capabilities.iter().cloned().collect(),
            granted: extension
                .granted
                .iter()
                .map(|capability| format!("{capability:?}"))
                .collect(),
            commands,
            panels,
            viewers,
        })
    }

    /// Run a command in an enabled extension. Returns the exit code reported by the
    /// module, or `None` when no extension contributed that id.
    ///
    /// # Errors
    /// When the module traps.
    pub fn invoke(&mut self, command: &str) -> Result<Option<i32>, ExtensionError> {
        for extension in self.extensions.values_mut() {
            if !extension.enabled {
                continue;
            }
            let Some(runtime) = &mut extension.runtime else {
                continue;
            };
            if !runtime.store.data().commands.iter().any(|entry| entry.id == command) {
                continue;
            }
            let memory = runtime
                .instance
                .get_memory(&runtime.store, "memory")
                .ok_or_else(|| ExtensionError::Runtime("the module does not export memory".to_owned()))?;
            let bytes = command.as_bytes();
            let offset = 1024usize;
            if memory.data_size(&runtime.store) < offset + bytes.len() {
                return Err(ExtensionError::Runtime("the module memory is too small".to_owned()));
            }
            memory
                .write(&mut runtime.store, offset, bytes)
                .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
            runtime
                .store
                .set_fuel(FUEL_PER_CALL)
                .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
            let code = runtime
                .on_command
                .call(&mut runtime.store, (offset as i32, bytes.len() as i32))
                .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
            self.logs.extend(runtime.store.data().logs.clone());
            self.denied.extend(runtime.store.data().denied.clone());
            return Ok(Some(code));
        }
        Ok(None)
    }

    /// The names and row counts an extension may inspect.
    pub fn set_datasets(&mut self, datasets: BTreeMap<String, usize>) {
        self.datasets = datasets;
    }

    /// The application's selection bus value.
    pub fn set_selection(&mut self, selection: impl Into<String>) {
        self.selection = selection.into();
    }

    /// Logs collected from extensions.
    pub fn logs(&self) -> &[PluginLog] {
        &self.logs
    }

    /// Host API denials.
    pub fn denied(&self) -> &[String] {
        &self.denied
    }

    /// The directory that owns installed extensions.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn rebuild(&mut self, id: &str) -> Result<(), ExtensionError> {
        let extension = self
            .extensions
            .get(id)
            .ok_or_else(|| ExtensionError::Unknown(id.to_owned()))?;
        self.verify_checksum(&extension.manifest, &extension.directory)?;
        let mut runtime = PluginRuntime::new(
            &extension.manifest,
            &extension.directory,
            extension.granted.clone(),
            self.datasets.clone(),
            self.selection.clone(),
        )?;
        runtime.init()?;
        self.logs.extend(runtime.store.data().logs.clone());
        self.denied.extend(runtime.store.data().denied.clone());
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::Unknown(id.to_owned()))?;
        extension.runtime = Some(runtime);
        Ok(())
    }

    fn verify_checksum(&self, manifest: &ExtensionManifest, directory: &Path) -> Result<(), ExtensionError> {
        let Some(expected) = &manifest.checksum else {
            return Ok(());
        };
        let path = manifest.module_path(directory);
        let bytes = std::fs::read(&path).map_err(|error| ExtensionError::Io(error.to_string()))?;
        let actual = format!("sha256:{}", hex(&Sha256::digest(&bytes)));
        if actual != *expected {
            return Err(ExtensionError::Checksum {
                expected: expected.clone(),
                actual,
            });
        }
        Ok(())
    }
}

impl PluginRuntime {
    fn new(
        manifest: &ExtensionManifest,
        directory: &Path,
        granted: BTreeSet<Capability>,
        datasets: BTreeMap<String, usize>,
        selection: String,
    ) -> Result<Self, ExtensionError> {
        let path = manifest.module_path(directory);
        let bytes = std::fs::read(&path).map_err(|error| ExtensionError::Io(error.to_string()))?;
        let mut config = Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, &bytes[..]).map_err(|error| ExtensionError::Runtime(error.to_string()))?;
        let state = HostState {
            extension: manifest.id.clone(),
            granted,
            datasets,
            selection,
            limiter: StoreLimitsBuilder::new().memory_size(MEMORY_LIMIT).build(),
            ..HostState::default()
        };
        let mut store = Store::new(&engine, state);
        store.limiter(|state| &mut state.limiter);
        store
            .set_fuel(FUEL_PER_CALL)
            .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
        let mut linker = Linker::new(&engine);
        host_api::link(&mut linker)?;
        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
        let init = instance
            .get_typed_func::<(), i32>(&store, "init")
            .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
        let on_command = instance
            .get_typed_func::<(i32, i32), i32>(&store, "on_command")
            .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
        Ok(Self {
            store,
            instance,
            init,
            on_command,
        })
    }

    fn init(&mut self) -> Result<(), ExtensionError> {
        self.store
            .set_fuel(FUEL_PER_CALL)
            .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
        let code = self
            .init
            .call(&mut self.store, ())
            .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
        if code != 0 {
            return Err(ExtensionError::Init(code));
        }
        Ok(())
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}
