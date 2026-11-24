//! A selection bus shared by a table, one or more plots, and a grid.
//!
//! Each [`LinkHub`] belongs to one dataset. Views publish row sets or a predicate and
//! read the current selection; the mode says whether other views highlight, filter, or
//! isolate the selected rows.

use std::collections::BTreeSet;

use super::Query;

/// How linked views react to a selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum LinkMode {
    /// Draw selected rows with emphasis, keep the rest visible.
    #[default]
    Highlight,
    /// Hide rows that are not selected.
    Filter,
    /// Show only the selected rows.
    Isolate,
}

/// One selection in a dataset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct LinkSelection {
    /// Selected row indices.
    pub rows: BTreeSet<usize>,
    /// The view that published the selection.
    pub source: Option<String>,
    /// A predicate used by a plot brush, preserved for a table filter.
    pub predicate: Option<String>,
}

impl LinkSelection {
    /// Whether no rows are selected.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.predicate.is_none()
    }

    /// Whether a row is selected.
    pub fn contains(&self, row: usize) -> bool {
        self.rows.contains(&row)
    }
}

/// The selection bus for one dataset.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkHub {
    /// Dataset key, such as a path or a document id.
    pub key: String,
    /// Current selection.
    pub selection: LinkSelection,
    /// How views react.
    pub mode: LinkMode,
    revision: u64,
}

impl LinkHub {
    /// A hub with an empty selection.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            selection: LinkSelection::default(),
            mode: LinkMode::Highlight,
            revision: 0,
        }
    }

    /// Replace the selection with rows.
    pub fn set_rows(&mut self, rows: impl IntoIterator<Item = usize>, source: impl Into<String>) {
        self.selection = LinkSelection {
            rows: rows.into_iter().collect(),
            source: Some(source.into()),
            predicate: None,
        };
        self.revision += 1;
    }

    /// Replace the selection with a predicate and the rows it matches.
    pub fn set_predicate(
        &mut self,
        predicate: impl Into<String>,
        rows: impl IntoIterator<Item = usize>,
        source: impl Into<String>,
    ) {
        self.selection = LinkSelection {
            rows: rows.into_iter().collect(),
            source: Some(source.into()),
            predicate: Some(predicate.into()),
        };
        self.revision += 1;
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        if !self.selection.is_empty() {
            self.selection = LinkSelection::default();
            self.revision += 1;
        }
    }

    /// The current selection.
    pub fn selection(&self) -> &LinkSelection {
        &self.selection
    }

    /// The mode.
    pub fn mode(&self) -> LinkMode {
        self.mode
    }

    /// Change the mode.
    pub fn set_mode(&mut self, mode: LinkMode) {
        if mode != self.mode {
            self.mode = mode;
            self.revision += 1;
        }
    }

    /// A revision that changes with the selection or the mode.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Rows matching `predicate` in `data`, for a table or plot brush.
    pub fn rows_for(&self, data: &super::Dataset, predicate: &str) -> Option<Vec<usize>> {
        Query::parse(predicate).ok().map(|query| query.matching_rows(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selections_and_modes_bump_the_revision() {
        let mut hub = LinkHub::new("trace.csv");
        assert!(hub.selection().is_empty());
        hub.set_rows([1, 3], "plot");
        assert!(hub.selection().contains(3));
        assert_eq!(hub.selection().source.as_deref(), Some("plot"));
        assert!(hub.revision() > 0);
        hub.set_mode(LinkMode::Filter);
        assert_eq!(hub.mode(), LinkMode::Filter);
        hub.clear();
        assert!(hub.selection().is_empty());
    }
}
