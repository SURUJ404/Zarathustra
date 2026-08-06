# @zarathustra/cli

npm distribution wrapper for the Zarathustra `astra` zkSNARK CLI (Rust binary).

## Usage

```sh
npx @zarathustra/cli prove compile circuit.astra
npx @zarathustra/cli prove prove -p 1 -r 2 circuit.astra
npx @zarathustra/cli audit .
```

Or install globally:

```sh
npm i -g @zarathustra/cli
astra prove compile circuit.astra
```

## How it works

- `bin/astra.js` spawns the native `astra` binary, forwarding exit codes and stdio.
- The binary is resolved from the matching platform package
  (`@zarathustra/cli-win32-x64`, `cli-darwin-arm64`, `cli-linux-x64`), falling back
  to `npm/astra/astra` when running from the repo.
- `install.js` (postinstall) copies a prebuilt `target/release/astra`, or builds it
  from source with `cargo build --release --bin astra`.

## Building the platform binaries

From the repo root:

```sh
cargo build --release --bin astra
npm run build:cli
```

The resulting `astra`/`astra.exe` is placed in each platform package's `bin/`
and in `npm/astra/`.

## Publishing

The platform packages (`@zarathustra/cli-*`) and the wrapper must be published
in lockstep at the same version. Replace the checked-in binaries in
`npm/<platform>/bin/` with binaries built for each target before publishing.
