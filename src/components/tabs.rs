//! The document tab strip component.

use egui::Ui;

use crate::Icons;
use crate::chrome::tabs::show_tab_strip;

pub use crate::chrome::tabs::{TabEvent, TabIcon, TabItem};

/// A document tab strip with a controls area.
///
/// This is a composite component: the caller supplies the tab models and the control
/// closure, and receives the selection/close event. It paints its whole `Ui::max_rect`,
/// so place it in a fixed-height panel or sized container.
///
pub struct TabStrip<'a> {
    tabs: &'a [TabItem<'a>],
    active: usize,
    close_tooltip: &'a str,
    controls_width: f32,
}

impl<'a> TabStrip<'a> {
    /// A strip over `tabs` with one active index.
    pub fn new(tabs: &'a [TabItem<'a>], active: usize) -> Self {
        Self {
            tabs,
            active,
            close_tooltip: "Close",
            controls_width: 276.0,
        }
    }

    /// Tooltip of the close buttons.
    pub fn close_tooltip(mut self, tooltip: &'a str) -> Self {
        self.close_tooltip = tooltip;
        self
    }

    /// Width reserved on the right for controls.
    pub fn controls_width(mut self, width: f32) -> Self {
        self.controls_width = width;
        self
    }

    /// Paint the strip and return the tab event.
    pub fn show(self, ui: &mut Ui, icons: &mut Icons, controls: impl FnOnce(&mut Ui, &mut Icons)) -> Option<TabEvent> {
        show_tab_strip(
            ui,
            icons,
            self.tabs,
            self.active,
            self.close_tooltip,
            self.controls_width,
            controls,
        )
    }
}
