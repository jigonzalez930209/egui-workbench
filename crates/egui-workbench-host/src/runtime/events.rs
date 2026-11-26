//! Events, queued actions, and the thread-safe action queue.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::patch::Patch;
use crate::spec::ToastSpec;

/// Something the host reports to the embedding application.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
#[non_exhaustive]
pub enum HostEvent {
    /// A button or command ran.
    Command {
        /// Command id.
        id: String,
    },
    /// A value changed.
    ValueChanged {
        /// Widget id.
        id: String,
        /// New value.
        value: Value,
    },
    /// A tab strip changed.
    TabChanged {
        /// Strip id.
        id: String,
        /// Active index.
        index: usize,
    },
    /// A tree selection changed.
    TreeSelected {
        /// Tree id.
        id: String,
        /// Node key.
        key: String,
    },
    /// A custom widget emitted an event.
    Custom {
        /// Widget kind.
        kind: String,
        /// Opaque payload.
        payload: Value,
    },
    /// The host window was closed.
    Closed,
}

/// An action queued by an event handler and applied on the next frame.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum HostAction {
    /// Apply a patch to the spec.
    Patch(Patch),
    /// Show a toast.
    Toast(ToastSpec),
    /// Change the window title.
    SetTitle(String),
    /// Request another frame.
    Repaint,
    /// Close the window.
    Close,
}

/// A cloneable handle to the action queue, safe from any thread.
///
/// The queue also holds the current egui context, so a push from a worker thread wakes
/// the window instead of waiting for the next input event.
#[derive(Clone, Default)]
pub struct ActionQueue {
    actions: Arc<Mutex<VecDeque<HostAction>>>,
    waker: Arc<Mutex<Option<egui::Context>>>,
}

impl ActionQueue {
    /// Queue an action and wake the window, when it is running.
    pub fn push(&self, action: HostAction) {
        {
            let mut actions = self.actions.lock().unwrap_or_else(|error| error.into_inner());
            actions.push_back(action);
        }
        if let Some(ctx) = self.waker.lock().unwrap_or_else(|error| error.into_inner()).as_ref() {
            ctx.request_repaint();
        }
    }

    /// Remember the context used to wake the window.
    pub fn set_waker(&self, ctx: &egui::Context) {
        *self.waker.lock().unwrap_or_else(|error| error.into_inner()) = Some(ctx.clone());
    }

    pub(super) fn drain(&self) -> Vec<HostAction> {
        let mut actions = self.actions.lock().unwrap_or_else(|error| error.into_inner());
        actions.drain(..).collect()
    }
}
