//! JSON test-vector verifier target.

use crate::VerifierTarget;

pub struct JsonTestVectors;

impl<VK> VerifierTarget<VK> for JsonTestVectors {
    fn name(&self) -> &'static str {
        "jt"
    }

    fn render(&self, _vk: &VK, _io: (usize, usize)) -> Result<String, String> {
        Err("json test-vector export is not yet implemented in v0.0".into())
    }
}
