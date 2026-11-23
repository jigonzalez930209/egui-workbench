//! Frame layout for a 3D plot: title room above, label room around the box.

use egui::{Rect, pos2};

use super::super::scene::Measure;
use super::config::Plot3DConfig;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Layout3D {
    pub rect: Rect,
    pub frame: Rect,
}

/// The 3D box is drawn inside `frame`; the margins leave room for titles, tick labels,
/// and the legend toolbar.
pub(super) fn layout(measure: &Measure, config: &Plot3DConfig, rect: Rect) -> Layout3D {
    let mut top = 8.0;
    if let Some(title) = &config.title {
        top += measure.text(title, config.title_size).1 + 6.0;
    }
    let side = 26.0;
    let bottom = 48.0;
    let frame = Rect::from_min_max(
        pos2(measure.snap(rect.left() + side), measure.snap(rect.top() + top)),
        pos2(measure.snap(rect.right() - side), measure.snap(rect.bottom() - bottom)),
    );
    Layout3D { rect, frame }
}
