//! Spreadsheet connector through `calamine`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use calamine::{Data, Reader};

use super::column::{Column, DataValue};
use super::dataset::Dataset;
use super::source::{ColumnSchema, DataError, DataSource};

/// One workbook with a dataset per sheet.
pub struct WorkbookSource {
    path: PathBuf,
    sheets: BTreeMap<String, Dataset>,
    schema: Vec<ColumnSchema>,
}

impl WorkbookSource {
    /// Open a workbook and read every sheet.
    ///
    /// # Errors
    /// When the workbook cannot be read.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = path.as_ref().to_path_buf();
        let mut workbook =
            calamine::open_workbook_auto(&path).map_err(|error| DataError::Connector(error.to_string()))?;
        let names = workbook.sheet_names().to_vec();
        let mut sheets = BTreeMap::new();
        for name in names {
            let range = workbook
                .worksheet_range(&name)
                .map_err(|error| DataError::Connector(error.to_string()))?;
            sheets.insert(name, dataset_from_range(&range));
        }
        let schema = sheets
            .values()
            .next()
            .map(|data| data.columns().iter().map(ColumnSchema::from_column).collect())
            .unwrap_or_default();
        Ok(Self { path, sheets, schema })
    }

    /// The workbook path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The sheet names.
    pub fn sheet_names(&self) -> impl Iterator<Item = &str> {
        self.sheets.keys().map(String::as_str)
    }

    /// A sheet dataset.
    pub fn sheet(&self, name: &str) -> Option<&Dataset> {
        self.sheets.get(name)
    }

    /// The first sheet.
    pub fn first(&self) -> Option<&Dataset> {
        self.sheets.values().next()
    }
}

impl DataSource for WorkbookSource {
    fn schema(&self) -> &[ColumnSchema] {
        &self.schema
    }

    fn row_count(&self) -> usize {
        self.first().map_or(0, Dataset::row_count)
    }

    fn read_range(&self, range: std::ops::Range<usize>) -> Result<Dataset, DataError> {
        let data = self
            .first()
            .ok_or_else(|| DataError::Connector("the workbook has no sheets".to_owned()))?;
        super::source::check_range(&range, data.row_count())?;
        Ok(data.slice(range))
    }
}

fn dataset_from_range(range: &calamine::Range<Data>) -> Dataset {
    let mut rows = range.rows();
    let Some(headers) = rows.next() else {
        return Dataset::new();
    };
    let names: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(index, value)| cell_text(value).unwrap_or_else(|| format!("column_{index}")))
        .collect();
    let values: Vec<Vec<DataValue>> = rows.map(|row| row.iter().map(cell_value).collect::<Vec<_>>()).collect();
    let columns: Vec<Column> = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            column_from_values(
                name,
                &values
                    .iter()
                    .map(|row| row.get(index).cloned().unwrap_or(DataValue::Null))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    Dataset::from_columns(columns)
}

fn cell_value(value: &Data) -> DataValue {
    match value {
        Data::Empty => DataValue::Null,
        Data::String(value) => DataValue::Text(value.clone()),
        Data::Float(value) => DataValue::Float(*value),
        Data::Int(value) => DataValue::Int(*value),
        Data::Bool(value) => DataValue::Bool(*value),
        Data::DateTime(value) => DataValue::Float(value.as_f64()),
        Data::DateTimeIso(value) | Data::DurationIso(value) => DataValue::Text(value.clone()),
        Data::Error(error) => DataValue::Text(format!("{error:?}")),
    }
}

fn cell_text(value: &Data) -> Option<String> {
    match cell_value(value) {
        DataValue::Null => None,
        DataValue::Text(value) => Some(value),
        DataValue::Float(value) => Some(value.to_string()),
        DataValue::Int(value) => Some(value.to_string()),
        DataValue::Bool(value) => Some(value.to_string()),
        DataValue::Date(value) | DataValue::Time(value) => Some(value.to_string()),
    }
}

fn column_from_values(name: &str, values: &[DataValue]) -> Column {
    let non_null: Vec<&DataValue> = values
        .iter()
        .filter(|value| !matches!(value, DataValue::Null))
        .collect();
    let all_int = !non_null.is_empty() && non_null.iter().all(|value| matches!(value, DataValue::Int(_)));
    let all_float = !non_null.is_empty()
        && non_null
            .iter()
            .all(|value| matches!(value, DataValue::Int(_) | DataValue::Float(_)));
    let column = if all_int {
        Column::ints(
            name,
            values
                .iter()
                .map(|value| if let DataValue::Int(value) = value { *value } else { 0 })
                .collect::<Vec<_>>(),
        )
    } else if all_float {
        Column::floats(
            name,
            values
                .iter()
                .map(|value| value.as_f64().unwrap_or(f64::NAN))
                .collect::<Vec<_>>(),
        )
    } else {
        Column::text(
            name,
            values
                .iter()
                .map(|value| match value {
                    DataValue::Null => String::new(),
                    DataValue::Text(value) => value.clone(),
                    other => format!("{other:?}"),
                })
                .collect::<Vec<_>>(),
        )
    };
    let valid: Vec<bool> = values.iter().map(|value| !matches!(value, DataValue::Null)).collect();
    if valid.iter().all(|valid| *valid) {
        column
    } else {
        column.with_validity(valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixture_workbook_has_sheets_and_rows() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sales.xlsx");
        let source = WorkbookSource::open(&path).expect("workbook");
        assert!(source.sheet_names().next().is_some());
        assert!(source.first().is_some_and(|sheet| sheet.row_count() > 0));
        assert!(source.row_count() > 0);
        assert!(source.read_range(0..1).is_ok());
    }
}
