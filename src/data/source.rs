//! Lazy, paged data sources without an async runtime.

use std::collections::BTreeMap;
use std::ops::Range;

use super::column::{Column, ColumnType};
use super::dataset::Dataset;

/// The schema of one column, without its values.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ColumnSchema {
    /// Column name.
    pub name: String,
    /// Column type.
    pub kind: ColumnType,
    /// Unit.
    pub unit: Option<String>,
    /// Metadata.
    pub metadata: BTreeMap<String, String>,
}

impl ColumnSchema {
    /// A schema from a name and type.
    pub fn new(name: impl Into<String>, kind: ColumnType) -> Self {
        Self {
            name: name.into(),
            kind,
            unit: None,
            metadata: BTreeMap::new(),
        }
    }

    /// A schema from a column.
    pub fn from_column(column: &Column) -> Self {
        Self {
            name: column.name.clone(),
            kind: column.kind(),
            unit: column.unit.clone(),
            metadata: column.metadata.clone(),
        }
    }
}

/// Why a data source could not read.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DataError {
    /// The requested range is outside the source.
    OutOfRange {
        /// Requested start.
        start: usize,
        /// Requested end.
        end: usize,
        /// Available rows.
        rows: usize,
    },
    /// The source cannot answer without more information.
    Unsupported(String),
    /// The connector failed.
    Connector(String),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfRange { start, end, rows } => {
                write!(formatter, "rows {start}..{end} are outside {rows} available rows")
            }
            Self::Unsupported(message) | Self::Connector(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for DataError {}

/// A lazy, paged table. A source can be backed by a file, a database, a channel, or a
/// thread; the trait is synchronous and never pulls in an async runtime.
pub trait DataSource {
    /// The column schema.
    fn schema(&self) -> &[ColumnSchema];

    /// The total number of rows, when known.
    fn row_count(&self) -> usize;

    /// Read rows in `range`. Implementations may return fewer rows at the end.
    ///
    /// # Errors
    /// When the range is invalid or the backing store fails.
    fn read_range(&self, range: Range<usize>) -> Result<Dataset, DataError>;

    /// A revision that changes when the source changes.
    fn revision(&self) -> u64 {
        0
    }

    /// Cancel an in-flight read, when the implementation supports it.
    fn cancel(&self) {}
}

/// A source backed by an in-memory dataset.
#[derive(Clone, Debug, PartialEq)]
pub struct MemorySource {
    schema: Vec<ColumnSchema>,
    data: Dataset,
    revision: u64,
}

impl MemorySource {
    /// A source over `data`.
    pub fn new(data: Dataset) -> Self {
        let schema = data.columns().iter().map(ColumnSchema::from_column).collect();
        Self {
            schema,
            data,
            revision: 0,
        }
    }

    /// The backing data.
    pub fn data(&self) -> &Dataset {
        &self.data
    }
}

impl DataSource for MemorySource {
    fn schema(&self) -> &[ColumnSchema] {
        &self.schema
    }

    fn row_count(&self) -> usize {
        self.data.row_count()
    }

    fn read_range(&self, range: Range<usize>) -> Result<Dataset, DataError> {
        check_range(&range, self.row_count())?;
        Ok(self.data.slice(range))
    }

    fn revision(&self) -> u64 {
        self.revision
    }
}

pub(crate) fn check_range(range: &Range<usize>, rows: usize) -> Result<(), DataError> {
    if range.start > range.end || range.end > rows {
        Err(DataError::OutOfRange {
            start: range.start,
            end: range.end,
            rows,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Column, ColumnType};

    #[test]
    fn a_memory_source_pages_and_reports_schema() {
        let data = Dataset::from_columns([Column::floats("v", vec![1.0, 2.0, 3.0])]);
        let source = MemorySource::new(data);
        assert_eq!(source.schema()[0].kind, ColumnType::Float);
        assert_eq!(source.row_count(), 3);
        let page = source.read_range(1..3).expect("page");
        assert_eq!(page.row_count(), 2);
        assert!(source.read_range(2..9).is_err());
    }
}
