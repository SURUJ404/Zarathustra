use bls12_381::Scalar;
use ff::Field;
use std::collections::HashMap;

use astra_ir::ir::{BinaryOp, Expr, Program, Stmt};
use astra_ir::types::{Constraint, ConstraintSystem, R1CSTriple};

pub fn compile(
    source: &str,
    public: &[Scalar],
    private: &[Scalar],
) -> Result<ConstraintSystem, String> {
    let program = astra_frontend::parse(source).map_err(|e| e.render())?;
    let mut comp = Compiler::new();
    comp.compile_program(&program, public, private)
}

pub fn validate_constraints(cs: &ConstraintSystem) -> Result<(), String> {
    let w = &cs.witness;
    for (j, (a_row, (b_row, c_row))) in cs.a.iter().zip(cs.b.iter().zip(cs.c.iter())).enumerate() {
        let mut a_sum = Scalar::ZERO;
        for &(idx, coeff) in a_row {
            if idx < w.len() {
                a_sum += w[idx] * coeff;
            }
        }
        let mut b_sum = Scalar::ZERO;
        for &(idx, coeff) in b_row {
            if idx < w.len() {
                b_sum += w[idx] * coeff;
            }
        }
        let mut c_sum = Scalar::ZERO;
        for &(idx, coeff) in c_row {
            if idx < w.len() {
                c_sum += w[idx] * coeff;
            }
        }
        if a_sum * b_sum - c_sum != Scalar::ZERO {
            return Err(format!("constraint {} not satisfied", j));
        }
    }
    Ok(())
}

struct Compiler {
    var_index: HashMap<String, usize>,
    var_values: Vec<Scalar>,
    var_names: Vec<String>,
    a: Vec<Vec<(usize, Scalar)>>,
    b: Vec<Vec<(usize, Scalar)>>,
    c: Vec<Vec<(usize, Scalar)>>,
    constraints: Vec<Constraint>,
    num_public: usize,
    num_private: usize,
}

impl Compiler {
    fn new() -> Self {
        let mut var_index = HashMap::new();
        let mut var_names = Vec::new();
        let mut var_values = Vec::new();

        var_index.insert("~one".into(), 0);
        var_names.push("~one".into());
        var_values.push(Scalar::ONE);

        Compiler {
            var_index,
            var_values,
            var_names,
            a: Vec::new(),
            b: Vec::new(),
            c: Vec::new(),
            constraints: Vec::new(),
            num_public: 1,
            num_private: 0,
        }
    }

    fn alloc_var(&mut self, name: String, is_public: bool) -> usize {
        let idx = self.var_values.len();
        self.var_index.insert(name.clone(), idx);
        self.var_names.push(name);
        self.var_values.push(Scalar::ZERO);
        if is_public {
            self.num_public += 1;
        } else {
            self.num_private += 1;
        }
        idx
    }

    fn get_var(&self, name: &str) -> Result<usize, String> {
        self.var_index
            .get(name)
            .copied()
            .ok_or_else(|| format!("unknown variable: {}", name))
    }

