//! The query expression tree and evaluator.

use super::super::Dataset;
use std::cmp::Ordering;

use super::super::column::{CellValue, DataValue};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    Literal(DataValue),
    Column(String),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Compare {
        op: CompareOp,
        left: Box<Self>,
        right: Box<Self>,
    },
    InRange {
        value: Box<Self>,
        start: Box<Self>,
        end: Box<Self>,
    },
    InList {
        value: Box<Self>,
        values: Vec<Self>,
    },
}

impl Expr {
    pub(super) fn evaluate(&self, data: &Dataset, row: usize) -> DataValue {
        match self {
            Self::Literal(value) => value.clone(),
            Self::Column(name) => data
                .column_named(name)
                .map_or(DataValue::Null, |column| match column.cell(row) {
                    CellValue::Null => DataValue::Null,
                    value => value.to_owned_value(),
                }),
            Self::Not(inner) => DataValue::Bool(!inner.evaluate(data, row).is_truthy()),
            Self::And(left, right) => {
                DataValue::Bool(left.evaluate(data, row).is_truthy() && right.evaluate(data, row).is_truthy())
            }
            Self::Or(left, right) => {
                DataValue::Bool(left.evaluate(data, row).is_truthy() || right.evaluate(data, row).is_truthy())
            }
            Self::Compare { op, left, right } => {
                let left = left.evaluate(data, row);
                let right = right.evaluate(data, row);
                DataValue::Bool(match op {
                    CompareOp::Eq => values_equal(&left, &right),
                    CompareOp::Ne => !values_equal(&left, &right),
                    _ => compare(&left, &right).is_some_and(|ordering| match op {
                        CompareOp::Lt => ordering == std::cmp::Ordering::Less,
                        CompareOp::Le => ordering != std::cmp::Ordering::Greater,
                        CompareOp::Gt => ordering == std::cmp::Ordering::Greater,
                        CompareOp::Ge => ordering != std::cmp::Ordering::Less,
                        CompareOp::Eq | CompareOp::Ne => false,
                    }),
                })
            }
            Self::InRange { value, start, end } => {
                let value = value.evaluate(data, row);
                let start = start.evaluate(data, row);
                let end = end.evaluate(data, row);
                DataValue::Bool(
                    compare(&value, &start).is_some_and(|ordering| ordering != std::cmp::Ordering::Less)
                        && compare(&value, &end).is_some_and(|ordering| ordering != std::cmp::Ordering::Greater),
                )
            }
            Self::InList { value, values } => {
                let value = value.evaluate(data, row);
                DataValue::Bool(
                    values
                        .iter()
                        .any(|candidate| values_equal(&value, &candidate.evaluate(data, row))),
                )
            }
        }
    }
}

impl Expr {
    pub(super) fn bind(&self, data: &Dataset) -> BoundExpr {
        match self {
            Self::Literal(value) => BoundExpr::Literal(value.clone()),
            Self::Column(name) => data.index_of(name).map_or(BoundExpr::Missing, BoundExpr::Column),
            Self::Not(inner) => BoundExpr::Not(Box::new(inner.bind(data))),
            Self::And(left, right) => BoundExpr::And(Box::new(left.bind(data)), Box::new(right.bind(data))),
            Self::Or(left, right) => BoundExpr::Or(Box::new(left.bind(data)), Box::new(right.bind(data))),
            Self::Compare { op, left, right } => BoundExpr::Compare {
                op: *op,
                left: Box::new(left.bind(data)),
                right: Box::new(right.bind(data)),
            },
            Self::InRange { value, start, end } => BoundExpr::InRange {
                value: Box::new(value.bind(data)),
                start: Box::new(start.bind(data)),
                end: Box::new(end.bind(data)),
            },
            Self::InList { value, values } => BoundExpr::InList {
                value: Box::new(value.bind(data)),
                values: values.iter().map(|value| value.bind(data)).collect(),
            },
        }
    }
}

pub(super) enum BoundExpr {
    Literal(DataValue),
    Column(usize),
    Missing,
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Compare {
        op: CompareOp,
        left: Box<Self>,
        right: Box<Self>,
    },
    InRange {
        value: Box<Self>,
        start: Box<Self>,
        end: Box<Self>,
    },
    InList {
        value: Box<Self>,
        values: Vec<Self>,
    },
}

impl BoundExpr {
    pub(super) fn evaluate(&self, data: &Dataset, row: usize) -> DataValue {
        match self {
            Self::Literal(value) => value.clone(),
            Self::Column(index) => data
                .column(*index)
                .map_or(DataValue::Null, |column| match column.cell(row) {
                    CellValue::Null => DataValue::Null,
                    value => value.to_owned_value(),
                }),
            Self::Missing => DataValue::Null,
            Self::Not(inner) => DataValue::Bool(!inner.evaluate(data, row).is_truthy()),
            Self::And(left, right) => {
                DataValue::Bool(left.evaluate(data, row).is_truthy() && right.evaluate(data, row).is_truthy())
            }
            Self::Or(left, right) => {
                DataValue::Bool(left.evaluate(data, row).is_truthy() || right.evaluate(data, row).is_truthy())
            }
            Self::Compare { op, left, right } => {
                let left = left.evaluate(data, row);
                let right = right.evaluate(data, row);
                DataValue::Bool(match op {
                    CompareOp::Eq => values_equal(&left, &right),
                    CompareOp::Ne => !values_equal(&left, &right),
                    _ => compare(&left, &right).is_some_and(|ordering| match op {
                        CompareOp::Lt => ordering == Ordering::Less,
                        CompareOp::Le => ordering != Ordering::Greater,
                        CompareOp::Gt => ordering == Ordering::Greater,
                        CompareOp::Ge => ordering != Ordering::Less,
                        CompareOp::Eq | CompareOp::Ne => false,
                    }),
                })
            }
            Self::InRange { value, start, end } => {
                let value = value.evaluate(data, row);
                let start = start.evaluate(data, row);
                let end = end.evaluate(data, row);
                DataValue::Bool(
                    compare(&value, &start).is_some_and(|ordering| ordering != Ordering::Less)
                        && compare(&value, &end).is_some_and(|ordering| ordering != Ordering::Greater),
                )
            }
            Self::InList { value, values } => {
                let value = value.evaluate(data, row);
                DataValue::Bool(
                    values
                        .iter()
                        .any(|candidate| values_equal(&value, &candidate.evaluate(data, row))),
                )
            }
        }
    }
}

impl DataValue {
    pub(super) fn is_truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Bool(value) => *value,
            Self::Float(value) => *value != 0.0 && !value.is_nan(),
            Self::Int(value) => *value != 0,
            Self::Text(value) => !value.is_empty(),
            Self::Date(value) | Self::Time(value) => *value != 0,
        }
    }
}

fn values_equal(left: &DataValue, right: &DataValue) -> bool {
    match (left, right) {
        (DataValue::Null, DataValue::Null) => true,
        (DataValue::Null, _) | (_, DataValue::Null) => false,
        (DataValue::Text(left), DataValue::Text(right)) => left == right,
        (DataValue::Bool(left), DataValue::Bool(right)) => left == right,
        _ => left
            .as_f64()
            .zip(right.as_f64())
            .is_some_and(|(left, right)| left == right),
    }
}

fn compare(left: &DataValue, right: &DataValue) -> Option<std::cmp::Ordering> {
    match (left, right) {
        (DataValue::Text(left), DataValue::Text(right)) => Some(left.cmp(right)),
        (DataValue::Bool(left), DataValue::Bool(right)) => Some(left.cmp(right)),
        _ => left.as_f64()?.partial_cmp(&right.as_f64()?),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
