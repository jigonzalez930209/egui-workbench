#![allow(missing_docs)]

//! Large-tree baseline: lay out and paint 10,000 expanded explorer rows.
//!
//! The explorer is not virtualized: it paints every visible row, so this number is the
//! budget that 0.6 docking and 0.7 data work must keep. Compare with
//! `cargo bench --bench large_tree`.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use egui_workbench::egui::{self, RawInput};
use egui_workbench::{Icons, RowIcon, TreeNode, TreeState, TreeView, show_tree};

fn nodes() -> Vec<TreeNode<usize>> {
    (0..10_000)
        .map(|index| {
            TreeNode::leaf(
                index,
                format!("chapter-{index:05}.md"),
                RowIcon::Glyph("file".to_owned()),
            )
        })
        .collect()
}

fn bench_tree(criterion: &mut Criterion) {
    let nodes = nodes();
    let mut group = criterion.benchmark_group("large_tree");
    group.sample_size(20);

    group.bench_function("10k_rows_900x600", |bencher| {
        let ctx = egui::Context::default();
        let mut icons = Icons::new();
        let mut state = TreeState::default();
        let view = TreeView {
            id_salt: "bench_tree",
            active: None,
            light: true,
        };

        bencher.iter(|| {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                let events = show_tree(
                    ui,
                    black_box(&mut icons),
                    black_box(&nodes),
                    &mut state,
                    &view,
                    |_, _, _| {},
                );
                black_box(events);
            });
            output.textures_delta.clear();
        });
    });
    group.finish();
}

criterion_group!(benches, bench_tree);
criterion_main!(benches);
