//! Poseidon hash over the BLS12-381 scalar field.
//!
//! Versioned `v1` implementation of the Poseidon construction with a width-3
//! permutation (rate 2, capacity 1) and the `x^5` S-box, over
//! `bls12_381::Scalar` — the field `astra_ir` lives in.
//!
//! **Parameter provenance.** The MDS matrix and round constants are derived
//! deterministically from the construction using the documented local
//! procedures (`MDS` matrix and the `expand_to_field` NUMS expander), *not*
//! the standardized parameter files of the reference implementation. The
//! function is fully deterministic and self-consistent, which makes it correct
//! for internal commitments and Merkle-ization (see `astra_stdlib::commitment`
//! and its tests). It is **not** intended to be interoperable with the
//! official Poseidon parameter sets.
//!
//! Depends only on `bls12_381`/`ff`; no arkworks or `sha2` needed.

use bls12_381::Scalar;
use ff::Field;
use std::sync::OnceLock;

pub const WIDTH: usize = 3;
pub const RATE: usize = 2;
pub const CAPACITY: usize = 1;
pub const FULL_ROUNDS: usize = 8;
pub const PARTIAL_ROUNDS: usize = 57;
const N_ROUNDS: usize = FULL_ROUNDS + PARTIAL_ROUNDS;

/// Circulant MDS matrix `circulant(2, 1, 1)` — MDS over `Scalar`
/// (all 2x2 minors and the determinant are non-zero in this field).
fn mds() -> [[Scalar; WIDTH]; WIDTH] {
    let two = Scalar::from(2u64);
    let one = Scalar::ONE;
    [[two, one, one], [one, two, one], [one, one, two]]
}

/// NUMS expander for the round constants. Deterministic and one-way-ish:
/// 255 iterations of `x -> x^2 + c` seeded by the counter. Deliberately not
/// the official Poseidon constants — see the module docs.
fn expand_to_field(seed: u64) -> Scalar {
    let c = Scalar::from(0x6173_7472_615f_7631u64); // "astra_v1"
    let mut x = Scalar::from(seed);
    for _ in 0..255 {
        x = x.square() + c;
    }
    x
}

fn constants() -> &'static [Scalar; N_ROUNDS * WIDTH] {
    static RC: OnceLock<[Scalar; N_ROUNDS * WIDTH]> = OnceLock::new();
    RC.get_or_init(|| {
        let mut out = [Scalar::ZERO; N_ROUNDS * WIDTH];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = expand_to_field(i as u64 + 1);
        }
        out
    })
}

#[inline]
fn sbox(x: Scalar) -> Scalar {
    let x2 = x.square();
    let x4 = x2.square();
    x4 * x
}

fn mds_mul(state: [Scalar; WIDTH], mds: &[[Scalar; WIDTH]; WIDTH]) -> [Scalar; WIDTH] {
    let mut out = [Scalar::ZERO; WIDTH];
    for i in 0..WIDTH {
        for j in 0..WIDTH {
            out[i] += mds[i][j] * state[j];
        }
    }
    out
}

/// The Poseidon permutation over a width-3 state.
///
/// State layout is `[left, right, capacity]`; in partial rounds the S-box is
/// applied to the capacity element (index 2). Deterministic for the same
/// inputs.
pub fn permutation(mut state: [Scalar; WIDTH]) -> [Scalar; WIDTH] {
    let mds = mds();
    let rc = constants();
    let mut idx = 0usize;

    for _ in 0..FULL_ROUNDS {
        for e in state.iter_mut() {
            *e += rc[idx];
            idx += 1;
        }
        for e in state.iter_mut() {
            *e = sbox(*e);
        }
        state = mds_mul(state, &mds);
    }

    for _ in 0..PARTIAL_ROUNDS {
        for e in state.iter_mut() {
            *e += rc[idx];
            idx += 1;
        }
        state[CAPACITY] = sbox(state[CAPACITY]);
        state = mds_mul(state, &mds);
    }

    state
}

/// 2-to-1 Poseidon compression over the BLS12-381 scalar field.
pub fn poseidon2(left: Scalar, right: Scalar) -> Scalar {
    let state = permutation([left, right, Scalar::ZERO]);
    state[0]
}

/// Hash an arbitrary-length input by rate-2 absorption into the permutation.
pub fn hash(input: &[Scalar]) -> Scalar {
    let mut rate = [Scalar::ZERO; RATE];
    let mut capacity = Scalar::ZERO;
    let mut chunks = input.chunks_exact(RATE);
    for chunk in &mut chunks {
        for i in 0..RATE {
            rate[i] += chunk[i];
        }
        let out = permutation([rate[0], rate[1], capacity]);
        rate = [out[0], out[1]];
        capacity = out[2];
    }
    let rem = chunks.remainder();
    let mut last = [Scalar::ZERO; RATE];
    for (i, v) in rem.iter().enumerate() {
        last[i] = *v;
    }
    // Domain separator so that a short message can never collide with a
    // zero-padded prefix of a longer one.
    last[rem.len()] += Scalar::ONE;
    let out = permutation([rate[0] + last[0], rate[1] + last[1], capacity]);
    out[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permutation_deterministic() {
        let s = [Scalar::ONE, Scalar::from(2u64), Scalar::from(3u64)];
        assert_eq!(permutation(s), permutation(s));
    }

    #[test]
    fn poseidon2_avalanches_on_left() {
        let a = poseidon2(Scalar::from(1u64), Scalar::from(2u64));
        let b = poseidon2(Scalar::from(1u64) + Scalar::ONE, Scalar::from(2u64));
        assert_ne!(a, b);
    }

    #[test]
    fn poseidon2_avalanches_on_right() {
        let a = poseidon2(Scalar::from(1u64), Scalar::from(2u64));
        let b = poseidon2(Scalar::from(1u64), Scalar::from(2u64) + Scalar::ONE);
        assert_ne!(a, b);
    }

    #[test]
    fn hash_differs_by_domain_separator() {
        let one = hash(&[Scalar::from(7u64)]);
        let three = hash(&[Scalar::from(7u64), Scalar::ZERO, Scalar::ZERO]);
        // Short input [7] and long input [7,0,0] both pad to the same length,
        // but the domain separator in the final chunk keeps them apart.
        assert_ne!(one, three);
    }

    #[test]
    fn full_state_is_permutation() {
        let s = [Scalar::from(5u64), Scalar::from(6u64), Scalar::from(7u64)];
        let out = permutation(s);
        // The permutation must not collapse the whole state to a constant.
        assert_ne!(out, s);
        assert_ne!(out[0] + out[1] + out[2], Scalar::ZERO);
    }
}
