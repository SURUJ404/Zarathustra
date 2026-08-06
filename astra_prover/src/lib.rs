//! `astra_prover` — proof-system registry.
//!
//! The active path is the ark-groth16 binding in [`groth16`]. The hand-rolled
//! implementation lives in [`legacy`] and is kept only for reference until it
//! is removed entirely.

#[doc(hidden)]
pub mod legacy;

pub mod backend;
pub mod groth16;

pub use groth16::{DefaultCurve, Groth16, Proof, ProvingKey, VerifyingKey};
