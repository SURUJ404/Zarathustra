//! ark-groth16 binding.
//!
//! Replaces the hand-rolled implementation in [`crate::legacy`] with the
//! community-audited arkworks Groth16 over ark-bn254. Any Zara circuit that
//! lowers to an `astra_ir` R1CS can be translated with [`to_ark_cs`] and then
//! proven/verified here.

use ark_ff::PrimeField as ArkPrimeField;
use ark_relations::r1cs::{
    ConstraintSystem, ConstraintSystemRef, LinearCombination, SynthesisError, Variable,
};
use bls12_381::Scalar as BlsScalar;
use ff::PrimeField as _;

pub use ark_bn254::Bn254 as DefaultCurve;
pub use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};

fn bls_scalar_to_ark<F: ArkPrimeField>(s: &BlsScalar) -> F {
    let bytes = s.to_repr();
    F::from_le_bytes_mod_order(bytes.as_ref())
}

fn lc_from_row<F: ArkPrimeField>(
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

/// Translate an `astra_ir` R1CS into an arkworks constraint system.
///
/// Both systems live over the BLS12-381 scalar field, so witness values and
/// coefficients are converted via canonical bytes.
pub fn to_ark_cs<F: ArkPrimeField>(
    cs: &astra_ir::types::ConstraintSystem,
) -> Result<ConstraintSystemRef<F>, SynthesisError> {
    let ark_cs = ConstraintSystemRef::new(ConstraintSystem::<F>::new());
    let mut vars = Vec::with_capacity(cs.num_variables);
    for (i, val) in cs.witness.iter().take(cs.num_variables).enumerate() {
        let f_val = bls_scalar_to_ark::<F>(val);
        let var = if i < cs.num_public {
            ark_cs.new_input_variable(|| Ok(f_val))?
        } else {
            ark_cs.new_witness_variable(|| Ok(f_val))?
        };
        vars.push(var);
    }
    for (a_row, (b_row, c_row)) in cs.a.iter().zip(cs.b.iter().zip(cs.c.iter())) {
        let a = lc_from_row(a_row, &vars)?;
        let b = lc_from_row(b_row, &vars)?;
        let c = lc_from_row(c_row, &vars)?;
        ark_cs.enforce_constraint(a, b, c)?;
    }
    Ok(ark_cs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Bn254;
    use ark_bn254::Fr;
    use ark_relations::r1cs::ConstraintSynthesizer;
    use astra_ir::types::scalar_from_dec_str;
    use rand::thread_rng;

    #[derive(Clone)]
    struct Circuit {
        x: Option<Fr>,
        y: Option<Fr>,
        z: Option<Fr>,
    }

    impl ConstraintSynthesizer<Fr> for Circuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
            let x = cs.new_witness_variable(|| self.x.ok_or(SynthesisError::AssignmentMissing))?;
            let y = cs.new_witness_variable(|| self.y.ok_or(SynthesisError::AssignmentMissing))?;
            let z = cs.new_witness_variable(|| self.z.ok_or(SynthesisError::AssignmentMissing))?;
            let x: LinearCombination<Fr> = x.into();
            let y: LinearCombination<Fr> = y.into();
            let z: LinearCombination<Fr> = z.into();
            cs.enforce_constraint(x, y, z)?;
            Ok(())
        }
    }

    #[test]
    fn test_ark_groth16_bn254_roundtrip() -> Result<(), String> {
        use ark_snark::SNARK;
        let circuit = Circuit {
            x: Some(Fr::from(3u64)),
            y: Some(Fr::from(5u64)),
            z: Some(Fr::from(15u64)),
        };
        let mut rng = thread_rng();
        let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit.clone(), &mut rng)
            .map_err(|e| format!("setup failed: {e:?}"))?;
        let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng)
            .map_err(|e| format!("prove failed: {e:?}"))?;
        let valid = Groth16::<Bn254>::verify(&vk, &[], &proof)
            .map_err(|e| format!("verify failed: {e:?}"))?;
        assert!(valid, "ark-groth16 proof must verify");
        Ok(())
    }

    #[test]
    fn test_to_ark_cs_translation() -> Result<(), String> {
        use astra_codegen::compile;
        let source = "def main(field a, field b) -> field {\n    field c = a * b;\n    assert(a * b == c);\n    return 1;\n}\n";
        let pub_v = vec![scalar_from_dec_str("3")?, scalar_from_dec_str("5")?];
        let cs = compile(source, &pub_v, &[]).map_err(|e| format!("compile failed: {e}"))?;
        let ark_cs = to_ark_cs::<Fr>(&cs).map_err(|e| format!("to_ark_cs failed: {e:?}"))?;
        let satisfied = ark_cs
            .is_satisfied()
            .map_err(|e| format!("is_satisfied failed: {e:?}"))?;
        if !satisfied {
            eprintln!(
                "DEBUG pub={} priv={} vars={} cons={}",
                cs.num_public, cs.num_private, cs.num_variables, cs.num_constraints
            );
            eprintln!(
                "DEBUG witness = {:?}",
                cs.witness
                    .iter()
                    .map(astra_ir::types::scalar_display)
                    .collect::<Vec<_>>()
            );
            for (j, (ar, (br, cr))) in cs.a.iter().zip(cs.b.iter().zip(cs.c.iter())).enumerate() {
                eprintln!("DEBUG con {j}: a={ar:?} b={br:?} c={cr:?}");
            }
        }
        assert!(satisfied, "translated constraints must be satisfied");
        assert!(
            ark_cs.num_constraints() >= 2,
            "expected at least the multiply + assert constraints"
        );
        Ok(())
    }
}
