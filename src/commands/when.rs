//! Context keys and the small `when` expression evaluated for each command.
//!
//! The language is intentionally small: identifiers, `!`, `&&`, `||`, and parentheses.
//! A command is available when its `when` expression is true for the current
//! [`WhenContext`]. An empty expression is always true.

use std::collections::BTreeSet;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
/// The keys a host can publish before drawing a frame.
///
/// The constants are the names the workbench itself uses; an application may add its
/// own. Keys are case-sensitive and must be identifiers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WhenContext {
    keys: BTreeSet<String>,
}

impl WhenContext {
    /// An empty context: every key is false.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key and return the context.
    pub fn with(mut self, key: impl Into<String>) -> Self {
        self.insert(key);
        self
    }

    /// Insert a key.
    pub fn insert(&mut self, key: impl Into<String>) {
        self.keys.insert(key.into());
    }

    /// Remove a key.
    pub fn remove(&mut self, key: &str) {
        self.keys.remove(key);
    }

    /// Whether a key is set.
    pub fn contains(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    /// Clear every key.
    pub fn clear(&mut self) {
        self.keys.clear();
    }

    /// The set keys, sorted.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.keys.iter().map(String::as_str)
    }
}

/// The standard context key for a focused text editor.
pub const EDITOR_FOCUS: &str = "editorFocus";
/// The standard context key for a non-empty selection.
pub const HAS_SELECTION: &str = "hasSelection";
/// The standard context key for an open modal.
pub const DIALOG_OPEN: &str = "dialogOpen";
/// The standard context key for an open command palette.
pub const PALETTE_OPEN: &str = "paletteOpen";
/// The standard context key for a read-only document.
pub const READ_ONLY: &str = "readOnly";
/// The standard context key for unsaved changes.
pub const DIRTY: &str = "dirty";

/// A parsed `when` expression.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct When {
    source: String,
    expression: Option<Expression>,
}

impl When {
    /// An expression that is always true.
    pub fn always() -> Self {
        Self::default()
    }

    /// Parse an expression. An empty or whitespace-only string is always true.
    ///
    /// # Errors
    /// When a token or parenthesis is missing or unexpected.
    pub fn parse(source: &str) -> Result<Self, WhenError> {
        if source.trim().is_empty() {
            return Ok(Self::always());
        }
        let mut parser = Parser::new(source)?;
        let expression = parser.expression()?;
        parser.expect_end()?;
        Ok(Self {
            source: source.to_owned(),
            expression: Some(expression),
        })
    }

    /// The source text, or an empty string for an always-true expression.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Whether the expression is always true.
    pub fn is_always(&self) -> bool {
        self.expression.is_none()
    }

    /// Evaluate the expression against `context`.
    pub fn evaluate(&self, context: &WhenContext) -> bool {
        self.expression
            .as_ref()
            .is_none_or(|expression| expression.evaluate(context))
    }
}

impl fmt::Display for When {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expression {
    Key(String),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl Expression {
    fn evaluate(&self, context: &WhenContext) -> bool {
        match self {
            Self::Key(key) => context.contains(key),
            Self::Not(inner) => !inner.evaluate(context),
            Self::And(left, right) => left.evaluate(context) && right.evaluate(context),
            Self::Or(left, right) => left.evaluate(context) || right.evaluate(context),
        }
    }
}

/// What went wrong while parsing a `when` expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhenError {
    message: String,
    offset: usize,
}

impl WhenError {
    /// A human-readable message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The byte offset of the problem.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl fmt::Display for WhenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.offset)
    }
}

