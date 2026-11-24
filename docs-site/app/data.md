---
title: Data fabric
description: Typed datasets, lazy sources, connectors, the query language, DataGrid, and the viewer registry.
---

# Data fabric

0.7 puts one typed data model under tables, viewers, and plots. A `Dataset` is columnar
and typed; a `DataSource` pages it lazily; a safe query language filters it; `DataGrid`
renders it; and `ViewerRegistry` opens a file with the right viewer.

## Dataset and columns

```rust
use egui_workbench::data::{Column, Dataset, Query};

let data = Dataset::from_columns([
    Column::floats("voltage", vec![0.1, 0.4, 0.2]).with_unit("V"),
    Column::ints("cycle", vec![1, 2, 3]),
    Column::text("state", ["idle", "run", "run"]),
]);
assert_eq!(data.row_count(), 3);
assert_eq!(data.column_named("voltage").map(|column| column.unit.as_deref()), Some(Some("V")));

let query = Query::parse("voltage > 0.3 && cycle in 2..4").expect("query");
assert_eq!(query.matching_rows(&data), [1]);
```

Columns are `Float`, `Int`, `Bool`, `Text`, `Date`, `Time`, or `Category`. A validity
mask marks null cells; `unit` and `metadata` travel with the column. `Dataset::slice`,
`take`, `matching_rows`, `stats`, and `xy_pairs` cover the common table and plot paths.
`Series::from_dataset(&data, "t", "v")` builds a line from two numeric columns.

## Sources and connectors

Sources implement:

```rust
pub trait DataSource {
    fn schema(&self) -> &[ColumnSchema];
    fn row_count(&self) -> usize;
    fn read_range(&self, range: Range<usize>) -> Result<Dataset, DataError>;
    fn revision(&self) -> u64 { 0 }
    fn cancel(&self) {}
}
```

The trait is synchronous and has no async runtime dependency. Built-in sources:

| Source | Feature | Notes |
|---|---|---|
| `MemorySource` | always | A dataset in memory |
| `CsvSource` | `csv` | CSV/TSV, type inference, quoted fields |
| `WorkbookSource` | `workbook` | Every sheet of an Excel/ODS workbook through `calamine` |
| `RingBuffer` | `ring` | Fixed capacity, oldest row dropped |
| `ChannelSource` | `channel` | Snapshots from a worker thread, `try_recv` polling |
| `ArrowSource` | `arrow` | Arrow IPC files |
| `ParquetSource` | `parquet` | Parquet, Arrow-backed |
| `SqliteSource` | `sqlite` | A SQLite query |

`csv`, `workbook`, `ring`, and `channel` are in the default feature set; `arrow`,
`parquet`, and `sqlite` are opt-in so a default build stays small.

```rust
# #[cfg(feature = "csv")]
# {
use egui_workbench::data::{CsvOptions, CsvSource};
let source = CsvSource::parse("trace.csv", "t,v\n0,1\n1,2\n", CsvOptions::csv()).expect("csv");
assert_eq!(source.schema().len(), 2);
# }
```

## Query language

A query can only read the current row. It supports `==`, `!=`, `<`, `<=`, `>`, `>=`,
`&&`, `||`, `!`, parentheses, `in 2..5`, `in [1, 4]`, strings, booleans, numbers, and
`null`. Errors carry a message and a byte offset. `query.bind(&data)` resolves column
names to indices once, which is what `matching_rows` and `DataGrid` use.

```text
voltage > 0.3 && cycle in 2..5
state == "run" && !(cycle < 3)
cycle in [1, 4] || missing == null
```

## DataGrid

`DataGrid` renders typed cells with the 0.5/0.6 visual language:

- sort by any column, multi-step through the header;
- filter with the query language, search every text cell;
- hide columns, resize widths, row and cell selection;
- copy TSV, Markdown, or JSON (`Copy` events);
- header hover statistics: type, unit, count, min, max;
- virtualized rows: only the visible range is painted.

```rust
use egui_workbench::components::{DataGrid, DataGridState};
use egui_workbench::data::Dataset;

fn grid(ui: &mut egui::Ui, data: &Dataset, state: &mut DataGridState) {
    let _ = DataGrid::new(data, state).height(420.0).show(ui);
}
```

## Viewer registry

`ViewerRegistry::show(ui, &ViewerFile)` detects by extension and content:

| Family | Detection |
|---|---|
| Table | `.csv`, `.tsv`, `.tab` |
| JSON / YAML / TOML | extension, then a tree |
| Image / SVG | `.png`, `.jpg`, `.jpeg`, `.svg` |
| Markdown | `.md`, `.markdown` |
| Logs | `.log`, with All/Warnings/Errors filters |
| Code | common source extensions, monospace |
| Hex | binary or unknown bytes |
| Text | UTF-8 fallback |

`ViewerRegistry::detect` is public, so a host can show the badge before opening the
file. The table viewer reuses `DataGrid`; the registry keeps its selection and log filter
between frames.

## The one-million-row budget

A release probe in `tests/data_budget.rs` (`--ignored`) opens a 1,000,000 × 20 CSV and
measures filter and sort on the reference machine:

| Step | Time |
|---|---|
| Open + type inference | 3.7 s |
| Typed payload | 152 MiB |
| Filter (`column_3 > 5000000`) | 17.9 ms |
| Sort one column | 8.2 ms |

The typed payload is far below the 0.3 `Vec<String>` cell baseline; the filter is just
above one 60 Hz frame and is the next thing to tune (0.9's level-of-detail work includes
the same column scan). Run it with:

```bash
cargo test --release --test data_budget -- --ignored --nocapture
```
