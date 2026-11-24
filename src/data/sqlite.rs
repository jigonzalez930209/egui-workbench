//! SQLite connector.

use std::path::{Path, PathBuf};

use rusqlite::types::ValueRef;

use super::column::{Column, DataValue};
use super::dataset::Dataset;
use super::source::{ColumnSchema, DataError, DataSource};

/// A SQLite query exposed as a paged data source.
pub struct SqliteSource {
    path: PathBuf,
    name: String,
    query: String,
    data: Dataset,
    schema: Vec<ColumnSchema>,
}

impl SqliteSource {
    /// Open a database and read `query` once.
    ///
    /// # Errors
    /// When the database or query fails.
    pub fn open(path: impl AsRef<Path>, query: impl Into<String>) -> Result<Self, DataError> {
        let path = path.as_ref().to_path_buf();
        let query = query.into();
        let data = read_sqlite(&path, &query)?;
        let schema = data.columns().iter().map(ColumnSchema::from_column).collect();
        Ok(Self {
            name: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("data.sqlite")
                .to_owned(),
            path,
            query,
            data,
            schema,
        })
    }

    /// The file name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The database path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// The parsed data.
    pub fn data(&self) -> &Dataset {
        &self.data
    }

    /// Re-run the query and replace the data.
    ///
    /// # Errors
    /// When the query fails.
    pub fn refresh(&mut self) -> Result<(), DataError> {
        self.data = read_sqlite(&self.path, &self.query)?;
        self.schema = self.data.columns().iter().map(ColumnSchema::from_column).collect();
        Ok(())
    }
}

impl DataSource for SqliteSource {
    fn schema(&self) -> &[ColumnSchema] {
        &self.schema
    }

    fn row_count(&self) -> usize {
        self.data.row_count()
    }

    fn read_range(&self, range: std::ops::Range<usize>) -> Result<Dataset, DataError> {
        super::source::check_range(&range, self.row_count())?;
        Ok(self.data.slice(range))
    }
}

/// Run a query and convert every row to a dataset.
///
/// # Errors
/// When the database or query fails.
pub fn read_sqlite(path: impl AsRef<Path>, query: &str) -> Result<Dataset, DataError> {
    let connection = rusqlite::Connection::open(path).map_err(|error| DataError::Connector(error.to_string()))?;
    let mut statement = connection
        .prepare(query)
        .map_err(|error| DataError::Connector(error.to_string()))?;
    let names: Vec<String> = statement.column_names().iter().map(|name| (*name).to_owned()).collect();
    let mut rows = statement
        .query([])
        .map_err(|error| DataError::Connector(error.to_string()))?;
    let mut values: Vec<Vec<DataValue>> = Vec::new();
    while let Some(row) = rows.next().map_err(|error| DataError::Connector(error.to_string()))? {
        let mut record = Vec::with_capacity(names.len());
        for index in 0..names.len() {
            let value = row
                .get_ref(index)
                .map_err(|error| DataError::Connector(error.to_string()))?;
            record.push(match value {
                ValueRef::Null => DataValue::Null,
                ValueRef::Integer(value) => DataValue::Int(value),
                ValueRef::Real(value) => DataValue::Float(value),
                ValueRef::Text(value) => DataValue::Text(String::from_utf8_lossy(value).into_owned()),
                ValueRef::Blob(value) => DataValue::Text(format!("0x{}", hex(value))),
            });
        }
        values.push(record);
    }
    Ok(dataset_from_values(&names, &values))
}

fn dataset_from_values(names: &[String], rows: &[Vec<DataValue>]) -> Dataset {
    let columns: Vec<Column> = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let values: Vec<DataValue> = rows
                .iter()
                .map(|row| row.get(index).cloned().unwrap_or(DataValue::Null))
                .collect();
            column_from_values(name, &values)
        })
        .collect();
    Dataset::from_columns(columns)
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
    with_validity(column, values)
}

fn with_validity(column: Column, values: &[DataValue]) -> Column {
    let valid: Vec<bool> = values.iter().map(|value| !matches!(value, DataValue::Null)).collect();
    if valid.iter().all(|valid| *valid) {
        column
    } else {
        column.with_validity(valid)
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_query_becomes_a_typed_dataset() {
        let path = std::env::temp_dir().join(format!("egui-workbench-sqlite-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        {
            let connection = rusqlite::Connection::open(&path).expect("open");
            connection
                .execute_batch("CREATE TABLE trace (t REAL, cycle INTEGER, state TEXT); INSERT INTO trace VALUES (0.5, 2, 'run'), (0.1, 1, 'idle');")
                .expect("setup");
        }
        let source = SqliteSource::open(&path, "SELECT t, cycle, state FROM trace ORDER BY cycle").expect("source");
        assert_eq!(source.row_count(), 2);
        assert_eq!(source.schema()[1].kind, crate::data::ColumnType::Int);
        assert_eq!(source.data().cell(1, 2), crate::data::CellValue::Text("run"));
        assert_eq!(source.read_range(0..1).expect("page").row_count(), 1);
        let _ = std::fs::remove_file(path);
    }
}
