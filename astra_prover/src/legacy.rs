//! Hand-rolled Groth16 — kept for reference/education only.
//!
//! Superseded by the ark-groth16 binding in [`crate::groth16`]. Types are
//! prefixed `Legacy` so they don't clash with the ark re-exports.

use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
use ff::Field;
use group::{prime::PrimeCurveAffine, Curve};
use rand::thread_rng;

use astra_ir::types::ConstraintSystem;

#[derive(Debug, Clone)]
pub struct LegacyProvingKey {
    pub alpha_g1: G1Affine,
    pub beta_g1: G1Affine,
    pub delta_g1: G1Affine,
    pub beta_g2: G2Affine,
    pub delta_g2: G2Affine,
    pub gamma_g2: G2Affine,
    pub tau_powers_g1: Vec<G1Affine>,
    pub tau_powers_g2: Vec<G2Affine>,
    pub abc_public_g1: Vec<G1Affine>,
    pub abc_private_g1: Vec<G1Affine>,
    pub h_crs_g1: Vec<G1Affine>,
    pub l: usize,
    pub m: usize,
    pub n: usize,
    pub u_coeffs: Vec<Vec<Scalar>>,
    pub v_coeffs: Vec<Vec<Scalar>>,
    pub w_coeffs: Vec<Vec<Scalar>>,
    pub z_coeffs: Vec<Scalar>,
}

#[derive(Debug, Clone)]
pub struct LegacyVerifyingKey {
    pub alpha_g1: G1Affine,
    pub beta_g2: G2Affine,
    pub gamma_g2: G2Affine,
    pub delta_g2: G2Affine,
    pub ic: Vec<G1Affine>,
}

#[derive(Debug, Clone)]
pub struct LegacyProof {
    pub a: G1Affine,
    pub b: G2Affine,
    pub c: G1Affine,
}

fn lagrange(points: &[(Scalar, Scalar)]) -> Vec<Scalar> {
    let n = points.len();
    let mut result = vec![Scalar::ZERO; n.max(1)];
    for i in 0..n {
        let (xi, yi) = points[i];
        let mut basis = vec![Scalar::ONE];
        for j in 0..n {
            if i == j {
                continue;
            }
            let xj = points[j].0;
            let d = (xi - xj).invert().unwrap_or(Scalar::ZERO);
            basis = poly_mul(&basis, &[-xj * d, d]);
        }
        result = poly_add(&result, &poly_scale(&basis, &yi));
    }
    result
}

fn poly_mul(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    if a.is_empty() || b.is_empty() {
        return vec![Scalar::ZERO];
    }
    let mut r = vec![Scalar::ZERO; a.len() + b.len() - 1];
    for (i, ac) in a.iter().enumerate() {
        if *ac == Scalar::ZERO {
            continue;
        }
        for (j, bc) in b.iter().enumerate() {
            if *bc == Scalar::ZERO {
                continue;
            }
            r[i + j] += *ac * *bc;
        }
    }
    r
}

fn poly_add(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    let m = a.len().max(b.len());
    let mut r = vec![Scalar::ZERO; m];
    for (i, c) in a.iter().enumerate() {
        r[i] += c;
    }
    for (i, c) in b.iter().enumerate() {
        r[i] += c;
    }
    r
}

fn poly_sub(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    let m = a.len().max(b.len());
    let mut r = vec![Scalar::ZERO; m];
    for (i, c) in a.iter().enumerate() {
        r[i] += c;
    }
    for (i, c) in b.iter().enumerate() {
        r[i] -= c;
    }
    r
}

fn poly_scale(a: &[Scalar], s: &Scalar) -> Vec<Scalar> {
    a.iter().map(|c| *c * s).collect()
}

fn poly_val(coeffs: &[Scalar], x: &Scalar) -> Scalar {
    let mut r = Scalar::ZERO;
    for c in coeffs.iter().rev() {
        r = r * x + c;
    }
    r
}

