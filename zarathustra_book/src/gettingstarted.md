# Getting Started

## Installation

### Online IDEs

To get a feel of the language, try the [Zarathustra playgound](https://play.zokrat.es).

To experiment with creating SNARKs and verifying them in the EVM, check out the Zarathustra plugin in the [Remix online IDE](https://remix.ethereum.org).


### One-line installation

We provide one-line installation for Linux, MacOS and FreeBSD:

```bash
curl -LSfs get.zokrat.es | sh
```

### From source

You can build Zarathustra from [source](https://github.com/ZoKrates/ZoKrates/) with the following commands:

```bash
git clone https://github.com/ZoKrates/ZoKrates
cd Zarathustra
export ZARATHUSTRA_STDLIB=$PWD/zarathustra_stdlib/stdlib
cargo build -p zarathustra_cli --release
cd target/release
```

### Docker

Zarathustra is available on Dockerhub.

```bash
docker run -ti zarathustra/zarathustra /bin/bash
```

From there on, you can use the `zarathustra` CLI.

## Hello Zarathustra!

First, create the text-file `root.zok` and implement your program. In this example, we will prove knowledge of the square root `a` of a number `b`:

```zarathustra
{{#include ../../zarathustra_cli/examples/book/factorize.zok}}
```

Some observations:
- The keyword `field` is the basic type we use, which is an element of a given prime field.
- The keyword `private` signals that we do not want to reveal this input, but still prove that we know its value.

Then run the different phases of the protocol:

```bash
# compile
zarathustra compile -i root.zok
# perform the setup phase
zarathustra setup
# execute the program
zarathustra compute-witness -a 337 113569
# generate a proof of computation
zarathustra generate-proof
# export a solidity verifier
zarathustra export-verifier
# or verify natively
zarathustra verify
```

The CLI commands are explained in more detail in the [CLI reference](toolbox/cli.md).
