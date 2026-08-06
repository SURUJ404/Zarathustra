//! `astra_publish` — verifier export targets.
//!
//! Concrete, honest exports live in [`jt`](fn@jt_from_artifacts) (JSON test
//! vectors, curve-agnostic and fully real) and the EVM target, which refuses to
//! emit a fake verifier for a curve Ethereum can't pair over. The
//! [`VerifierTarget`] trait below documents the intended generic surface for
//! future real targets (each returns an explicit error until implemented, so
//! the CLI can never ship a verifier that always returns `true`).

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

/// Assemble a real `jt` (JSON test-vector) bundle from the `proof.json` and
/// `vk.json` artifacts produced by `astra prove`/`astra prove setup`.
///
/// The bundle is machine-parseable and curve-agnostic, so it is a genuine cross-
/// platform artifact (usable by external verifiers and test runners) rather
/// than a placeholder.
pub fn jt_from_artifacts(
    proof: &serde_json::Value,
    vk: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let curve = proof.get("curve").and_then(|x| x.as_str()).unwrap_or("");
    let protocol = proof.get("protocol").and_then(|x| x.as_str()).unwrap_or("");
    if curve.is_empty() || protocol.is_empty() {
        return Err(
            "proof.json must contain `curve` and `protocol` (produced by `astra prove`)".into(),
        );
    }
    let null = serde_json::Value::Null;
    Ok(serde_json::json!({
        "version": 1,
        "protocol": protocol,
        "curve": curve,
        "public_inputs": proof.get("public").cloned().unwrap_or_else(|| serde_json::json!([])),
        "proof": {
            "a": proof.get("a").cloned().unwrap_or_else(|| null.clone()),
            "b": proof.get("b").cloned().unwrap_or_else(|| null.clone()),
            "c": proof.get("c").cloned().unwrap_or_else(|| null.clone()),
        },
        "verifying_key": vk.clone(),
    }))
}

/// Honest EVM verifier-export path.
///
/// Ethereum's `alt_bn128` precompile only pairs over BN254; the default Astra
/// backend is BLS12-381, which has no mainstream Ethereum pairing precompile.
/// Rather than ship a fake Solidity verifier (one that ignores the proof and
/// always returns `true`), this refuses loudly and points to the targets that
/// actually exist.
pub fn evm_render(proof: &serde_json::Value) -> Result<String, String> {
    let curve = proof.get("curve").and_then(|x| x.as_str()).unwrap_or("");
    if curve != "bn254" {
        return Err(format!(
            "cannot render a genuine EVM verifier for curve `{curve}`: Ethereum's alt_bn128 \
             precompile only supports BN254 pairing. Use `publish -t jt` for cross-platform \
             test vectors, or add a BN254 backend."
        ));
    }
    Err(
        "EVM verifier export for BN254 is not implemented yet (needs a Solidity pairing gadget \
         or precompile-based pairing verification)"
            .into(),
    )
}
