//! A chat panel for agent-driven applications: streaming messages, tool cards, and a
//! diff approval row.
//!
//! The component is transport-agnostic. An application appends messages as its model
//! streams, pushes a [`ToolCall`] when the model proposes a change, and reacts to
//! [`AgentAction`]. Nothing here calls a model or a network.

use egui::{RichText, Ui};

use super::Component;
use super::button::Button;
use super::input::TextArea;
use super::style::ButtonVariant;

/// Who wrote a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AgentRole {
    /// The person.
    User,
    /// The model.
    Assistant,
    /// A tool result.
    Tool,
}

/// One chat message.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    /// Author.
    pub role: AgentRole,
    /// Markdown text. A trailing cursor is drawn while `streaming` is true.
    pub text: String,
    /// Whether the model is still writing this message.
    pub streaming: bool,
}

impl ChatMessage {
    /// A user message.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: AgentRole::User,
            text: text.into(),
            streaming: false,
        }
    }

    /// An assistant message.
    pub fn assistant(text: impl Into<String>, streaming: bool) -> Self {
        Self {
            role: AgentRole::Assistant,
            text: text.into(),
            streaming,
        }
    }

    /// A tool result.
    pub fn tool(text: impl Into<String>) -> Self {
        Self {
            role: AgentRole::Tool,
            text: text.into(),
            streaming: false,
        }
    }
}

/// The state of a proposed tool call.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToolCallState {
    /// Waiting for the person.
    #[default]
    Pending,
    /// Approved and ready to run.
    Approved,
    /// Rejected.
    Rejected,
}

/// A tool call shown as a card, optionally with a diff to approve.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    /// Tool name, such as `apply_patch`.
    pub name: String,
    /// One-line summary.
    pub summary: String,
    /// Unified diff or patch text.
    pub diff: Option<String>,
    /// Approval state.
    pub state: ToolCallState,
}

impl ToolCall {
    /// A pending call without a diff.
    pub fn new(name: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            summary: summary.into(),
            diff: None,
            state: ToolCallState::Pending,
        }
    }

    /// Attach a diff.
    pub fn diff(mut self, diff: impl Into<String>) -> Self {
        self.diff = Some(diff.into());
        self
    }
}

/// What the panel wants the application to do.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AgentAction {
    /// Send the composer text.
    Submit(String),
    /// Approve a tool call by index.
    Approve(usize),
    /// Reject a tool call by index.
    Reject(usize),
}

/// Persistent agent-panel state.
#[derive(Clone, Debug, Default)]
pub struct AgentPanelState {
    /// Conversation.
    pub messages: Vec<ChatMessage>,
    /// Proposed tool calls.
    pub tool_calls: Vec<ToolCall>,
    /// Composer text.
    pub input: String,
    /// Keep the scroll area pinned to the newest message.
    pub stick_to_bottom: bool,
}

impl AgentPanelState {
    /// An empty panel.
    pub fn new() -> Self {
        Self {
            stick_to_bottom: true,
            ..Self::default()
        }
    }

    /// Append a message.
    pub fn push(&mut self, message: ChatMessage) -> &mut Self {
        self.messages.push(message);
        self
    }

    /// Append a proposed tool call.
    pub fn propose(&mut self, call: ToolCall) -> usize {
        self.tool_calls.push(call);
        self.tool_calls.len() - 1
    }
}

