//! Typed columns, validity, and cell values.

use std::collections::BTreeMap;
use std::sync::Arc;

/// The type of one column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum ColumnType {
    /// IEEE-754 double.
    Float,
    /// Signed 64-bit integer.
    Int,
    /// Boolean.
    Bool,
    /// UTF-8 text.
    Text,
    /// Days since the Unix epoch.
    Date,
    /// Nanoseconds since the Unix epoch.
    Time,
    /// A small set of text categories.
    Category,
}

impl ColumnType {
    /// A short name for a header or a schema.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Int => "int",
            Self::Bool => "bool",
            Self::Text => "text",
            Self::Date => "date",
            Self::Time => "time",
            Self::Category => "category",
        }
    }
}

/// The storage of one column. Strings are shared with `Arc<[String]>`, so a clone is
/// cheap and a derived column can keep the same backing storage.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ColumnValues {
    /// Floats.
    Float(Arc<[f64]>),
    /// Integers.
    Int(Arc<[i64]>),
    /// Booleans.
    Bool(Arc<[bool]>),
    /// Text.
    Text(Arc<[String]>),
    /// Dates as days since the epoch.
    Date(Arc<[i64]>),
    /// Times as nanoseconds since the epoch.
    Time(Arc<[i64]>),
    /// Categories as text.
    Category(Arc<[String]>),
}

impl ColumnValues {
    /// The number of values.
    pub fn len(&self) -> usize {
        match self {
            Self::Float(values) => values.len(),
            Self::Int(values) => values.len(),
            Self::Bool(values) => values.len(),
            Self::Text(values) => values.len(),
            Self::Date(values) => values.len(),
            Self::Time(values) => values.len(),
            Self::Category(values) => values.len(),
        }
    }

    /// Whether the column has no values.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The type of the storage.
    pub fn kind(&self) -> ColumnType {
        match self {
            Self::Float(_) => ColumnType::Float,
            Self::Int(_) => ColumnType::Int,
            Self::Bool(_) => ColumnType::Bool,
            Self::Text(_) => ColumnType::Text,
            Self::Date(_) => ColumnType::Date,
            Self::Time(_) => ColumnType::Time,
            Self::Category(_) => ColumnType::Category,
        }
    }
}

/// One column with its name, type, validity, and metadata.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Storage and type.
    pub values: ColumnValues,
    /// A false entry marks a null cell. `None` means every cell is valid.
    pub validity: Option<Vec<bool>>,
    /// Unit text, such as `V` or `s`.
    pub unit: Option<String>,
    /// Free-form metadata (source, precision, labels).
    pub metadata: BTreeMap<String, String>,
}

impl Column {
    /// A column from storage.
    pub fn new(name: impl Into<String>, values: ColumnValues) -> Self {
        Self {
            name: name.into(),
            values,
            validity: None,
            unit: None,
            metadata: BTreeMap::new(),
        }
    }

    /// A float column.
    pub fn floats(name: impl Into<String>, values: impl Into<Arc<[f64]>>) -> Self {
        Self::new(name, ColumnValues::Float(values.into()))
    }

    /// An integer column.
    pub fn ints(name: impl Into<String>, values: impl Into<Arc<[i64]>>) -> Self {
        Self::new(name, ColumnValues::Int(values.into()))
    }

    /// A boolean column.
    pub fn bools(name: impl Into<String>, values: impl Into<Arc<[bool]>>) -> Self {
        Self::new(name, ColumnValues::Bool(values.into()))
    }

    /// A text column.
    pub fn text(name: impl Into<String>, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let values: Arc<[String]> = values.into_iter().map(Into::into).collect();
        Self::new(name, ColumnValues::Text(values))
    }

    /// A date column (days since the Unix epoch).
    pub fn dates(name: impl Into<String>, values: impl Into<Arc<[i64]>>) -> Self {
        Self::new(name, ColumnValues::Date(values.into()))
    }

    /// A time column (nanoseconds since the Unix epoch).
    pub fn times(name: impl Into<String>, values: impl Into<Arc<[i64]>>) -> Self {
        Self::new(name, ColumnValues::Time(values.into()))
    }

    /// A category column.
    pub fn categories(name: impl Into<String>, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let values: Arc<[String]> = values.into_iter().map(Into::into).collect();
        Self::new(name, ColumnValues::Category(values))
    }

    /// Set the unit.
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set the validity mask. A shorter mask marks trailing cells invalid.
    pub fn with_validity(mut self, validity: impl IntoIterator<Item = bool>) -> Self {
        self.validity = Some(validity.into_iter().collect());
        self
    }

    /// The type.
    pub fn kind(&self) -> ColumnType {
        self.values.kind()
    }

    /// Number of values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the column is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether `row` has a value.
    pub fn is_valid(&self, row: usize) -> bool {
        self.validity
            .as_ref()
            .is_none_or(|validity| validity.get(row).copied().unwrap_or(false))
    }

