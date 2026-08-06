//! `astra_codegen` — lowering of Zara programs to R1CS + witness generation.
//!
//! Also hosts the [`Backend`] abstraction so proof systems (Groth16, Marlin,
//! Nova) can plug in behind a single interface. Backends communicate through
//! versioned JSON files (`pk.json`, `vk.json`, `proof.json`) written into a
//! working directory, which keeps the interface concrete enough to box behind
//! `dyn Backend` (no generics) while still allowing honest end-to-end flows.

pub mod compiler;

pub use compiler::{compile, validate_constraints};

use astra_ir::types::ConstraintSystem;
use std::path::{Path, PathBuf};

/// Proof-system abstraction.
///
/// Concrete implementations live in `astra_prover::backend` (ark-groth16 is
/// the default; the hand-rolled Groth16 is available behind `--backend
/// legacy`). Marlin/Nova implementers can add new entries to that registry
/// without touching the CLI.
pub trait Backend: Send + Sync {
    fn name(&self) -> &'static str;
    fn curve(&self) -> &'static str;

    /// Run trusted setup for `cs`, persisting `pk.json`/`vk.json` into `dir`.
    fn setup(&self, cs: &ConstraintSystem, dir: &Path) -> Result<(), String>;

    /// Produce a proof for `cs` (running setup first if the proving key is
    /// absent) and persist it to [`Self::proof_path`] with the public inputs
    /// bound in the circuit witness.
    fn prove(&self, cs: &ConstraintSystem, dir: &Path) -> Result<(), String>;

    /// Honestly verify the proof at [`Self::proof_path`] against the public
    /// inputs bound inside it. Returns `Ok(true)` for valid proofs,
    /// `Ok(false)` for invalid ones, and `Err` on a genuine error.
    fn verify(&self, dir: &Path) -> Result<bool, String>;

    fn proof_path(&self, dir: &Path) -> PathBuf {
        dir.join("proof.json")
    }
}
