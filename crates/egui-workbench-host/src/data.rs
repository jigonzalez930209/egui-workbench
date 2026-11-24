//! Large data buffers registered with a host.

use std::collections::BTreeMap;
use std::sync::Arc;

use egui_workbench::data::{Column, Dataset};
use serde::{Deserialize, Serialize};

/// A named dataset registered with a host.
#[derive(Clone, Debug)]
pub struct DataBuffer {
    /// Dataset name used by spec widgets and plot series.
    pub name: String,
    /// The data.
    pub dataset: Dataset,
}

impl DataBuffer {
    /// A buffer from a dataset.
    pub fn new(name: impl Into<String>, dataset: Dataset) -> Self {
        Self {
            name: name.into(),
            dataset,
        }
    }
}

/// A column moved over the protocol. Floats/ints are contiguous vectors; strings are
/// UTF-8 values; validity is optional.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum BufferColumn {
    /// Floats.
    Float {
        /// Name.
        name: String,
        /// Values.
        values: Vec<f64>,
        /// Validity.
        validity: Option<Vec<bool>>,
    },
    /// Integers.
    Int {
        /// Name.
        name: String,
        /// Values.
        values: Vec<i64>,
        /// Validity.
        validity: Option<Vec<bool>>,
    },
    /// Booleans.
    Bool {
        /// Name.
        name: String,
        /// Values.
        values: Vec<bool>,
        /// Validity.
        validity: Option<Vec<bool>>,
    },
    /// Text.
    Text {
        /// Name.
        name: String,
        /// Values.
        values: Vec<String>,
        /// Validity.
        validity: Option<Vec<bool>>,
    },
}

impl BufferColumn {
    /// Convert to a typed column.
    pub fn to_column(&self) -> Column {
        match self {
            Self::Float { name, values, validity } => with_validity(Column::floats(name, values.clone()), validity),
            Self::Int { name, values, validity } => with_validity(Column::ints(name, values.clone()), validity),
            Self::Bool { name, values, validity } => with_validity(Column::bools(name, values.clone()), validity),
            Self::Text { name, values, validity } => with_validity(Column::text(name, values.clone()), validity),
        }
    }
}

fn with_validity(column: Column, validity: &Option<Vec<bool>>) -> Column {
    match validity {
        Some(validity) => column.with_validity(validity.iter().copied()),
        None => column,
    }
}

/// Build a dataset from protocol columns.
pub fn dataset_from_columns(columns: impl IntoIterator<Item = BufferColumn>) -> Dataset {
    Dataset::from_columns(columns.into_iter().map(|column| column.to_column()))
}

/// A map of named datasets, shared by the runtime.
pub type Datasets = BTreeMap<String, Arc<Dataset>>;
