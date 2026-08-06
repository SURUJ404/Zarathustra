//! Constraint-system types, scalar helpers and the Plonkish-first constraint
//! kinds that replace the single-shape `a * b = c` representation.

use bls12_381::Scalar;
use ff::PrimeField;

pub fn scalar_from_dec_str(s: &str) -> Result<Scalar, String> {
    if let Some(rest) = s.strip_prefix('-') {
        let pos = scalar_from_dec_str(rest)?;
        return Ok(-pos);
    }
    let n = num_bigint::BigUint::parse_bytes(s.as_bytes(), 10)
        .ok_or_else(|| format!("invalid number: {}", s))?;
    let bytes = n.to_bytes_le();
    let mut repr = <Scalar as PrimeField>::Repr::default();
    let len = bytes.len().min(repr.as_ref().len());
    repr.as_mut()[..len].copy_from_slice(&bytes[..len]);
    Option::from(Scalar::from_repr(repr)).ok_or_else(|| format!("invalid field element: {}", s))
}

pub fn scalar_display(s: &Scalar) -> String {
    let bytes = s.to_repr();
    let n = num_bigint::BigUint::from_bytes_le(bytes.as_ref());
    n.to_string()
}

#[derive(Debug, Clone)]
pub struct ConstraintSystem {
    pub num_public: usize,
    pub num_private: usize,
    pub num_variables: usize,
    pub num_constraints: usize,
    pub a: Vec<Vec<(usize, Scalar)>>,
    pub b: Vec<Vec<(usize, Scalar)>>,
    pub c: Vec<Vec<(usize, Scalar)>>,
    pub witness: Vec<Scalar>,
    /// Plonkish-first constraint kinds produced by the compiler. The `R1CS`
    /// variant mirrors the legacy `a/b/c` triples; `Plonkish/CustomGate/Lookup/
    /// Range` are the forward targets for Plonk/Nova/folding backends.
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct R1cs {
    pub a: Vec<Vec<(usize, Scalar)>>,
    pub b: Vec<Vec<(usize, Scalar)>>,
    pub c: Vec<Vec<(usize, Scalar)>>,
}

/// A linear combination over witness indices with scalar coefficients.
pub type LinComb = Vec<(usize, Scalar)>;

#[derive(Debug, Clone)]
pub struct R1CSTriple {
    pub a: LinComb,
    pub b: LinComb,
    pub c: LinComb,
}

#[derive(Debug, Clone)]
pub struct PlonkishTriple {
    pub a: LinComb,
    pub b: LinComb,
    pub c: LinComb,
    /// Scalar/rotation lookups attached to the row.
    pub lookups: Vec<(String, LinComb, LinComb)>,
}

/// Multi-shape constraint kind. R1CS stays for compatibility with the legacy
/// lowering; Plonkish / CustomGate / Lookup / Range are the forward targets
/// for Plonk, Nova and folding backends.
#[derive(Debug, Clone)]
pub enum Constraint {
    Plonkish(PlonkishTriple),
    R1CS(R1CSTriple),
    CustomGate {
        name: String,
        args: Vec<LinComb>,
    },
    Lookup {
        table: String,
        key: LinComb,
        val: LinComb,
    },
    Range {
        var: usize,
        bits: usize,
    },
}
