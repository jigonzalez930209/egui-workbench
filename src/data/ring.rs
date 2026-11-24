//! A fixed-capacity ring buffer for streaming measurements.

use std::collections::VecDeque;

use super::column::{Column, DataValue};
use super::dataset::Dataset;
use super::source::{ColumnSchema, DataError, DataSource};

/// A ring of the newest rows with a fixed schema.
#[derive(Clone, Debug)]
pub struct RingBuffer {
    capacity: usize,
    schema: Vec<ColumnSchema>,
    rows: VecDeque<Vec<DataValue>>,
    revision: u64,
}

impl RingBuffer {
    /// A ring with `capacity` rows. A capacity of zero becomes one.
    pub fn new(capacity: usize, schema: impl IntoIterator<Item = ColumnSchema>) -> Self {
        Self {
            capacity: capacity.max(1),
            schema: schema.into_iter().collect(),
            rows: VecDeque::new(),
            revision: 0,
        }
    }

    /// The schema.
    pub fn schema(&self) -> &[ColumnSchema] {
        &self.schema
    }

    /// Number of buffered rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether the ring is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Append one row. The oldest row is dropped when the ring is full.
    ///
    /// # Errors
    /// When the row length does not match the schema.
    pub fn push(&mut self, row: impl IntoIterator<Item = DataValue>) -> Result<(), DataError> {
        let row: Vec<DataValue> = row.into_iter().collect();
        if row.len() != self.schema.len() {
            return Err(DataError::Connector(format!(
                "row has {} values, schema has {}",
                row.len(),
                self.schema.len()
            )));
        }
        if self.rows.len() == self.capacity {
            self.rows.pop_front();
        }
        self.rows.push_back(row);
        self.revision += 1;
        Ok(())
    }

    /// Remove every row.
    pub fn clear(&mut self) {
        self.rows.clear();
        self.revision += 1;
    }

    /// A dataset of the buffered rows, oldest first.
    pub fn snapshot(&self) -> Dataset {
        let mut columns = Vec::with_capacity(self.schema.len());
        for (index, schema) in self.schema.iter().enumerate() {
            columns.push(column_from_schema(
                schema,
                self.rows.iter().map(|row| row[index].clone()).collect(),
            ));
        }
        Dataset::from_columns(columns)
    }
}

fn column_from_schema(schema: &ColumnSchema, values: Vec<DataValue>) -> Column {
    let mut column = match schema.kind {
        super::column::ColumnType::Float => Column::floats(
            &schema.name,
            values
                .iter()
                .map(|value| value.as_f64().unwrap_or(f64::NAN))
                .collect::<Vec<_>>(),
        ),
        super::column::ColumnType::Int => Column::ints(
            &schema.name,
            values
                .iter()
                .map(|value| if let DataValue::Int(value) = value { *value } else { 0 })
                .collect::<Vec<_>>(),
        ),
        super::column::ColumnType::Bool => Column::bools(
            &schema.name,
            values
                .iter()
                .map(|value| matches!(value, DataValue::Bool(true)))
                .collect::<Vec<_>>(),
        ),
        super::column::ColumnType::Date => Column::dates(
            &schema.name,
            values
                .iter()
                .map(|value| if let DataValue::Date(value) = value { *value } else { 0 })
                .collect::<Vec<_>>(),
        ),
        super::column::ColumnType::Time => Column::times(
            &schema.name,
            values
                .iter()
                .map(|value| if let DataValue::Time(value) = value { *value } else { 0 })
                .collect::<Vec<_>>(),
        ),
        super::column::ColumnType::Category => Column::categories(
            &schema.name,
            values
                .iter()
                .map(|value| value.as_text().unwrap_or_default().to_owned())
                .collect::<Vec<_>>(),
        ),
        _ => Column::text(
            &schema.name,
            values
                .iter()
                .map(|value| value.as_text().unwrap_or_default().to_owned())
                .collect::<Vec<_>>(),
        ),
    };
    let validity: Vec<bool> = values.iter().map(|value| !matches!(value, DataValue::Null)).collect();
    if validity.iter().any(|valid| !valid) {
        column = column.with_validity(validity);
    }
    column.metadata = schema.metadata.clone();
    if let Some(unit) = &schema.unit {
        column = column.with_unit(unit.clone());
    }
    column
}

impl DataSource for RingBuffer {
    fn schema(&self) -> &[ColumnSchema] {
        &self.schema
    }

    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn read_range(&self, range: std::ops::Range<usize>) -> Result<Dataset, DataError> {
        super::source::check_range(&range, self.rows.len())?;
        Ok(self.snapshot().slice(range))
    }

    fn revision(&self) -> u64 {
        self.revision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ColumnType;

    fn ring() -> RingBuffer {
        RingBuffer::new(
            3,
            [
                ColumnSchema::new("t", ColumnType::Float),
                ColumnSchema::new("v", ColumnType::Float),
            ],
        )
    }

    #[test]
    fn the_oldest_row_is_dropped() {
        let mut ring = ring();
        for index in 0..5 {
            ring.push([DataValue::Float(index as f64), DataValue::Float(index as f64 * 2.0)])
                .expect("push");
        }
        assert_eq!(ring.len(), 3);
        let snapshot = ring.snapshot();
        assert_eq!(snapshot.row_count(), 3);
        assert_eq!(snapshot.cell(0, 0), crate::data::CellValue::Float(2.0));
        assert_eq!(ring.read_range(1..3).expect("read").row_count(), 2);
    }

    #[test]
    fn row_width_is_checked() {
        let mut ring = ring();
        assert!(ring.push([DataValue::Float(1.0)]).is_err());
        assert!(ring.is_empty());
    }
}