fn poly_trim(mut a: Vec<Scalar>) -> Vec<Scalar> {
    while a.len() > 1 && a[a.len() - 1] == Scalar::ZERO {
        a.pop();
    }
    a
}

fn poly_div(a: &[Scalar], b: &[Scalar]) -> Vec<Scalar> {
    let mut a = a.to_vec();
    let b = poly_trim(b.to_vec());
    if b.len() > a.len() {
        return vec![Scalar::ZERO];
    }
    let b_deg = b.len() - 1;
    let lead_inv = b[b_deg].invert().unwrap_or(Scalar::ZERO);
    let q_len = a.len() - b_deg;
    let mut q = vec![Scalar::ZERO; q_len];
    for i in (0..q_len).rev() {
        let coeff = a[i + b_deg] * lead_inv;
        q[i] = coeff;
        for j in 0..=b_deg {
            a[i + j] -= coeff * b[j];
        }
    }
    poly_trim(q)
}

fn poly_combine(coeffs_list: &[Vec<Scalar>], weights: &[Scalar]) -> Vec<Scalar> {
    let max_deg = coeffs_list.iter().map(|c| c.len()).max().unwrap_or(0);
    let mut result = vec![Scalar::ZERO; max_deg];
    for (i, coeffs) in coeffs_list.iter().enumerate() {
        let w = weights.get(i).copied().unwrap_or(Scalar::ZERO);
        if w == Scalar::ZERO {
            continue;
        }
        for (j, c) in coeffs.iter().enumerate() {
            result[j] += w * c;
        }
    }
    poly_trim(result)
}

fn eval_g1(coeffs: &[Scalar], tau_powers: &[G1Affine]) -> G1Projective {
    let mut r = G1Projective::identity();
    for (i, c) in coeffs.iter().enumerate() {
        if *c != Scalar::ZERO && i < tau_powers.len() {
            r += tau_powers[i].to_curve() * c;
        }
    }
    r
}

fn eval_g2(coeffs: &[Scalar], tau_powers: &[G2Affine]) -> G2Projective {
    let mut r = G2Projective::identity();
    for (i, c) in coeffs.iter().enumerate() {
        if *c != Scalar::ZERO && i < tau_powers.len() {
            r += tau_powers[i].to_curve() * c;
        }
    }
    r
}

fn scalar_pow(s: &Scalar, exp: usize) -> Scalar {
    let mut r = Scalar::ONE;
    for _ in 0..exp {
        r *= s;
    }
    r
}

