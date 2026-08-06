# Astra — Improvement & Maintenance Report

Status as of **v0.1.0** (day-1 archetype). Verified by: full feature walkthrough
of the release binary, CI runs, and a code audit of the workspace.

Severity scale: **CRITICAL** (correctness/security, must fix first) ·
**HIGH** (feature-parity blockers) · **MEDIUM** (robustness / risk) ·
**LOW** (polish / ops).

## Resolution status (PR "10/10" pass)

- **C-1, C-2, C-3, C-4 — DONE.** The CLI now defaults to the ark-groth16
  backend (`astra_prover::groth16` over `ark_bls12_381`) with `--backend
  legacy` kept as a differential reference. Setup persists `pk.json`/`vk.json`;
  `prove` persists a versioned `proof.json` (curve, protocol, hex `a/b/c`,
  bound `public`, canonical `raw`); `verify` loads and honestly verifies the
  stored proof against its bound public inputs (tampered proof/key ⇒ reject,
  exit 1). Input parsing propagates errors instead of coercing to zero.
- **H-1, H-2 — DONE.** The compiler emits `Constraint::R1CS`, both backends
  consume the enum, and `astra_prover::backend` is a real registry
  (`ark-groth16` + `legacy`) behind `astra prove --backend <name>`.
- **H-3 — PARTIAL.** `publish -t jt` writes real, machine-parseable JSON test
  vectors from the proof/vk artifacts. EVM refuses honestly (BLS12-381 has no
  Ethereum pairing precompile; a BN254 backend is required first). Starknet/
  wasm/aleo still return explicit "not implemented" errors.
- **H-4 — DONE.** Setup keys are persisted and verified against; `prove`
  auto-derives keys when absent so the web flow stays one-shot.
- **H-5 — PARTIAL.** `astra_stdlib` ships a real, tested Poseidon (width-3
  permutation + 2-to-1 + sponge, locally-derived constants, documented) and a
  Merkle tree with inclusion-proof verification over the BLS12-381 field.
  SHA-256/Pedersen/EdDSA/recursion remain scaffolding.

Remaining after this pass: H-3 EVM/Starknet/wasm/aleo export, H-5 rest, and the
MEDIUM/LOW items below.

---

## CRITICAL — correctness & security

### C-1. The CLI still runs hand-rolled Groth16, not ark
`astra_cli/src/ops/prove.rs:3` imports `astra_prover::legacy::{prove, setup,
verify}`. The community-audited ark-groth16 binding (`astra_prover/src/groth16.rs`)
is only exercised by unit tests. The production proof path is un-audited
hand-rolled crypto — the exact risk the ark migration was meant to remove.

**Fix:** default the CLI to the ark path (`to_ark_cs` → `Groth16` over
`ark_bls12_381`); keep `legacy` behind a `--legacy` flag as a differential
test reference. This also gives a real ProvingKey/VerifyingKey to serialize.

### C-2. "verify" does not verify — it re-proves and re-verifies the result
`run_verify` (`ops/prove.rs:108`) recompiles, calls `setup(&cs)` **again**, and
calls `prove(&pk, &cs)` to build a fresh proof, then checks it. It never reads
`proof.json` and never checks the user's witness against the original proof.
A "verify" that regenerates the proof is meaningless as a verifier.

**Fix:** `verify` must load the serialized proof + public inputs + the
published VerifyingKey (from setup), and verify *that* proof. Add a negative
test (tampered proof / wrong public input → reject).

### C-3. Proofs are `Debug`-formatted Rust strings
`ops/prove.rs:96-100` writes `proof.json` as `format!("{:?}", proof.a)` — e.g.
`"x: 0x0be3…, infinity: Choice(0)"`. Not machine-parsable, not interoperable,
contains no curve/field metadata, and cannot be consumed by the web verifier.

**Fix:** serialize via ark (`proof.serialize()` / hex) with a versioned JSON
schema: `{ "curve": "bls12-381", "protocol": "groth16", "a": […], "b": […], "c": […], "public": [...] }`.

### C-4. Silent input coercion to zero
`parse_inputs` (`ops/prove.rs:17-22`) does `.unwrap_or(Scalar::ZERO)`: a typo
like `-p 3x` silently becomes `0`. In a prover this is a footgun for soundness.

**Fix:** propagate parse errors and exit non-zero with the offending input.

---

## HIGH — feature parity / blockers

### H-1. Plonkish-first IR is declared, not used
`Constraint::{Plonkish, CustomGate, Lookup, Range}` (`astra_ir/src/types.rs`)
has no producers or consumers. `to_ark_cs` only reads the legacy `a/b/c`
triples. The "Plonkish-first" direction is just a type.

