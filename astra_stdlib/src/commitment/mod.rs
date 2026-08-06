//! Commitment gadgets.
//!
//! Implemented: Merkle trees over the BLS12-381 scalar field (see
//! [`merkle`]). Planned: Pedersen commitments.

pub mod merkle;
pub mod pedersen;
