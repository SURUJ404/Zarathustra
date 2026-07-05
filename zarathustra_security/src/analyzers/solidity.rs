use crate::patterns::VulnerabilityPattern;
use crate::Finding;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static PATTERNS: LazyLock<Vec<VulnerabilityPattern>> =
    LazyLock::new(|| crate::patterns::solidity_vulnerability_patterns());

pub fn analyze_solidity(
    file_path: &Path,
    verbose: bool,
) -> Result<Vec<Finding>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut findings = Vec::new();
    let file_path_str = file_path.to_string_lossy().to_string();

    for pattern in PATTERNS.iter() {
        match &pattern.pattern_type {
            crate::patterns::PatternType::Regex(regex_str) => {
                let re = Regex::new(regex_str)?;
                for cap in re.find_iter(&content) {
                    let line_number = content[..cap.start()].lines().count() + 1;
                    findings.push(Finding {
                        severity: pattern.severity.clone(),
                        category: pattern.category.clone(),
                        title: pattern.name.clone(),
                        description: pattern.description.clone(),
                        file_path: file_path_str.clone(),
                        line_number: Some(line_number),
                        snippet: Some(cap.as_str().to_string()),
                        recommendation: pattern.recommendation.clone(),
                    });
                }
            }
            crate::patterns::PatternType::KeywordMatch(keywords) => {
                for keyword in keywords {
                    if content.contains(keyword) {
                        findings.push(Finding {
                            severity: pattern.severity.clone(),
                            category: pattern.category.clone(),
                            title: pattern.name.clone(),
                            description: pattern.description.clone(),
                            file_path: file_path_str.clone(),
                            line_number: None,
                            snippet: Some(keyword.clone()),
                            recommendation: pattern.recommendation.clone(),
                        });
                    }
                }
            }
            crate::patterns::PatternType::Semantic => {
                let semantic_findings = run_semantic_solidity_checks(&content, &file_path_str, pattern)?;
                findings.extend(semantic_findings);
            }
        }
    }

    if verbose {
        eprintln!("[{}] scanned: {} findings", file_path_str, findings.len());
    }

    Ok(findings)
}

fn run_semantic_solidity_checks(
    content: &str,
    file_path: &str,
    pattern: &VulnerabilityPattern,
) -> Result<Vec<Finding>, Box<dyn std::error::Error>> {
    let mut findings = Vec::new();

    match pattern.id.as_str() {
        "SOL-001" => {
            let re = Regex::new(r"\b(\w+)\.call\s*\{[^}]*\}\s*\(").unwrap();
            for cap in re.captures_iter(content) {
                let line_number = content[..cap.get(0).unwrap().start()].lines().count() + 1;
                let has_return_check = content[cap.get(0).unwrap().start()..]
                    .lines()
                    .next()
                    .map(|l| l.contains("require") || l.contains("success"))
                    .unwrap_or(false);

                if !has_return_check {
                    findings.push(Finding {
                        severity: pattern.severity.clone(),
                        category: pattern.category.clone(),
                        title: pattern.name.clone(),
                        description: format!("Unchecked call to {}", cap.get(1).map_or("", |m| m.as_str())),
                        file_path: file_path.to_string(),
                        line_number: Some(line_number),
                        snippet: Some(cap.get(0).map_or("", |m| m.as_str()).to_string()),
                        recommendation: pattern.recommendation.clone(),
                    });
                }
            }
        }
        "SOL-002" => {
            let re = Regex::new(r"(\w+)\.call\s*\{[^}]*\}\s*\([^)]*\)\s*;").unwrap();
            for cap in re.captures_iter(content) {
                let pos = cap.get(0).unwrap().start();
                let line_number = content[..pos].lines().count() + 1;
                let after_call = &content[pos..];
                let has_state_change = after_call.contains("storage")
                    || after_call.contains("transfer")
                    || after_call.contains("send");
                if has_state_change {
                    findings.push(Finding {
                        severity: pattern.severity.clone(),
                        category: pattern.category.clone(),
                        title: pattern.name.clone(),
                        description: pattern.description.clone(),
                        file_path: file_path.to_string(),
                        line_number: Some(line_number),
                        snippet: Some(cap.get(0).map_or("", |m| m.as_str()).to_string()),
                        recommendation: pattern.recommendation.clone(),
                    });
                }
            }
        }
        _ => {}
    }

    Ok(findings)
}