pub fn setup(cs: &ConstraintSystem) -> (LegacyProvingKey, LegacyVerifyingKey) {
    let mut rng = thread_rng();
    let l = cs.num_public;
    let m = cs.num_variables;
    let n = cs.num_constraints;

    let tau = Scalar::random(&mut rng);
    let alpha = Scalar::random(&mut rng);
    let beta = Scalar::random(&mut rng);
    let gamma = Scalar::random(&mut rng);
    let delta = Scalar::random(&mut rng);

    let pts: Vec<Scalar> = (0..n).map(|i| Scalar::from(i as u64)).collect();

    let mut u_c = Vec::new();
    let mut v_c = Vec::new();
    let mut w_c = Vec::new();

    for var in 0..m {
        let mut ap = Vec::new();
        let mut bp = Vec::new();
        let mut cp = Vec::new();
        for (j, p) in pts.iter().enumerate() {
            let ac = cs.a[j]
                .iter()
                .find(|(vi, _)| *vi == var)
                .map(|(_, c)| *c)
                .unwrap_or(Scalar::ZERO);
            let bc = cs.b[j]
                .iter()
                .find(|(vi, _)| *vi == var)
                .map(|(_, c)| *c)
                .unwrap_or(Scalar::ZERO);
            let cc = cs.c[j]
                .iter()
                .find(|(vi, _)| *vi == var)
                .map(|(_, c)| *c)
                .unwrap_or(Scalar::ZERO);
            ap.push((*p, ac));
            bp.push((*p, bc));
            cp.push((*p, cc));
        }
        u_c.push(ap);
        v_c.push(bp);
        w_c.push(cp);
    }

    let u_poly: Vec<Vec<Scalar>> = u_c.iter().map(|pts| lagrange(pts)).collect();
    let v_poly: Vec<Vec<Scalar>> = v_c.iter().map(|pts| lagrange(pts)).collect();
    let w_poly: Vec<Vec<Scalar>> = w_c.iter().map(|pts| lagrange(pts)).collect();

    let tp_size = 2 * n + 2;
    let tau_powers_g1: Vec<G1Affine> = (0..tp_size)
        .map(|i| (G1Projective::generator() * scalar_pow(&tau, i)).to_affine())
        .collect();
    let tau_powers_g2: Vec<G2Affine> = (0..tp_size)
        .map(|i| (G2Projective::generator() * scalar_pow(&tau, i)).to_affine())
        .collect();

    let z_poly: Vec<Scalar> = {
        let mut zp = vec![Scalar::ONE];
        for p in &pts {
            zp = poly_mul(&zp, &[-*p, Scalar::ONE]);
        }
        zp
    };

    let z_tau = poly_val(&z_poly, &tau);

    let alpha_g1 = (G1Projective::generator() * alpha).to_affine();
    let beta_g1 = (G1Projective::generator() * beta).to_affine();
    let delta_g1 = (G1Projective::generator() * delta).to_affine();
    let beta_g2 = (G2Projective::generator() * beta).to_affine();
    let gamma_g2 = (G2Projective::generator() * gamma).to_affine();
    let delta_g2 = (G2Projective::generator() * delta).to_affine();

    let gamma_inv = gamma.invert().unwrap_or(Scalar::ZERO);
    let delta_inv = delta.invert().unwrap_or(Scalar::ZERO);

    let mut abc_public_g1 = Vec::new();
    let mut abc_private_g1 = Vec::new();

    for i in 0..m {
        let uv = beta * poly_val(&u_poly[i], &tau);
        let av = alpha * poly_val(&v_poly[i], &tau);
        let wv = poly_val(&w_poly[i], &tau);
        let combined = uv + av + wv;
        if i < l {
            let coeff = combined * gamma_inv;
            abc_public_g1.push((G1Projective::generator() * coeff).to_affine());
        } else {
            let coeff = combined * delta_inv;
            abc_private_g1.push((G1Projective::generator() * coeff).to_affine());
        }
    }

    let h_crs_g1: Vec<G1Affine> = (0..n.max(1))
        .map(|i| {
            let tj = scalar_pow(&tau, i);
            let val = tj * z_tau * delta_inv;
            (G1Projective::generator() * val).to_affine()
        })
        .collect();

    let pk = LegacyProvingKey {
        alpha_g1,
        beta_g1,
        delta_g1,
        beta_g2,
        delta_g2,
        gamma_g2,
        tau_powers_g1,
        tau_powers_g2,
        abc_public_g1: abc_public_g1.clone(),
        abc_private_g1,
        h_crs_g1,
        l,
        m,
        n,
        u_coeffs: u_poly,
        v_coeffs: v_poly,
        w_coeffs: w_poly,
        z_coeffs: z_poly,
    };

    let vk = LegacyVerifyingKey {
        alpha_g1,
        beta_g2,
        gamma_g2,
        delta_g2,
        ic: abc_public_g1,
    };

    (pk, vk)
}

