pub mod ir;
pub mod parser;
pub mod compiler;
pub mod groth16;

pub use ir::*;
pub use compiler::compile;
pub use groth16::{setup, prove, verify, ProvingKey, VerifyingKey, Proof};
