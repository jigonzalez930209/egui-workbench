//! A lazily built min/max pyramid for out-of-core range queries.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use super::column::CellValue;
use super::source::{DataError, DataSource};

/// One block of a level: its x span and the y minimum/maximum in that span.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LodTile {
    /// First x in the block.
    pub x_min: f64,
    /// Last x in the block.
    pub x_max: f64,
    /// Smallest y in the block.
    pub y_min: f64,
    /// Largest y in the block.
    pub y_max: f64,
    /// First row (inclusive).
    pub row_start: usize,
    /// Last row (exclusive).
    pub row_end: usize,
}

impl LodTile {
    /// Four points that outline the tile for a line plot.
    pub fn outline(self) -> [[f64; 2]; 4] {
        [
            [self.x_min, self.y_min],
            [self.x_min, self.y_max],
            [self.x_max, self.y_min],
            [self.x_max, self.y_max],
        ]
    }
}

/// A min/max pyramid over a paged source. Levels are built on first use and blocks are
/// cached, so a plot asks only for the tiles its viewport needs.
pub struct MinMaxPyramid {
    source: Arc<dyn DataSource>,
    x_column: usize,
    y_column: usize,
    block: usize,
    levels: RefCell<Vec<Option<Vec<LodTile>>>>,
    cache: RefCell<HashMap<(usize, usize), LodTile>>,
    revision: u64,
}

impl MinMaxPyramid {
    /// A pyramid over two columns of `source`, with `block` rows per tile.
    pub fn new(source: Arc<dyn DataSource>, x_column: usize, y_column: usize, block: usize) -> Self {
        Self {
            source,
            x_column,
            y_column,
            block: block.max(2),
            levels: RefCell::new(Vec::new()),
            cache: RefCell::new(HashMap::new()),
            revision: 0,
        }
    }

    /// The number of rows in the source.
    pub fn row_count(&self) -> usize {
        self.source.row_count()
    }

    /// The block size at level 0.
    pub fn block(&self) -> usize {
        self.block
    }

    /// The source revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Invalidate every tile after the source changed.
    pub fn invalidate(&mut self) {
        self.levels.get_mut().clear();
        self.cache.get_mut().clear();
        self.revision = self.revision.wrapping_add(1);
    }

    /// The tile at `level` and `index`, building the level when needed.
    ///
    /// # Errors
    /// When a source page cannot be read.
    pub fn tile(&self, level: usize, index: usize) -> Result<Option<LodTile>, DataError> {
        if let Some(tile) = self.cache.borrow().get(&(level, index)) {
            return Ok(Some(*tile));
        }
        // A level past the width of `usize` has no tiles; building it would shift out of
        // range and reserve `level` empty slots.
        if self
            .block
            .checked_shl(u32::try_from(level).unwrap_or(u32::MAX))
            .is_none_or(|block| block >> level != self.block)
        {
            return Ok(None);
        }
        self.build_level(level)?;
        let tile = self
            .levels
            .borrow()
            .get(level)
            .and_then(|level| level.as_ref())
            .and_then(|tiles| tiles.get(index).copied());
        if let Some(tile) = tile {
            self.cache.borrow_mut().insert((level, index), tile);
        }
        Ok(tile)
    }

    /// Tiles that cover `range` at a level chosen to fit `budget`.
    ///
    /// # Errors
    /// When a source page cannot be read.
    pub fn range(&self, range: std::ops::Range<usize>, budget: usize) -> Result<Vec<LodTile>, DataError> {
        let start = range.start.min(self.row_count());
        let end = range.end.min(self.row_count()).max(start);
        if end <= start {
            return Ok(Vec::new());
        }
        let span = end - start;
        let mut level = 0;
        while (span / (self.block << level)).max(1) > budget.max(1) {
            level += 1;
        }
        self.build_level(level)?;
        let mut tiles = Vec::new();
        if let Some(Some(level_tiles)) = self.levels.borrow().get(level) {
            for (index, tile) in level_tiles.iter().enumerate() {
                if tile.row_end > start && tile.row_start < end {
                    tiles.push(*tile);
                    if tiles.len() >= budget {
                        break;
                    }
                }
                let _ = index;
            }
        }
        Ok(tiles)
    }

