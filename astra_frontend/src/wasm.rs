//! WASM bindings for browser-native compilation.
//!
//! Build with:
//!
//! ```text
//! wasm-pack build astra_frontend --target web --release --features wasm
//! ```
//!
//! and serve the output under `web/public/pkg/`. The playground prefers this
//! module when present and only falls back to the `/api/*` backend otherwise.

use crate::parser;

/// Compile a Zara program, returning a description of its AST.
///
/// Placeholder until `astra_codegen` is WASM-safe; today it proves the parse
/// frontend runs natively in the browser.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn compile(source: &str) -> Result<String, String> {
    let program = parser::parse(source).map_err(|e| e.render())?;
    Ok(format!("{:#?}", program))
}
