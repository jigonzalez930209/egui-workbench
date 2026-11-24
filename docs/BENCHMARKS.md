# Benchmark baseline

The 0.3.x hardening release records a baseline so later releases can say "faster than
0.3" with a number. Criterion stores the raw samples under `target/criterion`; the
weekly `Benchmarks` workflow uploads them as an artifact.

## Reference machine

| | |
|---|---|
| CPU | AMD Ryzen 9 6900HX (16 threads) |
| OS | Linux 7.0.0-44-generic, x86_64 |
| Rust | 1.97.1 |
| Profile | `cargo bench` (release, LTO off) |
| Display | X11 under Xvfb (offscreen, 900×600 logical) |

Numbers move with the machine, the GPU driver, and the load. Compare against a fresh run
on the same machine, not across machines.

## How to run

```bash
cargo bench --workspace --no-run          # compile only, as CI does
cargo bench --bench plot_decimation       # 1M-point line
cargo bench --bench large_tree            # 10k explorer rows
cargo bench --bench csv                   # 1M rows × 20 columns
cargo bench --bench palette_filter        # 10k palette rows
```

`csv` is deliberately slow to set up; use `--sample-size 10 --measurement-time 15` for a
quick check.

## 2026-11-24 baseline (0.3.1)

| Benchmark | Time | What it measures |
|---|---|---|
| `plot_decimation/line_1m_900x600` | **8.49 ms** | Paint a million-point line, per-pixel M4 decimation and RDP simplification |
| `large_tree/10k_rows_900x600` | **38.0 ms** | Lay out and paint 10,000 expanded explorer rows (not virtualized) |
| `csv/open_1m_rows_20_columns` | **1.52 s** | Parse a 1,000,000-row, 20-column CSV into the current `String`-cell table |
| `csv/first_frame_1m_rows_900x600` | **13.0 ms** | Paint the first visible frame of that table |
| `palette_filter/empty` | **85 ns** | Empty query, first 12 rows (the cap short-circuits) |
| `palette_filter/prefix` | **388 µs** | 10,000 rows, query `command.number.09` |
| `palette_filter/shortcut` | **393 µs** | 10,000 rows, query by shortcut detail |

## What later releases promise against these numbers

- **0.4** keeps palette filtering under 1 ms on the 10k fixture while adding fuzzy
  ranking and recent commands.
- **0.7** opens, scrolls, sorts, and filters the 1M × 20 CSV inside the frame budget
  (16.6 ms) and with less memory than the 0.3 string-cell table. The 1.52 s open time is
  the thing to beat; the 13.0 ms first frame is the frame budget to hold.
- **0.9** adds GPU rendering and level-of-detail; the CPU plot at 8.49 ms is the fallback
  and the snapshot reference.
- **0.10** keeps export and the new chart families inside the same frame budget.
- **0.17** records the final numbers and checks every budget.

A regression larger than 10 % on the same machine fails review. The budget gates land in
0.9 and 0.17; until then the workflow is a recorded trend, not a hard gate.