**Fix:** make `astra_codegen` emit `Constraint` (R1CS variant today, others
soon), and make backends consume the enum instead of the raw triples.

### H-2. `Backend` trait has zero implementations
`astra_codegen` declares the trait; nothing implements it. Without a
`Backend` registry, the "multiple proof systems" story is unimplemented.

**Fix:** implement `Backend` for ark-groth16 (and legacy behind a flag), add a
registry + `astra prove --backend <name>`.

### H-3. `publish` is a string printer
`astra_publish` targets (`evm/starknet/wasm/aleo/jt`) return
`"…not yet implemented in v0.0"`. `deploy verifier` deliberately refuses.
The flagship "export & deploy a verifier" feature is at 0%.

**Fix:** land **EVM first** (real Solidity verifier over the ark
`VerifyingKey`, with CEI-safe patterns from `astra_security::cei`), then wasm.

### H-4. Setup key material is ephemeral
`run_setup` computes `(pk, vk)` and **discards** them
(`let (_pk, _vk) = setup(&cs);`). There is no CRS file, no trusted-setup
flow, no vk distribution. Real use requires persistent, auditable keys.

**Fix:** write `vk`/`pk` to files (versioned), support `prove --vk`, and verify
against the published `vk`.

### H-5. Stdlib gadgets are empty shells
`astra_stdlib` (sha256, poseidon, pedersen, merkle, eddsa, recursion) is
module scaffolding with no implementations.

**Fix:** implement **poseidon + merkle first** (highest demand in zk apps);
versioned + dependency-free as designed.

---

## MEDIUM — robustness

- **M-1. Range checks / field semantics.** `CIR-002` fires "Missing Range
  Check" on every `.zara` file that uses `field` (regex heuristic, saw it live).
  Real range analysis needs integer-wrapping semantics at IR level.
- **M-2. Signal-flow is coarse.** `referenced_vars` treats *any* appearance as
  "constrained" (e.g. `assert(x == x)` counts x as constrained — that's the
  unsafe.zara false-negative we saw). Needs actual dataflow (does the input
  affect a *non-trivially* constrained output?).
- **M-3. Parser coverage.** No negative-grammar tests, no property/fuzz tests.
  `validate_constraints` (compiler.rs) duplicates the constraint check that
  ark's `is_satisfied` also does — keep both, but drive them from one source.
- **M-4. Test breadth.** ~a handful of tests; the ark-translation test was
  broken until this session. Add: differential tests legacy-vs-ark (same proof
  sets), tamper tests, wrong-input tests, round-trip serialize/deserialize.
- **M-5. `show_cs` hardcodes labels** `["~one","a","b","c"]`
  (`ops/prove.rs:36`) instead of using `compiler.var_names`. Cosmetic, but
  misleads for >3 inputs.
- **M-6. WASM not shipped.** Playground always falls back to the API because
  `web/public/pkg/` is never built. Add a CI wasm-pack step (once `astra_ir`
  deps are confirmed wasm32-clean) and serve `pkg/`.

---

## LOW — ops / maintenance

- **L-1.** Actions Node 20 deprecation warnings — bump `actions/*` majors.
- **L-2.** Add supply-chain scanning: `cargo-audit` (Rust deps) and
  `npm audit` (web deps) as CI jobs; pin and lock everything (`--locked` in
  `ci.yml` test step).
- **L-3.** `ci.yml` builds the workspace; the release workflow rebuilds
  everything per tag — consider `cargo cache` (swatinem) to cut the 3-platform
  build time.
- **L-4.** Repo hygiene deltas: add `CHANGELOG.md`, a `ROADMAP.md` with the
  priority order below, and a `docs/` index.
- **L-5.** Document the local-dev story for this machine: Smart App Control
  blocks unsigned Rust binaries → use WSL or the GitHub release binaries (as
  done in the feature walkthrough).

---

## Suggested priority order

1. **C-1 + C-2 + C-3 + C-4** (make the shipped proof path ark-based and
   honest: real setup keys, real verify of a real proof, real serialization).
2. **H-4** (persistent CRS/vk) then **H-3 EVM verifier** — the first
   externally valuable feature.
3. **H-1/H-2** (Constraint enum + Backend registry) — unlocks everything else.
4. **M-2** (real signal-flow) and **M-4** (test depth) — before scaling usage.
5. **H-5 poseidon/merkle**, **M-6 WASM**, then the remaining polish/ops items.

---

*Generated 2026-08-06. Items verified against the codebase and live runs; fix
the CRITICALs before any public/permissionless deployment of the prover.*
