//! ark-groth16 binding (default proof system).
//!
//! Replaces the hand-rolled implementation in [`crate::legacy`] with the
//! community-audited arkworks Groth16 over ark-bls12-381. The `astra_ir` R1CS
//! lives over the BLS12-381 scalar field, so the ark curve is chosen to match
//! (`ark_bls12_381` shares that scalar field). Any Zara circuit that lowers to
//! an `astra_ir` R1CS can be synthesized by [`ZkCircuit`] (consuming the
//! Plonkish-first `Constraint::R1CS` kinds) and then trusted-setup, proven and
//! verified here.
//!
//! Keys and proofs are persisted as versioned JSON files (`pk.json`,
//! `vk.json`, `proof.json`) so that `prove verify` performs an honest
//! verification of the produced proof rather than re-computing a witness.

use std::fs;
use std::path::Path;

use ark_relations::r1cs::{
    ConstraintSystem as ArkConstraintSystem, ConstraintSystemRef, LinearCombination,
    SynthesisError, Variable,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use bls12_381::Scalar as BlsScalar;
use ff::Field;
use ff::PrimeField as _;
use rand::thread_rng;

use astra_ir::types::Constraint as ZConstraint;
use astra_ir::types::ConstraintSystem as ZirCS;

pub use ark_bls12_381::Bls12_381 as DefaultCurve;
pub use ark_bls12_381::{Bls12_381, Fr, G1Affine, G2Affine};
pub use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};

pub const DEFAULT_CURVE: &str = "bls12-381";
pub const DEFAULT_PROTOCOL: &str = "groth16";

pub const PK_FILE: &str = "pk.json";
pub const VK_FILE: &str = "vk.json";
pub const PROOF_FILE: &str = "proof.json";

fn bls_scalar_to_ark<F: ark_ff::PrimeField>(s: &BlsScalar) -> F {
    let bytes = s.to_repr();
    F::from_le_bytes_mod_order(bytes.as_ref())
}

fn lc_from_row<F: ark_ff::PrimeField>(
    row: &[(usize, BlsScalar)],
    vars: &[Variable],
) -> Result<LinearCombination<F>, SynthesisError> {
    let mut lc = LinearCombination::zero();
    for (idx, coeff) in row {
        let var = vars.get(*idx).ok_or(SynthesisError::AssignmentMissing)?;
        lc += (bls_scalar_to_ark::<F>(coeff), *var);
    }
    Ok(lc)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if !s.len().is_multiple_of(2) {
        return Err("hex string has odd length".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|_| format!("invalid hex byte near index {i}"))
        })
        .collect()
}

/// Synthesizes an `astra_ir` constraint system into arkworks. Used for the
/// low-level translation test in [`to_ark_cs`] and by [`ZkCircuit`] during
/// Groth16 setup/proving.
fn fill_circuit<F: ark_ff::PrimeField>(
    cs: &ZirCS,
    env: &ConstraintSystemRef<F>,
) -> Result<(), SynthesisError> {
    let mut vars = Vec::with_capacity(cs.num_variables);
    for i in 0..cs.num_variables {
        let val = cs.witness.get(i).cloned().unwrap_or(BlsScalar::ZERO);
        // Index 0 is the `~one` constant: ark already reserves instance
        // variable 0 for the implicit `1`, so it maps to `Variable::One`
        // rather than a fresh input variable.
        if i == 0 {
            vars.push(Variable::One);
            continue;
        }
        let f_val = bls_scalar_to_ark::<F>(&val);
        let var = if i < cs.num_public {
            env.new_input_variable(|| Ok(f_val))?
        } else {
            env.new_witness_variable(|| Ok(f_val))?
        };
        vars.push(var);
    }
    for c in &cs.constraints {
        let t = match c {
            ZConstraint::R1CS(t) => t,
            // Only R1CS is realisable under Groth16; anything else fails
            // loudly instead of silently producing a different circuit.
            _ => return Err(SynthesisError::Unsatisfiable),
        };
        let a = lc_from_row(&t.a, &vars)?;
        let b = lc_from_row(&t.b, &vars)?;
        let c = lc_from_row(&t.c, &vars)?;
        env.enforce_constraint(a, b, c)?;
    }
    Ok(())
}

/// Translate an `astra_ir` R1CS into an arkworks constraint system.
/// Both systems live over the BLS12-381 scalar field.
pub fn to_ark_cs<F: ark_ff::PrimeField>(
    cs: &ZirCS,
) -> Result<ConstraintSystemRef<F>, SynthesisError> {
    let ark_cs = ConstraintSystemRef::new(ArkConstraintSystem::<F>::new());
    fill_circuit(cs, &ark_cs)?;
    Ok(ark_cs)
}

/// A Groth16 circuit that synthesizes a whole Zara constraint system.
pub struct ZkCircuit {
    pub cs: ZirCS,
}

