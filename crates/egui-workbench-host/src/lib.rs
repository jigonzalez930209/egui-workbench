//! A declarative host for egui-workbench.
//!
//! An application is a versioned [`HostSpec`] (window, theme, commands, menus, explorer,
//! layout, panels, status, and a widget tree). A host applies [`Patch`] values, renders
//! the tree with the egui-workbench components, and emits [`HostEvent`]s. The same
//! messages travel in-process, over stdio/sockets with a length-prefixed frame, or
//! through the `egui-workbench` sidecar.
//!
//! ```no_run
//! use egui_workbench_host::{Host, HostSpec};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let spec = HostSpec::default();
//! let mut host = Host::new(spec)?;
//! host.run()?;
//! # Ok(())
//! # }
//! ```

pub mod custom;
pub mod data;
pub mod patch;
pub mod protocol;
pub mod runtime;
pub mod spec;
mod view;

pub use custom::{CustomWidget, CustomWidgetRegistry};
pub use data::DataBuffer;
pub use patch::{Patch, PatchError};
pub use protocol::{FrameError, Message, PROTOCOL_VERSION, ProtocolError, Request, Response, read_frame, write_frame};
pub use runtime::{Host, HostError, HostEvent};
pub use spec::{
    CommandSpec, DialogSpec, GroupSpec, HostSpec, LayoutSpec, MenuItemSpec, MenuSpec, PanelsSpec, SPEC_VERSION,
    SeriesSpec, StatusSpec, TabSpec, ThemeSpec, ToastSpec, TreeSpec, WidgetSpec, WindowSpec,
};
