#![allow(missing_docs)]

//! Plot decimation baseline: paint a million-point line in a 900×600 frame.
//!
//! The frame keeps the same number of screen columns as the documentation plots, so the
//! per-pixel M4 path and the Ramer–Douglas–Peucker simplification both run. Compare a
//! change against `cargo bench --bench plot_decimation`.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use egui_workbench::Icons;
use egui_workbench::components::plot::{Plot, PlotState, Series};
use egui_workbench::egui::{self, RawInput};

fn million_points() -> Vec<[f64; 2]> {
    (0..1_000_000)
        .map(|index| {
            let t = index as f64 * 0.001;
            [t, (t * 0.7).sin() + (t * 3.1).cos() * 0.15]
        })
        .collect()
}

fn bench_plot(criterion: &mut Criterion) {
    let data = million_points();
    let mut group = criterion.benchmark_group("plot_decimation");
    group.sample_size(30);

    group.bench_function("line_1m_900x600", |bencher| {
        let ctx = egui::Context::default();
        let mut icons = Icons::new();
        let mut state = PlotState::default();
        // Warm the text and texture caches; the measurement is the plot frame.
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            let _ = Plot::new("million")
                .height(600.0)
                .series(Series::line("signal", &data))
                .show(ui, &mut icons, &mut state);
        });

        bencher.iter(|| {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                let response = Plot::new("million")
                    .height(600.0)
                    .series(Series::line("signal", black_box(&data)))
                    .show(ui, black_box(&mut icons), black_box(&mut state));
                black_box(response);
            });
            output.textures_delta.clear();
        });
    });
    group.finish();
}

criterion_group!(benches, bench_plot);
criterion_main!(benches);
