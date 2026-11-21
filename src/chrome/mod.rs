//! Window chrome: the undecorated frame, title bar, tab strip, status bar, and split.
//!
//! These components draw the fixed shell around an application's content. They return
//! events and never own application state.

pub(crate) mod split;
pub(crate) mod status;
pub(crate) mod tabs;
pub(crate) mod title_bar;
pub(crate) mod window;
