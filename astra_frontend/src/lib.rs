//! `astra_frontend` — lexing/parsing and IDE-facing hooks for the Zara
//! circuit language.
//!
//! Produces the [`astra_ir`] AST. The `wasm` feature adds browser-native
//! bindings so the playground can compile without a backend round-trip.

pub mod error;
pub mod lsp;
pub mod parser;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use error::ParseError;
pub use parser::parse;
