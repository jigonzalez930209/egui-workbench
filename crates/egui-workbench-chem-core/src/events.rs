//! Typed events shared through the host event bus.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::id::{AtomId, DocumentId};
use crate::jobs::RepaintWaker;
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

/// A bounded FIFO of chemistry events that can wake the UI on push.
#[derive(Clone)]
pub struct EventQueue {
    events: VecDeque<ChemEvent>,
    capacity: usize,
    waker: Option<RepaintWaker>,
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::with_capacity(1024)
    }
}

impl std::fmt::Debug for EventQueue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EventQueue")
            .field("events", &self.events)
            .field("capacity", &self.capacity)
            .field("waker", &self.waker.is_some())
            .finish()
    }
}

impl EventQueue {
    /// A queue that keeps at most `capacity` events, dropping the oldest.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity.min(1024)),
            capacity: capacity.max(1),
            waker: None,
        }
    }

    /// A queue that calls `waker` after each push, for `ctx.request_repaint()`.
    pub fn with_waker(capacity: usize, waker: impl Fn() + Send + Sync + 'static) -> Self {
        let mut queue = Self::with_capacity(capacity);
        queue.set_waker(waker);
        queue
    }

    /// Replace the repaint callback.
    pub fn set_waker(&mut self, waker: impl Fn() + Send + Sync + 'static) {
        self.waker = Some(std::sync::Arc::new(waker));
    }

    /// Push one event and wake the UI.
    pub fn push(&mut self, event: ChemEvent) {
        if self.events.len() == self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
        if let Some(waker) = &self.waker {
            waker();
        }
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
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

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

    #[test]
    fn pushing_wakes_the_ui() {
        let wakes = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&wakes);
        let mut queue = EventQueue::with_waker(8, move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });
        queue.push(ChemEvent::Request3DConformers {
            document: DocumentId::new(1),
        });
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
    }
}
