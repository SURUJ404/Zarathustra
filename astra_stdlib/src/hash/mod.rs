//! Hash gadgets.
//!
//! Implemented: Poseidon (width-3 permutation + 2-to-1 compression + sponge,
//! see [`poseidon`]). Planned: SHA-256.

pub mod poseidon;
pub mod sha256;
