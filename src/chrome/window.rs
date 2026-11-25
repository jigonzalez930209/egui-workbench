//! The undecorated native window: options, edge resizing, and window commands.
//!
//! The workbench draws its own title bar, so the OS decorations are off. The window is
//! moved by dragging the title bar's center, resized from 4 pt bands on the west, east,
//! and south edges (the north edge belongs to the title bar), and controlled through
//! [`WindowAction`].

use egui::{Context, ResizeDirection, ViewportCommand};

use crate::metrics::RESIZE_EDGE;

/// A window control request from the title bar.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum WindowAction {
    /// Minimize the window.
    Minimize,
    /// Maximize or restore the window.
    ToggleMaximize,
    /// Close through the application's normal close path.
    Close,
}

/// Native options for a workbench window: no OS decorations, the glow renderer,
/// 980×720 by default, at least 640×420.
///
/// Not available on wasm; use `eframe::WebRunner` with `eframe::WebOptions` there.
#[cfg(not(target_arch = "wasm32"))]
pub fn native_options(title: &str) -> eframe::NativeOptions {
    native_options_sized(title, [980.0, 720.0], [640.0, 420.0])
}

/// Native options with an explicit initial and minimum inner size.
///
/// Not available on wasm; use `eframe::WebRunner` with `eframe::WebOptions` there.
#[cfg(not(target_arch = "wasm32"))]
pub fn native_options_sized(title: &str, size: [f32; 2], min_size: [f32; 2]) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_min_inner_size(min_size)
            .with_decorations(false)
            .with_title(title),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    }
}

/// Force the OS decorations off once per viewport on Windows.
///
/// Windows can keep a native frame after the initial `with_decorations(false)` when the
/// window is created hidden or restored from a previous session. Re-sending the viewport
/// command removes that strip, which is the source of the vertical pointer offset seen on
/// some Windows builds.
pub fn ensure_undecorated(ctx: &Context) {
    #[cfg(target_os = "windows")]
    {
        let id = egui::Id::unique("workbench_force_undecorated");
        let sent = ctx.data(|data| data.get_temp::<bool>(id).unwrap_or(false));
        if !sent {
            ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
            ctx.data_mut(|data| data.insert_temp(id, true));
        }
    }
    #[cfg(not(target_os = "windows"))]
    let _ = ctx;
}

/// Whether the window is maximized.
pub fn is_maximized(ctx: &Context) -> bool {
    ctx.input(|input| input.viewport().maximized.unwrap_or(false))
}

/// Run a window action. `Close` sends the viewport close command, which arrives back at
/// the application as a close request it can still cancel (for unsaved changes).
pub fn apply_window_action(ctx: &Context, action: WindowAction) {
    match action {
        WindowAction::Minimize => ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
        WindowAction::ToggleMaximize => ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_maximized(ctx))),
        WindowAction::Close => ctx.send_viewport_cmd(ViewportCommand::Close),
    }
}

/// Start an OS resize when the primary button goes down on a window edge. Call once per
/// frame, before the panels.
pub fn handle_window_edges(ctx: &Context) {
    if !ctx.input(|input| input.pointer.primary_pressed()) {
        return;
    }
    let Some(position) = ctx.pointer_latest_pos() else {
        return;
    };
    let rect = ctx.content_rect();
    let west = position.x <= rect.left() + RESIZE_EDGE;
    let east = position.x >= rect.right() - RESIZE_EDGE;
    let south = position.y >= rect.bottom() - RESIZE_EDGE;
    let direction = match (west, east, south) {
        (true, _, true) => Some(ResizeDirection::SouthWest),
        (_, true, true) => Some(ResizeDirection::SouthEast),
        (true, _, _) => Some(ResizeDirection::West),
        (_, true, _) => Some(ResizeDirection::East),
        (_, _, true) => Some(ResizeDirection::South),
        _ => None,
    };
    if let Some(direction) = direction {
        ctx.send_viewport_cmd(ViewportCommand::BeginResize(direction));
    }
}

/// Keep the OS window title in sync ("name • — App" style is up to the caller).
pub fn set_window_title(ctx: &Context, title: &str) {
    ctx.send_viewport_cmd(ViewportCommand::Title(title.to_owned()));
}
