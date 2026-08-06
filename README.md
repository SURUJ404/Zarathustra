# Zarathustra — Astra zkSNARK toolkit

A zero-knowledge proof toolkit in Rust with a first-class language for
circuits, **Zara** (`.zara`). Astra's goal is broad feature parity with —
and eventually to overtake — [ZoKrates](https://github.com/Zokrates/ZoKrates):
friendly syntax, multiple proof backends, static security analysis, and
first-class deployment of verifiers.

**Status: day-1 archetype.** The workspace builds, the CLI runs a full
compile → setup → prove → verify pipeline via ark-groth16, and the security
analyzers run real checks. `cargo test` / `cargo clippy` are enforced in CI.

## Why Zara

```zara
// Prove you know x, y such that x + y == sum
def main(field public sum, field private x, field private y) -> field {
    assert(x + y == sum);
    return 1;
}
```

- **Readable.** Imperative, typed, `assert`-based constraints.
- **Structured errors.** Parse/compile diagnostics carry source spans.
- **Built for tooling.** An LSP hook (`DocumentService` trait) and
  browser-native WASM frontend ship in the repo.

## Quick start

```console
cargo build --workspace
astra prove compile    -p 8    -r 3,5 examples/sum.zara   # → R1CS + witness
astra prove setup      -p 8    -r 3,5 examples/sum.zara   # → CRS
astra prove prove      -p 8    -r 3,5 examples/sum.zara   # → proof.json
astra prove verify     -p 8    -r 3,5 examples/sum.zara   # → true
```

## CLI

The `astra` binary is the zkSNARK toolkit. Every build target drops artifacts in
the **current working directory** (`pk.json`, `vk.json`, `proof.json`). See
`docs/CLI.md` for the complete reference.

```console
# top-level
astra                          # usage summary
astra <cmd> --help             # per-command help
```

| Command | Purpose |
| --- | --- |
| `astra prove compile\|setup\|prove\|verify` | Full proof pipeline on a `.zara` file |
| `astra audit [-f json] [--deny high] [target]` | Static security scan (Zara, Rust, Solidity) |
| `astra deploy init\|verifier\|verify` | Circuit projects + verifier export |
| `astra publish -t evm\|starknet\|wasm\|aleo\|jt` | Verifier export targets |

### Proof pipeline (`astra prove`)

`compile` parses a Zara source file, lowers it to R1CS, evaluates the witness,
and prints the circuit summary. `setup` generates the CRS (`pk.json` +
`vk.json`). `prove` generates the proof (`proof.json`). `verify` validates
`proof.json` against the public inputs **bound into the proof** (exit code 0 /
1).

```console
astra prove compile -p 8 -r 3,5 circuit.zara   # circuit + witness
astra prove setup   -p 8 -r 3,5 circuit.zara   # → pk.json, vk.json
astra prove prove   -p 8 -r 3,5 circuit.zara   # → proof.json
astra prove verify  -p 8 -r 3,5 circuit.zara   # → ✓ PROOF VERIFIED
```

- `-p <n,n>` public inputs, `-r <n,n>` private (comma-separated).
- `--backend ark-groth16` (default) or `legacy`.
- `verify` refuses tampered proofs / mismatched keys with a non-zero exit.

### Security audit

```console
astra audit .                       # terminal report
astra audit --format json -o audit.json .
astra audit --deny high .           # exit non-zero on HIGH+ findings
```

Check IDs: `CIR-001…006` (circuits), `SOL-001…006` (Solidity). Runs a
**signal-flow analysis** that flags unconstrained public inputs (HIGH) and a
**CEI / reentrancy** check for Solidity verifiers. See `SECURITY.md`.

### Project scaffolding

```console
astra deploy init project       # creates project/src/main.zara
astra deploy verifier           # Solidity export (errors out for now)
astra deploy verify             # simulated on-chain verify (pending)
```

### Verifier publishing

```console
astra publish -t jt             # → zara_test_vectors.json (real proof+VK)
astra publish -t evm            # BLS12-381 refuses; BN254 verifier pending
```

Full option list and artifacts are documented in `docs/CLI.md`.

## Workspace

| Crate | Role |
| --- | --- |
| `astra_ir` | Core IR: AST, R1CS, Plonkish-first `Constraint` kinds. No proof crypto. |
| `astra_frontend` | Zara lexer/parser, span-aware errors, LSP hook, WASM bindings. |
| `astra_codegen` | Lowers Zara to R1CS + witnesses; `Backend` trait. |
| `astra_prover` | Proof-system registry: ark-groth16 (default) + hand-rolled reference. |
| `astra_stdlib` | Versioned gadget library (hash / signature / commitment / snark / primitive). |
| `astra_publish` | Verifier export: EVM, Starknet, WASM, Aleo, JSON test vectors. |
| `astra_security` | Static analyzers: signal flow, CEI/reentrancy, regex patterns. |
| `astra_cli` | The `astra` binary. |

## Playground

`web/public/playground.html` is a Monaco-based IDE. When the WASM bundle is
present (`web/public/pkg/`), **compile** runs natively in the browser with
no server; otherwise it falls back to the `/api/*` backend. Prove/verify
currently require the backend (Groth16 is not WASM-safe yet). Build the
WASM bundle with:

```console
wasm-pack build astra_frontend --target web --release --features wasm
```

## Development

```console
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

All four are enforced in `.github/workflows/ci.yml`. See
`CONTRIBUTING.md` and `ARCHITECTURE.md`.

## License

Apache-2.0. See `LICENSE`.