    fn compile_program(
        &mut self,
        program: &Program,
        public_inputs: &[Scalar],
        private_inputs: &[Scalar],
    ) -> Result<ConstraintSystem, String> {
        let f = &program.main;
        let mut pub_idx = 0;
        let mut priv_idx = 0;

        for (name, is_private) in &f.params {
            let idx = self.alloc_var(name.clone(), !is_private);
            let val = if *is_private {
                if priv_idx >= private_inputs.len() {
                    return Err(format!("missing private input: {}", name));
                }
                let v = private_inputs[priv_idx];
                priv_idx += 1;
                v
            } else {
                if pub_idx >= public_inputs.len() {
                    return Err(format!("missing public input: {}", name));
                }
                let v = public_inputs[pub_idx];
                pub_idx += 1;
                v
            };
            self.var_values[idx] = val;
        }

        for stmt in &f.body {
            self.compile_stmt(stmt)?;
        }

        let num_vars = self.var_values.len();

        let mut r1cs = Vec::with_capacity(self.constraints.len());
        for (k, (a_row, (b_row, c_row))) in self
            .a
            .iter()
            .zip(self.b.iter().zip(self.c.iter()))
            .enumerate()
        {
            let triple = match self.constraints.get(k) {
                Some(Constraint::R1CS(t)) => t.clone(),
                _ => R1CSTriple {
                    a: a_row.clone(),
                    b: b_row.clone(),
                    c: c_row.clone(),
                },
            };
            r1cs.push(Constraint::R1CS(triple));
        }

        Ok(ConstraintSystem {
            num_public: self.num_public,
            num_private: self.num_private,
            num_variables: num_vars,
            num_constraints: self.a.len(),
            a: self.a.clone(),
            b: self.b.clone(),
            c: self.c.clone(),
            witness: self.var_values.clone(),
            constraints: r1cs,
        })
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<LinearCombination, String> {
        match expr {
            Expr::Number(val) => {
                let mut lc = LinearCombination::new();
                lc.add_term(0, *val);
                Ok(lc)
            }
            Expr::Variable(name) => {
                let idx = self.get_var(name)?;
                let mut lc = LinearCombination::new();
                lc.add_term(idx, Scalar::ONE);
                Ok(lc)
            }
            Expr::Binary(op, left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                match op {
                    BinaryOp::Add => Ok(l + r),
                    BinaryOp::Sub => Ok(l - r),
                    BinaryOp::Mul => {
                        let result_name = format!("~mult_{}", self.a.len());
                        let result_idx = self.alloc_var(result_name, false);

                        let l_eval = self.eval_lc(&l);
                        let r_eval = self.eval_lc(&r);
                        self.var_values[result_idx] = l_eval * r_eval;

                        let a_lc = LinearCombination::new() + l;
                        let b_lc = LinearCombination::new() + r;
                        let mut c_lc = LinearCombination::new();
                        c_lc.add_term(result_idx, Scalar::ONE);

                        self.a.push(self.lc_to_vec(&a_lc));
                        self.b.push(self.lc_to_vec(&b_lc));
                        self.c.push(self.lc_to_vec(&c_lc));
                        self.constraints.push(Constraint::R1CS(R1CSTriple {
                            a: self.lc_to_vec(&a_lc),
                            b: self.lc_to_vec(&b_lc),
                            c: self.lc_to_vec(&c_lc),
                        }));

                        let mut result = LinearCombination::new();
                        result.add_term(result_idx, Scalar::ONE);
                        Ok(result)
                    }
                }
            }
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Declare { name, init } => {
                let idx = self.alloc_var(name.clone(), false);
                if let Some(init_expr) = init {
                    let lc = self.compile_expr(init_expr)?;
                    self.var_values[idx] = self.eval_lc(&lc);
                }
                Ok(())
            }
            Stmt::Constrain { left, right } => {
                let left_lc = self.compile_expr(left)?;
                let right_lc = self.compile_expr(right)?;
                let diff = left_lc - right_lc;
                let mut a_lc = LinearCombination::new();
                a_lc.add_term(0, Scalar::ONE);
                self.a.push(self.lc_to_vec(&a_lc));
                self.b.push(self.lc_to_vec(&diff));
                self.c.push(Vec::new());
                self.constraints.push(Constraint::R1CS(R1CSTriple {
                    a: self.lc_to_vec(&a_lc),
                    b: self.lc_to_vec(&diff),
                    c: Vec::new(),
                }));
                Ok(())
            }
            Stmt::Return(_) => Ok(()),
            Stmt::If { cond, body } => {
                let cond_lc = self.compile_expr(cond)?;
                let _cond_val = self.eval_lc(&cond_lc);
                for s in body {
                    self.compile_stmt(s)?;
                }
                Ok(())
            }
        }
    }

    fn eval_lc(&self, lc: &LinearCombination) -> Scalar {
        let mut result = Scalar::ZERO;
        for &(idx, coeff) in &lc.terms {
            result += self.var_values[idx] * coeff;
        }
        result
    }

    fn lc_to_vec(&self, lc: &LinearCombination) -> Vec<(usize, Scalar)> {
        lc.terms.clone()
    }
}

/// Oracle assertions pinning the editor of [[crate::compiler::compile]] to the
/// mathematical ground truth in `docs/CONSTRAINT_MODEL.md`. These are the
/// exact invariants fuzzing and code review rely on: deterministic index
/// layout, per-construct constraint counts, and row shapes.
#[cfg(test)]
mod ground_truth_tests {
    use super::*;

    /// helper: decimal-string input → Scalar (avoids scalar-in-0 issues).
    fn sc(s: &str) -> Scalar {
        astra_ir::types::scalar_from_dec_str(s).expect("valid scalar")
    }

    /// `==` on a `(usize, Scalar)` pair, in any term order.
    fn has_term(row: &[(usize, Scalar)], idx: usize, coeff: &str) -> bool {
        row.iter().any(|(i, c)| *i == idx && *c == sc(coeff))
    }

