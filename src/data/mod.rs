//! The typed, lazy data model shared by tables, viewers, and plots.
//!
//! A [`Dataset`] is columnar: floats, integers, booleans, text, dates, times, and
//! categories, with a validity mask and per-column units/metadata. A [`DataSource`] adds
//! paging, change notification, and cancellation without an async runtime. The query
//! language filters rows, the CSV connector opens delimited files, and the ring and
//! channel sources cover streaming data.

#[cfg(feature = "arrow")]
mod arrow_ipc;
#[cfg(feature = "channel")]
mod channel;
mod column;
#[cfg(feature = "csv")]
mod csv;
mod dataset;
mod link;
#[cfg(feature = "parquet")]
mod parquet_file;
mod query;
#[cfg(feature = "ring")]
mod ring;
mod source;
#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "workbook")]
mod workbook;

#[cfg(feature = "arrow")]
pub use arrow_ipc::{ArrowSource, dataset_from_batch, read_arrow_ipc};
#[cfg(feature = "channel")]
pub use channel::ChannelSource;
pub use column::{CellValue, Column, ColumnType, ColumnValues, DataValue};
#[cfg(feature = "csv")]
pub use csv::{CsvOptions, CsvSource, parse_csv};
pub use dataset::{ColumnStats, Dataset};
pub use link::{LinkHub, LinkMode, LinkSelection};
#[cfg(feature = "parquet")]
pub use parquet_file::{ParquetSource, read_parquet_bytes};
pub use query::{Query, QueryError};
#[cfg(feature = "ring")]
pub use ring::RingBuffer;
pub use source::{ColumnSchema, DataError, DataSource, MemorySource};
#[cfg(feature = "sqlite")]
pub use sqlite::{SqliteSource, read_sqlite};
#[cfg(feature = "workbook")]
pub use workbook::WorkbookSource;
