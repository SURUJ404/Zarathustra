use crate::patterns::{
    circuit_vulnerability_patterns, solidity_vulnerability_patterns, PatternType,
};
use crate::Finding;
use regex::Regex;

pub fn analyze(code: &str, ext: &str, file: &str) -> Vec<Finding> {
    match ext {
        "zara" => analyze_circuit(code, file),
        "rs" => super::analyze_rust::analyze(code, file),
        "sol" => analyze_solidity(code, file),
        _ => Vec::new(),
    }
}

fn run_semantic_checks(
    code: &str,
    patterns: &[crate::patterns::VulnerabilityPattern],
    file: &str,
    is_solidity: bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    let has_assert = code.contains("assert")
        || code.contains("===")
        || code.contains("==")
        || (is_solidity && code.contains("require"));
    let has_nonce = code.contains("nonce") || code.contains("nullifier");
    let has_verify = code.contains("verifyProof") || code.contains("verifyTx");
    let has_field = code.contains("field");
    let has_range_type =
        code.contains("u8") || code.contains("u16") || code.contains("u32") || code.contains("u64");
    let _public_inputs: Vec<&str> = if !is_solidity {
        code.lines()
            .filter(|l| l.contains("field") && (l.contains("private")))
            .flat_map(|l| {
                let parts: Vec<&str> = l.split([',', '(', ')', ':']).collect();
                parts
                    .into_iter()
                    .filter(|p| p.trim().starts_with("field") && p.contains("private"))
                    .map(|p| p.split_whitespace().last().unwrap_or(""))
                    .collect::<Vec<&str>>()
            })
            .collect()
    } else {
        Vec::new()
    };

    for pattern in patterns {
        match &pattern.pattern_type {
            PatternType::Regex(re_str) => {
                if let Ok(re) = Regex::new(re_str) {
                    for (lineno, line) in lines.iter().enumerate() {
                        if re.is_match(line) {
                            findings.push(Finding {
                                severity: pattern.severity.to_string(),
                                category: pattern.category.to_string(),
                                title: pattern.title.to_string(),
                                file: file.to_string(),
                                line: Some(lineno + 1),
                                description: pattern.description.to_string(),
                                snippet: line.trim().to_string(),
                                fix: pattern.fix.to_string(),
                            });
                        }
                    }
                }
            }
            PatternType::Semantic => match pattern.id {
                "CIR-002" if has_field && has_assert && !has_range_type => {
                    findings.push(Finding {
                        severity: pattern.severity.to_string(),
                        category: pattern.category.to_string(),
                        title: pattern.title.to_string(),
                        file: file.to_string(),
                        line: None,
                        description: pattern.description.to_string(),
                        snippet: "field type used".to_string(),
                        fix: pattern.fix.to_string(),
                    });
                }
                "SOL-003" if has_verify && !has_nonce => {
                    findings.push(Finding {
                        severity: pattern.severity.to_string(),
                        category: pattern.category.to_string(),
                        title: pattern.title.to_string(),
                        file: file.to_string(),
                        line: None,
                        description: pattern.description.to_string(),
                        snippet: "verifyProof found without nonce/nullifier".to_string(),
                        fix: pattern.fix.to_string(),
                    });
                }
                _ => {}
            },
            PatternType::KeywordMatch(kws) => {
                for kw in kws {
                    for (lineno, line) in lines.iter().enumerate() {
                        if line.contains(kw) {
                            findings.push(Finding {
                                severity: pattern.severity.to_string(),
                                category: pattern.category.to_string(),
                                title: pattern.title.to_string(),
                                file: file.to_string(),
                                line: Some(lineno + 1),
                                description: pattern.description.to_string(),
                                snippet: line.trim().to_string(),
                                fix: pattern.fix.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    findings
}

fn analyze_circuit(code: &str, file: &str) -> Vec<Finding> {
    let patterns = circuit_vulnerability_patterns();
    let mut findings = run_semantic_checks(code, &patterns, file, false);

    for pattern in &patterns {
        if let PatternType::Regex(re_str) = &pattern.pattern_type {
            if let Ok(re) = Regex::new(re_str) {
                for (lineno, line) in code.lines().enumerate() {
                    if re.is_match(line) {
                        findings.push(Finding {
                            severity: pattern.severity.to_string(),
                            category: pattern.category.to_string(),
                            title: pattern.title.to_string(),
                            file: file.to_string(),
                            line: Some(lineno + 1),
                            description: pattern.description.to_string(),
                            snippet: line.trim().to_string(),
                            fix: pattern.fix.to_string(),
                        });
                    }
                }
            }
        }
    }

    if let Ok(program) = astra_frontend::parse(code) {
        findings.extend(super::signal_flow::check(&program, file));
    }

    findings
}

fn analyze_solidity(code: &str, file: &str) -> Vec<Finding> {
    let patterns = solidity_vulnerability_patterns();
    let findings = run_semantic_checks(code, &patterns, file, true);

    let mut re_findings = Vec::new();
    for pattern in &patterns {
        if let PatternType::Regex(re_str) = &pattern.pattern_type {
            if let Ok(re) = Regex::new(re_str) {
                for (lineno, line) in code.lines().enumerate() {
                    if re.is_match(line) {
                        re_findings.push(Finding {
                            severity: pattern.severity.to_string(),
                            category: pattern.category.to_string(),
                            title: pattern.title.to_string(),
                            file: file.to_string(),
                            line: Some(lineno + 1),
                            description: pattern.description.to_string(),
                            snippet: line.trim().to_string(),
                            fix: pattern.fix.to_string(),
                        });
                    }
                }
            }
        }
    }

    let mut all = findings;
    all.extend(re_findings);
    all.extend(super::cei::check_solidity(code, file));
    all
}
