//! Arrow IPC connector.

use arrow::array::{
    Array, ArrayRef, BooleanArray, Date32Array, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array,
    Int64Array, StringArray, TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use std::path::Path;

use super::column::Column;
use super::dataset::Dataset;
use super::source::{ColumnSchema, DataError, DataSource};

/// Read an Arrow IPC file (the `.arrow`/`.feather` file format) into a dataset.
///
/// # Errors
/// When the file is not a valid Arrow IPC file or a column type is unsupported.
pub fn read_arrow_ipc(bytes: &[u8]) -> Result<Dataset, DataError> {
    let reader = arrow::ipc::reader::FileReader::try_new(std::io::Cursor::new(bytes), None)
        .map_err(|error| DataError::Connector(error.to_string()))?;
    let schema = reader.schema();
    let batches: Vec<RecordBatch> = reader
        .map(|batch| batch.map_err(|error| DataError::Connector(error.to_string())))
        .collect::<Result<_, _>>()?;
    if batches.is_empty() {
        return Ok(Dataset::new());
    }
    let batch =
        arrow::compute::concat_batches(&schema, &batches).map_err(|error| DataError::Connector(error.to_string()))?;
    dataset_from_batch(&batch)
}

/// Convert an Arrow record batch into a typed dataset.
///
/// # Errors
/// When a column type has no dataset representation.
pub fn dataset_from_batch(batch: &RecordBatch) -> Result<Dataset, DataError> {
    let mut columns = Vec::with_capacity(batch.num_columns());
    for (index, field) in batch.schema().fields().iter().enumerate() {
        let array = batch.column(index);
        columns.push(column_from_array(field.name(), array)?);
    }
    Ok(Dataset::from_columns(columns))
}

pub(crate) fn column_from_array(name: &str, array: &ArrayRef) -> Result<Column, DataError> {
    macro_rules! primitive {
        ($array_ty:ty, $variant:ident) => {{
            let values = array
                .as_any()
                .downcast_ref::<$array_ty>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not {}", stringify!($array_ty))))?;
            let data: Vec<_> = (0..values.len())
                .map(|row| {
                    if values.is_null(row) {
                        None
                    } else {
                        Some(values.value(row))
                    }
                })
                .collect();
            let valid: Vec<bool> = data.iter().map(Option::is_some).collect();
            let values: Vec<_> = data.into_iter().map(Option::unwrap_or_default).collect();
            let column = Column::$variant(name, values);
            Some(if valid.iter().all(|value| *value) {
                column
            } else {
                column.with_validity(valid)
            })
        }};
    }
    let column = match array.data_type() {
        DataType::Float64 => primitive!(Float64Array, floats),
        DataType::Float32 => {
            let values = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not Float32")))?;
            Some(with_validity(
                Column::floats(
                    name,
                    values
                        .values()
                        .iter()
                        .map(|value| f64::from(*value))
                        .collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::Int64 => primitive!(Int64Array, ints),
        DataType::Int32 => {
            let values = array
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not Int32")))?;
            Some(with_validity(
                Column::ints(
                    name,
                    values
                        .values()
                        .iter()
                        .map(|value| i64::from(*value))
                        .collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::Int16 => {
            let values = array
                .as_any()
                .downcast_ref::<Int16Array>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not Int16")))?;
            Some(with_validity(
                Column::ints(
                    name,
                    values
                        .values()
                        .iter()
                        .map(|value| i64::from(*value))
                        .collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::Int8 => {
            let values = array
                .as_any()
                .downcast_ref::<Int8Array>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not Int8")))?;
            Some(with_validity(
                Column::ints(
                    name,
                    values
                        .values()
                        .iter()
                        .map(|value| i64::from(*value))
                        .collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::UInt64 => {
            let values = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not UInt64")))?;
            Some(with_validity(
                Column::ints(
                    name,
                    values.values().iter().map(|value| *value as i64).collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::UInt32 => {
            let values = array
                .as_any()
                .downcast_ref::<UInt32Array>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not UInt32")))?;
            Some(with_validity(
                Column::ints(
                    name,
                    values
                        .values()
                        .iter()
                        .map(|value| i64::from(*value))
                        .collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::UInt16 => {
            let values = array
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not UInt16")))?;
            Some(with_validity(
                Column::ints(
                    name,
                    values
                        .values()
                        .iter()
                        .map(|value| i64::from(*value))
                        .collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::UInt8 => {
            let values = array
                .as_any()
                .downcast_ref::<UInt8Array>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not UInt8")))?;
            Some(with_validity(
                Column::ints(
                    name,
                    values
                        .values()
                        .iter()
                        .map(|value| i64::from(*value))
                        .collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::Boolean => primitive!(BooleanArray, bools),
        DataType::Utf8 => {
            let values = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not Utf8")))?;
            Some(with_validity(
                Column::text(
                    name,
                    values
                        .iter()
                        .map(|value| value.unwrap_or_default().to_owned())
                        .collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::Date32 => {
            let values = array
                .as_any()
                .downcast_ref::<Date32Array>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not Date32")))?;
            Some(with_validity(
                Column::dates(
                    name,
                    values
                        .values()
                        .iter()
                        .map(|value| i64::from(*value))
                        .collect::<Vec<_>>(),
                ),
                values,
            ))
        }
        DataType::Timestamp(_, _) => {
            let values = array
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .ok_or_else(|| DataError::Unsupported(format!("column `{name}` is not a nanosecond timestamp")))?;
            Some(with_validity(
                Column::times(name, values.values().iter().copied().collect::<Vec<_>>()),
                values,
            ))
        }
        other => return Err(DataError::Unsupported(format!("unsupported Arrow type {other:?}"))),
    };
    column.ok_or_else(|| DataError::Unsupported(format!("column `{name}`")))
}

fn with_validity<A: Array>(column: Column, array: &A) -> Column {
    let valid: Vec<bool> = (0..array.len()).map(|row| !array.is_null(row)).collect();
    if valid.iter().all(|value| *value) {
        column
    } else {
        column.with_validity(valid)
    }
}

/// An Arrow IPC file exposed as a paged data source.
pub struct ArrowSource {
    name: String,
    data: Dataset,
    schema: Vec<ColumnSchema>,
}

impl ArrowSource {
    /// Read an Arrow IPC file from disk.
    ///
    /// # Errors
    /// When the file cannot be read or parsed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|error| DataError::Connector(error.to_string()))?;
        Self::from_bytes(
            path.file_name().and_then(|name| name.to_str()).unwrap_or("data.arrow"),
            &bytes,
        )
    }

    /// Read Arrow IPC bytes.
    ///
    /// # Errors
    /// When the bytes are not a valid Arrow IPC file.
    pub fn from_bytes(name: impl Into<String>, bytes: &[u8]) -> Result<Self, DataError> {
        let data = read_arrow_ipc(bytes)?;
        let schema = data.columns().iter().map(ColumnSchema::from_column).collect();
        Ok(Self {
            name: name.into(),
            data,
            schema,
        })
    }

    /// The file name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The parsed data.
    pub fn data(&self) -> &Dataset {
        &self.data
    }
}

impl DataSource for ArrowSource {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use arrow::array::{Float64Array, Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};

    fn batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("n", DataType::Int64, false),
            Field::new("v", DataType::Float64, false),
            Field::new("name", DataType::Utf8, false),
        ]));
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(vec![1, 2])),
                Arc::new(Float64Array::from(vec![0.5, 1.5])),
                Arc::new(StringArray::from(vec!["a", "b"])),
            ],
        )
        .expect("batch")
    }

    #[test]
    fn arrow_ipc_round_trips_through_a_dataset() {
        let batch = batch();
        let mut bytes = Vec::new();
        {
            let mut writer = arrow::ipc::writer::FileWriter::try_new(&mut bytes, &batch.schema()).expect("writer");
            writer.write(&batch).expect("write");
            writer.finish().expect("finish");
        }
        let dataset = read_arrow_ipc(&bytes).expect("read");
        assert_eq!(dataset.row_count(), 2);
        assert_eq!(dataset.cell(1, 0), crate::data::CellValue::Int(2));
        assert_eq!(dataset.cell(0, 2), crate::data::CellValue::Text("a"));
        let source = ArrowSource::from_bytes("test.arrow", &bytes).expect("source");
        assert_eq!(source.schema().len(), 3);
        assert_eq!(source.read_range(1..2).expect("page").row_count(), 1);
    }
}
