//! A generic document list with one active document.
//!
//! Applications implement [`Document`] for their own tab model and keep the actual
//! content in `T`. [`Documents`] only manages order, the active index, and the dirty
//! query used by the close guard. It performs no I/O and knows nothing about paths.

/// One tab in a [`Documents`] list.
pub trait Document {
    /// The label shown on the tab, without the dirty marker.
    fn title(&self) -> &str;

    /// Whether closing this document needs confirmation.
    fn is_dirty(&self) -> bool {
        false
    }
}

/// An ordered list of documents with one active document.
///
/// Removing a document keeps the same document active when possible: removing a
/// document before the active one shifts the index, removing the active one selects
/// its successor (or the new last document).
///
/// ```
/// # use egui_workbench::app::{Document, Documents};
/// struct Doc(&'static str);
/// impl Document for Doc {
///     fn title(&self) -> &str { self.0 }
/// }
/// let mut docs = Documents::with_items(vec![Doc("a"), Doc("b")]);
/// assert_eq!(docs.active().map(Document::title), Some("a"));
/// docs.select_next();
/// assert_eq!(docs.active().map(Document::title), Some("b"));
/// docs.remove(0);
/// assert_eq!(docs.active_index(), 0);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Documents<T> {
    items: Vec<T>,
    active: usize,
}

impl<T> Default for Documents<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Documents<T> {
    /// An empty list.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active: 0,
        }
    }

    /// A list with the first document active.
    pub fn with_items(items: Vec<T>) -> Self {
        Self { items, active: 0 }
    }

    /// The number of documents.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether there are no documents.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Every document in tab order.
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Every document in tab order, mutably.
    pub fn items_mut(&mut self) -> &mut [T] {
        &mut self.items
    }

    /// The active index, or 0 when empty.
    pub fn active_index(&self) -> usize {
        self.active
    }

    /// The active document.
    pub fn active(&self) -> Option<&T> {
        self.items.get(self.active)
    }

    /// The active document, mutably.
    pub fn active_mut(&mut self) -> Option<&mut T> {
        self.items.get_mut(self.active)
    }

    /// A document by index.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    /// A document by index, mutably.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index)
    }

    /// Select an existing document. Returns `false` when the index is out of range.
    pub fn set_active(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            self.active = index;
            true
        } else {
            false
        }
    }

    /// Append a document. Returns its index. The active document does not change.
    pub fn push(&mut self, item: T) -> usize {
        let index = self.items.len();
        self.items.push(item);
        index
    }

    /// Append a document and select it. Returns its index.
    pub fn push_active(&mut self, item: T) -> usize {
        let index = self.push(item);
        self.active = index;
        index
    }

    /// Insert a document at `index` (clamped to the end). Returns its index.
    ///
    /// When inserting at or before the active document, the active document stays
    /// selected and its index shifts by one.
    pub fn insert(&mut self, index: usize, item: T) -> usize {
        let index = index.min(self.items.len());
        if !self.items.is_empty() && index <= self.active {
            self.active += 1;
        }
        self.items.insert(index, item);
        index
    }

    /// Remove a document and return it. Returns `None` when the index is out of range.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.items.len() {
            return None;
        }
        let removed = self.items.remove(index);
        if self.items.is_empty() {
            self.active = 0;
        } else if index < self.active {
            self.active -= 1;
        } else if index == self.active {
            self.active = index.min(self.items.len() - 1);
        }
        Some(removed)
    }

    /// Remove the active document and return it.
    pub fn close_active(&mut self) -> Option<T> {
        self.remove(self.active)
    }

    /// Select the next document, wrapping around. Returns `false` when empty.
    pub fn select_next(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }
        self.active = (self.active + 1) % self.items.len();
        true
    }

    /// Select the previous document, wrapping around. Returns `false` when empty.
    pub fn select_previous(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }
        self.active = (self.active + self.items.len() - 1) % self.items.len();
        true
    }

    /// Select the first document matching `predicate`. Returns its index.
    pub fn select_by(&mut self, predicate: impl FnMut(&T) -> bool) -> Option<usize> {
        let index = self.items.iter().position(predicate)?;
        self.active = index;
        Some(index)
    }

    /// Remove every document.
    pub fn clear(&mut self) {
        self.items.clear();
        self.active = 0;
    }

    /// The title of the active document.
    pub fn active_title(&self) -> Option<&str>
    where
        T: Document,
    {
        self.active().map(Document::title)
    }

    /// Whether any document is dirty.
    pub fn any_dirty(&self) -> bool
    where
        T: Document,
    {
        self.items.iter().any(Document::is_dirty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Doc(&'static str, bool);

    impl Document for Doc {
        fn title(&self) -> &str {
            self.0
        }

        fn is_dirty(&self) -> bool {
            self.1
        }
    }

    fn docs() -> Documents<Doc> {
        Documents::with_items(vec![Doc("a", false), Doc("b", true), Doc("c", false)])
    }

    #[test]
    fn removing_before_active_keeps_the_same_document() {
        let mut list = docs();
        assert!(list.set_active(2));
        list.remove(0);
        assert_eq!(list.active_title(), Some("c"));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn removing_active_selects_the_successor() {
        let mut list = docs();
        assert!(list.set_active(1));
        list.remove(1);
        assert_eq!(list.active_title(), Some("c"));
        list.close_active();
        assert_eq!(list.active_title(), Some("a"));
        list.close_active();
        assert!(list.is_empty());
        assert_eq!(list.active_index(), 0);
    }

    #[test]
    fn removing_after_active_does_not_move_it() {
        let mut list = docs();
        list.remove(2);
        assert_eq!(list.active_title(), Some("a"));
    }

    #[test]
    fn navigation_wraps_and_handles_empty_lists() {
        let mut list = docs();
        assert!(list.select_previous());
        assert_eq!(list.active_title(), Some("c"));
        assert!(list.select_next());
        assert_eq!(list.active_title(), Some("a"));
        list.clear();
        assert!(!list.select_next());
        assert!(!list.select_previous());
        assert!(!list.set_active(0));
    }

    #[test]
    fn insert_shifts_the_active_index() {
        let mut list = docs();
        assert!(list.set_active(1));
        list.insert(0, Doc("x", false));
        assert_eq!(list.active_title(), Some("b"));
        list.insert(99, Doc("z", false));
        assert_eq!(list.active_index(), 2);
        assert!(list.any_dirty());
    }

    #[test]
    fn select_by_finds_a_document() {
        let mut list = docs();
        assert_eq!(list.select_by(|doc| doc.0 == "c"), Some(2));
        assert_eq!(list.active_title(), Some("c"));
        assert_eq!(list.select_by(|doc| doc.0 == "missing"), None);
    }
}
