//! LSP-style document service hooks for the Zara language.
//!
//! This is the integration surface for a future `tower-lsp` backend and a VS
//! Code extension. The trait shapes the frontend so IDE features (hover,
//! go-to-definition, references, diagnostics) can be implemented later in one
//! place without touching the transport layer.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone)]
pub struct Hover {
    pub content: String,
    pub range: Option<Range>,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Language-service surface for `.zara` documents.
///
/// A `tower-lsp` backend or a VS Code extension implements this trait; the
/// rest of the workspace only ever talks to this interface.
pub trait DocumentService {
    fn hover(&self, pos: Position) -> Option<Hover>;
    fn definition(&self, pos: Position) -> Option<Location>;
    fn references(&self, pos: Position) -> Vec<Location>;
    fn diagnostics(&self) -> Vec<Diagnostic>;
}