    #[test]
    fn example_a_add_chain_layout() {
        // def main(field public c, field private a, field private b) {
        //     field t = a + b; assert(t == c); return 1; }
        let cs = compile(
            "def main(field public c, field private a, field private b) -> field {\n\
             field t = a + b;\n\
             assert(t == c);\n\
             return 1;\n\
             }",
            &[sc("7")],
            &[sc("3"), sc("4")],
        )
        .expect("compiles");

        // Ground truth: num_public=2 (~one+c), num_private=3 (a,b,t),
        // num_variables=5, num_constraints=1 (only the assert; + is linear).
        assert_eq!(cs.num_public, 2);
        assert_eq!(cs.num_private, 3);
        assert_eq!(cs.num_variables, 5);
        assert_eq!(cs.num_constraints, 1);

        // witness: [~one=1, c=7, a=3, b=4, t=7]
        assert_eq!(cs.witness[0], sc("1"));
        assert_eq!(cs.witness[1], sc("7"));
        assert_eq!(cs.witness[2], sc("3"));
        assert_eq!(cs.witness[3], sc("4"));
        assert_eq!(cs.witness[4], sc("7"));

        // The single row: 1 * (t - c) = 0  →  A=[(0,1)], B=[(4,1),(1,-1)], C=[]
        assert!(has_term(&cs.a[0], 0, "1"));
        assert!(has_term(&cs.b[0], 4, "1"));
        assert!(has_term(&cs.b[0], 1, "-1"));
        assert!(cs.c[0].is_empty());

        // And the whole system is satisfied under the witness.
        validate_constraints(&cs).expect("example_a ground truth must hold");
    }

    #[test]
    fn example_b_mul_gate_layout() {
        // def main(field public c, field private a, field private b) {
        //     field t = a * b; assert(t == c); return 1; }
        let cs = compile(
            "def main(field public c, field private a, field private b) {\n\
             field t = a * b;\n\
             assert(t == c);\n\
             return 1;\n\
             }",
            &[sc("15")],
            &[sc("3"), sc("5")],
        )
        .expect("example_b must compile");

        // Witness: [one, c=15, a=3, b=5, t=15, ~mult_0=15], var count = 6.
        assert_eq!(cs.num_variables, 6);
        assert_eq!(cs.num_private, 4); // a, b, t, ~mult_0
        assert_eq!(cs.num_constraints, 2);

        // row 0: a·b = ~mult_0  -> A=[(2,1)] B=[(3,1)] C=[(5,1)]
        assert!(has_term(&cs.a[0], 2, "1"));
        assert!(has_term(&cs.b[0], 3, "1"));
        assert!(has_term(&cs.c[0], 5, "1"));

        // row 1: t − c = 0
        assert!(has_term(&cs.a[1], 0, "1"));
        assert!(has_term(&cs.b[1], 4, "1"));
        assert!(has_term(&cs.b[1], 1, "-1"));
        assert!(cs.c[1].is_empty());

        assert_eq!(cs.witness[5], sc("15"));
        validate_constraints(&cs).expect("example_b ground truth must hold");
    }

    #[test]
    fn mul_intermediate_is_fresh_private_var() {
        // Nested: c = (a*b) with no explicit declare → ~mult_0 then result.
        let cs = compile(
            "def main(field public c, field private a, field private b) {\n\
             assert((a * b) == c);\n\
             return 1;\n\
             }",
            &[sc("12")],
            &[sc("3"), sc("4")],
        )
        .expect("mul intermediate must compile");

        // one + 3 params + 1 ~mult = 5 vars; 1 mul row + 1 assert row = 2.
        assert_eq!(cs.num_variables, 5);
        assert_eq!(cs.num_constraints, 2);
        assert_eq!(cs.witness[4], sc("12")); // ~mult_0 = 3*4
        validate_constraints(&cs).expect("mul intermediate ground truth must hold");
    }
}

#[derive(Clone, Debug)]
struct LinearCombination {
    terms: Vec<(usize, Scalar)>,
}

impl LinearCombination {
    fn new() -> Self {
        LinearCombination { terms: Vec::new() }
    }

    fn add_term(&mut self, idx: usize, coeff: Scalar) {
        for (i, (existing_idx, existing_coeff)) in self.terms.iter_mut().enumerate() {
            if *existing_idx == idx {
                let new_coeff = *existing_coeff + coeff;
                if new_coeff == Scalar::ZERO {
                    self.terms.swap_remove(i);
                } else {
                    *existing_coeff = new_coeff;
                }
                return;
            }
        }
        if coeff != Scalar::ZERO {
            self.terms.push((idx, coeff));
        }
    }
}

impl std::ops::Add for LinearCombination {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        for (idx, coeff) in rhs.terms {
            self.add_term(idx, coeff);
        }
        self
    }
}

impl std::ops::Sub for LinearCombination {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self {
        for (idx, coeff) in rhs.terms {
            self.add_term(idx, -coeff);
        }
        self
    }
}
