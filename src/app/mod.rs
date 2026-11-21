//! Optional application runtime built on the components.
//!
//! The components in the crate root stay presentation-only. This module adds the
//! pieces every workbench application repeats:
//!
//! - [`ThemeState`]: requested/effective theme, follow-system, reduced motion.
//! - [`Documents`] and [`Document`]: tab order, active document, dirty query.
//! - [`Keymap`]: one ordered shortcut table that consumes handled keys.
//! - [`CloseGuard`]: cancels close requests while documents are dirty.
//! - [`WorkbenchApp`] and [`show_workbench`]: the fixed panel order.
//!
//! All of it is optional. An application can keep its own state and call individual
//! components or panels directly.
//!
//! ```no_run
//! use egui_workbench::app::{ThemeState, show_workbench, WorkbenchApp};
//! use egui_workbench::{eframe, egui};
//!
//! struct App {
//!     theme: ThemeState,
//!     docs: egui_workbench::app::Documents<Doc>,
//! }
//! # struct Doc;
//!
//! impl eframe::App for App {
//!     fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//!         self.theme.sync_system(ctx);
//!     }
//!
//!     fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
//!         show_workbench(ui, self);
//!     }
//! }
//! # impl WorkbenchApp for App {
//! #     type Command = ();
//! # }
//! ```

mod close_guard;
mod documents;
mod keymap;
mod shell;
mod theme_state;

pub use close_guard::CloseGuard;
pub use documents::{Document, Documents};
pub use keymap::Keymap;
pub use shell::{
    BOTTOM_PANEL, BOTTOM_PANEL_MIN_HEIGHT, CONTENT_MIN_WIDTH, RIGHT_PANEL, RIGHT_PANEL_MIN_WIDTH, SIDEBAR_MIN_WIDTH,
    SIDEBAR_PANEL, STATUS_BAR_PANEL, TAB_STRIP_PANEL, TITLE_BAR_PANEL, WorkbenchApp, show_workbench,
};
pub use theme_state::ThemeState;