impl std::error::Error for WhenError {}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Result<Self, WhenError> {
        Ok(Self {
            source,
            bytes: source.as_bytes(),
            offset: 0,
        })
    }

    fn expression(&mut self) -> Result<Expression, WhenError> {
        let mut expression = self.term()?;
        loop {
            self.skip_whitespace();
            if self.take("||") {
                let right = self.term()?;
                expression = Expression::Or(Box::new(expression), Box::new(right));
            } else {
                return Ok(expression);
            }
        }
    }

    fn term(&mut self) -> Result<Expression, WhenError> {
        let mut expression = self.unary()?;
        loop {
            self.skip_whitespace();
            if self.take("&&") {
                let right = self.unary()?;
                expression = Expression::And(Box::new(expression), Box::new(right));
            } else {
                return Ok(expression);
            }
        }
    }

    fn unary(&mut self) -> Result<Expression, WhenError> {
        self.skip_whitespace();
        if self.take("!") {
            return Ok(Expression::Not(Box::new(self.unary()?)));
        }
        if self.take("(") {
            let expression = self.expression()?;
            self.skip_whitespace();
            if !self.take(")") {
                return Err(self.error("expected `)`"));
            }
            return Ok(expression);
        }
        let key = self.identifier()?;
        Ok(Expression::Key(key))
    }

    fn identifier(&mut self) -> Result<String, WhenError> {
        self.skip_whitespace();
        let start = self.offset;
        while let Some(byte) = self.bytes.get(self.offset) {
            if byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'.' || *byte == b'-' {
                self.offset += 1;
            } else {
                break;
            }
        }
        if self.offset == start {
            return Err(self.error("expected a context key"));
        }
        Ok(self.source[start..self.offset].to_owned())
    }

    fn expect_end(&mut self) -> Result<(), WhenError> {
        self.skip_whitespace();
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(self.error("unexpected input after the expression"))
        }
    }

    fn skip_whitespace(&mut self) {
        while self.bytes.get(self.offset).is_some_and(u8::is_ascii_whitespace) {
            self.offset += 1;
        }
    }

    fn take(&mut self, text: &str) -> bool {
        if self.source[self.offset..].starts_with(text) {
            self.offset += text.len();
            true
        } else {
            false
        }
    }

    fn error(&self, message: &str) -> WhenError {
        WhenError {
            message: message.to_owned(),
            offset: self.offset,
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::When;

    impl serde::Serialize for When {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.source())
        }
    }

    impl<'de> Deserialize<'de> for When {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let text = String::deserialize(deserializer)?;
            Self::parse(&text).map_err(serde::de::Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(keys: &[&str]) -> WhenContext {
        let mut context = WhenContext::new();
        for key in keys {
            context.insert(*key);
        }
        context
    }

    #[test]
    fn empty_is_always_true() {
        let when = When::parse("").expect("empty");
        assert!(when.is_always());
        assert!(when.evaluate(&WhenContext::new()));
    }

    #[test]
    fn keys_and_negation_follow_the_context() {
        let when = When::parse("editorFocus && !dialogOpen").expect("expression");
        assert!(when.evaluate(&context(&[EDITOR_FOCUS])));
        assert!(!when.evaluate(&context(&[EDITOR_FOCUS, DIALOG_OPEN])));
        assert!(!when.evaluate(&WhenContext::new()));
    }

    #[test]
    fn precedence_is_not_then_and_then_or() {
        let when = When::parse("!dialogOpen && editorFocus || hasSelection").expect("expression");
        assert!(when.evaluate(&context(&[EDITOR_FOCUS])));
        assert!(when.evaluate(&context(&[HAS_SELECTION])));
        assert!(!when.evaluate(&context(&[EDITOR_FOCUS, DIALOG_OPEN])));
        let grouped = When::parse("!(dialogOpen && editorFocus)").expect("grouped");
        assert!(grouped.evaluate(&context(&[EDITOR_FOCUS])));
    }

    #[test]
    fn errors_have_an_offset() {
        let error = When::parse("editorFocus &&").expect_err("missing operand");
        assert_eq!(error.offset(), 14);
        assert!(When::parse("(editorFocus").is_err());
        assert!(When::parse("editorFocus dialogOpen").is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn expressions_round_trip_through_json() {
        let when = When::parse("editorFocus && !dialogOpen").expect("expression");
        let json = serde_json::to_string(&when).expect("serialize");
        assert_eq!(json, "\"editorFocus && !dialogOpen\"");
        let back: When = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, when);
    }
}
