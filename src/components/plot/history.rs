//! Back and forward through zoom levels.

use super::Bounds;

/// A bounded back/forward history of plot views.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewHistory {
    entries: Vec<Option<Bounds>>,
    index: usize,
}

impl ViewHistory {
    /// A history starting at the automatic view (`None`).
    pub fn new() -> Self {
        Self {
            entries: vec![None],
            index: 0,
        }
    }

    /// A history starting at a view.
    pub fn with_view(view: Bounds) -> Self {
        Self {
            entries: vec![Some(view)],
            index: 0,
        }
    }

    /// Push a view, dropping the forward entries. Identical views are ignored.
    pub fn push(&mut self, view: Option<Bounds>) {
        if self.current() == view.as_ref() {
            return;
        }
        self.entries.truncate(self.index + 1);
        self.entries.push(view);
        self.index = self.entries.len() - 1;
        if self.entries.len() > 64 {
            self.entries.remove(0);
            self.index = self.index.saturating_sub(1);
        }
    }

    /// The current view.
    pub fn current(&self) -> Option<&Bounds> {
        self.entries.get(self.index).and_then(Option::as_ref)
    }

    /// Go back. Returns whether the index moved.
    pub fn back(&mut self) -> bool {
        if self.index == 0 {
            return false;
        }
        self.index -= 1;
        true
    }

    /// Go forward. Returns whether the index moved.
    pub fn forward(&mut self) -> bool {
        if self.index + 1 >= self.entries.len() {
            return false;
        }
        self.index += 1;
        true
    }

    /// Whether back is possible.
    pub fn can_back(&self) -> bool {
        self.index > 0
    }

    /// Whether forward is possible.
    pub fn can_forward(&self) -> bool {
        self.index + 1 < self.entries.len()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the history has only the initial view.
    pub fn is_empty(&self) -> bool {
        self.entries.len() <= 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_and_forward_walk_the_zoom_history() {
        let mut history = ViewHistory::new();
        let first = Bounds::from_corners([0.0, 0.0], [1.0, 1.0]);
        let second = Bounds::from_corners([0.2, 0.2], [0.8, 0.8]);
        history.push(Some(first));
        history.push(Some(second));
        assert_eq!(history.current(), Some(&second));
        assert!(history.back());
        assert_eq!(history.current(), Some(&first));
        assert!(history.forward());
        assert_eq!(history.current(), Some(&second));
        history.push(None);
        assert!(!history.can_forward());
    }

    #[test]
    fn duplicate_views_do_not_grow_the_history() {
        let mut history = ViewHistory::with_view(Bounds::from_corners([0.0, 0.0], [1.0, 1.0]));
        history.push(Some(Bounds::from_corners([0.0, 0.0], [1.0, 1.0])));
        assert_eq!(history.len(), 1);
    }
}
