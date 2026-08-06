//! `astra_codegen` — lowering of Zara programs to R1CS + witness generation.
//!
//! Also hosts the [`Backend`] abstraction so proof systems (Groth16, Marlin,
//! Nova) can plug in behind a single interface.

pub mod compiler;

pub use compiler::{compile, validate_constraints};

use astra_ir::types::ConstraintSystem;
use bls12_381::Scalar;

/// Proof-system abstraction.
///
/// Structural skeleton: type signatures only. Concrete implementations land in
/// `astra_prover` (ark-groth16 today, Marlin/Nova next).
pub trait Backend {
    type Proof;
    type Vk;

    fn prove(&self, cs: &ConstraintSystem, witness: &[Scalar]) -> Result<Self::Proof, String>;
    fn verify(&self, vk: &Self::Vk, public: &[Scalar], proof: &Self::Proof) -> bool;
}
