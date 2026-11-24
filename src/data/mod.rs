//! The typed, lazy data model shared by tables, viewers, and plots.
//!
//! A [`Dataset`] is columnar: floats, integers, booleans, text, dates, times, and
//! categories, with a validity mask and per-column units/metadata. A [`DataSource`] adds
//! paging, change notification, and cancellation without an async runtime. The query
//! language filters rows, the CSV connector opens delimited files, and the ring and
//! channel sources cover streaming data.

mod channel;
mod column;
mod csv;
mod dataset;
mod query;
mod ring;
mod source;

pub use channel::ChannelSource;
pub use column::{CellValue, Column, ColumnType, ColumnValues, DataValue};
pub use csv::{CsvOptions, CsvSource, parse_csv};
pub use dataset::{ColumnStats, Dataset};
pub use query::{Query, QueryError};
pub use ring::RingBuffer;
pub use source::{ColumnSchema, DataError, DataSource, MemorySource};