impl ark_relations::r1cs::ConstraintSynthesizer<Fr> for ZkCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        fill_circuit(&self.cs, &cs)
    }
}

/// The public inputs known to a verifier: every variable index in
/// `[1, num_public)` — the `~one` constant at index 0 is excluded, matching
/// ark's public-input convention.
pub fn public_inputs_of(cs: &ZirCS) -> Vec<BlsScalar> {
    cs.witness
        .iter()
        .skip(1)
        .take(cs.num_public.saturating_sub(1))
        .cloned()
        .collect()
}

/// Run the arkworks trusted setup for a circuit: returns `(pk, vk)`.
pub fn setup(cs: &ZirCS) -> Result<(ProvingKey<Bls12_381>, VerifyingKey<Bls12_381>), String> {
    let mut rng = thread_rng();
    let circuit = ZkCircuit { cs: cs.clone() };
    Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng)
        .map_err(|e| format!("trusted setup failed: {e:?}"))
}

/// Produce a Groth16 proof that the circuit (with its witness, incl. the
/// signed public inputs) is satisfied.
pub fn prove(cs: &ZirCS, pk: &ProvingKey<Bls12_381>) -> Result<Proof<Bls12_381>, String> {
    let mut rng = thread_rng();
    let circuit = ZkCircuit { cs: cs.clone() };
    Groth16::<Bls12_381>::prove(pk, circuit, &mut rng).map_err(|e| format!("prove failed: {e:?}"))
}

/// Honestly verify a proof against the public inputs bound in the proof.
/// Returns `Ok(true)` when valid, `Ok(false)` when invalid, and `Err` on a
/// genuine verification error.
pub fn verify(
    vk: &VerifyingKey<Bls12_381>,
    public_inputs: &[BlsScalar],
    proof: &Proof<Bls12_381>,
) -> Result<bool, String> {
    let public_fr: Vec<Fr> = public_inputs.iter().map(bls_scalar_to_ark).collect();
    Groth16::<Bls12_381>::verify(vk, &public_fr, proof).map_err(|e| format!("verify failed: {e:?}"))
}

// ---- key & proof serialization ----

fn point_hex<T: CanonicalSerialize>(val: &T) -> String {
    let mut buf = Vec::new();
    val.serialize_compressed(&mut buf).expect("serialize point");
    hex_encode(&buf)
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| format!("encode json: {e}"))?;
    fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

pub fn write_pk_json(dir: &Path, pk: &ProvingKey<Bls12_381>) -> Result<(), String> {
    let mut buf = Vec::new();
    pk.serialize_compressed(&mut buf)
        .map_err(|e| format!("serialize proving key: {e}"))?;
    let value = serde_json::json!({
        "protocol": DEFAULT_PROTOCOL,
        "curve": DEFAULT_CURVE,
        "proving_key": hex_encode(&buf),
    });
    write_json(&dir.join(PK_FILE), &value)
}

pub fn read_pk_json(dir: &Path) -> Result<ProvingKey<Bls12_381>, String> {
    let value = read_json(&dir.join(PK_FILE))?;
    let hex = value
        .get("proving_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{} missing `proving_key`", PK_FILE))?;
    let bytes = hex_decode(hex)?;
    ProvingKey::deserialize_compressed(bytes.as_slice())
        .map_err(|e| format!("decode proving key: {e}"))
}

pub fn write_vk_json(dir: &Path, vk: &VerifyingKey<Bls12_381>) -> Result<(), String> {
    let mut buf = Vec::new();
    vk.serialize_compressed(&mut buf)
        .map_err(|e| format!("serialize verifying key: {e}"))?;
    let value = serde_json::json!({
        "protocol": DEFAULT_PROTOCOL,
        "curve": DEFAULT_CURVE,
        "verifying_key": hex_encode(&buf),
    });
    write_json(&dir.join(VK_FILE), &value)
}

pub fn read_vk_json(dir: &Path) -> Result<VerifyingKey<Bls12_381>, String> {
    let value = read_json(&dir.join(VK_FILE))?;
    let hex = value
        .get("verifying_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{} missing `verifying_key`", VK_FILE))?;
    let bytes = hex_decode(hex)?;
    VerifyingKey::deserialize_compressed(bytes.as_slice())
        .map_err(|e| format!("decode verifying key: {e}"))
}

