//! `astra_ir` — core IR for the Zara circuit language.
//!
//! Deliberately free of proof-system crypto: AST ([`ir`]) and constraint
//! types ([`types`]) only. Downstream crates lower from here.

pub mod ir;
pub mod types;

pub use ir::*;
pub use types::*;
