//! Starknet (Cairo) verifier target.

use crate::VerifierTarget;

pub struct Starknet;

impl<VK> VerifierTarget<VK> for Starknet {
    fn name(&self) -> &'static str {
        "starknet"
    }

    fn render(&self, _vk: &VK, _io: (usize, usize)) -> Result<String, String> {
        Err("starknet verifier export is not yet implemented in v0.0".into())
    }
}
