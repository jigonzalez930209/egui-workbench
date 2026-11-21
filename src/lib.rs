//! The MD View desktop workbench as reusable egui components.
//!
//! The crate holds the shell, not the content: nine themes, the size tokens, the
//! undecorated window with a custom title bar and menus, the document tab strip, the
//! explorer tree, modal prompts, the settings scaffold, the command palette, the
//! keyboard menu, the status bar, icon buttons, form controls, and the icon cache.
//!
//! Components never translate text or own application state: they take translated
//! strings and return events. The `components` builders additionally share variants and
//! sizes; the original functions remain for existing code. See `WORKBENCH.md` for the
//! layout rules, `docs/COMPONENTS.md` for the component guide, and
//! `examples/showcase` for a complete application skeleton.
//!
//! # Layout of the source
//!
//! - [`metrics`] and [`theme`]: sizes, theme tokens, and the icon caches (the
//!   `foundation` layer).
//! - `chrome`: the window frame, title bar, tab strip, status bar, and split view.
//! - `widgets`: the function API for buttons, controls, dialogs, overlays, and the tree.
//! - [`components`]: the shadcn-style builder layer with shared variants, tones, and
//!   composite blocks. Start here for new screens.
//! - [`app`]: optional runtime helpers ([`ThemeState`](app::ThemeState),
//!   [`Documents`](app::Documents), [`Keymap`](app::Keymap), [`CloseGuard`](app::CloseGuard),
//!   and [`show_workbench`](app::show_workbench)).

pub mod app;
pub mod components;

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
pub use widgets::palette::{PaletteItem, PaletteLabels, PaletteState, filter as filter_palette, show_palette};
pub use widgets::settings::{SettingsDialog, settings_row, settings_section, settings_switch, show_settings_dialog};
pub use widgets::tree::{RowIcon, TreeEvent, TreeNode, TreeState, TreeView, section_header, show_tree};

pub use eframe;
pub use egui;
pub use theme::{Theme, ThemeId, ThemeMode};
