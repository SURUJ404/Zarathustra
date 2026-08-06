//! Aleo (Leo) verifier target.

use crate::VerifierTarget;

pub struct Aleo;

impl<VK> VerifierTarget<VK> for Aleo {
    fn name(&self) -> &'static str {
        "aleo"
    }

    fn render(&self, _vk: &VK, _io: (usize, usize)) -> Result<String, String> {
        Err("aleo verifier export is not yet implemented in v0.0".into())
    }
}
