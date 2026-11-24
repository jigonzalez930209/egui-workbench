//! Streaming throughput probe. Run in release:
//!
//! `cargo test --release --test stream_budget -- --ignored --nocapture`

#![allow(clippy::expect_used)]

use std::time::Instant;

use egui_workbench::data::StreamingSeries;

#[test]
#[ignore = "streaming throughput probe; run in release"]
fn one_million_points_per_second() {
    let mut series = StreamingSeries::new(1_000_000);
    let started = Instant::now();
    for index in 0..1_000_000 {
        let x = index as f64 * 1e-6;
        series.push([x, (x * 1000.0).sin()]);
    }
    let elapsed = started.elapsed();
    let rate = 1_000_000.0 / elapsed.as_secs_f64();
    println!("streaming: {elapsed:?}; {rate:.0} points/s");
    assert!(rate >= 1_000_000.0, "only {rate:.0} points/s");
    assert_eq!(series.len(), 1_000_000);
}
