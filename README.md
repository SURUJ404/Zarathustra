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

| Command | Purpose |
| --- | --- |
| `astra prove compile\|setup\|prove\|verify` | Full proof pipeline on a `.zara` file |
| `astra audit [-f json] [--deny high] [target]` | Static security scan (Zara, Rust, Solidity) |
| `astra deploy init\|verifier\|verify` | Circuit projects + verifier export |
| `astra publish -t evm\|starknet\|wasm\|aleo\|jt` | Verifier export targets (day-1 skeleton) |

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

## Security auditing

```console
astra audit .                       # terminal report
astra audit --format json -o audit.json .
astra audit --deny high .           # exit non-zero on HIGH+ findings
```

Check IDs: `CIR-001…006` (circuits), `SOL-001…006` (Solidity). Beyond
pattern matching, `astra_security` runs a **signal-flow analysis** that
flags unconstrained public inputs (HIGH) and a **CEI / reentrancy** check
for Solidity verifiers. See `SECURITY.md`.

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
