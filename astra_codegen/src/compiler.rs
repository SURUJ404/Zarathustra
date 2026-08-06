use bls12_381::Scalar;
use ff::Field;
use std::collections::HashMap;

use astra_ir::ir::{BinaryOp, Expr, Program, Stmt};
use astra_ir::types::ConstraintSystem;

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

        Ok(ConstraintSystem {
            num_public: self.num_public,
            num_private: self.num_private,
            num_variables: num_vars,
            num_constraints: self.a.len(),
            a: self.a.clone(),
            b: self.b.clone(),
            c: self.c.clone(),
            witness: self.var_values.clone(),
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
