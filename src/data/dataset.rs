//! A columnar, typed table.

use std::collections::BTreeMap;
use std::ops::Range;

use super::column::{CellValue, Column, ColumnType, ColumnValues};

/// A typed table: columns of equal length with dataset metadata.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dataset {
    columns: Vec<Column>,
    len: usize,
    metadata: BTreeMap<String, String>,
}

impl Dataset {
    /// An empty dataset.
    pub fn new() -> Self {
        Self::default()
    }

    /// A dataset from columns. The longest column sets the row count; shorter columns
    /// are treated as invalid at the end.
    pub fn from_columns(columns: impl IntoIterator<Item = Column>) -> Self {
        let columns: Vec<Column> = columns.into_iter().collect();
        let len = columns.iter().map(Column::len).max().unwrap_or(0);
        Self {
            columns,
            len,
            metadata: BTreeMap::new(),
        }
    }

    /// A dataset with `rows` empty rows and no columns.
    pub fn with_rows(rows: usize) -> Self {
        Self {
            len: rows,
            ..Self::default()
        }
    }

    /// Number of rows.
    pub fn row_count(&self) -> usize {
        self.len
    }

    /// Number of columns.
    pub fn width(&self) -> usize {
        self.columns.len()
    }

    /// Whether the dataset has no columns and no rows.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty() && self.len == 0
    }

    /// The columns.
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// A column by index.
    pub fn column(&self, index: usize) -> Option<&Column> {
        self.columns.get(index)
    }

    /// A column by name.
    pub fn column_named(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == name)
    }

    /// A column index by name.
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|column| column.name == name)
    }

    /// The dataset metadata.
    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    /// Add or replace metadata.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Append a column, padding shorter existing columns with nulls.
    pub fn push_column(&mut self, column: Column) {
        self.len = self.len.max(column.len());
        self.columns.push(column);
    }

    /// A cell by row and column.
    pub fn cell(&self, row: usize, column: usize) -> CellValue<'_> {
        self.columns
            .get(column)
            .map_or(CellValue::Null, |column| column.cell(row))
    }

    /// The row as owned values, for a query or a channel.
    pub fn row(&self, row: usize) -> Vec<super::column::DataValue> {
        self.columns
            .iter()
            .map(|column| column.cell(row).to_owned_value())
            .collect()
    }

    /// A dataset of rows in `range`.
    pub fn slice(&self, range: Range<usize>) -> Self {
        let start = range.start.min(self.len);
        let end = range.end.min(self.len).max(start);
        let columns = self.columns.iter().map(|column| column.slice(start..end)).collect();
        Self {
            columns,
            len: end - start,
            metadata: self.metadata.clone(),
        }
    }

    /// Rows whose predicate returns true, in order.
    pub fn matching_rows(&self, mut predicate: impl FnMut(usize, &Self) -> bool) -> Vec<usize> {
        (0..self.len).filter(|row| predicate(*row, self)).collect()
    }

    /// A dataset with only `rows`, in the given order.
    pub fn take(&self, rows: &[usize]) -> Self {
        let columns = self
            .columns
            .iter()
            .map(|column| {
                let values: Vec<super::column::DataValue> =
                    rows.iter().map(|row| column.cell(*row).to_owned_value()).collect();
                column_from_values(
                    &column.name,
                    &column.values,
                    &values,
                    &column.metadata,
                    column.unit.as_deref(),
                )
            })
            .collect();
        Self {
            columns,
            len: rows.len(),
            metadata: self.metadata.clone(),
        }
    }

    /// Points from two numeric columns, skipping null or non-numeric rows.
    pub fn xy_pairs(&self, x: &str, y: &str) -> Option<Vec<[f64; 2]>> {
        let x = self.column_named(x)?;
        let y = self.column_named(y)?;
        let mut points = Vec::new();
        for row in 0..self.len {
            if let (Some(x), Some(y)) = (
                super::column::DataValue::as_f64(&x.cell(row).to_owned_value()),
                super::column::DataValue::as_f64(&y.cell(row).to_owned_value()),
            ) && x.is_finite()
                && y.is_finite()
            {
                points.push([x, y]);
            }
        }
        Some(points)
    }

    /// One (x, y) point at `row`, when both values are finite numbers.
    pub fn point_at_row(&self, x: &str, y: &str, row: usize) -> Option<[f64; 2]> {
        let x = self.column_named(x)?;
        let y = self.column_named(y)?;
        let x = super::column::DataValue::as_f64(&x.cell(row).to_owned_value())?;
        let y = super::column::DataValue::as_f64(&y.cell(row).to_owned_value())?;
        (x.is_finite() && y.is_finite()).then_some([x, y])
    }

    /// Rows whose two numeric columns fall inside `bounds`.
    #[cfg(feature = "plot")]
    pub fn rows_in_bounds(&self, x: &str, y: &str, bounds: crate::components::plot::Bounds) -> Vec<usize> {
        (0..self.row_count())
            .filter(|row| {
                self.point_at_row(x, y, *row).is_some_and(|[x, y]| {
                    x >= bounds.min_x && x <= bounds.max_x && y >= bounds.min_y && y <= bounds.max_y
                })
            })
            .collect()
    }

    /// Statistics for one column: count, min, max, mean, and nulls.
    pub fn stats(&self, column: usize) -> Option<ColumnStats> {
        let column = self.columns.get(column)?;
        let mut count = 0;
        let mut nulls = 0;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut sum = 0.0;
        for row in 0..self.len {
            if let Some(value) = column.cell(row).to_owned_value().as_f64() {
                count += 1;
                sum += value;
                min = min.min(value);
                max = max.max(value);
            } else if column.cell(row).is_null() {
                nulls += 1;
            }
        }
        Some(ColumnStats {
            count,
            nulls,
            min: (count > 0).then_some(min),
            max: (count > 0).then_some(max),
            mean: (count > 0).then_some(sum / count as f64),
        })
    }
}

