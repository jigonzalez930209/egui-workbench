//! The command palette component.

use egui::Context;

use crate::widgets::palette::show_palette;

pub use crate::widgets::palette::{PaletteItem, PaletteLabels, PaletteState};

/// A command palette/Quick Open overlay.
pub struct CommandPalette<'a, T> {
    state: &'a mut PaletteState,
    labels: PaletteLabels<'a>,
    items: &'a [PaletteItem<T>],
}

impl<'a, T> CommandPalette<'a, T> {
    /// A palette over `items`, controlled by `state`.
    pub fn new(state: &'a mut PaletteState, labels: PaletteLabels<'a>, items: &'a [PaletteItem<T>]) -> Self {
        Self { state, labels, items }
    }

    /// Show the palette and return the chosen value.
    pub fn show(self, ctx: &Context) -> Option<T>
    where
        T: Clone,
    {
        show_palette(ctx, self.state, self.labels, self.items)
    }
}
