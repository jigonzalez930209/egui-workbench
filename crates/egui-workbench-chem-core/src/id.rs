//! Stable document, atom, and bond identifiers.

use serde::{Deserialize, Serialize};

/// The identifier of one chemical document.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentId(u64);

impl DocumentId {
    /// Wrap a numeric document id.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The numeric value.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "document-{}", self.0)
    }
}

/// A generational atom identifier.
///
/// A slot can be reused after an atom is removed; the generation keeps stale ids from
/// resolving to the new occupant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AtomId {
    slot: u32,
    generation: u32,
}

impl AtomId {
    /// Build an id from a slot and generation.
    pub const fn new(slot: u32, generation: u32) -> Self {
        Self { slot, generation }
    }

    /// The slot.
    pub const fn slot(self) -> u32 {
        self.slot
    }

    /// The generation.
    pub const fn generation(self) -> u32 {
        self.generation
    }
}

impl std::fmt::Display for AtomId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "atom-{}.{}", self.slot, self.generation)
    }
}

/// A generational bond identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BondId {
    slot: u32,
    generation: u32,
}

impl BondId {
    /// Build an id from a slot and generation.
    pub const fn new(slot: u32, generation: u32) -> Self {
        Self { slot, generation }
    }

    /// The slot.
    pub const fn slot(self) -> u32 {
        self.slot
    }

    /// The generation.
    pub const fn generation(self) -> u32 {
        self.generation
    }
}

impl std::fmt::Display for BondId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "bond-{}.{}", self.slot, self.generation)
    }
}

/// A generational slot allocator that serializes with the document.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IdPool {
    generations: Vec<u32>,
    free: Vec<u32>,
    live: usize,
}

impl IdPool {
    /// An empty pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate `(slot, generation)`.
    pub fn allocate(&mut self) -> (u32, u32) {
        // A pool read from a document can list a free slot that has no generation; skip
        // those instead of indexing past the end below.
        let mut reused = None;
        while let Some(slot) = self.free.pop() {
            if let Some(generation) = self.generations.get_mut(slot as usize) {
                *generation = generation.wrapping_add(1);
                reused = Some(slot);
                break;
            }
        }
        let slot = if let Some(slot) = reused {
            slot
        } else {
            let slot = self.generations.len() as u32;
            self.generations.push(1);
            slot
        };
        self.live += 1;
        let generation = self.generations[slot as usize];
        (slot, generation)
    }

    /// Release `slot`; returns whether it was live.
    pub fn release(&mut self, slot: u32) -> bool {
        if !self.is_live(slot) {
            return false;
        }
        self.free.push(slot);
        self.live = self.live.saturating_sub(1);
        true
    }

    /// Whether `(slot, generation)` is currently live.
    pub fn contains(&self, slot: u32, generation: u32) -> bool {
        self.is_live(slot) && self.generations.get(slot as usize) == Some(&generation)
    }

    /// Number of live slots.
    pub fn live(&self) -> usize {
        self.live
    }

    fn is_live(&self, slot: u32) -> bool {
        self.generations.get(slot as usize).is_some() && !self.free.contains(&slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_generational_and_never_resolve_stale_ids() {
        let mut pool = IdPool::new();
        let (first, first_generation) = pool.allocate();
        assert!(pool.contains(first, first_generation));
        assert!(pool.release(first));
        let (second, second_generation) = pool.allocate();
        assert_eq!(first, second);
        assert_ne!(first_generation, second_generation);
        assert!(!pool.contains(first, first_generation));
        assert!(pool.contains(second, second_generation));
        assert_eq!(pool.live(), 1);
    }

    #[test]
    fn releasing_twice_is_a_no_op() {
        let mut pool = IdPool::new();
        let (slot, _) = pool.allocate();
        assert!(pool.release(slot));
        assert!(!pool.release(slot));
        assert_eq!(pool.live(), 0);
    }
}

#[cfg(test)]
mod hardening_tests {
    use super::*;

    #[test]
    fn a_pool_with_a_dangling_free_slot_does_not_panic() {
        let mut pool: IdPool = serde_json::from_str(r#"{"generations":[],"free":[7],"live":0}"#).expect("json");
        let (slot, generation) = pool.allocate();
        assert_eq!((slot, generation), (0, 1));
        assert!(pool.contains(slot, generation));
    }
}
