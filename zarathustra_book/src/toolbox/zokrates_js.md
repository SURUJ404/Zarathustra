# zarathustra.js

JavaScript bindings for [Zarathustra](https://github.com/Zarathustra/Zarathustra).

```bash
npm install zarathustra-js
```

## Importing

#### ES modules

```js
import { initialize } from "zarathustra-js";
```

#### CommonJS

```js
let { initialize } = await import("zarathustra-js");
```

#### CDN

```html
<script src="https://unpkg.com/zarathustra-js@latest/umd.min.js"></script>
<script>
  zarathustra.initialize().then((zarathustraProvider) => {
    /* ... */
  });
</script>
```

## Example

```js
initialize().then((zarathustraProvider) => {
  const source = "def main(private field a) -> field { return a * a; }";

  // compilation
  const artifacts = zarathustraProvider.compile(source);

  // computation
  const { witness, output } = zarathustraProvider.computeWitness(artifacts, ["2"]);

  // run setup
  const keypair = zarathustraProvider.setup(artifacts.program);

  // generate proof
  const proof = zarathustraProvider.generateProof(
    artifacts.program,
    witness,
    keypair.pk
  );

  // export solidity verifier
  const verifier = zarathustraProvider.exportSolidityVerifier(keypair.vk);

  // or verify off-chain
  const isVerified = zarathustraProvider.verify(keypair.vk, proof);
});
```

## API

##### initialize()

Returns an initialized `ZarathustraProvider` as a promise.

```js
initialize().then((zarathustraProvider) => {
  // call api functions here
});
```

Returns: `Promise<ZarathustraProvider>`

##### withOptions(options)

Returns a `ZarathustraProvider` configured with given options.

```js
initialize().then((defaultProvider) => {
  let zarathustraProvider = defaultProvider.withOptions({
    backend: "ark",
    curve: "bls12_381",
    scheme: "g16",
  });
  // ...
});
```

Options:

- `backend` - Backend (options: `ark` | `bellman`, default: `ark`)
- `curve` - Elliptic curve (options: `bn128` | `bls12_381` | `bls12_377` | `bw6_761`, default: `bn128`)
- `scheme` - Proving scheme (options: `g16` | `gm17` | `marlin`, default: `g16`)

Returns: `ZarathustraProvider`

##### compile(source[, options])

Compiles source code into Zarathustra internal representation of arithmetic circuits.

Parameters:

- `source` - Source code to compile
- `options` - Compilation options

Returns: `CompilationArtifacts`

**Examples:**

Compilation:

```js
const artifacts = zarathustraProvider.compile("def main() { return; }");
```

Compilation with custom options:

```js
const source = "...";
const options = {
  location: "main.zok", // location of the root module
  resolveCallback: (currentLocation, importLocation) => {
    console.log(currentLocation + " is importing " + importLocation);
    return {
      source: "def main() { return; }",
      location: importLocation,
    };
  },
};
const artifacts = zarathustraProvider.compile(source, options);
```

**Note:** The `resolveCallback` function is used to resolve dependencies.
This callback receives the current module location and the import location of the module which is being imported.
The callback must synchronously return either an error, `null` or a valid `ResolverResult` object like shown in the example above.
A simple file system resolver in a node environment can be implemented as follows:

```js
import fs from "fs";
import path from "path";

const fileSystemResolver = (from, to) => {
  const location = path.resolve(path.dirname(path.resolve(from)), to);
  const source = fs.readFileSync(location).toString();
  return { source, location };
};
```

##### computeWitness(artifacts, args[, options])

Computes a valid assignment of the variables, which include the results of the computation.

Parameters:

- `artifacts` - Compilation artifacts
- `args` - Array of arguments (eg. `["1", "2", true]`)
- `options` - Computation options

Returns: `ComputationResult`

**Example:**

```js
const code = "def main(private field a) -> field { return a * a; }";
const artifacts = zarathustraProvider.compile(code);

const { witness, output } = zarathustraProvider.computeWitness(artifacts, ["2"]);

console.log(witness); // Resulting witness which can be used to generate a proof
console.log(output); // Computation output: "4"
```

##### setup(program[, entropy])

Generates a trusted setup for the compiled program.

Parameters:

- `program` - Compiled program
- `entropy` - User provided randomness (optional)

Returns: `SetupKeypair`

##### universalSetup(size[, entropy])

Performs the universal phase of a trusted setup. Only available for the `marlin` scheme.

Parameters:

- `size` - Size of the trusted setup passed as an exponent. For example, `8` for `2**8`.
- `entropy` - User provided randomness (optional)

Returns: `Uint8Array`

##### setupWithSrs(srs, program)

Generates a trusted setup with universal public parameters for the compiled program. Only available for `marlin` scheme.

Parameters:

- `srs` - Universal public parameters from the universal setup phase
- `program` - Compiled program

Returns: `SetupKeypair`

##### generateProof(program, witness, provingKey[, entropy])

Generates a proof for a computation of the compiled program.

Parameters:

- `program` - Compiled program
- `witness` - Witness (valid assignment of the variables) from the computation result
- `provingKey` - Proving key from the setup keypair
- `entropy` - User provided randomness (optional)

Returns: `Proof`

##### verify(verificationKey, proof)

Verifies the generated proof.

Parameters:

- `verificationKey` - Verification key from the setup keypair
- `proof` - Generated proof

Returns: `boolean`

##### exportSolidityVerifier(verificationKey)

Generates a Solidity contract which contains the generated verification key and a public function to verify proofs of computation of the compiled program.

Parameters:

- `verificationKey` - Verification key from the setup keypair

Returns: `string`

##### utils.formatProof(proof)

Formats the proof into an array of field elements that are compatible as input to the generated solidity contract

Parameters:

- `proof` - Generated proof

Returns: `array`
