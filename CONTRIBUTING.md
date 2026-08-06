# Contributing to Zarathustra

Thanks for your interest! This project is a day-1 archetype architected to
overtake ZoKrates in usability, backend coverage, and developer tooling.

## Code of Conduct

Read and follow `CODE_OF_CONDUCT.md`.

## Getting started

```console
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

All four gates run in CI. Keep them green locally before opening a PR.

## Workspace map

The repo is an intentional 8-crate split so that each layer has a narrow,
security-relevant surface (`ARCHITECTURE.md` has the full picture):

| Layer | Crate | Craft rule |
| --- | --- | --- |
| IR | `astra_ir` | **No proof-system crypto** allowed |
| Frontend | `astra_frontend` | Parser/errors/LSP/WASM only |
| Lowering | `astra_codegen` | Zara → R1CS; `Backend` trait |
| Prover | `astra_prover` | Proof backends (ark-groth16 default) |
| Stdlib | `astra_stdlib` | Versioned gadgets, dependency-free |
| Publish | `astra_publish` | Verifier export targets |
| Security | `astra_security` | Static analyzers |
| CLI | `astra_cli` | Thin binary over the layers |

## Contribution workflow

1. Fork and create a branch (`docs/…`, `feat/…`, `fix/…`).
2. Make the change; add or update tests where behaviour changes.
3. Run the four gates above.
4. Open a PR against `main` using the template.
5. Keep PRs small and focused. Prefer several small PRs to one large one.

## Finding issues

Use the issue templates. For security reports, see `SECURITY.md`.

## Commit style

- Imperative subject line (`add`, `fix`, `refactor`, `docs:`).
- A short summary of *why* in the body when non-obvious.
- Follow the existing history: small, atomic, well-described commits.