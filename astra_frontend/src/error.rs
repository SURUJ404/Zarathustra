//! Structured parse errors with source spans.
//!
//! Used by the parser and later rendered by the LSP / CLI as
//! `error: <path>:<line>:<col>` blocks.

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub span: Option<std::ops::Range<usize>>,
}

impl ParseError {
    pub fn new(msg: impl Into<String>) -> Self {
        ParseError {
            msg: msg.into(),
            span: None,
        }
    }

    pub fn at(msg: impl Into<String>, pos: usize) -> Self {
        ParseError {
            msg: msg.into(),
            span: Some(pos..pos + 1),
        }
    }

    /// Render as `message (at byte <start>)`.
    pub fn render(&self) -> String {
        match &self.span {
            Some(span) => format!("{} (at byte {})", self.msg, span.start),
            None => self.msg.clone(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}

impl std::error::Error for ParseError {}