/// Show the panel and return the requested action.
pub fn show_agent_panel(ui: &mut Ui, state: &mut AgentPanelState) -> Option<AgentAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Agent").strong());
        if !state.tool_calls.is_empty() {
            let pending = state
                .tool_calls
                .iter()
                .filter(|call| call.state == ToolCallState::Pending)
                .count();
            ui.label(RichText::new(format!("{pending} pending")).weak());
        }
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .stick_to_bottom(state.stick_to_bottom)
        .max_height(280.0)
        .show(ui, |ui| {
            for message in &state.messages {
                message_ui(ui, message);
            }
            for (index, call) in state.tool_calls.iter().enumerate() {
                if let Some(choice) = tool_call_ui(ui, index, call) {
                    action = Some(choice);
                }
                ui.add_space(4.0);
            }
        });
    ui.separator();
    TextArea::new(&mut state.input)
        .hint("Ask the agent, or describe a change…")
        .rows(2)
        .show(ui);
    ui.horizontal(|ui| {
        let can_send = !state.input.trim().is_empty();
        if Button::new("Send")
            .variant(ButtonVariant::Primary)
            .enabled(can_send)
            .show(ui)
            .clicked()
        {
            action = Some(AgentAction::Submit(state.input.trim().to_owned()));
            state.input.clear();
        }
        if Button::new("Clear").variant(ButtonVariant::Ghost).show(ui).clicked() {
            state.messages.clear();
            state.tool_calls.clear();
        }
    });
    action
}

fn message_ui(ui: &mut Ui, message: &ChatMessage) {
    let (label, color) = match message.role {
        AgentRole::User => ("You", ui.visuals().text_color()),
        AgentRole::Assistant => ("Assistant", ui.visuals().text_color()),
        AgentRole::Tool => ("Tool", ui.visuals().weak_text_color()),
    };
    ui.label(RichText::new(label).small().color(color));
    markdown(ui, &message.text);
    if message.streaming {
        ui.label(RichText::new("▍").color(ui.visuals().hyperlink_color));
    }
    ui.add_space(6.0);
}

fn markdown(ui: &mut Ui, text: &str) {
    let mut code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            code = !code;
            continue;
        }
        if code {
            ui.label(RichText::new(line).monospace());
        } else if let Some(heading) = line.strip_prefix('#') {
            ui.label(RichText::new(heading.trim_start_matches('#').trim()).strong());
        } else if let Some(item) = line.strip_prefix("- ") {
            ui.label(format!("• {item}"));
        } else if line.trim().is_empty() {
            ui.add_space(4.0);
        } else {
            ui.label(line);
        }
    }
}

fn tool_call_ui(ui: &mut Ui, index: usize, call: &ToolCall) -> Option<AgentAction> {
    let mut action = None;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&call.name).monospace().strong());
            match call.state {
                ToolCallState::Pending => ui.label(RichText::new("pending").weak()),
                ToolCallState::Approved => ui.label(RichText::new("approved").color(ui.visuals().hyperlink_color)),
                ToolCallState::Rejected => ui.label(RichText::new("rejected").weak()),
            };
        });
        ui.label(&call.summary);
        if let Some(diff) = &call.diff {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                for line in diff.lines() {
                    let color = if line.starts_with('+') {
                        ui.visuals().hyperlink_color
                    } else if line.starts_with('-') {
                        ui.visuals().error_fg_color
                    } else {
                        ui.visuals().weak_text_color()
                    };
                    ui.label(RichText::new(line).monospace().color(color));
                }
            });
        }
        if call.state == ToolCallState::Pending {
            ui.horizontal(|ui| {
                if Button::new("Approve")
                    .variant(ButtonVariant::Primary)
                    .show(ui)
                    .clicked()
                {
                    action = Some(AgentAction::Approve(index));
                }
                if Button::new("Reject").variant(ButtonVariant::Ghost).show(ui).clicked() {
                    action = Some(AgentAction::Reject(index));
                }
            });
        }
    });
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_and_tool_calls_render() {
        let ctx = egui::Context::default();
        let mut state = AgentPanelState::new();
        state.push(ChatMessage::user("hello"));
        state.push(ChatMessage::assistant("## Plan\n- one\n```\ncode\n```", true));
        state.propose(ToolCall::new("apply_patch", "change a label").diff("+after\n-before"));
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let _ = show_agent_panel(ui, &mut state);
        });
        output.textures_delta.clear();
        assert_eq!(state.messages.len(), 2);
        assert_eq!(state.tool_calls.len(), 1);
    }
}
