//! Composite blocks.
//!
//! These are larger, stateful pieces that already exist in the shell. They are
//! re-exported here so an application can reach every visual block from
//! [`super`] without learning the internal module layout.

pub use crate::{
    KeyboardMenuState, MenuEntry, MenuGroup, STACK_BELOW, SettingsDialog, TitleBar, TitleBarOutput, WindowLabels,
    banner, close_glyph, compact_menu, dialog_footer, dialog_frame, dialog_header, editor_scroll_style,
    menu_tool_button, open_from_keyboard, section_header, settings_row, settings_section, settings_switch,
    show_choice_window, show_empty_state, show_keyboard_menu, show_popover, show_progress, show_settings_dialog,
    show_title_bar, split_panes, title_menu, with_editor_scrollbars,
};
