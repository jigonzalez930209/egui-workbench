//! Typed events shared through the host event bus.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::id::{AtomId, DocumentId};
use crate::model::UnitCell;
use crate::selection::ChemSelection;

/// One chemistry change.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ChemEvent {
    /// An atom, bond, or label changed.
    StructureModified {
        /// Document that changed.
        document: DocumentId,
        /// The atom involved, when there is one.
        atom: Option<AtomId>,
    },
    /// The shared selection changed.
    SelectionChanged {
        /// Document the selection belongs to.
        document: DocumentId,
        /// The new selection.
        selection: ChemSelection,
    },
    /// Coordinates changed without a topology change.
    GeometryUpdated {
        /// Document that changed.
        document: DocumentId,
        /// The atom involved, when there is one.
        atom: Option<AtomId>,
    },
    /// A panel wants 3D conformers for the document.
    Request3DConformers {
        /// Document to embed.
        document: DocumentId,
    },
    /// The crystal cell changed.
    CrystalCellUpdated {
        /// Document that changed.
        document: DocumentId,
        /// The new cell.
        cell: UnitCell,
    },
}

/// A bounded FIFO of chemistry events.
#[derive(Clone, Debug)]
pub struct EventQueue {
    events: VecDeque<ChemEvent>,
    capacity: usize,
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::with_capacity(1024)
    }
}

impl EventQueue {
    /// A queue that keeps at most `capacity` events, dropping the oldest.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity.min(1024)),
            capacity: capacity.max(1),
        }
    }

    /// Push one event.
    pub fn push(&mut self, event: ChemEvent) {
        if self.events.len() == self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Take every queued event.
    pub fn drain(&mut self) -> impl Iterator<Item = ChemEvent> + '_ {
        self.events.drain(..)
    }

    /// Queued event count.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// The configured capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Remove every event.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_queue_is_fifo_and_bounded() {
        let document = DocumentId::new(1);
        let mut queue = EventQueue::with_capacity(2);
        queue.push(ChemEvent::StructureModified { document, atom: None });
        queue.push(ChemEvent::Request3DConformers { document });
        queue.push(ChemEvent::GeometryUpdated { document, atom: None });
        assert_eq!(queue.len(), 2);
        let events = queue.drain().collect::<Vec<_>>();
        assert!(matches!(events[0], ChemEvent::Request3DConformers { .. }));
        assert!(matches!(events[1], ChemEvent::GeometryUpdated { .. }));
        assert!(queue.is_empty());
    }
}
