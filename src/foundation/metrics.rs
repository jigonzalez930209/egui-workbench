//! Size tokens for the workbench chrome, in points.
//!
//! Keeping every chrome size in one place stops the title bar, tabs, explorer, and
//! dialogs from drifting apart.

/// Glyphs inside dense menus, steppers, and window buttons.
pub const ICON_SMALL: f32 = 14.0;
/// Glyphs of title-bar actions, tabs, and tree rows.
pub const ICON_MEDIUM: f32 = 16.0;
/// The product mark.
pub const ICON_LARGE: f32 = 20.0;

/// Height of the title bar.
pub const TITLE_HEIGHT: f32 = 34.0;
/// Height of the document tab strip.
pub const TAB_HEIGHT: f32 = 34.0;
/// Left inset of the first tab in the strip.
pub const TAB_STRIP_PADDING: f32 = 8.0;
/// Height of the status bar.
pub const STATUS_HEIGHT: f32 = 24.0;
/// Height of an explorer tree row.
pub const TREE_ROW_HEIGHT: f32 = 21.0;
/// Indentation per tree depth.
pub const TREE_INDENT: f32 = 12.0;
/// Width of the expand/collapse slot in front of every tree icon.
pub const CHEVRON_SIZE: f32 = 14.0;
/// Default explorer width.
pub const SIDEBAR_WIDTH: f32 = 260.0;

/// Square title-bar action button.
pub const ACTION_BUTTON: f32 = 28.0;
/// Square tab-strip tool button.
pub const TOOL_BUTTON: f32 = 26.0;
/// Square explorer header button.
pub const SMALL_BUTTON: f32 = 21.0;
/// Width of a window control cell (minimize, maximize, close).
pub const WINDOW_BUTTON_WIDTH: f32 = 44.0;
/// Height of a window control cell.
pub const WINDOW_BUTTON_HEIGHT: f32 = 32.0;

/// Height of text fields, dialog buttons, combo boxes, and steppers.
pub const CONTROL_HEIGHT: f32 = 30.0;
/// Height of a context-menu row.
pub const MENU_ROW_HEIGHT: f32 = 22.0;
/// Height of a palette or keyboard-menu row.
pub const LIST_ROW_HEIGHT: f32 = 26.0;
/// Pixels from the window edge that start a resize on an undecorated window.
pub const RESIZE_EDGE: f32 = 4.0;
