//! Append-only streaming series with live/frozen mode, a rolling window, and triggers.

use std::collections::VecDeque;

/// Whether a streaming series accepts new points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum StreamMode {
    /// New points replace old ones.
    #[default]
    Live,
    /// The current window is held; pushes are ignored until Live.
    Frozen,
}

/// A level crossing that fires once per edge.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trigger {
    /// Threshold.
    pub level: f64,
    /// Fire when the value rises through the level.
    pub rising: bool,
    armed: bool,
}

impl Trigger {
    /// A rising-edge trigger.
    pub fn rising(level: f64) -> Self {
        Self {
            level,
            rising: true,
            armed: true,
        }
    }

    /// A falling-edge trigger.
    pub fn falling(level: f64) -> Self {
        Self {
            level,
            rising: false,
            armed: true,
        }
    }

    /// Re-arm the trigger.
    pub fn arm(&mut self) {
        self.armed = true;
    }

    /// Feed one value; true on the edge that fires.
    pub fn update(&mut self, value: f64) -> bool {
        let crossed = if self.rising {
            value >= self.level
        } else {
            value <= self.level
        };
        if crossed && self.armed {
            self.armed = false;
            true
        } else if !crossed {
            self.armed = true;
            false
        } else {
            false
        }
    }
}

/// A bounded, append-only (x, y) series for live data.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamingSeries {
    capacity: usize,
    x_window: Option<f64>,
    mode: StreamMode,
    points: VecDeque<[f64; 2]>,
    trigger: Option<Trigger>,
    revision: u64,
    fired: u64,
}

impl StreamingSeries {
    /// A live series holding `capacity` points. A capacity of zero is unlimited.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            x_window: None,
            mode: StreamMode::Live,
            points: VecDeque::new(),
            trigger: None,
            revision: 0,
            fired: 0,
        }
    }

    /// Keep only points within `span` of the newest x.
    pub fn with_window(mut self, span: f64) -> Self {
        self.x_window = (span > 0.0).then_some(span);
        self
    }

    /// Attach a trigger.
    pub fn with_trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// The mode.
    pub fn mode(&self) -> StreamMode {
        self.mode
    }

    /// Switch live/frozen. Frozen drops pushes until Live.
    pub fn set_mode(&mut self, mode: StreamMode) {
        if mode != self.mode {
            self.mode = mode;
            self.revision += 1;
        }
    }

    /// Number of points.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Whether there are no points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// The current points, oldest first.
    pub fn points(&self) -> Vec<[f64; 2]> {
        self.points.iter().copied().collect()
    }

    /// A revision that changes on every accepted push, mode change, or trim.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// A revision that changes only when the trigger fires.
    pub fn fired(&self) -> u64 {
        self.fired
    }

    /// Append a point. Returns whether the trigger fired on this point.
    pub fn push(&mut self, point: [f64; 2]) -> bool {
        if self.mode == StreamMode::Frozen || !point[0].is_finite() || !point[1].is_finite() {
            return false;
        }
        if self.capacity > 0 && self.points.len() == self.capacity {
            self.points.pop_front();
        }
        self.points.push_back(point);
        if let Some(window) = self.x_window {
            let min_x = point[0] - window;
            while self.points.front().is_some_and(|front| front[0] < min_x) {
                self.points.pop_front();
            }
        }
        self.revision += 1;
        let fired = self.trigger.as_mut().is_some_and(|trigger| trigger.update(point[1]));
        if fired {
            self.fired += 1;
        }
        fired
    }

    /// Remove every point.
    pub fn clear(&mut self) {
        self.points.clear();
        self.revision += 1;
    }

    /// The newest point.
    pub fn last(&self) -> Option<[f64; 2]> {
        self.points.back().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_and_window_trim_the_oldest_points() {
        let mut series = StreamingSeries::new(3).with_window(2.0);
        for index in 0..6 {
            series.push([index as f64, index as f64]);
        }
        assert_eq!(series.len(), 3);
        assert_eq!(series.last(), Some([5.0, 5.0]));
        assert!(series.points().iter().all(|point| point[0] >= 3.0));
    }

    #[test]
    fn frozen_mode_holds_the_window() {
        let mut series = StreamingSeries::new(8);
        series.push([1.0, 1.0]);
        series.set_mode(StreamMode::Frozen);
        assert!(!series.push([2.0, 2.0]));
        assert_eq!(series.len(), 1);
        series.set_mode(StreamMode::Live);
        let _ = series.push([2.0, 2.0]);
        assert_eq!(series.len(), 2);
    }

    #[test]
    fn a_trigger_fires_once_on_the_edge() {
        let mut series = StreamingSeries::new(16).with_trigger(Trigger::rising(1.0));
        assert!(!series.push([0.0, 0.5]));
        assert!(series.push([1.0, 1.1]));
        assert!(!series.push([2.0, 1.2]));
        assert!(!series.push([3.0, 0.9]));
        assert!(series.push([4.0, 1.3]));
        assert_eq!(series.fired(), 2);
    }
}
