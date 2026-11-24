//! Named commands, contexts, rebindable keys, and the palette built on them.
//!
//! A workbench application describes each action once as a [`Command`], registers it in
//! a [`CommandRegistry`], and lets menus, the palette, the title bar, context menus, and
//! keyboard shortcuts all read that registry. Nothing has to be wired three times.
//!
//! ```
//! use egui_workbench::commands::{Command, CommandId, CommandRegistry, WhenContext};
//!
//! let mut registry = CommandRegistry::new();
//! registry.register(Command::new("file.save", "Save").shortcut("Ctrl+S"));
//! registry.register(
//!     Command::new("edit.paste", "Paste")
//!         .shortcut("Ctrl+V")
//!         .when("editorFocus && !dialogOpen"),
//! );
//!
//! let context = WhenContext::new().with("editorFocus");
//! let items = registry.palette_items(&context);
//! assert_eq!(items.len(), 2);
//! let save = CommandId::new("file.save");
//! assert_eq!(registry.binding(&save).map(ToString::to_string), Some("Ctrl+S".to_owned()));
//! ```
//!
//! The registry never runs application code. It returns a [`CommandId`] and the
//! application maps it to its own handler, which keeps command metadata serializable and
//! testable without a UI.

mod chord;
mod command;
mod editor;
mod keybindings;
mod menu;
mod palette;
mod registry;
mod when;

pub use chord::{KeyChord, KeySequence};
pub use command::{ArgKind, ArgSchema, BindingError, Command, CommandId, CommandState};
pub use editor::{KeybindingChange, KeybindingsEditorState, KeybindingsLabels, show_keybindings_editor};
pub use keybindings::{KeybindingEntry, Keybindings, KeybindingsError};
pub use menu::{command_menu_groups, show_command_menu};
pub use palette::{CommandPaletteLabels, command_palette_items, show_command_palette};
pub use registry::{BindingConflict, CommandRegistry};
pub use when::{
    DIALOG_OPEN, DIRTY, EDITOR_FOCUS, HAS_SELECTION, PALETTE_OPEN, READ_ONLY, When, WhenContext, WhenError,
};
