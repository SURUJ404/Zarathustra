pub enum PatternType {
    Regex(String),
    KeywordMatch(Vec<String>),
    Semantic,
}

pub struct VulnerabilityPattern {
    pub id: &'static str,
    pub title: &'static str,
    pub severity: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub fix: &'static str,
    pub pattern_type: PatternType,
}

pub fn circuit_vulnerability_patterns() -> Vec<VulnerabilityPattern> {
    vec![
        VulnerabilityPattern {
            id: "CIR-001",
            title: "Unconstrained Signal",
            severity: "CRITICAL",
            category: "Constraint Safety",
            description: "Public input is never constrained by any assertion",
            fix: "Add an assert statement involving this public input",
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "CIR-002",
            title: "Missing Range Check",
            severity: "HIGH",
            category: "Input Validation",
            description: "Field element used without range check, may cause unexpected overflow or truncation behavior",
            fix: "Add explicit range assertions for field elements that represent bounded integers",
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "CIR-003",
            title: "Unused Public Input",
            severity: "LOW",
            category: "Input Hygiene",
            description: "Public input is declared but never referenced in any constraint",
            fix: "Remove unused public inputs or add constraints that use them",
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "CIR-004",
            title: "Assertion on Private Input Only",
            severity: "MEDIUM",
            category: "Proof Completeness",
            description: "Critical assertions rely solely on private inputs, making them invisible to verifiers",
            fix: "Include at least one public input in critical assertions",
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "CIR-005",
            title: "Hardcoded Constant in Constraint",
            severity: "INFO",
            category: "Code Quality",
            description: "Large hardcoded constant detected in circuit",
            fix: "Consider making significant constants configurable as public inputs",
            pattern_type: PatternType::Regex(r"\b(field|u8|u16|u32|u64)\s*\(\s*\d{5,}\s*\)".to_string()),
        },
        VulnerabilityPattern {
            id: "CIR-006",
            title: "Potential Under-Constraint Path",
            severity: "HIGH",
            category: "Constraint Safety",
            description: "Conditional branch without else may leave signals unconstrained",
            fix: "Add an else branch to ensure all signals remain constrained in every execution path",
            pattern_type: PatternType::Semantic,
        },
    ]
}

pub fn solidity_vulnerability_patterns() -> Vec<VulnerabilityPattern> {
    vec![
        VulnerabilityPattern {
            id: "SOL-001",
            title: "Unchecked External Call",
            severity: "CRITICAL",
            category: "Call Safety",
            description: "External call return value is not checked",
            fix: "Check the return value of the external call and revert on failure",
            pattern_type: PatternType::Regex(r"\.call\s*\{[^}]*\}\s*\([^)]*\)".to_string()),
        },
        VulnerabilityPattern {
            id: "SOL-002",
            title: "Re-entrancy Risk",
            severity: "HIGH",
            category: "Execution Safety",
            description: "State changes after external call, enabling re-entrancy",
            fix: "Apply checks-effects-interactions pattern or use re-entrancy guards",
            pattern_type: PatternType::Regex(r"\.call\s*\{[^}]*\}\s*\([^)]*\)\s*;".to_string()),
        },
        VulnerabilityPattern {
            id: "SOL-003",
            title: "Verifier Replay Attack",
            severity: "CRITICAL",
            category: "Cryptographic",
            description: "Proof verification lacks nonce or nullifier mechanism",
            fix: "Include a nonce or nullifier in the public inputs to prevent proof replay",
            pattern_type: PatternType::Semantic,
        },
        VulnerabilityPattern {
            id: "SOL-004",
            title: "Unrestricted Verification",
            severity: "HIGH",
            category: "Access Control",
            description: "verifyProof function is publicly callable without access control",
            fix: "Add onlyOwner modifier or msg.sender check to verifyProof",
            pattern_type: PatternType::Regex(r"function\s+verifyProof".to_string()),
        },
        VulnerabilityPattern {
            id: "SOL-005",
            title: "Gas Limit Issues",
            severity: "MEDIUM",
            category: "Gas Optimization",
            description: "Loop with gas-dependent logic may cause out-of-gas",
            fix: "Avoid gas-dependent logic inside loops; consider bounded iterations",
            pattern_type: PatternType::Regex(r"\bfor\s*\([^)]*\)\s*\{[^}]*\bgas\b".to_string()),
        },
        VulnerabilityPattern {
            id: "SOL-006",
            title: "Timestamp Dependence",
            severity: "LOW",
            category: "Timing",
            description: "Use of block.timestamp which can be manipulated by miners",
            fix: "Avoid critical logic based on block.timestamp; use block.number instead",
            pattern_type: PatternType::Regex(r"\bblock\s*\.\s*timestamp\b".to_string()),
        },
    ]
}
