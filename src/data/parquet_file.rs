//! Parquet connector (Arrow-backed).

use std::path::Path;

use bytes::Bytes;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use super::dataset::Dataset;
use super::source::{ColumnSchema, DataError, DataSource};

/// Read Parquet bytes into a dataset.
///
/// # Errors
/// When the bytes are not a valid Parquet file or a column type is unsupported.
pub fn read_parquet_bytes(bytes: &[u8]) -> Result<Dataset, DataError> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::copy_from_slice(bytes))
        .map_err(|error| DataError::Connector(error.to_string()))?;
    let schema = builder.schema().clone();
    let reader = builder
        .build()
        .map_err(|error| DataError::Connector(error.to_string()))?;
    let batches: Vec<arrow::record_batch::RecordBatch> = reader
        .map(|batch| batch.map_err(|error| DataError::Connector(error.to_string())))
        .collect::<Result<_, _>>()?;
    if batches.is_empty() {
        return Ok(Dataset::new());
    }
    let batch =
        arrow::compute::concat_batches(&schema, &batches).map_err(|error| DataError::Connector(error.to_string()))?;
    super::arrow_ipc::dataset_from_batch(&batch)
}

/// A Parquet file exposed as a paged data source.
pub struct ParquetSource {
    name: String,
    data: Dataset,
    schema: Vec<ColumnSchema>,
}

impl ParquetSource {
    /// Read a Parquet file from disk.
    ///
    /// # Errors
    /// When the file cannot be read or parsed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = path.as_ref();
        let bytes = super::source::read_limited(path)?;
        let data = read_parquet_bytes(&bytes)?;
        let schema = data.columns().iter().map(ColumnSchema::from_column).collect();
        Ok(Self {
            name: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("data.parquet")
                .to_owned(),
            data,
            schema,
        })
    }

    /// Read Parquet bytes.
    ///
    /// # Errors
    /// When the bytes are not a valid Parquet file.
    pub fn from_bytes(name: impl Into<String>, bytes: &[u8]) -> Result<Self, DataError> {
        let data = read_parquet_bytes(bytes)?;
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

impl DataSource for ParquetSource {
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

    use arrow::array::{Float64Array, Int64Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::ArrowWriter;

    #[test]
    fn parquet_round_trips_through_a_dataset() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("n", DataType::Int64, false),
            Field::new("v", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(Float64Array::from(vec![0.1, 0.2, 0.3])),
            ],
        )
        .expect("batch");
        let mut bytes = Vec::new();
        {
            let mut writer = ArrowWriter::try_new(&mut bytes, schema, None).expect("writer");
            writer.write(&batch).expect("write");
            writer.close().expect("close");
        }
        let dataset = read_parquet_bytes(&bytes).expect("read");
        assert_eq!(dataset.row_count(), 3);
        assert_eq!(dataset.cell(2, 0), crate::data::CellValue::Int(3));
        let source = ParquetSource::from_bytes("test.parquet", &bytes).expect("source");
        assert_eq!(source.data().row_count(), 3);
    }
}
