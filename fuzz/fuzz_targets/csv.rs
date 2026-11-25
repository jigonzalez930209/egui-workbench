//! The CSV reader must return `Ok` or `Err` for any bytes, never panic.
#![no_main]

use egui_workbench::data::{CsvOptions, parse_csv};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let _ = parse_csv("fuzz", bytes, CsvOptions::csv());
    let _ = parse_csv("fuzz", bytes, CsvOptions::tsv());
});
