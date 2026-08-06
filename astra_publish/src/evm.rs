//! EVM (Solidity) verifier target.

use crate::VerifierTarget;

pub struct Evm;

impl<VK> VerifierTarget<VK> for Evm {
    fn name(&self) -> &'static str {
        "evm"
    }

    fn render(&self, _vk: &VK, _io: (usize, usize)) -> Result<String, String> {
        Err("evm verifier export is not yet implemented in v0.0".into())
    }
}
