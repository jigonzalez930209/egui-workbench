# Benchmarks

> The criterion suites for plot decimation, a large tree, a one-million-row CSV, and palette filtering, with the recorded 0.3.1 baseline. Live page: <https://jigonzalez930209.github.io/egui-workbench/reference/benchmarks.html>

The 0.3.x baseline is recorded in
[`docs/BENCHMARKS.md`](https://github.com/jigonzalez930209/egui-workbench/blob/main/docs/BENCHMARKS.md).
It is the number later releases compare against.

## Suites

| Bench | Fixture | Records |
|---|---|---|
| `plot_decimation` | 1,000,000-point line in a 900×600 frame | CPU painting, per-pixel M4, RDP simplification |
| `large_tree` | 10,000 expanded explorer rows | Layout and paint of a large tree |
| `csv` | 1,000,000 rows × 20 columns | Opening the file and the first visible frame |
| `palette_filter` | 10,000 rows | Empty, prefix, and shortcut queries |

## Run it

```bash
cargo bench --workspace --no-run
cargo bench --bench plot_decimation
cargo bench --bench large_tree
cargo bench --bench csv -- --sample-size 10 --measurement-time 15
cargo bench --bench palette_filter
```

The raw samples are under `target/criterion`. The weekly `Benchmarks` workflow uploads
them as an artifact, so a regression has a comparable history.

## 0.3.1 baseline

| Benchmark | Time |
|---|---|
| `plot_decimation/line_1m_900x600` | 8.49 ms |
| `large_tree/10k_rows_900x600` | 38.0 ms |
| `csv/open_1m_rows_20_columns` | 1.52 s |
| `csv/first_frame_1m_rows_900x600` | 13.0 ms |
| `palette_filter/empty` | 85 ns |
| `palette_filter/prefix` | 388 µs |
| `palette_filter/shortcut` | 393 µs |

Reference machine: AMD Ryzen 9 6900HX, Linux 7.0, Rust 1.97.1, X11 under Xvfb. Compare
on the same machine; a change larger than 10 % is reviewed.

## 0.7 data probe

`cargo test --release --test data_budget -- --ignored --nocapture` reports:

| Step | Time |
|---|---|
| Open 1M × 20 CSV + type inference | 3.7 s |
| Typed payload | 152 MiB |
| Filter `column_3 > 5000000` | 17.9 ms |
| Sort one numeric column | 8.2 ms |

The typed payload is far below the 0.3 string-cell baseline. The filter sits just above a
60 Hz frame and is the scan 0.9's level-of-detail work targets.

## 0.9 engine

`cargo test --all-features --test gpu_plot` renders a line through a real headless wgpu
device, so the `gpu` feature is exercised locally by that test (the hosting CI workflow is
manual). `MinMaxPyramid` and `StreamingSeries`
cover the out-of-core and live paths. The streaming probe reports **52.7 M points/s**
(1,000,000 points in 18.9 ms), above the 1 M points/s target. The interactive 100M-point
GPU target is recorded with each tagged run.
