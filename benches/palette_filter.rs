#![allow(missing_docs)]

//! Palette filtering baseline: rank and take 12 rows from 10,000 commands.
//!
//! The 0.4 fuzzy palette must stay inside this budget. Run with
//! `cargo bench --bench palette_filter`.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use egui_workbench::{PaletteItem, filter_palette};

fn items() -> Vec<PaletteItem<usize>> {
    (0..10_000)
        .map(|index| PaletteItem {
            label: format!("command.number.{index:05}"),
            detail: format!("Ctrl+Alt+Shift+{index}"),
            value: index,
        })
        .collect()
}

fn bench_palette(criterion: &mut Criterion) {
    let items = items();
    let mut group = criterion.benchmark_group("palette_filter");

    for (name, query) in [
        ("empty", ""),
        ("prefix", "command.number.09"),
        ("shortcut", "ctrl+alt+shift+999"),
    ] {
        group.bench_function(name, |bencher| {
            bencher.iter(|| {
                let matches = filter_palette(black_box(&items), black_box(query));
                black_box(matches.len());
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_palette);
criterion_main!(benches);
