//! A source that receives whole datasets from another thread.

use std::sync::mpsc::{Receiver, TryRecvError};

use super::dataset::Dataset;
use super::source::{ColumnSchema, DataError, DataSource};

/// A source whose current dataset can be replaced by a channel message.
///
/// The sender lives on a worker thread; [`ChannelSource::poll`] uses `try_recv`, so the
/// UI never blocks waiting for the next snapshot.
pub struct ChannelSource {
    schema: Vec<ColumnSchema>,
    data: Dataset,
    receiver: Option<Receiver<Dataset>>,
    revision: u64,
}

impl ChannelSource {
    /// A source with an initial dataset and no channel.
    pub fn new(data: Dataset) -> Self {
        let schema = data.columns().iter().map(ColumnSchema::from_column).collect();
        Self {
            schema,
            data,
            receiver: None,
            revision: 0,
        }
    }

    /// Attach the worker channel.
    pub fn attach(&mut self, receiver: Receiver<Dataset>) {
        self.receiver = Some(receiver);
    }

    /// Take every finished snapshot. Returns whether the data changed.
    pub fn poll(&mut self) -> bool {
        let Some(receiver) = &self.receiver else {
            return false;
        };
        let mut changed = false;
        loop {
            match receiver.try_recv() {
                Ok(data) => {
                    self.schema = data.columns().iter().map(ColumnSchema::from_column).collect();
                    self.data = data;
                    self.revision += 1;
                    changed = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.receiver = None;
                    break;
                }
            }
        }
        changed
    }

    /// The current dataset.
    pub fn data(&self) -> &Dataset {
        &self.data
    }
}

impl DataSource for ChannelSource {
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

    fn revision(&self) -> u64 {
        self.revision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Column;

    #[test]
    fn a_channel_replaces_the_snapshot_without_blocking() {
        let initial = Dataset::from_columns([Column::ints("n", vec![1])]);
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut source = ChannelSource::new(initial);
        source.attach(receiver);
        assert!(!source.poll());
        sender
            .send(Dataset::from_columns([Column::ints("n", vec![2, 3])]))
            .expect("send");
        assert!(source.poll());
        assert_eq!(source.row_count(), 2);
        assert!(source.revision() > 0);
        assert!(source.read_range(0..2).is_ok());
    }
}
