//! A desktop workbench shell for [egui](https://github.com/emilk/egui).
//!
//! The crate holds the shell, not the content: nine themes, the size tokens, the
//! undecorated window with a custom title bar and menus, the document tab strip, the
//! explorer tree, modal prompts, the settings scaffold, the command palette, the
//! keyboard menu, the status bar, toasts, interactive plots with new chart families and
//! legend animations, a native file viewer, icon buttons, form controls, and the icon cache.
//!
//! Components never translate text or own application state: they take translated
//! strings and return events. The [`components`] builders share variants and sizes; the
//! original functions remain for existing code.
//!
//! - Guide and live demos: <https://jigonzalez930209.github.io/egui-workbench/>
//! - Layout specification, component guide, examples, and the application template:
//!   <https://github.com/jigonzalez930209/egui-workbench>
//!
//! ```no_run
//! use egui_workbench::{ThemeId, eframe, theme};
//!
//! fn main() -> eframe::Result {
//!     eframe::run_native(
//!         "My App",
//!         egui_workbench::native_options("My App"),
//!         Box::new(|cc| {
//!             theme::apply_theme(&cc.egui_ctx, ThemeId::GithubLight, false);
//!             Ok(Box::new(MyApp))
//!         }),
//!     )
//! }
//!
//! struct MyApp;
//!
//! impl eframe::App for MyApp {
//!     fn ui(&mut self, ui: &mut egui_workbench::egui::Ui, _frame: &mut eframe::Frame) {
//!         ui.label("Hello, workbench");
//!     }
//! }
//! ```
//!
//! # Layout of the source
//!
//! - [`metrics`] and [`theme`]: sizes, theme tokens, and the icon caches (the
//!   `foundation` layer).
//! - `chrome`: the window frame, title bar, tab strip, status bar, and split view.
//! - `widgets`: the function API for buttons, controls, dialogs, overlays, and the tree.
//! - [`components`]: the shadcn-style builder layer with shared variants, tones,
//!   composite blocks, charts, and the data viewer. Start here for new screens.
//! - [`app`]: optional runtime helpers ([`ThemeState`](app::ThemeState),
//!   [`Documents`](app::Documents), [`Keymap`](app::Keymap), [`CloseGuard`](app::CloseGuard),
//!   and [`show_workbench`](app::show_workbench)).

// docs.rs: label items that need an optional feature.
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod app;
pub mod commands;
pub mod components;

pub use commands::{
    ArgKind, ArgSchema, BindingError, Command, CommandId, CommandRegistry, CommandState, DIALOG_OPEN, DIRTY,
    EDITOR_FOCUS, HAS_SELECTION, KeyChord, KeySequence, Keybindings, KeybindingsEditorState, PALETTE_OPEN, READ_ONLY,
    When, WhenContext,
};

mod chrome;
mod foundation;
mod widgets;

pub use foundation::icons::{FULL_UV, Icons};
#[cfg(feature = "material-icons")]
pub use foundation::material::MaterialIcons;
pub use foundation::{metrics, style, theme};

pub use chrome::split::{STACK_BELOW, split_panes};
pub use chrome::status::{
    StatusItem, editor_scroll_style, ellipsize_start, show_empty_state, show_status_bar, with_editor_scrollbars,
};
pub use chrome::tabs::{TabEvent, TabIcon, TabItem, show_tab_strip};
pub use chrome::title_bar::{Pill, TitleBar, TitleBarOutput, WindowLabels, show_title_bar, title_menu};
pub use chrome::window::{WindowAction, apply_window_action, handle_window_edges, is_maximized, set_window_title};
#[cfg(not(target_arch = "wasm32"))]
pub use chrome::window::{native_options, native_options_sized};
pub use widgets::buttons::{
    IconState, Segment, action_button, menu_tool_button, segmented, small_button, tool_button, tool_button_response,
};
pub use widgets::controls::{
    close_glyph, combo, compact_menu, menu_button, menu_item, menu_toggle, numeric_stepper, primary_button,
    secondary_button, switch, text_field,
};
pub use widgets::dialogs::{
    ConfirmPrompt, NamePrompt, PromptAction, banner, dialog_footer, dialog_frame, dialog_header, popover_frame,
    show_choice_window, show_confirm_prompt, show_name_prompt, show_popover, show_progress,
};
pub use widgets::keyboard_menu::{KeyboardMenuState, MenuEntry, MenuGroup, open_from_keyboard, show_keyboard_menu};
pub use widgets::palette::{
    PaletteFeed, PaletteItem, PaletteLabels, PaletteMode, PaletteState, filter as filter_palette, fuzzy_filter,
    show_palette,
};
pub use widgets::settings::{SettingsDialog, settings_row, settings_section, settings_switch, show_settings_dialog};
pub use widgets::tree::{RowIcon, TreeEvent, TreeNode, TreeState, TreeView, section_header, show_tree};

pub use eframe;
pub use egui;
pub use theme::{Theme, ThemeId, ThemeMode};
