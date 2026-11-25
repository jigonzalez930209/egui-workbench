//! The workbook reader must return `Ok` or `Err` for any file, never panic.
//!
//! `WorkbookSource` reads from a path and picks the format by extension, so each input is
//! written to a per-process scratch file under every supported extension.
#![no_main]

use egui_workbench::data::WorkbookSource;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let directory = std::env::temp_dir().join(format!("egui-workbench-fuzz-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&directory);
    for extension in ["xlsx", "xls", "ods"] {
        let path = directory.join(format!("input.{extension}"));
        if std::fs::write(&path, bytes).is_ok() {
            let _ = WorkbookSource::open(&path);
        }
    }
});
