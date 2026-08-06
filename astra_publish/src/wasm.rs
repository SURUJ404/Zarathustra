//! WASM verifier target.

use crate::VerifierTarget;

pub struct Wasm;

impl<VK> VerifierTarget<VK> for Wasm {
    fn name(&self) -> &'static str {
        "wasm"
    }

    fn render(&self, _vk: &VK, _io: (usize, usize)) -> Result<String, String> {
        Err("wasm verifier export is not yet implemented in v0.0".into())
    }
}
