//! Command metadata, ids, arguments, and binding errors.

use std::fmt;

use super::{KeySequence, When, WhenContext};

/// A stable, namespaced command identifier such as `file.save`.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandId(String);

impl CommandId {
    /// An identifier from a string. Prefer a namespaced `category.action` name.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the identifier is non-empty and has no whitespace.
    pub fn is_valid(&self) -> bool {
        !self.0.is_empty() && !self.0.chars().any(char::is_whitespace)
    }

    /// Whether the identifier is in `category.action` form.
    pub fn is_namespaced(&self) -> bool {
        self.0.contains('.') && self.is_valid()
    }
}

impl fmt::Display for CommandId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<&str> for CommandId {
    fn from(id: &str) -> Self {
        Self(id.to_owned())
    }
}

impl From<String> for CommandId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl AsRef<str> for CommandId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for CommandId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for CommandId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CommandId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        <String as serde::Deserialize>::deserialize(deserializer).map(Self)
    }
}

/// The kind of argument a command asks for after it is chosen.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArgKind {
    /// Free text.
    Text,
    /// A number.
    Number,
    /// One of a fixed set of values.
    Choice(Vec<String>),
    /// A path, with an optional directory/file filter.
    Path,
}

impl ArgKind {
    /// A short name for the palette prompt.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Choice(_) => "choice",
            Self::Path => "path",
        }
    }
}

/// The optional argument a command asks for.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ArgSchema {
    /// The argument name, used in the prompt and the command journal.
    pub name: String,
    /// The prompt shown to the user.
    pub prompt: String,
    /// What the palette should collect.
    pub kind: ArgKind,
    /// Whether the command can run without it.
    pub required: bool,
}

impl ArgSchema {
    /// A required text argument.
    pub fn text(name: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prompt: prompt.into(),
            kind: ArgKind::Text,
            required: true,
        }
    }

    /// Make the argument optional.
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Change the kind.
    pub fn kind(mut self, kind: ArgKind) -> Self {
        self.kind = kind;
        self
    }
}

/// Enabled and checked state, updated by the application before drawing a frame.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandState {
    /// Whether the command can run now.
    pub enabled: bool,
    /// Whether the command is a checked toggle.
    pub checked: bool,
}

impl Default for CommandState {
    fn default() -> Self {
        Self {
            enabled: true,
            checked: false,
        }
    }
}

/// Metadata for one action.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Command {
    /// Stable namespaced identifier.
    pub id: CommandId,
    /// Visible title.
    pub title: String,
    /// Group shown in menus and the palette.
    pub category: String,
    /// Optional icon name.
    pub icon: Option<String>,
    /// Optional secondary text.
    pub description: Option<String>,
    /// Search terms beyond the title.
    pub keywords: Vec<String>,
    /// The default shortcut.
    pub default_shortcut: Option<KeySequence>,
    /// The context in which the command is available.
    pub when: When,
    /// Enabled and checked state.
    pub state: CommandState,
    /// Optional argument schema.
    pub args: Option<ArgSchema>,
}

impl Command {
    /// A command with the default category (`General`).
    pub fn new(id: impl Into<CommandId>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category: "General".to_owned(),
            icon: None,
            description: None,
            keywords: Vec::new(),
            default_shortcut: None,
            when: When::always(),
            state: CommandState::default(),
            args: None,
        }
    }

    /// Set the menu category.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    /// Set the icon name.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the secondary text.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add search terms.
    pub fn keywords(mut self, keywords: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keywords.extend(keywords.into_iter().map(Into::into));
        self
    }

    /// Set the default shortcut. An invalid string is ignored; use
    /// [`Self::try_shortcut`] when the caller must see the error.
    pub fn shortcut(mut self, shortcut: impl AsRef<str>) -> Self {
        if let Some(sequence) = KeySequence::parse(shortcut.as_ref()) {
            self.default_shortcut = Some(sequence);
        }
        self
    }

    /// Set the default shortcut and report an invalid string.
    pub fn try_shortcut(mut self, shortcut: impl AsRef<str>) -> Result<Self, BindingError> {
        let text = shortcut.as_ref();
        let sequence = KeySequence::parse(text).ok_or_else(|| BindingError::InvalidSequence(text.to_owned()))?;
        self.default_shortcut = Some(sequence);
        Ok(self)
    }

    /// Set the availability context.
    pub fn when(mut self, when: impl AsRef<str>) -> Self {
        if let Ok(when) = When::parse(when.as_ref()) {
            self.when = when;
        }
        self
    }

    /// Set the availability context and report a parse error.
    pub fn try_when(mut self, when: impl AsRef<str>) -> Result<Self, BindingError> {
        self.when = When::parse(when.as_ref()).map_err(|error| BindingError::InvalidWhen(error.to_string()))?;
        Ok(self)
    }

    /// Attach an argument schema.
    pub fn args(mut self, schema: ArgSchema) -> Self {
        self.args = Some(schema);
        self
    }

    /// Whether the command is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.state.enabled
    }

    /// Whether the command is currently checked.
    pub fn is_checked(&self) -> bool {
        self.state.checked
    }

    /// Whether the command applies in `context`.
    pub fn applies(&self, context: &WhenContext) -> bool {
        self.when.evaluate(context)
    }
}

/// Why a binding could not be applied.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BindingError {
    /// The command does not exist.
    UnknownCommand(CommandId),
    /// The shortcut text is not a valid chord or sequence.
    InvalidSequence(String),
    /// The `when` expression is invalid.
    InvalidWhen(String),
    /// Another command already owns the sequence.
    Conflict {
        /// The sequence that is already bound.
        sequence: KeySequence,
        /// The command that owns it.
        existing: CommandId,
    },
}

impl fmt::Display for BindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand(id) => write!(formatter, "unknown command `{id}`"),
            Self::InvalidSequence(text) => write!(formatter, "`{text}` is not a valid key sequence"),
            Self::InvalidWhen(text) => write!(formatter, "invalid when expression: {text}"),
            Self::Conflict { sequence, existing } => {
                write!(formatter, "{sequence} is already bound to `{existing}`")
            }
        }
    }
}

impl std::error::Error for BindingError {}
