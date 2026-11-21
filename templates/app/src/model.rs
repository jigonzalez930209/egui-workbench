//! The application model.
//!
//! `Tab` is the shell metadata every workbench needs. `TabData` is the only
//! domain-specific part: replace its fields with the state one tab owns (a document,
//! a plot, a session, a form, …). The shell, the close guard, and the tab strip only
//! read `title` and `dirty`.
//!
//! The starter keeps the gallery's demo values in `TabData::demo` so the test app can
//! show every component. Delete `DemoState` and `Gallery` when replacing the screen.

use egui_workbench::app::Document;

/// One section of the component gallery.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GallerySection {
    /// Searchable Lucide icon browser.
    Icons,
    /// Buttons and icon buttons.
    #[default]
    Buttons,
    /// Text fields, text areas, switches, checkboxes, selects, steppers, segmented.
    Inputs,
    /// Toasts, badges, banners, labels, separators, cards.
    Feedback,
    /// Menus and menu rows.
    Menus,
    /// Tree, tab strip, and split.
    Data,
    /// Interactive plots: zoom, pan, selection, fits, and live data.
    Charts,
    /// Palette and dialog commands.
    Actions,
}

impl GallerySection {
    /// Every section, in navigator order.
    pub const ALL: [Self; 8] = [
        Self::Icons,
        Self::Buttons,
        Self::Inputs,
        Self::Feedback,
        Self::Menus,
        Self::Data,
        Self::Charts,
        Self::Actions,
    ];

    /// The tab and navigator label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Icons => "Icons",
            Self::Buttons => "Buttons",
            Self::Inputs => "Inputs",
            Self::Feedback => "Feedback",
            Self::Menus => "Menus",
            Self::Data => "Data",
            Self::Charts => "Charts",
            Self::Actions => "Actions",
        }
    }

    /// A one-line description for the section header.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Icons => "2,410 Lucide icons; the grid fills the width and follows the window size.",
            Self::Buttons => "Variants, sizes, disabled and toggle states.",
            Self::Inputs => "State is passed by &mut and returned in the same frame.",
            Self::Feedback => "Toasts, badges, labels, banners, cards, and separators.",
            Self::Menus => "Text menus, icon menus, rows, and toggles.",
            Self::Data => "The explorer tree, a second tab strip, and a split.",
            Self::Charts => "Zoom, pan, box zoom, area selection, hover popups, fits, and live data.",
            Self::Actions => "Commands reuse the same palette and dialogs as the shell.",
        }
    }
}

/// One open editor tab.
pub struct Tab {
    /// Stable identity, useful for scroll state and async results.
    pub id: u64,
    /// Tab label.
    pub title: String,
    /// Whether closing this tab needs confirmation.
    pub dirty: bool,
    /// The gallery section this tab shows.
    pub section: GallerySection,
    /// Replace this with your domain state.
    pub data: TabData,
}

impl Tab {
    /// A clean tab showing the Buttons section.
    pub fn new(id: u64) -> Self {
        Self::example(id, GallerySection::Buttons)
    }

    /// A clean tab for one gallery section.
    pub fn example(id: u64, section: GallerySection) -> Self {
        Self {
            id,
            title: section.label().to_owned(),
            dirty: false,
            section,
            data: TabData {
                demo: DemoState {
                    switch: true,
                    check: true,
                    number: 1.0,
                    split: 0.5,
                    ..DemoState::default()
                },
            },
        }
    }
}

impl Document for Tab {
    fn title(&self) -> &str {
        &self.title
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Per-tab application state.
///
/// Keep this `Default` so `Tab::new` stays simple. Replace the placeholder field with
/// the data your product actually keeps in a tab.
#[derive(Default)]
pub struct TabData {
    /// State of the starter gallery. Replace it with the product state.
    pub demo: DemoState,
}

/// Values used by the starter gallery controls.
///
/// This is demo-only state, kept in the tab so every tab remembers its own values.
#[derive(Default)]
pub struct DemoState {
    /// Text field sample.
    pub text: String,
    /// Text area sample.
    pub area: String,
    /// Switch sample.
    pub switch: bool,
    /// Checkbox sample.
    pub check: bool,
    /// Select option index.
    pub select: usize,
    /// Stepper value.
    pub number: f32,
    /// Mini tab strip selection.
    pub mini_tab: usize,
    /// Split pane ratio.
    pub split: f32,
    /// Segmented control selection.
    pub mode: usize,
    /// Last menu/action message.
    pub message: String,
    /// Whether the demo popover is open.
    pub popover: bool,
    /// Lucide search query.
    pub icon_query: String,
    /// Index into the icon size options (Auto, 16, 24, 32, 48).
    pub icon_size: usize,
    /// The last icon clicked in the browser.
    pub icon_selected: Option<String>,
    /// Plots, their view state, and the live stream.
    pub charts: crate::charts::ChartsDemo,
    /// The loading toast the "Finish export" button resolves.
    pub toast_loading: Option<egui_workbench::components::ToastId>,
    /// 0 = grid, 1 = list.
    pub icon_layout: usize,
}
