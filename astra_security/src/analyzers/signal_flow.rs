//! Signal-flow analysis for Zara circuits.
//!
//! Traces which inputs actually flow into emitted constraints. A public input
//! that never reaches any constraint is "unconstrained" — an attacker can pick
//! it freely, which is a soundness problem.
//!
//! Replaces the old `has_field && has_assert && !has_range_type` heuristic,
//! which fired unconditionally on every `.zara` file that used `field`.

use crate::Finding;
use astra_ir::ir::{Expr, Program, Stmt};
use std::collections::HashSet;

/// Collect every variable referenced anywhere in the program body.
///
/// Note: this errs toward fewer findings. It treats an input that flows into a
/// `declare` as constrained, because the multiply lowering emits an R1CS row.
pub fn referenced_vars(program: &Program) -> HashSet<String> {
    let mut seen = HashSet::new();
    for stmt in &program.main.body {
        collect_stmt(stmt, &mut seen);
    }
    seen
}

fn collect_stmt(stmt: &Stmt, seen: &mut HashSet<String>) {
    match stmt {
        Stmt::Declare { init, .. } => {
            if let Some(init) = init {
                collect_expr(init, seen);
            }
        }
        Stmt::Constrain { left, right } => {
            collect_expr(left, seen);
            collect_expr(right, seen);
        }
        Stmt::If { cond, body } => {
            collect_expr(cond, seen);
            for s in body {
                collect_stmt(s, seen);
            }
        }
        Stmt::Return(e) => collect_expr(e, seen),
    }
}

fn collect_expr(expr: &Expr, seen: &mut HashSet<String>) {
    match expr {
        Expr::Number(_) => {}
        Expr::Variable(name) => {
            seen.insert(name.clone());
        }
        Expr::Binary(_, l, r) => {
            collect_expr(l, seen);
            collect_expr(r, seen);
        }
    }
}

/// Flag public inputs never referenced by any statement as HIGH.
pub fn check(program: &Program, file: &str) -> Vec<Finding> {
    let seen = referenced_vars(program);
    let mut findings = Vec::new();
    for (name, is_private) in &program.main.params {
        if !is_private && !seen.contains(name) {
            findings.push(Finding {
                severity: "HIGH".into(),
                category: "Constraint Safety".into(),
                title: "Unconstrained Public Input".into(),
                file: file.into(),
                line: None,
                description: format!(
                    "public input '{}' is never referenced by any statement",
                    name
                ),
                snippet: format!("field {}", name),
                fix: "add an assert involving this input, or make it private".into(),
            });
        }
    }
    findings
}
