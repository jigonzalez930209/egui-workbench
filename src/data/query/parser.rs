//! The query lexer and parser.

use super::super::column::DataValue;
use super::QueryError;
use super::ast::{CompareOp, Expr};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Token {
    Ident(String),
    Int(i64),
    Float(f64),
    Text(String),
    True,
    False,
    Null,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    In,
    DotDot,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    End,
}

pub(super) struct Lexer<'a> {
    source: &'a str,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self { source, offset: 0 }
    }

    pub(super) fn tokens(mut self) -> Result<Vec<(Token, usize)>, QueryError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            let start = self.offset;
            let Some(character) = self.peek() else {
                tokens.push((Token::End, start));
                return Ok(tokens);
            };
            let token = match character {
                '(' => self.single(Token::LParen),
                ')' => self.single(Token::RParen),
                '[' => self.single(Token::LBracket),
                ']' => self.single(Token::RBracket),
                ',' => self.single(Token::Comma),
                '!' => {
                    self.bump();
                    if self.take('=') { Token::Ne } else { Token::Not }
                }
                '=' => {
                    self.bump();
                    if self.take('=') {
                        Token::Eq
                    } else {
                        return Err(self.error("expected `==`", start));
                    }
                }
                '<' => {
                    self.bump();
                    if self.take('=') { Token::Le } else { Token::Lt }
                }
                '>' => {
                    self.bump();
                    if self.take('=') { Token::Ge } else { Token::Gt }
                }
                '&' => {
                    self.bump();
                    if self.take('&') {
                        Token::And
                    } else {
                        return Err(self.error("expected `&&`", start));
                    }
                }
                '|' => {
                    self.bump();
                    if self.take('|') {
                        Token::Or
                    } else {
                        return Err(self.error("expected `||`", start));
                    }
                }
                '.' => {
                    self.bump();
                    if self.take('.') {
                        Token::DotDot
                    } else {
                        return Err(self.error("expected `..`", start));
                    }
                }
                '"' | '\'' => Token::Text(self.string(character)?),
                character if character.is_ascii_digit() || character == '-' => self.number()?,
                character if character.is_ascii_alphabetic() || character == '_' => {
                    let ident = self.identifier();
                    match ident.as_str() {
                        "true" => Token::True,
                        "false" => Token::False,
                        "null" => Token::Null,
                        "in" => Token::In,
                        "and" => Token::And,
                        "or" => Token::Or,
                        "not" => Token::Not,
                        _ => Token::Ident(ident),
                    }
                }
                _ => return Err(self.error("unexpected character", start)),
            };
            tokens.push((token, start));
        }
    }

    fn single(&mut self, token: Token) -> Token {
        self.bump();
        token
    }

    fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.offset += character.len_utf8();
        Some(character)
    }

    fn take(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    fn identifier(&mut self) -> String {
        let start = self.offset;
        while let Some(character) = self.peek() {
            if character.is_ascii_alphanumeric() || character == '_' || character == '.' {
                self.bump();
            } else {
                break;
            }
        }
        self.source[start..self.offset].to_owned()
    }

    fn number(&mut self) -> Result<Token, QueryError> {
        let start = self.offset;
        if self.peek() == Some('-') {
            self.bump();
        }
        while self.peek().is_some_and(|character| character.is_ascii_digit()) {
            self.bump();
        }
        let mut float = false;
        if self.peek() == Some('.')
            && self.source[self.offset..]
                .chars()
                .nth(1)
                .is_some_and(|c| c.is_ascii_digit())
        {
            float = true;
            self.bump();
            while self.peek().is_some_and(|character| character.is_ascii_digit()) {
                self.bump();
            }
        }
        let text = &self.source[start..self.offset];
        if float {
            text.parse::<f64>()
                .map(Token::Float)
                .map_err(|_| self.error("invalid float", start))
        } else {
            text.parse::<i64>()
                .map(Token::Int)
                .map_err(|_| self.error("invalid integer", start))
        }
    }

    fn string(&mut self, quote: char) -> Result<String, QueryError> {
        let start = self.offset;
        self.bump();
        let mut value = String::new();
        loop {
            match self.bump() {
                Some(character) if character == quote => return Ok(value),
                Some('\\') => match self.bump() {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('\\') => value.push('\\'),
                    Some(character) => value.push(character),
                    None => return Err(self.error("unterminated escape", start)),
                },
                Some(character) => value.push(character),
                None => return Err(self.error("unterminated string", start)),
            }
        }
    }

    fn error(&self, message: &str, offset: usize) -> QueryError {
        QueryError {
            message: message.to_owned(),
            offset,
        }
    }
}

pub(super) struct Parser {
    tokens: Vec<(Token, usize)>,
    index: usize,
}

