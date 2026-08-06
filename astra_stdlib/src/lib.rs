//! `astra_stdlib` — standard gadget library for Zara.
//!
//! Structural skeleton. Gadgets land here as versioned, dependency-free
//! modules so other Rust crates can `cargo add astra_stdlib` and use them
//! from their own programs — the moment that works, Astra leaves the
//! single-binary world.

pub mod commitment;
pub mod hash;
pub mod primitive;
pub mod signature;
pub mod snark;
