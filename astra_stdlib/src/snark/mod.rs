//! Recursive-SNARK primitives.
//!
//! Planned: in-circuit `verify_groth16` / `verify_plonk` so circuits can
//! verify other proofs (recursion / IVC with folding backends like Nova).

pub mod recursion;
