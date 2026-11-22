---
title: Data viewer
description: Native Excel workbooks with sheet tabs, CSV and TSV files, and virtualized plain-text viewing.
---

# Data viewer

`components::viewer::FileViewer` opens the files a data workbench usually needs to read:

| File | Rendering |
|---|---|
| `.xlsx`, `.xlsm`, `.xls`, `.xlsb`, `.ods` | One tab per sheet, typed cells, header row |
| `.csv` | One sheet, RFC 4180 quoting, numbers right-aligned |
| `.tsv`, `.tab` | Same as CSV with tabs |
| Anything else | UTF-8 plain text with line numbers |

<LiveDemo
  scene="viewer"
  height="520"
  alt="The file viewer with a workbook, CSV, TSV, and text"
  caption="Switch formats: the XLSX tab strip is live, and the text view keeps line numbers and horizontal scrolling."
/>

```rust
use egui_workbench::components::viewer::FileViewer;

struct Tab {
    viewer: FileViewer,
}

// Native: read the file once, when the tab opens.
let viewer = FileViewer::open("reports/sales.xlsx")?;

// Or parse bytes already in memory (a browser download, an attachment, a worker).
let viewer = FileViewer::from_bytes("sales.xlsx", bytes)?;

// Inside `ui()`: no I/O, only painting.
viewer.show(ui);
```

## What the viewer paints

- A toolbar with the format badge, the file name, dimensions, and toggles.
- A horizontally scrolled tab strip when the workbook has more than one sheet.
- The **first row is a header** switch, on by default when the first row has text.
- The text view's **line numbers** switch.
- A virtualized grid: only rows in the viewport are laid out, so a very large sheet opens
  without building a widget per cell.
- Numeric cells are right-aligned; booleans, dates, and spreadsheet errors have their own
  colors; empty cells keep the column width.
- Hovering a truncated cell shows its full content.

## Data model

`FileViewer` wraps a `Document`:

```rust
pub enum Document {
    Text(TextFile),
    Workbook(Workbook),
}

pub struct Workbook { pub sheets: Vec<Sheet> }
pub struct Sheet { pub name: String, pub table: Table }
```

`Table` stores rows of `Cell { text, kind }`; `CellKind` is `Empty`, `Text`, `Number`,
`Bool`, `Date`, or `Error`. Build one directly when the application already has rows:

```rust
use egui_workbench::components::viewer::{Cell, FileViewer, Sheet, Table, Workbook};

let table = Table::new([
    [Cell::text("Region"), Cell::text("Units")],
    [Cell::text("North"), Cell::number("12")],
]);
```

For in-memory text, `FileViewer::text(name, text)` needs no parsing and cannot fail.

## Errors

`FileViewer::open` and `from_bytes` return `ViewerError`:

| Variant | When |
|---|---|
| `Io(String)` | The path could not be read |
| `Spreadsheet(String)` | `calamine` rejected the workbook |
| `Binary` | The bytes are not UTF-8 and the extension is not a workbook |
| `Empty` | A workbook or delimited file has no sheets or cells |

The desktop-only `open` is available on native targets; `from_bytes` works everywhere,
including WebAssembly.

## Dependencies and features

Workbook support uses [`calamine`](https://crates.io/crates/calamine) (MIT) for XLSX,
XLS, XLSB, and ODS. CSV/TSV parsing and the text reader are part of the crate. Loading
happens once in `open`/`from_bytes`; painting never performs file I/O.
