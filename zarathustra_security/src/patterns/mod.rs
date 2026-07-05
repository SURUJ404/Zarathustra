use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityPattern {
    pub id: String,
    pub name: String,
    pub severity: super::Severity,
    pub category: String,
    pub description: String,
    pub recommendation: String,
    pub pattern_type: PatternType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Regex(String),
    KeywordMatch(Vec<String>),
    Semantic,
}

pub fn circuit_vulnerability_patterns() -> Vec<VulnerabilityPattern> {
    vec![
        VulnerabilityPattern {
            id: "CIR-001".into(),
            name: "Unconstrained Signal".into(),
            severity: super::Severity::Critical,
            category: "Constraint Safety".into(),
            description: "Signal may be under-constrained, allowing prover to choose arbitrary values without satisfying meaningful constraints.".into(),
            recommendation: "Ensure all signals are properly constrained with linear or quadratic constraints.".into(),
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "CIR-002".into(),
            name: "Missing Range Check".into(),
            severity: super::Severity::High,
            category: "Input Validation".into(),
            description: "Field element used without range check, may cause unexpected overflow or truncation behavior.".into(),
            recommendation: "Add explicit range assertions for field elements that represent bounded integers.".into(),
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "CIR-003".into(),
            name: "Unused Public Input".into(),
            severity: super::Severity::Low,
            category: "Input Hygiene".into(),
            description: "Public input declared but never constrained in the circuit.".into(),
            recommendation: "Remove unused public inputs or add the necessary constraints.".into(),
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "CIR-004".into(),
            name: "Assertion on Private Input Only".into(),
            severity: super::Severity::Medium,
            category: "Proof Completeness".into(),
            description: "Critical assertions depend only on private inputs, making them invisible to verifiers.".into(),
            recommendation: "Include public inputs in key assertions to enable public verifiability.".into(),
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "CIR-005".into(),
            name: "Hardcoded Constant in Constraint".into(),
            severity: super::Severity::Info,
            category: "Code Quality".into(),
            description: "Magic numbers or hardcoded constants used directly in constraints.".into(),
            recommendation: "Define constants with descriptive names for maintainability.".into(),
            pattern_type: PatternType::Regex(r"\b(field|u8|u16|u32|u64)\s*\(\s*\d{5,}\s*\)".into()),
        },
        VulnerabilityPattern {
            id: "CIR-006".into(),
            name: "Potential Under-Constraint Path".into(),
            severity: super::Severity::High,
            category: "Constraint Safety".into(),
            description: "Conditional branch may leave some signals unconstrained on certain paths.".into(),
            recommendation: "Verify all execution paths produce equivalent constraints on public outputs.".into(),
            pattern_type: PatternType::Semantic,
        },
    ]
}

pub fn solidity_vulnerability_patterns() -> Vec<VulnerabilityPattern> {
    vec![
        VulnerabilityPattern {
            id: "SOL-001".into(),
            name: "Unchecked External Call".into(),
            severity: super::Severity::Critical,
            category: "Call Safety".into(),
            description: "External call made without checking return value.".into(),
            recommendation: "Always check return values of external calls or use a wrapper that does.".into(),
            pattern_type: PatternType::Regex(r"\b(address\s*\(\s*\w+\s*\)\s*\.\s*call(?!\s*\.\s*success))".into()),
        },
        VulnerabilityPattern {
            id: "SOL-002".into(),
            name: "Re-entrancy Risk".into(),
            severity: super::Severity::High,
            category: "Execution Safety".into(),
            description: "State changes after external call, enabling re-entrancy.".into(),
            recommendation: "Apply checks-effects-interactions pattern or use re-entrancy guards.".into(),
            pattern_type: PatternType::Regex(r"\.call\s*\{[^}]*\}\s*\([^)]*\)\s*;".into()),
        },
        VulnerabilityPattern {
            id: "SOL-003".into(),
            name: "Verifier Replay Attack".into(),
            severity: super::Severity::Critical,
            category: "Cryptographic".into(),
            description: "Verifier contract may accept proofs without checking public input freshness.".into(),
            recommendation: "Include a nonce or timestamp in public inputs to prevent proof replay.".into(),
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "SOL-004".into(),
            name: "Unrestricted Verification".into(),
            severity: super::Severity::High,
            category: "Access Control".into(),
            description: "verifyProof function callable by any address without access control.".into(),
            recommendation: "Add access control modifiers or msg.sender checks to verification functions.".into(),
            pattern_type: PatternType::Regex(r"function\s+verifyProof".into()),
        },
        VulnerabilityPattern {
            id: "SOL-005".into(),
            name: "Gas Limit Issues".into(),
            severity: super::Severity::Medium,
            category: "Gas Optimization".into(),
            description: "Potential gas exhaustion due to loops or large computations.".into(),
            recommendation: "Use bounded loops and optimize pairing check parameters.".into(),
            pattern_type: PatternType::Regex(r"\bfor\s*\([^)]*\)\s*\{[^}]*\bgas\b".into()),
        },
        VulnerabilityPattern {
            id: "SOL-006".into(),
            name: "Timestamp Dependence".into(),
            severity: super::Severity::Low,
            category: "Timing".into(),
            description: "Block timestamp used in critical logic, can be manipulated by miners.".into(),
            recommendation: "Use block number instead of timestamp for time-dependent logic.".into(),
            pattern_type: PatternType::Regex(r"\bblock\s*\.\s*timestamp\b".into()),
        },
    ]
}
