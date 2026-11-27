//! The permission model of one MCP session.

use std::collections::HashSet;

/// Read/write permissions for one MCP session.
#[derive(Clone, Debug)]
pub struct Permissions {
    /// When true (the default), every mutating tool is denied.
    pub read_only: bool,
    /// When set, commands outside this set are denied.
    pub allowed_commands: Option<HashSet<String>>,
    /// Commands that need `arguments.confirmation` to match `confirmation_token`.
    pub sensitive_commands: HashSet<String>,
    /// The confirmation value for sensitive commands.
    pub confirmation_token: Option<String>,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            read_only: true,
            allowed_commands: None,
            sensitive_commands: HashSet::new(),
            confirmation_token: None,
        }
    }
}

impl Permissions {
    /// Allow reads and writes.
    pub fn read_write() -> Self {
        Self {
            read_only: false,
            ..Self::default()
        }
    }

    /// Allow only these commands.
    pub fn allow_commands<I, S>(mut self, commands: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_commands = Some(commands.into_iter().map(Into::into).collect());
        self
    }

    /// Require a confirmation value for these commands.
    pub fn confirm<I, S>(mut self, commands: I, token: impl Into<String>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.sensitive_commands = commands.into_iter().map(Into::into).collect();
        self.confirmation_token = Some(token.into());
        self
    }

    /// Require confirmation for these commands without configuring a value.
    ///
    /// Every call is refused with [`crate::mcp::ToolError::ConfirmationRequired`] until a
    /// token is configured, which is the safe default for `--read-write` without a secret.
    pub fn require_confirmation<I, S>(mut self, commands: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.sensitive_commands = commands.into_iter().map(Into::into).collect();
        self.confirmation_token = None;
        self
    }
}
