//! `astra_publish` — verifier export targets.
//!
//! Every target implements [`VerifierTarget`]. Targets render a verifier for a
//! given verifying key. None emit fake contracts — unimplemented targets
//! return an explicit error, so the CLI can never ship a verifier that always
//! returns `true`.

pub mod aleo;
pub mod evm;
pub mod json_test_vectors;
pub mod starknet;
pub mod wasm;

/// A compile-time-named export target.
///
/// Generic over the verifying-key type so the crate carries no dependency on a
/// concrete proof system; `render` returns an error until a target is real.
pub trait VerifierTarget<VK> {
    fn name(&self) -> &'static str;
    fn render(&self, vk: &VK, io: (usize, usize)) -> Result<String, String>;
}

/// The set of known target names, for CLI validation.
pub fn available_targets() -> [&'static str; 5] {
    ["evm", "starknet", "wasm", "aleo", "jt"]
}
