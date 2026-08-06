# Astra CLI Reference

The `astra` binary is the zkSNARK toolkit command line interface. All proof
pipeline commands operate on Zara circuit source files (`.zara`) and write
their artifacts to the **current working directory**.

Every subcommand accepts `--help` (`-h`) for its exact usage.

---

## Top-level

```console
astra [COMMAND] [OPTIONS]
```

Running `astra` with no arguments prints the subcommand list:

```
astra 0.1.0 - zkSNARK toolkit
Subcommands: prove, audit, deploy, publish
Use 'astra <cmd> --help' for details
```

**Subcommands**

| Command | Description |
| --- | --- |
| `prove` | Full proof pipeline: `compile`, `setup`, `prove`, `verify`. |
| `audit` | Static security audit of circuits & smart contracts. |
| `deploy` | Circuit project scaffolding + verifier export. |
| `publish` | Export verifiers for target platforms. |

---

## Prove pipeline

### `astra prove compile [OPTIONS] <input>`

Compile a circuit source file into R1CS and evaluate the witness.

| Option | Description |
| --- | --- |
| `<input>` | Source file (`.zara`). |
| `-p, --public <inputs>` | Comma-separated public inputs. |
| `-r, --private <inputs>` | Comma-separated private inputs. |

Prints the circuit summary (variables, public/private counts, constraints) and
the witness. Advances without writing artifacts — later steps re-derive the
constraint system.

### `astra prove setup [OPTIONS] <input>`

Generate the trusted setup (CRS) for the circuit.

| Option | Description |
| --- | --- |
| `<input>` | Source file. |
| `-p, --public <inputs>` | Comma-separated public inputs. |
| `-r, --private <inputs>` | Comma-separated private inputs. |
| `--backend <name>` | Proof system: `ark-groth16` (default) or `legacy`. |

Writes **`pk.json`** (proving key) and **`vk.json`** (verifying key).

```bash
astra prove setup -p 8 -r 3,5 circuit.zara
```

### astra prove prove [OPTIONS] <input>

Generate a zkSNARK proof for the circuit.

| Option | Description |
| --- | --- |
| `<input>` | Source file. |
| `-p, --public <inputs>` | Comma-separated public inputs. |
| `-r, --private <inputs>` | Comma-separated private inputs. |
| `--backend <backend>` | Proof system: `ark-groth16` (default) or `legacy`. |

Writes **`proof.json`** (the groth16 proof `A`/`B`/`C` plus the bound public
inputs). Prints the point coordinates and the save path.

```bash
astra prove prove -p 9 -r 1 circuit.zara
```

### astra prove verify [OPTIONS] [input]

Verify a proof against the public inputs bound in `proof.json`.

| Option | Description |
| --- | --- |
| `[input]` | Source file (kept for CLI compatibility; verify reads `proof.json`). |
| `-p, --public` | Comma-separated public inputs (validated against the bound ones). |
| `-r, --private` | Private inputs (accepted for symmetry). |
| `--backend <backend>` | Proof system: `ark-groth16` (default) or `legacy`. |

Reads `proof.json` and `vk.json`, re-derives the statement from the bound
public inputs, and returns:

- `✓ PROOF VERIFIED` on success (exit `0`).
- `✗ PROOF REJECTED` or a decode error, exit `1` — for tampered proofs,
  mismatched keys, or missing files.
- Warnings if the provided `-p` inputs differ from those bound in the proof
  (it always verifies against the bound inputs).

Requires `proof.json` — run `astra prove prove` (with the same source) first.

---

## Audit

### astra audit [FLAGS] [OPTIONS] [target]

Static security audit for circuits and smart contracts.

| Flag / Option | Description |
| --- | --- |
| `[target]` | File or directory to scan (default `.`). |
| `-v, --verbose` | Verbose output. |
| `-f, --format <format>` | `terminal` \| `json` \| `html` (default `terminal`). |
| `-o, --output <path>` | Write report to this file. |
| `--deny <severity>` | `critical` \| `high` \| `medium` \| `any` — exit non-zero if findings at/above this severity exist. |

Report IDs: `CIR-001…006` (circuits), `SOL-001…006` (Solidity).

```bash
astra audit .
astra audit --format=json --output=audit.json .
astra audit --deny=high .
```

---

## Deployment

### `astra deploy init <name>`

Scaffold a new circuit project.

```bash
astra deploy init mpc-scheme
```

Creates `<name>/src/main.zara` with a starter circuit:

```zara
def main(field a, field b) -> field {
    field c = a * b;
    return c;
}
```

### `astra deploy verifier`

Export a Solidity verifier. Currently honest-by-default: it refuses to
produce a real per-circuit contract and errors out with guidance to use
`astra publish -t evm` once implemented (rather than emitting a stub that
always returns `true`).

### `astra deploy verify`

Simulated on-chain verification. Not yet implemented.

---

## Publishing

### `astra publish [OPTIONS]`

Export verifiers / test vectors for target platforms.

| Option | Description |
| --- | --- |
| `-t, --target <target>` | `evm` \| `starknet` \| `wasm` \| `aleo` \| `jt` (default `evm`). |

- `jt` reads a valid `proof.json` + `vk.json` (from `astra prove setup`/prove)
  and writes **`zara_test_vectors.json`** with real, verified artifacts
  (`version`, `protocol`, `curve`, `public_inputs`, `proof`, `verifying_key`).
- `evm` is honest: BLS12-381 has no Ethereum pairing precompile, so an EVM
  verifier is only meaningful for BN254; the target currently rejects
  BLS12-381 rather than emitting an invalid contract.
- `starknet`, `wasm`, `aleo` are explicit "not implemented".

```bash
astra prove setup  -p 3 -r 7 circuit.zara
astra prove prove  -p 3 -r 7 circuit.zara
astra publish -t jt    # → zara_test_vectors.json
```

---

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success (including `✓ PROOF VERIFIED`). |
| `1` | Error — invalid input/flag, compile failure, unsatisfied circuit, tampered/missing proof, or verification rejected. |

## Notes

- Default backend is **ark-groth16** over `ark_bls12_381`; `--backend legacy`
  selects the hand-rolled reference backend.
- `verify` is honest: it loads `vk.json` + `proof.json` and verifies the bound
  public inputs — tampering either file is rejected.
- The npm distribution wrapper (`@zarathustra/cli`) invokes this same binary;
  see `npm/astra/README.md`.