    /// A borrowed cell value.
    pub fn cell(&self, row: usize) -> CellValue<'_> {
        if !self.is_valid(row) {
            return CellValue::Null;
        }
        match &self.values {
            ColumnValues::Float(values) => values
                .get(row)
                .map_or(CellValue::Null, |value| CellValue::Float(*value)),
            ColumnValues::Int(values) => values.get(row).map_or(CellValue::Null, |value| CellValue::Int(*value)),
            ColumnValues::Bool(values) => values.get(row).map_or(CellValue::Null, |value| CellValue::Bool(*value)),
            ColumnValues::Text(values) => values
                .get(row)
                .map_or(CellValue::Null, |value| CellValue::Text(value.as_str())),
            ColumnValues::Date(values) => values.get(row).map_or(CellValue::Null, |value| CellValue::Date(*value)),
            ColumnValues::Time(values) => values.get(row).map_or(CellValue::Null, |value| CellValue::Time(*value)),
            ColumnValues::Category(values) => values
                .get(row)
                .map_or(CellValue::Null, |value| CellValue::Category(value.as_str())),
        }
    }

    /// A copy of rows in `range`.
    pub fn slice(&self, range: std::ops::Range<usize>) -> Self {
        let start = range.start.min(self.len());
        let end = range.end.min(self.len()).max(start);
        let values = match &self.values {
            ColumnValues::Float(values) => ColumnValues::Float(Arc::from(&values[start..end])),
            ColumnValues::Int(values) => ColumnValues::Int(Arc::from(&values[start..end])),
            ColumnValues::Bool(values) => ColumnValues::Bool(Arc::from(&values[start..end])),
            ColumnValues::Text(values) => ColumnValues::Text(Arc::from(&values[start..end])),
            ColumnValues::Date(values) => ColumnValues::Date(Arc::from(&values[start..end])),
            ColumnValues::Time(values) => ColumnValues::Time(Arc::from(&values[start..end])),
            ColumnValues::Category(values) => ColumnValues::Category(Arc::from(&values[start..end])),
        };
        Self {
            name: self.name.clone(),
            values,
            validity: self.validity.as_ref().map(|validity| {
                (start..end)
                    .map(|row| validity.get(row).copied().unwrap_or(false))
                    .collect()
            }),
            unit: self.unit.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// A borrowed cell value.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CellValue<'a> {
    /// No value.
    Null,
    /// Float.
    Float(f64),
    /// Integer.
    Int(i64),
    /// Boolean.
    Bool(bool),
    /// Text.
    Text(&'a str),
    /// Date.
    Date(i64),
    /// Time.
    Time(i64),
    /// Category.
    Category(&'a str),
}

impl CellValue<'_> {
    /// Whether the cell is null.
    pub fn is_null(self) -> bool {
        matches!(self, Self::Null)
    }

    /// A text representation for a grid or a copy.
    pub fn to_text(self) -> String {
        match self {
            Self::Null => String::new(),
            Self::Float(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Bool(value) => value.to_string(),
            Self::Text(value) | Self::Category(value) => value.to_owned(),
            Self::Date(value) => format!("date:{value}"),
            Self::Time(value) => format!("time:{value}"),
        }
    }

    /// An owned value.
    pub fn to_owned_value(self) -> DataValue {
        match self {
            Self::Null => DataValue::Null,
            Self::Float(value) => DataValue::Float(value),
            Self::Int(value) => DataValue::Int(value),
            Self::Bool(value) => DataValue::Bool(value),
            Self::Text(value) | Self::Category(value) => DataValue::Text(value.to_owned()),
            Self::Date(value) => DataValue::Date(value),
            Self::Time(value) => DataValue::Time(value),
        }
    }
}

/// An owned value, used by the query evaluator and streaming buffers.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DataValue {
    /// No value.
    Null,
    /// Float.
    Float(f64),
    /// Integer.
    Int(i64),
    /// Boolean.
    Bool(bool),
    /// Text.
    Text(String),
    /// Date.
    Date(i64),
    /// Time.
    Time(i64),
}

impl DataValue {
    /// A numeric value as an f64, when it is numeric.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            Self::Int(value) => Some(*value as f64),
            Self::Date(value) | Self::Time(value) => Some(*value as f64),
            _ => None,
        }
    }

    /// A text value, for text comparisons.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_carry_types_units_and_validity() {
        let column = Column::floats("voltage", vec![1.0, f64::NAN, 3.0])
            .with_unit("V")
            .with_validity([true, false, true]);
        assert_eq!(column.kind(), ColumnType::Float);
        assert_eq!(column.unit.as_deref(), Some("V"));
        assert!(column.is_valid(0));
        assert!(!column.is_valid(1));
        assert_eq!(column.cell(2), CellValue::Float(3.0));
        assert_eq!(column.cell(1), CellValue::Null);
    }

    #[test]
    fn slices_keep_metadata_and_validity() {
        let column = Column::ints("n", vec![0, 1, 2, 3]).with_validity([true, true, false, true]);
        let slice = column.slice(1..3);
        assert_eq!(slice.len(), 2);
        assert_eq!(slice.cell(0), CellValue::Int(1));
        assert_eq!(slice.cell(1), CellValue::Null);
    }

    #[test]
    fn owned_values_round_trip() {
        let column = Column::text("name", ["a", "b"]);
        assert_eq!(column.cell(0).to_owned_value(), DataValue::Text("a".to_owned()));
        assert_eq!(DataValue::Int(2).as_f64(), Some(2.0));
    }
}
