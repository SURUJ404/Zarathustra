# Architecture

Astra is split into a layered Rust workspace so each concern has a narrow,
auditable boundary and a single source of truth for its data model.

## Layering

```
  .zara source
     │  parse
     ▼
astra_frontend ──► astra_ir (AST / Program)         ◄─ document-service (LSP), WASM bindings
     │  lower
     ▼
astra_codegen  ──► R1CS + witness  (Constraint / ConstraintSystem)
     │
     ├──► astra_prover  ──► ark-groth16 (default) / hand-rolled (reference)
     ├──► astra_publish ──► VerifierTarget: evm, starknet, wasm, aleo, jt
     └──► astra_security──► analyzers (signal-flow, CEI/reentrancy, regex)
astra_stdlib::gadgets  ◄ versioned, dependency-free, consumed by codegen
```

## Boundaries

- **`astra_ir`** is the shared vocabulary: `Expr`, `Statement`, `Program`,
  plus a **Plonkish-first** constraint model:
  `Constraint::{Plonkish, R1cs, CustomGate, Lookup, Range}`. It must stay
  free of any proof-system crypto so the IR is portable across backends.
- **`astra_frontend`** owns the grammar, source-span errors, the
  `DocumentService` hook (hover / definition / references / diagnostics),
  and the browser-native WASM surface (`wasm` feature, `cdylib`).
- **`astra_codegen`** lowers Zara → R1CS/witness and defines a `Backend`
  trait so proof systems plug in uniformly.
- **`astra_prover`** registers proof systems. ark-groth16 is the default
  binding; the original hand-rolled Groth16 is kept (`legacy.rs`) as an
  audit reference and to catch regressions in the ark translation.
- **`astra_publish`** exposes a `VerifierTarget` with EVM / Starknet /
  WASM / Aleo / JSON-test-vector targets (day-1 skeletons that return
  `Err("… not yet implemented in v0.0")` rather than emit a fake verifier).
- **`astra_security`** runs deterministic static analysis:
  - signal-flow → unconstrained public input (HIGH),
  - Solidity CEI / reentrancy,
  - `CIR-001…006` and `SOL-001…006` regex patterns.
- **`astra_cli`** is a thin binary (`clap`) over the layers.

## data-flow

- Circuit: `parse → lower → R1CS → (setup → prove → verify)`.
- Audit: `parse + regex → findings → terminal/json/html`.
- Playground: WASM compile in-browser when present, else `/api/*` backend;
  prove/verify require the backend until Groth16 is WASM-safe.

## Adding a proof backend

1. Implement the `Backend` trait in `astra_codegen` (or the prover's
   backend registry).
2. Add its verifier export to a `VerifierTarget` in `astra_publish`.
3. Register it under `astra publish -t <name>`.
4. Keep the language-layer crates (`astra_ir`, `astra_frontend`) untouched.