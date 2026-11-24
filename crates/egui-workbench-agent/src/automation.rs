//! Recorded flows and headless replay.
//!
//! A flow is a list of commands and patches. It can be recorded from a journal and
//! replayed against a `Host` with no window, which is the layer above `egui_kittest` for
//! CI: find a node by accessible name, run the command, compare the resulting spec.

use egui_workbench::app::CommandJournal;
use egui_workbench_host::{Host, HostEvent, Message, Patch, Request};

use crate::access::{AccessNode, find_node};

/// One recorded step.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "step", rename_all = "snake_case")]
#[non_exhaustive]
pub enum AutomationStep {
    /// Run a command id.
    Command {
        /// Command id.
        id: String,
    },
    /// Apply a patch.
    Patch {
        /// The patch.
        patch: Patch,
    },
}

/// A recorded flow.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Automation {
    /// Steps in order.
    pub steps: Vec<AutomationStep>,
}

impl Automation {
    /// An empty flow.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record every command in a journal as a replayable flow.
    pub fn from_journal(journal: &CommandJournal) -> Self {
        Self {
            steps: journal
                .entries
                .iter()
                .map(|entry| AutomationStep::Command {
                    id: entry.command.to_string(),
                })
                .collect(),
        }
    }

    /// Append a command step.
    pub fn command(&mut self, id: impl Into<String>) -> &mut Self {
        self.steps.push(AutomationStep::Command { id: id.into() });
        self
    }

    /// Append a patch step.
    pub fn patch(&mut self, patch: Patch) -> &mut Self {
        self.steps.push(AutomationStep::Patch { patch });
        self
    }

    /// Replay the flow on a host and return every event.
    ///
    /// # Errors
    /// When a patch cannot be applied.
    pub fn replay(&self, host: &mut Host) -> Result<Vec<HostEvent>, AutomationError> {
        let mut events = Vec::new();
        for step in &self.steps {
            match step {
                AutomationStep::Command { id } => {
                    let _ = host.handle(Message::Request {
                        id: 0,
                        request: Request::RunCommand { id: id.clone() },
                    });
                }
                AutomationStep::Patch { patch } => {
                    host.apply(patch)
                        .map_err(|error| AutomationError::Patch(error.to_string()))?;
                }
            }
            events.extend(host.frame());
        }
        Ok(events)
    }

    /// The step that targets a node, when the node was found by accessible name.
    pub fn target(nodes: &[AccessNode], name: &str, role: Option<&str>) -> Option<String> {
        find_node(nodes, name, role).and_then(|node| node.id.clone())
    }
}

/// Why a replay failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AutomationError {
    /// A patch was rejected.
    Patch(String),
}

impl std::fmt::Display for AutomationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Patch(error) => write!(formatter, "patch failed: {error}"),
        }
    }
}

impl std::error::Error for AutomationError {}
