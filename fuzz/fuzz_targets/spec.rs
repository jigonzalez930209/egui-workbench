//! The host spec reader must return `Ok` or `Err` for any text, never panic.
#![no_main]

use egui_workbench_host::HostSpec;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(text) = std::str::from_utf8(bytes) {
        let _ = HostSpec::from_json(text);
    }
});