/// Write a proof with its bound public inputs to `dir/proof.json`.
pub fn write_proof_json(
    dir: &Path,
    proof: &Proof<Bls12_381>,
    public: &[BlsScalar],
) -> Result<(), String> {
    let mut raw = Vec::new();
    proof
        .serialize_compressed(&mut raw)
        .map_err(|e| format!("serialize proof: {e}"))?;
    let public_str: Vec<String> = public.iter().map(astra_ir::types::scalar_display).collect();
    let value = serde_json::json!({
        "version": 1,
        "protocol": DEFAULT_PROTOCOL,
        "curve": DEFAULT_CURVE,
        "a": point_hex(&proof.a),
        "b": point_hex(&proof.b),
        "c": point_hex(&proof.c),
        "public": public_str,
        "raw": hex_encode(&raw),
    });
    write_json(&dir.join(PROOF_FILE), &value)
}

/// Read back the proof and its bound public inputs from `dir/proof.json`.
pub fn read_proof_json(dir: &Path) -> Result<(Proof<Bls12_381>, Vec<BlsScalar>), String> {
    let value = read_json(&dir.join(PROOF_FILE))?;
    let raw = value
        .get("raw")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{} missing `raw`", PROOF_FILE))?;
    let bytes = hex_decode(raw)?;
    let proof = Proof::deserialize_compressed(bytes.as_slice())
        .map_err(|e| format!("decode proof: {e}"))?;
    let public = match value.get("public") {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(astra_ir::types::scalar_from_dec_str)
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    };
    Ok((proof, public))
}

/// Human-readable A/B/C hex for CLI display.
pub fn proof_components_hex(proof: &Proof<Bls12_381>) -> (String, String, String) {
    (
        point_hex(&proof.a),
        point_hex(&proof.b),
        point_hex(&proof.c),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use astra_ir::types::R1CSTriple;
    use ff::Field;

    #[test]
    fn test_ark_groth16_bls12_381_from_zara_cs() -> Result<(), String> {
        let cs = ZirCS {
            num_public: 3,
            num_private: 1,
            num_variables: 4,
            num_constraints: 1,
            a: vec![vec![(1, BlsScalar::ONE)]],
            b: vec![vec![(2, BlsScalar::ONE)]],
            c: vec![vec![(3, BlsScalar::ONE)]],
            witness: vec![
                BlsScalar::ONE,
                BlsScalar::from(3u64),
                BlsScalar::from(5u64),
                BlsScalar::from(15u64),
            ],
            constraints: vec![ZConstraint::R1CS(R1CSTriple {
                a: vec![(1, BlsScalar::ONE)],
                b: vec![(2, BlsScalar::ONE)],
                c: vec![(3, BlsScalar::ONE)],
            })],
        };
        let (pk, vk) = setup(&cs)?;
        let proof = prove(&cs, &pk)?;
        let public = public_inputs_of(&cs);
        assert_eq!(public.len(), 2, "expected two user public inputs");
        let valid = verify(&vk, &public, &proof)?;
        assert!(valid, "ark-groth16 proof must verify");

        let wrong = vec![BlsScalar::from(4u64), BlsScalar::from(5u64)];
        let bad = verify(&vk, &wrong, &proof)?;
        assert!(!bad, "proof must fail against wrong public inputs");
        Ok(())
    }

    #[test]
    fn test_proof_roundtrips_through_files() -> Result<(), String> {
        let dir = std::env::temp_dir().join(format!("zara_roundtrip_{}", std::process::id()));
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let cs = ZirCS {
            num_public: 3,
            num_private: 1,
            num_variables: 4,
            num_constraints: 1,
            a: vec![vec![(1, BlsScalar::ONE)]],
            b: vec![vec![(2, BlsScalar::ONE)]],
            c: vec![vec![(3, BlsScalar::ONE)]],
            witness: vec![
                BlsScalar::ONE,
                BlsScalar::from(3u64),
                BlsScalar::from(5u64),
                BlsScalar::from(15u64),
            ],
            constraints: vec![ZConstraint::R1CS(R1CSTriple {
                a: vec![(1, BlsScalar::ONE)],
                b: vec![(2, BlsScalar::ONE)],
                c: vec![(3, BlsScalar::ONE)],
            })],
        };
        let (pk, vk) = setup(&cs)?;
        write_pk_json(&dir, &pk)?;
        write_vk_json(&dir, &vk)?;
        let pk2 = read_pk_json(&dir)?;
        let vk2 = read_vk_json(&dir)?;

        let proof = prove(&cs, &pk)?;
        let public = public_inputs_of(&cs);
        write_proof_json(&dir, &proof, &public)?;
        let (proof2, public2) = read_proof_json(&dir)?;

        // Round-tripped key must still verify the round-tripped proof.
        let valid = verify(&vk2, &public2, &proof2)?;
        assert!(valid, "round-tripped proof must verify");
        // A fresh proof from the round-tripped pk verifies against the round-tripped proof.
        let proof3 = prove(&cs, &pk2)?;
        let valid2 = verify(&vk2, &public, &proof3)?;
        assert!(valid2, "proof from round-tripped pk must verify");
        let _ = &proof;
        Ok(())
    }
}
