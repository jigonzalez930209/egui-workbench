//! A shadcn-style component layer over the workbench primitives.
//!
//! The crate root keeps the original function API. This module adds builders with
//! shared [`Size`], [`ButtonVariant`], [`Tone`], and state handling, so an application
//! can read like a component tree:
//!
//! ```
//! use egui_workbench::components::{Button, TextField, prelude::*};
//! use egui_workbench::egui;
//!
//! # let ctx = egui::Context::default();
//! # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
//! let mut name = String::new();
//! TextField::new(&mut name).label("Name").width(240.0).show(ui);
//! if Button::new("Save").variant(ButtonVariant::Primary).show(ui).clicked() {
//!     // Reactivity stays immediate: the state above is the source of truth.
//! }
//! # });
//! # output.textures_delta.clear();
//! ```
//!
//! Components never own application state. Interactive components take `&mut` state and
//! return the change in the same frame (`bool`, `Option<T>`, or `egui::Response`); the
//! caller owns the model. This is the shadcn composition model adapted to egui's
//! immediate mode: variants and tokens are shared, but state is passed in, not hidden
//! in a component tree.
//!
//! Composite blocks (`TabStrip`, `CommandPalette`, `Tree`, `Settings`, `Split`,
//! `Banner`, dialogs) wrap the existing shell functions and are reachable through
//! [`blocks`] or the crate root.
//!
//! See `docs/COMPONENTS.md` for the full table and authoring rules, and run
//! `cargo run --example catalog` for the live catalog.

use egui::Ui;

use crate::Icons;

mod badge;
pub mod blocks;
mod button;
mod card;
mod checkbox;
mod dialog;
mod feedback;
mod icon_grid;
mod icon_preview;
mod input;
mod label;
mod menu;
mod nav;
mod palette;
#[cfg(feature = "plot")]
pub mod plot;
mod segmented;
mod select;
mod separator;
mod status;
mod stepper;
mod style;
mod switch;
mod tabs;
mod toast;
mod tree;
#[cfg(feature = "viewer")]
pub mod viewer;

#[cfg(feature = "viewer")]
pub use viewer::{Cell, CellKind, Document, FileFormat, FileViewer, Sheet, Table, TextFile, ViewerError, Workbook};

pub use badge::Badge;
pub use button::{Button, CloseButton, IconButton};
pub use card::Card;
pub use checkbox::Checkbox;
pub use dialog::{ConfirmDialog, Dialog, NameDialog, ProgressDialog, PromptAction};
pub use feedback::{Banner, Popover};
pub use icon_grid::{IconGrid, IconLayout, IconSize};
pub use icon_preview::IconPreview;
pub use input::{TextArea, TextField};
pub use label::Label;
pub use menu::{Menu, MenuButton, MenuIconButton, MenuItem, MenuToggle};
pub use nav::NavTab;
pub use palette::{CommandPalette, PaletteFeed, PaletteItem, PaletteLabels, PaletteMode, PaletteState};
pub use segmented::SegmentedControl;
pub use select::Select;
pub use separator::{Orientation, Separator};
pub use status::StatusBar;
pub use stepper::Stepper;
pub use style::{ButtonVariant, IconButtonSize, Size, Tone};
pub use switch::Switch;
pub use tabs::{TabEvent, TabIcon, TabItem, TabStrip};
pub use toast::{Toast, ToastEvent, ToastId, ToastKind, ToastPosition, Toasts};
pub use tree::{RowIcon, Tree, TreeEvent, TreeNode, TreeState};

/// A component that takes only a [`Ui`] and returns a value.
///
/// Implement it for builders that can be rendered with `Component::show`. Components
/// that need the icon cache implement [`IconComponent`] instead. Dialogs take a
/// [`egui::Context`] and are called directly; they are composition, not a `Ui` child.
pub trait Component {
    /// What the component returns after painting.
    type Output;

    /// Paint the component.
    fn show(self, ui: &mut Ui) -> Self::Output;
}

/// A component that paints glyphs from the shared [`Icons`] cache.
pub trait IconComponent {
    /// What the component returns after painting.
    type Output;

    /// Paint the component with the application's icon cache.
    fn show(self, ui: &mut Ui, icons: &mut Icons) -> Self::Output;
}

/// The names most applications import.
pub mod prelude {
    pub use super::{
        Badge, Banner, Button, ButtonVariant, Card, Checkbox, CloseButton, CommandPalette, Component, ConfirmDialog,
        Dialog, IconButton, IconButtonSize, IconComponent, IconGrid, IconLayout, IconPreview, IconSize, Label, Menu,
        MenuButton, MenuIconButton, MenuItem, MenuToggle, NameDialog, NavTab, Orientation, PaletteItem, PaletteLabels,
        PaletteState, Popover, ProgressDialog, PromptAction, RowIcon, SegmentedControl, Select, Separator, Size,
        StatusBar, Stepper, Switch, TabEvent, TabIcon, TabItem, TabStrip, TextArea, TextField, Toast, ToastEvent,
        ToastKind, ToastPosition, Toasts, Tone, Tree, TreeEvent, TreeNode, TreeState,
    };
    #[cfg(feature = "viewer")]
    pub use super::{Cell, CellKind, Document, FileFormat, FileViewer, Sheet, Table, TextFile, ViewerError, Workbook};
}
