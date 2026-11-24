//! A small, safe expression language for filters and derived columns.
//!
//! It supports comparison, `&&`, `||`, `!`, parentheses, membership in a range or a
//! list, and string/boolean/number/null literals. There are no function calls, no
//! assignments, and no I/O: a query can only read the row it is given.

use std::fmt;

use super::Dataset;

mod ast;
mod parser;

use ast::Expr;
use parser::{Lexer, Parser};

/// A parsed filter expression.
#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    source: String,
    expression: Expr,
}

impl Query {
    /// Parse an expression.
    ///
    /// # Errors
    /// When the token or the grammar is invalid, with a byte offset.
    pub fn parse(source: &str) -> Result<Self, QueryError> {
        let tokens = Lexer::new(source).tokens()?;
        let mut parser = Parser::new(tokens);
        let expression = parser.expression()?;
        parser.expect_end()?;
        Ok(Self {
            source: source.to_owned(),
            expression,
        })
    }

    /// The source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Whether the row matches. A missing column is an error at evaluation time and
    /// evaluates to false.
    pub fn matches(&self, data: &Dataset, row: usize) -> bool {
        self.expression.evaluate(data, row).is_truthy()
    }

    /// The matching rows, in order.
    pub fn matching_rows(&self, data: &Dataset) -> Vec<usize> {
        (0..data.row_count()).filter(|row| self.matches(data, *row)).collect()
    }
}

impl fmt::Display for Query {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.source)
    }
}

/// What went wrong while parsing or evaluating a query.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct QueryError {
    message: String,
    offset: usize,
}

impl QueryError {
    /// A human-readable message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The byte offset of the problem.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.offset)
    }
}

impl std::error::Error for QueryError {}
