//! Excel and OpenDocument workbooks through `calamine`, one sheet per tab.

use std::io::Cursor;

use calamine::{Data, Reader, open_workbook_auto_from_rs};

use super::ViewerError;
use super::table::{Cell, Sheet, Table, Workbook};

/// Read every sheet of a workbook from bytes.
pub(super) fn workbook(bytes: &[u8]) -> Result<Workbook, ViewerError> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook =
        open_workbook_auto_from_rs(cursor).map_err(|error| ViewerError::Spreadsheet(error.to_string()))?;
    let names = workbook.sheet_names();
    let mut sheets = Vec::with_capacity(names.len());
    for name in names {
        let range = workbook
            .worksheet_range(&name)
            .map_err(|error| ViewerError::Spreadsheet(error.to_string()))?;
        let width = range.width();
        let rows: Vec<Vec<Cell>> = range
            .rows()
            .map(|row| {
                let mut cells: Vec<Cell> = row.iter().map(cell).collect();
                cells.resize(width, Cell::empty());
                cells
            })
            .collect();
        sheets.push(Sheet::new(name, Table::new(rows)));
    }
    if sheets.is_empty() {
        return Err(ViewerError::Empty);
    }
    Ok(Workbook::new(sheets))
}

fn cell(value: &Data) -> Cell {
    match value {
        Data::Int(number) => Cell::number(number.to_string()),
        Data::Float(number) => Cell::number(number.to_string()),
        Data::String(text) => Cell::text(text.clone()),
        Data::Bool(value) => Cell::boolean(*value),
        Data::DateTime(value) => Cell::date(value.to_string()),
        Data::DateTimeIso(value) => Cell::date(value.clone()),
        Data::DurationIso(value) => Cell::text(value.clone()),
        Data::Error(error) => Cell::error(error.to_string()),
        Data::Empty => Cell::empty(),
    }
}