pub fn prove(pk: &LegacyProvingKey, cs: &ConstraintSystem) -> LegacyProof {
    let mut rng = thread_rng();
    let r = Scalar::random(&mut rng);
    let s = Scalar::random(&mut rng);
    let w = &cs.witness;

    let u_combined = poly_combine(&pk.u_coeffs, w);
    let v_combined = poly_combine(&pk.v_coeffs, w);
    let w_combined = poly_combine(&pk.w_coeffs, w);

    let uv = poly_mul(&u_combined, &v_combined);
    let h_num = poly_sub(&uv, &w_combined);
    let h_poly = poly_div(&h_num, &pk.z_coeffs);

    let u_g1 = eval_g1(&u_combined, &pk.tau_powers_g1);
    let v_g2 = eval_g2(&v_combined, &pk.tau_powers_g2);
    let v_g1 = eval_g1(&v_combined, &pk.tau_powers_g1);

    let a = (pk.alpha_g1.to_curve() + u_g1 + pk.delta_g1.to_curve() * r).to_affine();
    let b = (pk.beta_g2.to_curve() + v_g2 + pk.delta_g2.to_curve() * s).to_affine();

    let mut c = G1Projective::identity();
    for (idx, var_idx) in (pk.l..pk.m).enumerate() {
        if var_idx < w.len() {
            let weight = w[var_idx];
            if weight != Scalar::ZERO && idx < pk.abc_private_g1.len() {
                c += pk.abc_private_g1[idx].to_curve() * weight;
            }
        }
    }

    let mut h_g1 = G1Projective::identity();
    for (j, coeff) in h_poly.iter().enumerate() {
        if *coeff != Scalar::ZERO && j < pk.h_crs_g1.len() {
            h_g1 += pk.h_crs_g1[j].to_curve() * coeff;
        }
    }
    c += h_g1;

    let b_g1 = pk.beta_g1.to_curve() + v_g1 + pk.delta_g1.to_curve() * s;
    c = c + a.to_curve() * s + b_g1 * r - pk.delta_g1.to_curve() * (r * s);

    LegacyProof {
        a,
        b,
        c: c.to_affine(),
    }
}

pub fn verify(vk: &LegacyVerifyingKey, public_inputs: &[Scalar], proof: &LegacyProof) -> bool {
    let mut ic = vk.ic[0].to_curve();
    for (i, input) in public_inputs.iter().enumerate() {
        if i + 1 < vk.ic.len() {
            ic = ic + vk.ic[i + 1].to_curve() * input;
        }
    }
    let ic = ic.to_affine();

    let a_b = bls12_381::pairing(&proof.a, &proof.b);
    let ab = bls12_381::pairing(&vk.alpha_g1, &vk.beta_g2);
    let ic_g = bls12_381::pairing(&ic, &vk.gamma_g2);
    let c_d = bls12_381::pairing(&proof.c, &vk.delta_g2);

    a_b == ab + ic_g + c_d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_cs() -> ConstraintSystem {
        ConstraintSystem {
            num_public: 3,
            num_private: 1,
            num_variables: 4,
            num_constraints: 1,
            a: vec![vec![(1, Scalar::ONE)]],
            b: vec![vec![(2, Scalar::ONE)]],
            c: vec![vec![(3, Scalar::ONE)]],
            witness: vec![
                Scalar::ONE,
                Scalar::from(3u64),
                Scalar::from(5u64),
                Scalar::from(15u64),
            ],
        }
    }

    #[test]
    fn test_pairing_basic() {
        let one = Scalar::ONE;
        let two = Scalar::from(2u64);
        let three = Scalar::from(3u64);
        let g1 = G1Projective::generator();
        let g2 = G2Projective::generator();
        let p1 = (g1 * one).to_affine();
        let p2 = (g1 * two).to_affine();
        let p3 = (g1 * three).to_affine();
        let q = (g2 * one).to_affine();
        let e1 = bls12_381::pairing(&p1, &q);
        let e2 = bls12_381::pairing(&p2, &q);
        let e3 = bls12_381::pairing(&p3, &q);
        assert_eq!(e3, e1 + e2, "bilinearity failed");
    }

    #[test]
    fn test_groth16_basic() {
        let cs = make_simple_cs();
        let (pk, vk) = setup(&cs);
        let proof = prove(&pk, &cs);
        let public_inputs = vec![Scalar::from(3u64), Scalar::from(5u64)];
        let result = verify(&vk, &public_inputs, &proof);
        assert!(result, "Groth16 proof should verify");
    }
}