impl Parser {
    pub(super) fn new(tokens: Vec<(Token, usize)>) -> Self {
        Self { tokens, index: 0 }
    }

    pub(super) fn expression(&mut self) -> Result<Expr, QueryError> {
        self.or()
    }

    fn or(&mut self) -> Result<Expr, QueryError> {
        let mut expression = self.and()?;
        while self.take(&Token::Or) {
            expression = Expr::Or(Box::new(expression), Box::new(self.and()?));
        }
        Ok(expression)
    }

    fn and(&mut self) -> Result<Expr, QueryError> {
        let mut expression = self.not()?;
        while self.take(&Token::And) {
            expression = Expr::And(Box::new(expression), Box::new(self.not()?));
        }
        Ok(expression)
    }

    fn not(&mut self) -> Result<Expr, QueryError> {
        if self.take(&Token::Not) {
            return Ok(Expr::Not(Box::new(self.not()?)));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr, QueryError> {
        let left = self.primary()?;
        let op = match self.peek_token() {
            Token::Eq => Some(CompareOp::Eq),
            Token::Ne => Some(CompareOp::Ne),
            Token::Lt => Some(CompareOp::Lt),
            Token::Le => Some(CompareOp::Le),
            Token::Gt => Some(CompareOp::Gt),
            Token::Ge => Some(CompareOp::Ge),
            Token::In => None,
            _ => return Ok(left),
        };
        if let Some(op) = op {
            self.advance();
            let right = self.primary()?;
            return Ok(Expr::Compare {
                op,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        self.advance();
        if self.take(&Token::LBracket) {
            let mut values = Vec::new();
            if !self.take(&Token::RBracket) {
                loop {
                    values.push(self.primary()?);
                    if self.take(&Token::RBracket) {
                        break;
                    }
                    if !self.take(&Token::Comma) {
                        return Err(self.error("expected `,` or `]`", self.offset()));
                    }
                }
            }
            return Ok(Expr::InList {
                value: Box::new(left),
                values,
            });
        }
        let start = self.primary()?;
        if self.take(&Token::DotDot) {
            let end = self.primary()?;
            return Ok(Expr::InRange {
                value: Box::new(left),
                start: Box::new(start),
                end: Box::new(end),
            });
        }
        let mut values = vec![start];
        while self.take(&Token::Comma) {
            values.push(self.primary()?);
        }
        Ok(Expr::InList {
            value: Box::new(left),
            values,
        })
    }

    fn primary(&mut self) -> Result<Expr, QueryError> {
        let (token, offset) = self.peek().clone();
        self.advance();
        match token {
            Token::Ident(name) => Ok(Expr::Column(name)),
            Token::Int(value) => Ok(Expr::Literal(DataValue::Int(value))),
            Token::Float(value) => Ok(Expr::Literal(DataValue::Float(value))),
            Token::Text(value) => Ok(Expr::Literal(DataValue::Text(value))),
            Token::True => Ok(Expr::Literal(DataValue::Bool(true))),
            Token::False => Ok(Expr::Literal(DataValue::Bool(false))),
            Token::Null => Ok(Expr::Literal(DataValue::Null)),
            Token::LParen => {
                let expression = self.expression()?;
                if !self.take(&Token::RParen) {
                    return Err(self.error("expected `)`", offset));
                }
                Ok(expression)
            }
            Token::LBracket => {
                let mut values = Vec::new();
                if !self.take(&Token::RBracket) {
                    loop {
                        values.push(self.primary()?);
                        if self.take(&Token::RBracket) {
                            break;
                        }
                        if !self.take(&Token::Comma) {
                            return Err(self.error("expected `,` or `]`", offset));
                        }
                    }
                }
                // A bracketed list is only meaningful on the right of `in`; the parser
                // represents it as a literal list and the comparator rejects it.
                Ok(Expr::InList {
                    value: Box::new(Expr::Literal(DataValue::Null)),
                    values,
                })
            }
            _ => Err(self.error("expected a value", offset)),
        }
    }

    pub(super) fn expect_end(&mut self) -> Result<(), QueryError> {
        if matches!(self.peek_token(), Token::End) {
            Ok(())
        } else {
            Err(self.error("unexpected input after the expression", self.offset()))
        }
    }

    fn peek(&self) -> &(Token, usize) {
        &self.tokens[self.index.min(self.tokens.len() - 1)]
    }

    fn peek_token(&self) -> &Token {
        &self.peek().0
    }

    fn advance(&mut self) {
        self.index = (self.index + 1).min(self.tokens.len() - 1);
    }

    fn take(&mut self, expected: &Token) -> bool {
        if self.peek_token() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn offset(&self) -> usize {
        self.peek().1
    }

    fn error(&self, message: &str, offset: usize) -> QueryError {
        QueryError {
            message: message.to_owned(),
            offset,
        }
    }
}
