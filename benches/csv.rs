#![allow(missing_docs)]

//! One-million-row CSV baseline: parse a 20-column delimited file and paint one frame.
//!
//! The 0.7 data fabric must open, scroll, sort, and filter this file inside the frame
//! budget with less memory than the current `String`-cell table. This is the 0.3.x
//! reference. Run with `cargo bench --bench csv`.

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use egui_workbench::components::viewer::FileViewer;
use egui_workbench::egui::{self, RawInput};

const ROWS: usize = 1_000_000;
const COLUMNS: usize = 20;

fn csv() -> String {
    let mut text = String::with_capacity(ROWS * COLUMNS * 6);
    for column in 0..COLUMNS {
        if column > 0 {
            text.push(',');
        }
        let _ = write!(text, "column_{column}");
    }
    text.push('\n');
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            if column > 0 {
                text.push(',');
            }
            let _ = write!(text, "{}", row * COLUMNS + column);
        }
        text.push('\n');
    }
    text
}

fn bench_csv(criterion: &mut Criterion) {
    let bytes = csv();
    let mut group = criterion.benchmark_group("csv");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));

    group.bench_function("open_1m_rows_20_columns", |bencher| {
        bencher.iter(|| {
            let viewer = FileViewer::from_bytes("million.csv", black_box(bytes.as_bytes()));
            let _ = black_box(viewer.map(|viewer| (viewer.name().to_owned(), viewer.format())));
        });
    });

    group.bench_function("first_frame_1m_rows_900x600", |bencher| {
        let ctx = egui::Context::default();
        let mut viewer = match FileViewer::from_bytes("million.csv", &bytes) {
            Ok(viewer) => viewer,
            Err(error) => {
                eprintln!("could not build the CSV fixture: {error}");
                return;
            }
        };
        bencher.iter(|| {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                viewer.show(black_box(ui));
            });
            output.textures_delta.clear();
        });
    });
    group.finish();
}

criterion_group!(benches, bench_csv);
criterion_main!(benches);
