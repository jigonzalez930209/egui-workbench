//! An icon segmented control.

use egui::Ui;

use crate::Icons;
use crate::components::IconComponent;
use crate::{Segment, segmented};

/// The View/Edit/Split style icon segmented control.
pub struct SegmentedControl<'a> {
    id_salt: &'a str,
    segments: &'a [Segment<'a>],
    selected: usize,
    enabled: bool,
}

impl<'a> SegmentedControl<'a> {
    /// A control over `segments`, with one selected index.
    pub fn new(id_salt: &'a str, segments: &'a [Segment<'a>], selected: usize) -> Self {
        Self {
            id_salt,
            segments,
            selected,
            enabled: true,
        }
    }

    /// Disable the whole control.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl IconComponent for SegmentedControl<'_> {
    type Output = Option<usize>;

    fn show(self, ui: &mut Ui, icons: &mut Icons) -> Option<usize> {
        ui.add_enabled_ui(self.enabled, |ui| {
            segmented(ui, icons, self.id_salt, self.segments, self.selected)
        })
        .inner
    }
}
