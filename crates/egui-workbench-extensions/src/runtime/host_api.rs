//! Host functions imported by extension modules.
//!
//! Kept apart from the manager so each file stays under the 500-line rule.

use wasmi::{Caller, Extern, Linker, Memory};

use super::{
    CommandContribution, ExtensionError, HostState, MAX_CONTRIBUTIONS, MAX_LOGS, PanelContribution, PluginLog,
    ViewerContribution,
};
use crate::manifest::Capability;

/// Register every `workbench.*` import on `linker`.
pub(super) fn link(linker: &mut Linker<HostState>) -> Result<(), ExtensionError> {
    linker
        .func_wrap(
            "workbench",
            "host_register",
            |mut caller: Caller<'_, HostState>, kind: i32, ptr: i32, len: i32| -> i32 {
                let Some(memory) = memory(&caller) else {
                    return -1;
                };
                let Some(text) = read_string(&memory, &caller, ptr, len) else {
                    return -2;
                };
                let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
                    return -3;
                };
                let state = caller.data_mut();
                if state.commands.len() + state.panels.len() + state.viewers.len() >= MAX_CONTRIBUTIONS {
                    return -6;
                }
                let capability = match kind {
                    0 => Capability::Commands,
                    1 => Capability::Panels,
                    2 => Capability::Viewers,
                    _ => return -4,
                };
                if !state.granted.contains(&capability) {
                    state.deny(format!("{capability:?}"));
                    return -5;
                }
                let id = value
                    .get("id")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_owned();
                if id.is_empty() {
                    return -7;
                }
                let title = value
                    .get("title")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_owned();
                match kind {
                    0 => state.commands.push(CommandContribution { id, title }),
                    1 => state.panels.push(PanelContribution { id, title }),
                    2 => {
                        let extensions = value
                            .get("extensions")
                            .and_then(|value| value.as_array())
                            .map(|values| {
                                values
                                    .iter()
                                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                                    .collect()
                            })
                            .unwrap_or_default();
                        state.viewers.push(ViewerContribution { id, extensions });
                    }
                    _ => {}
                }
                0
            },
        )
        .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
    linker
        .func_wrap(
            "workbench",
            "host_log",
            |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
                let Some(memory) = memory(&caller) else {
                    return;
                };
                let Some(message) = read_string(&memory, &caller, ptr, len) else {
                    return;
                };
                let extension = caller.data().extension.clone();
                let logs = &mut caller.data_mut().logs;
                if logs.len() < MAX_LOGS {
                    logs.push(PluginLog { extension, message });
                }
            },
        )
        .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
    linker
        .func_wrap(
            "workbench",
            "host_read_file",
            |mut caller: Caller<'_, HostState>, path_ptr: i32, path_len: i32, out_ptr: i32, out_cap: i32| -> i32 {
                let Some(memory) = memory(&caller) else {
                    return -1;
                };
                let Some(path) = read_string(&memory, &caller, path_ptr, path_len) else {
                    return -2;
                };
                let granted = caller
                    .data()
                    .granted
                    .iter()
                    .filter_map(|capability| match capability {
                        Capability::FilesystemRead { prefix } => Some(prefix.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let Ok(path) = std::fs::canonicalize(&path) else {
                    return -4;
                };
                let allowed = granted.iter().any(|prefix| {
                    std::fs::canonicalize(prefix)
                        .map(|prefix| path.starts_with(prefix))
                        .unwrap_or(false)
                });
                if !allowed {
                    caller.data_mut().deny(format!("read {}", path.display()));
                    return -3;
                }
                let Ok(metadata) = std::fs::metadata(&path) else {
                    return -4;
                };
                // A FIFO or a device node reports a zero length and never ends.
                if !metadata.is_file() {
                    return -4;
                }
                let capacity = out_cap.max(0) as u64;
                if metadata.len() > capacity {
                    return -5;
                }
                // Read at most one byte past the capacity: the file may have grown since
                // `metadata`, and the cap must hold for what is read, not what was seen.
                let mut bytes = Vec::new();
                let Ok(file) = std::fs::File::open(&path) else {
                    return -4;
                };
                if std::io::Read::read_to_end(&mut std::io::Read::take(file, capacity + 1), &mut bytes).is_err() {
                    return -4;
                }
                if bytes.len() as u64 > capacity {
                    return -5;
                }
                if memory.write(&mut caller, out_ptr as usize, &bytes).is_err() {
                    return -6;
                }
                bytes.len() as i32
            },
        )
        .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
    linker
        .func_wrap(
            "workbench",
            "host_dataset_rows",
            |caller: Caller<'_, HostState>, ptr: i32, len: i32| -> i32 {
                let Some(memory) = memory(&caller) else {
                    return -1;
                };
                let Some(name) = read_string(&memory, &caller, ptr, len) else {
                    return -2;
                };
                if !caller.data().granted.contains(&Capability::Data) {
                    return -3;
                }
                caller
                    .data()
                    .datasets
                    .get(&name)
                    .map(|rows| i32::try_from(*rows).unwrap_or(i32::MAX))
                    .unwrap_or(-4)
            },
        )
        .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
    linker
        .func_wrap(
            "workbench",
            "host_selection",
            |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
                let Some(memory) = memory(&caller) else {
                    return;
                };
                let Some(value) = read_string(&memory, &caller, ptr, len) else {
                    return;
                };
                if caller.data().granted.contains(&Capability::Selection) {
                    caller.data_mut().selection = value;
                } else {
                    caller.data_mut().deny("selection".to_owned());
                }
            },
        )
        .map_err(|error| ExtensionError::Runtime(error.to_string()))?;
    Ok(())
}

fn memory(caller: &Caller<'_, HostState>) -> Option<Memory> {
    match caller.get_export("memory") {
        Some(Extern::Memory(memory)) => Some(memory),
        _ => None,
    }
}

fn read_string(memory: &Memory, caller: &Caller<'_, HostState>, ptr: i32, len: i32) -> Option<String> {
    if ptr < 0 || len < 0 {
        return None;
    }
    let (start, length) = (ptr as usize, len as usize);
    // Check the range before allocating: `len` comes from the module and can be 2 GiB.
    if start.checked_add(length)? > memory.data_size(caller) {
        return None;
    }
    let mut buffer = vec![0u8; length];
    memory.read(caller, start, &mut buffer).ok()?;
    String::from_utf8(buffer).ok()
}
