# Naming, trademark & brand

This project ships two product names. Use them consistently in code, docs,
marketing, and packaging.

| Term | What it is | Used for |
| --- | --- | --- |
| **Astra** (`.astra` … the CLI `astra`) | The zkSNARK toolkit / engine | Binary, main crate, UI branding |
| **Zara** (`.zara`) | The circuit language | Source files, parser, LSP, highlighting |

## Repository naming

- The repository is **Zarathustra** (the over-arching project name).
- The binary is `astra` (see `astra_cli/Cargo.toml` → `[[bin]] name`).
- The frontend language is **Zara**.

## Conventions

- Crate names use the `astra_` prefix: `astra_ir`, `astra_frontend`,
  `astra_codegen`, `astra_prover`, `astra_stdlib`, `astra_publish`,
  `astra_security`, `astra_cli`.
- Zara source files use the `.zara` extension.
- Browser products brand as **Astra Playground**; the docs site also uses
  **Zarathustra**.

## Trademark note

"Astra" and "Zara" are descriptive product names in this project. If you
distribute or rebrand this software, review any trademarks that may apply
in your jurisdiction. This project does not assert trademarks over the
words themselves.