//! The 0.7 one-million-row budget probe. Run in release:
//!
//! `cargo test --release --test data_budget -- --ignored --nocapture`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Instant;

use egui_workbench::components::{DataGridState, SortDirection};
use egui_workbench::data::{CsvOptions, CsvSource, DataSource};

const ROWS: usize = 1_000_000;
const COLUMNS: usize = 20;

fn csv() -> String {
    let mut text = String::with_capacity(ROWS * COLUMNS * 6);
    for column in 0..COLUMNS {
        if column > 0 {
            text.push(',');
        }
        text.push_str(&format!("column_{column}"));
    }
    text.push('\n');
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            if column > 0 {
                text.push(',');
            }
            text.push_str(&format!("{}", row * COLUMNS + column));
        }
        text.push('\n');
    }
    text
}

#[test]
#[ignore = "one-million-row budget probe; run in release"]
fn one_million_rows_open_filter_and_sort() {
    let bytes = csv();
    let started = Instant::now();
    let source = CsvSource::parse("million.csv", &bytes, CsvOptions::csv()).expect("parse");
    let open = started.elapsed();
    assert_eq!(source.row_count(), ROWS);
    assert_eq!(source.schema().len(), COLUMNS);

    let data = source.data();
    let string_cells = (ROWS * COLUMNS) as u64;
    let typed_bytes: u64 = data.columns().iter().map(|column| column.len() as u64 * 8).sum();
    println!(
        "open: {open:?}; rows: {}; typed payload: {} MiB for {} cells (string-cell Vec<String> baseline is much larger)",
        data.row_count(),
        typed_bytes / (1024 * 1024),
        string_cells
    );

    let mut state = DataGridState {
        query: "column_3 > 5000000".to_owned(),
        ..DataGridState::default()
    };
    let started = Instant::now();
    let filtered = state.visible_rows(data).expect("filter");
    let filter = started.elapsed();
    assert!(!filtered.is_empty());
    println!("filter: {filter:?} ({} rows)", filtered.len());

    state.query.clear();
    state.sort = Some((0, SortDirection::Descending));
    let started = Instant::now();
    let sorted = state.visible_rows(data).expect("sort");
    let sort = started.elapsed();
    assert_eq!(sorted.len(), ROWS);
    println!("sort: {sort:?}; first row key: {}", data.cell(sorted[0], 0).to_text());

    assert!(open.as_secs_f64() < 20.0, "open took {open:?}");
    assert!(filter.as_secs_f64() < 2.0, "filter took {filter:?}");
    assert!(sort.as_secs_f64() < 2.0, "sort took {sort:?}");
}