    /// A decimated polyline with four points per tile.
    ///
    /// # Errors
    /// When a source page cannot be read.
    pub fn polyline(&self, range: std::ops::Range<usize>, budget: usize) -> Result<Vec<[f64; 2]>, DataError> {
        let mut points = Vec::new();
        for tile in self.range(range, budget)? {
            points.extend(tile.outline());
        }
        Ok(points)
    }

    fn build_level(&self, level: usize) -> Result<(), DataError> {
        if self.levels.borrow().get(level).is_some_and(Option::is_some) {
            return Ok(());
        }
        let block = self.block << level;
        let rows = self.row_count();
        let mut tiles = Vec::with_capacity(rows.div_ceil(block).max(1));
        let mut start = 0;
        while start < rows {
            let end = (start + block).min(rows);
            let page = self.source.read_range(start..end)?;
            let mut x_min = f64::INFINITY;
            let mut x_max = f64::NEG_INFINITY;
            let mut y_min = f64::INFINITY;
            let mut y_max = f64::NEG_INFINITY;
            for row in 0..page.row_count() {
                let x = numeric(page.cell(row, self.x_column));
                let y = numeric(page.cell(row, self.y_column));
                if let (Some(x), Some(y)) = (x, y) {
                    x_min = x_min.min(x);
                    x_max = x_max.max(x);
                    y_min = y_min.min(y);
                    y_max = y_max.max(y);
                }
            }
            if x_min.is_finite() && y_min.is_finite() {
                tiles.push(LodTile {
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                    row_start: start,
                    row_end: end,
                });
            }
            start = end;
        }
        let mut levels = self.levels.borrow_mut();
        while levels.len() <= level {
            levels.push(None);
        }
        levels[level] = Some(tiles);
        Ok(())
    }
}

fn numeric(cell: CellValue<'_>) -> Option<f64> {
    match cell {
        CellValue::Float(value) => value.is_finite().then_some(value),
        CellValue::Int(value) => Some(value as f64),
        CellValue::Date(value) | CellValue::Time(value) => Some(value as f64),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Column, Dataset, MemorySource};

    fn source() -> Arc<dyn DataSource> {
        let x: Vec<f64> = (0..10_000).map(|index| index as f64 * 0.01).collect();
        let y: Vec<f64> = x.iter().map(|x| x.sin()).collect();
        Arc::new(MemorySource::new(Dataset::from_columns([
            Column::floats("t", x),
            Column::floats("v", y),
        ])))
    }

    #[test]
    fn tiles_keep_the_extrema_and_outline() {
        let pyramid = MinMaxPyramid::new(source(), 0, 1, 256);
        let tile = pyramid.tile(0, 0).expect("read").expect("tile");
        assert!(tile.x_min <= tile.x_max);
        assert!(tile.y_min <= tile.y_max);
        assert_eq!(tile.outline().len(), 4);
        assert_eq!(tile.row_start, 0);
        assert_eq!(tile.row_end, 256);
    }

    #[test]
    fn a_range_chooses_a_level_within_the_budget() {
        let pyramid = MinMaxPyramid::new(source(), 0, 1, 64);
        let tiles = pyramid.range(0..10_000, 32).expect("range");
        assert!(!tiles.is_empty());
        assert!(tiles.len() <= 32);
        let points = pyramid.polyline(0..10_000, 32).expect("polyline");
        assert_eq!(points.len(), tiles.len() * 4);
    }

    #[test]
    fn invalidation_rebuilds() {
        let mut pyramid = MinMaxPyramid::new(source(), 0, 1, 64);
        let _ = pyramid.tile(0, 0).expect("tile");
        let revision = pyramid.revision();
        pyramid.invalidate();
        assert_ne!(pyramid.revision(), revision);
        assert!(pyramid.tile(0, 0).expect("tile").is_some());
    }
}
