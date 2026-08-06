//! Checks-Effects-Interactions analysis for Solidity verifiers.
//!
//! Basic brace-tracking pass: if a state mutation appears after an external
//! call (`.call` / `.transfer` / `.send`) inside the same function body, the
//! contract may be vulnerable to reentrancy. This is intentionally
//! conservative — a real checker belongs in a dedicated crate.

use crate::Finding;

fn is_external_call(line: &str) -> bool {
    line.contains(".call") || line.contains(".transfer") || line.contains(".send")
}

fn is_state_mutation(line: &str) -> bool {
    let trimmed = line.trim_start();
    let assignment = line.contains('=')
        && !line.trim_start().starts_with("//")
        && !line.trim_start().starts_with("==")
        && !line.trim_start().starts_with("!=");
    let storage = trimmed.contains("storage")
        || trimmed.starts_with("_balances")
        || trimmed.contains("mapping[")
        || trimmed.contains(".push(")
        || trimmed.starts_with("balances[");
    assignment && storage
}

/// Flag storage mutations that follow an external call in the same function.
pub fn check_solidity(src: &str, file: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut depth = 0i32;
    let mut saw_external = false;
    let mut last_external = 0usize;

    for (i, line) in src.lines().enumerate() {
        let stripped = line.split("//").next().unwrap_or("");
        depth += stripped.matches('{').count() as i32;
        depth -= stripped.matches('}').count() as i32;

        if saw_external && depth > 0 && is_state_mutation(line) {
            findings.push(Finding {
                severity: "HIGH".into(),
                category: "CEI Violation".into(),
                title: "State Mutation After External Call".into(),
                file: file.into(),
                line: Some(i + 1),
                description: format!(
                    "state is written on line {} after an external call on line {} within the same function — reentrancy risk",
                    i + 1,
                    last_external + 1
                ),
                snippet: line.trim().to_string(),
                fix: "move effects before interactions, or use a reentrancy guard".into(),
            });
        }

        if is_external_call(line) {
            saw_external = true;
            last_external = i;
        }

        if depth <= 0 {
            saw_external = false;
        }
    }

    findings
}
