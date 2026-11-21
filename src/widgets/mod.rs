//! Reusable widgets: buttons, form controls, dialogs, overlays, and the explorer tree.
//!
//! Every widget reads colors and sizes from `foundation` and reports interaction
//! through its return value or an event type.

pub(crate) mod buttons;
pub(crate) mod controls;
pub(crate) mod dialogs;
pub(crate) mod keyboard_menu;
pub(crate) mod palette;
pub(crate) mod settings;
pub(crate) mod tree;
