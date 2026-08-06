# Security Policy

## Reporting a vulnerability

Please **do not open a public issue** for security problems.

Report privately to the maintainers by opening a GitHub Security Advisory
(repository → *Security* → *Report a vulnerability*). If you cannot use the
advisory flow, email the address listed in the project settings.

We ask for:

- a description of the issue,
- the affected versions / commit,
- a minimal reproducer (circuit, input, or snippet),
- your suggested impact and (if known) fix.

## What we care about

Astra generates proofs and verifiers; soundness bugs are critical. This
includes, but is not limited to:

- **Unconstrained inputs** — a public input that no constraint depends on
  can be chosen freely by a prover.
- **Missing or weak range checks** — field element overflow / wrap-around.
- **Verifier bugs** — wrong public input binding, nonce/nullifier misuse,
  reentrancy in `verifyProof`.
- **Invalid proof acceptance** — a verifier accepting a malformed proof.

## Response

We aim to acknowledge reports within 72 hours and to ship a fix (or a
coordinated disclosure plan) for confirmed critical issues promptly.

## Self-audit

The repo ships `astra audit`, a static analyzer that flags many of the
classes above (`CIR-*`, `SOL-*`, signal-flow, CEI). Run it before release:

```console
astra audit --deny high .
```

## Supported versions

Security fixes land on `main` and are backported to the latest tagged
release when practical. Pre-1.0 releases receive fixes on `main` only.
