//! The filter and transform pipeline owned by a chart.

use serde::{Deserialize, Serialize};

use super::filter::{FilterKind, FilterSettings, apply_filters};
use super::series::Series;
use super::transforms::{TransformKind, TransformSettings, apply_transform};

/// The ordered filter chain and the optional transform.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pipeline {
    /// Filters applied in this order.
    pub filters: Vec<FilterKind>,
    /// The transform applied after the filters.
    pub transform: Option<TransformKind>,
    /// Filter parameters.
    pub filter_settings: FilterSettings,
    /// Transform parameters.
    pub transform_settings: TransformSettings,
    #[serde(skip)]
    revision: u64,
}

impl Pipeline {
    /// Whether any filter or transform is active.
    pub fn is_active(&self) -> bool {
        !self.filters.is_empty() || self.transform.is_some()
    }

    /// Whether `filter` is in the chain.
    pub fn is_filter_enabled(&self, filter: FilterKind) -> bool {
        self.filters.contains(&filter)
    }

    /// Add or remove `filter`. Returns whether it is now enabled.
    pub fn toggle_filter(&mut self, filter: FilterKind) -> bool {
        if let Some(position) = self.filters.iter().position(|candidate| *candidate == filter) {
            self.filters.remove(position);
            self.touch();
            false
        } else {
            self.filters.push(filter);
            self.touch();
            true
        }
    }

    /// Enable or disable `filter` explicitly.
    pub fn set_filter(&mut self, filter: FilterKind, enabled: bool) {
        if enabled && !self.is_filter_enabled(filter) {
            self.filters.push(filter);
            self.touch();
        } else if !enabled {
            self.filters.retain(|candidate| *candidate != filter);
            self.touch();
        }
    }

    /// Replace the transform; `None` disables it.
    pub fn set_transform(&mut self, transform: Option<TransformKind>) {
        if self.transform != transform {
            self.transform = transform;
            self.touch();
        }
    }

    /// Remove every filter and the transform.
    pub fn clear(&mut self) {
        if self.is_active() {
            self.filters.clear();
            self.transform = None;
            self.touch();
        }
    }

    /// A counter that changes whenever the pipeline changes.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Apply the pipeline to `series`.
    ///
    /// Filters preserve every series. A transform replaces the data with its result
    /// (cross-correlation uses the first two visible series).
    pub fn process(&self, series: &[Series<'_>]) -> Vec<Series<'static>> {
        if !self.is_active() {
            return Vec::new();
        }
        let filtered = series
            .iter()
            .map(|series| {
                let data = apply_filters(series.data(), &self.filters, &self.filter_settings);
                (series, data)
            })
            .collect::<Vec<_>>();
        if let Some(transform) = self.transform {
            let inputs = filtered.iter().map(|(_, data)| data.as_slice()).collect::<Vec<_>>();
            let output = apply_transform(transform, &inputs, &self.transform_settings);
            let Some((first, _)) = filtered.first().filter(|_| !output.is_empty()) else {
                return Vec::new();
            };
            let mut processed = first.with_data(output, true);
            processed.name = format!("{} · {}", first.name(), transform.label());
            return vec![processed];
        }
        filtered
            .into_iter()
            .map(|(series, data)| series.with_data(data, false))
            .collect()
    }

    fn touch(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::plot::Series;

    fn series() -> Vec<Series<'static>> {
        vec![
            Series::line("a", vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0], [3.0, 1.0], [4.0, 0.0]]),
            Series::line("b", vec![[0.0, 1.0], [1.0, 0.0], [2.0, 1.0], [3.0, 0.0], [4.0, 1.0]]),
        ]
    }

    #[test]
    fn toggling_filters_updates_the_chain_and_revision() {
        let mut pipeline = Pipeline::default();
        let revision = pipeline.revision();
        assert!(pipeline.toggle_filter(FilterKind::Gaussian));
        assert!(pipeline.is_filter_enabled(FilterKind::Gaussian));
        assert!(pipeline.revision() > revision);
        assert!(!pipeline.toggle_filter(FilterKind::Gaussian));
        assert!(!pipeline.is_active());
    }

    #[test]
    fn filters_preserve_every_series() {
        let mut pipeline = Pipeline::default();
        pipeline.set_filter(FilterKind::MovingAverage, true);
        let processed = pipeline.process(&series());
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].name(), "a");
        assert_eq!(processed[1].data().len(), 5);
    }

    #[test]
    fn a_transform_replaces_the_data_and_needs_two_series_for_cross_correlation() {
        let mut pipeline = Pipeline::default();
        pipeline.set_transform(Some(TransformKind::Fft));
        let processed = pipeline.process(&series());
        assert_eq!(processed.len(), 1);
        assert!(processed[0].name().contains("FFT"));

        pipeline.set_transform(Some(TransformKind::CrossCorrelation));
        assert_eq!(pipeline.process(&series()).len(), 1);
        assert!(pipeline.process(&series()[..1]).is_empty());
    }

    #[test]
    fn clearing_the_pipeline_disables_everything() {
        let mut pipeline = Pipeline::default();
        pipeline.set_filter(FilterKind::Median, true);
        pipeline.set_transform(Some(TransformKind::Autocorrelation));
        pipeline.clear();
        assert!(!pipeline.is_active());
        assert!(pipeline.filters.is_empty());
        assert_eq!(pipeline.transform, None);
    }
}