fn column_from_values(
    name: &str,
    template: &ColumnValues,
    values: &[super::column::DataValue],
    metadata: &BTreeMap<String, String>,
    unit: Option<&str>,
) -> Column {
    let mut column = match template.kind() {
        ColumnType::Float => Column::floats(
            name,
            values
                .iter()
                .map(|value| value.as_f64().unwrap_or(f64::NAN))
                .collect::<Vec<_>>(),
        ),
        ColumnType::Int => Column::ints(
            name,
            values
                .iter()
                .map(|value| match value {
                    super::column::DataValue::Int(value) => *value,
                    super::column::DataValue::Float(value) => *value as i64,
                    _ => 0,
                })
                .collect::<Vec<_>>(),
        ),
        ColumnType::Bool => Column::bools(
            name,
            values
                .iter()
                .map(|value| matches!(value, super::column::DataValue::Bool(true)))
                .collect::<Vec<_>>(),
        ),
        ColumnType::Text | ColumnType::Category => {
            let texts: Vec<String> = values
                .iter()
                .map(|value| value.as_text().unwrap_or_default().to_owned())
                .collect();
            if template.kind() == ColumnType::Category {
                Column::categories(name, texts)
            } else {
                Column::text(name, texts)
            }
        }
        ColumnType::Date => Column::dates(
            name,
            values
                .iter()
                .map(|value| {
                    if let super::column::DataValue::Date(value) = value {
                        *value
                    } else {
                        0
                    }
                })
                .collect::<Vec<_>>(),
        ),
        ColumnType::Time => Column::times(
            name,
            values
                .iter()
                .map(|value| {
                    if let super::column::DataValue::Time(value) = value {
                        *value
                    } else {
                        0
                    }
                })
                .collect::<Vec<_>>(),
        ),
    };
    let validity: Vec<bool> = values
        .iter()
        .map(|value| !matches!(value, super::column::DataValue::Null))
        .collect();
    if validity.iter().any(|valid| !valid) {
        column = column.with_validity(validity);
    }
    column.metadata = metadata.clone();
    if let Some(unit) = unit {
        column = column.with_unit(unit);
    }
    column
}

/// Summary statistics of one numeric column.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct ColumnStats {
    /// Non-null numeric values.
    pub count: usize,
    /// Null values.
    pub nulls: usize,
    /// Minimum, when there is a value.
    pub min: Option<f64>,
    /// Maximum, when there is a value.
    pub max: Option<f64>,
    /// Mean, when there is a value.
    pub mean: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dataset() -> Dataset {
        Dataset::from_columns([
            Column::floats("voltage", vec![0.1, 0.4, 0.2, 0.5]),
            Column::ints("cycle", vec![1, 2, 3, 4]),
            Column::text("name", ["a", "b", "c", "d"]),
        ])
    }

    #[test]
    fn columns_and_cells_are_typed() {
        let data = dataset();
        assert_eq!(data.row_count(), 4);
        assert_eq!(data.width(), 3);
        assert_eq!(data.cell(1, 0), CellValue::Float(0.4));
        assert_eq!(data.cell(1, 1), CellValue::Int(2));
        assert_eq!(data.index_of("name"), Some(2));
        assert!(data.column_named("missing").is_none());
    }

    #[test]
    fn slices_and_take_keep_types_and_metadata() {
        let mut data = dataset();
        data.set_metadata("source", "test");
        let slice = data.slice(1..3);
        assert_eq!(slice.row_count(), 2);
        assert_eq!(slice.cell(0, 0), CellValue::Float(0.4));
        assert_eq!(slice.metadata().get("source").map(String::as_str), Some("test"));
        let taken = data.take(&[3, 1]);
        assert_eq!(taken.row_count(), 2);
        assert_eq!(taken.cell(0, 1), CellValue::Int(4));
        assert_eq!(taken.cell(1, 2), CellValue::Text("b"));
    }

    #[test]
    fn statistics_ignore_nulls() {
        let data =
            Dataset::from_columns([Column::floats("v", vec![1.0, f64::NAN, 3.0]).with_validity([true, false, true])]);
        let stats = data.stats(0).expect("stats");
        assert_eq!(stats.count, 2);
        assert_eq!(stats.nulls, 1);
        assert_eq!(stats.min, Some(1.0));
        assert_eq!(stats.max, Some(3.0));
        assert_eq!(stats.mean, Some(2.0));
    }
}
