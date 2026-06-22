#!/bin/bash
set -e

bin=$1; stdlib=$2

function zarathustra() {
  ZARATHUSTRA_STDLIB=$stdlib $bin "$@"
}

zarathustra compile -i get_hash.zok -o get_hash && zarathustra inspect -i get_hash
zarathustra compute-witness --verbose -i get_hash -a 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15

mkdir alice bob

cp reveal_bit.zok bob/reveal_bit.zok
cp reveal_bit.zok alice/reveal_bit.zok

cd bob

zarathustra compile -i reveal_bit.zok -o reveal_bit
zarathustra setup -i reveal_bit

cd ..
cp bob/proving.key alice/proving.key

cd alice
zarathustra compile -i reveal_bit.zok -o reveal_bit
zarathustra compute-witness --verbose -i reveal_bit -a 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 510
zarathustra generate-proof -i reveal_bit

cd ..
cp alice/proof.json bob/proof.json

cd bob
zarathustra verify
zarathustra export-verifier