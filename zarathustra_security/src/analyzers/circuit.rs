use crate::patterns::VulnerabilityPattern;
use crate::Finding;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static PATTERNS: LazyLock<Vec<VulnerabilityPattern>> =
    LazyLock::new(|| crate::patterns::circuit_vulnerability_patterns());

pub fn analyze_circuit(
    file_path: &Path,
    verbose: bool,
) -> Result<Vec<Finding>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut findings = Vec::new();
    let file_path_str = file_path.to_string_lossy().to_string();

    for pattern in PATTERNS.iter() {
        match &pattern.pattern_type {
            crate::patterns::PatternType::Regex(regex_str) => {
                let re = match Regex::new(regex_str) {
                    Ok(r) => r,
                    Err(e) => {
                        if verbose {
                            eprintln!("[warning] bad regex pattern '{}': {}", regex_str, e);
                        }
                        continue;
                    }
                };
                for cap in re.find_iter(&content) {
                    let line_number = count_lines_before(&content, cap.start());
                    findings.push(Finding {
                        severity: pattern.severity.clone(),
                        category: pattern.category.clone(),
                        title: pattern.name.clone(),
                        description: pattern.description.clone(),
                        file_path: file_path_str.clone(),
                        line_number,
                        snippet: Some(cap.as_str().to_string()),
                        recommendation: pattern.recommendation.clone(),
                    });
                }
            }
            crate::patterns::PatternType::KeywordMatch(keywords) => {
                for keyword in keywords {
                    let mut search_start = 0;
                    while let Some(pos) = content[search_start..].find(keyword) {
                        let abs_pos = search_start + pos;
                        let line_number = count_lines_before(&content, abs_pos);
                        let start = abs_pos.saturating_sub(40);
                        let end = (abs_pos + keyword.len() + 40).min(content.len());
                        findings.push(Finding {
                            severity: pattern.severity.clone(),
                            category: pattern.category.clone(),
                            title: pattern.name.clone(),
                            description: pattern.description.clone(),
                            file_path: file_path_str.clone(),
                            line_number,
                            snippet: Some(content[start..end].to_string()),
                            recommendation: pattern.recommendation.clone(),
                        });
                        search_start = abs_pos + keyword.len();
                    }
                }
            }
            crate::patterns::PatternType::Semantic => {
                findings.extend(run_semantic_checks(&content, &file_path_str, pattern)?);
            }
        }
    }

    if verbose {
        eprintln!("[{}] scanned: {} findings", file_path_str, findings.len());
    }

    Ok(findings)
}

fn run_semantic_checks(
    content: &str,
    file_path: &str,
    pattern: &VulnerabilityPattern,
) -> Result<Vec<Finding>, Box<dyn std::error::Error>> {
    let mut findings = Vec::new();

    match pattern.id.as_str() {
        "CIR-001" => {
            let public_inputs = extract_public_inputs(content);
            let constrained = find_constrained_signals(content);
            for input in &public_inputs {
                if !constrained.iter().any(|c| c.contains(input.as_str())) {
                    let line_number = find_line_for_signal(content, input);
                    findings.push(Finding {
                        severity: pattern.severity.clone(),
                        category: pattern.category.clone(),
                        title: pattern.name.clone(),
                        description: format!(
                            "Public input '{}' may be under-constrained. {}",
                            input, pattern.description
                        ),
                        file_path: file_path.to_string(),
                        line_number,
                        snippet: Some(format!("public {}", input)),
                        recommendation: pattern.recommendation.clone(),
                    });
                }
            }
        }
        "CIR-002" => {
            if detect_missing_range_checks(content) {
                findings.push(Finding {
                    severity: pattern.severity.clone(),
                    category: pattern.category.clone(),
                    title: pattern.name.clone(),
                    description: pattern.description.clone(),
                    file_path: file_path.to_string(),
                    line_number: None,
                    snippet: None,
                    recommendation: pattern.recommendation.clone(),
                });
            }
        }
        "CIR-003" => {
            let imports = find_unused_imports(content);
            for imp in &imports {
                findings.push(Finding {
                    severity: pattern.severity.clone(),
                    category: pattern.category.clone(),
                    title: pattern.name.clone(),
                    description: format!("Unused import: {}", imp),
                    file_path: file_path.to_string(),
                    line_number: None,
                    snippet: Some(imp.clone()),
                    recommendation: pattern.recommendation.clone(),
                });
            }
        }
        "CIR-006" => {
            let if_blocks = detect_if_without_else(content);
            for (line, snippet) in &if_blocks {
                findings.push(Finding {
                    severity: pattern.severity.clone(),
                    category: pattern.category.clone(),
                    title: pattern.name.clone(),
                    description: pattern.description.clone(),
                    file_path: file_path.to_string(),
                    line_number: Some(*line),
                    snippet: Some(snippet.clone()),
                    recommendation: pattern.recommendation.clone(),
                });
            }
        }
        "SOL-003" => {
            if detect_replay_vulnerability(content) {
                findings.push(Finding {
                    severity: pattern.severity.clone(),
                    category: pattern.category.clone(),
                    title: pattern.name.clone(),
                    description: pattern.description.clone(),
                    file_path: file_path.to_string(),
                    line_number: None,
                    snippet: None,
                    recommendation: pattern.recommendation.clone(),
                });
            }
        }
        _ => {}
    }

    Ok(findings)
}

fn extract_public_inputs(content: &str) -> Vec<String> {
    let re = Regex::new(r"\bdef\s+main\s*\(([^)]*)").unwrap();
    let mut inputs = Vec::new();
    if let Some(cap) = re.captures(content) {
        let params = cap.get(1).map_or("", |m| m.as_str());
        for param in params.split(',') {
            let param = param.trim();
            if !param.contains("private") && !param.is_empty() {
                let name = param.split_whitespace().last().unwrap_or("");
                if !name.is_empty() {
                    inputs.push(name.to_string());
                }
            }
        }
    }
    inputs
}

fn find_constrained_signals(content: &str) -> Vec<String> {
    let re = Regex::new(r"(assert|===|==)\s*\(?[^)]*\)?").unwrap();
    let mut constrained = Vec::new();
    for cap in re.find_iter(content) {
        constrained.push(cap.as_str().to_string());
    }
    constrained
}

fn find_line_for_signal(content: &str, signal: &str) -> Option<usize> {
    for (i, line) in content.lines().enumerate() {
        if line.contains(signal) {
            return Some(i + 1);
        }
    }
    None
}

fn detect_missing_range_checks(content: &str) -> bool {
    let has_field_type = content.contains("field");
    let has_assert = content.contains("assert");
    let has_range_check = content.contains("u8")
        || content.contains("u16")
        || content.contains("u32")
        || content.contains("u64");

    has_field_type && !has_range_check && has_assert
}

fn find_unused_imports(content: &str) -> Vec<String> {
    let re = Regex::new(r#"(?:from\s+)?"([^"]+)"(?:\s+import\s+(\w+))?"#).unwrap();
    let mut unused = Vec::new();
    for cap in re.captures_iter(content) {
        let import_path = cap.get(1).map_or("", |m| m.as_str());
        let symbol = cap.get(2).map_or("main", |m| m.as_str());
        if !content.contains(&format!(" as {}", symbol))
            && !content.contains(&format!(" {}(", symbol))
            && !content.contains(&format!(" {}.", symbol))
        {
            unused.push(import_path.to_string());
        }
    }
    unused
}

fn detect_if_without_else(content: &str) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let re = Regex::new(r"\bif\s+").unwrap();
    for cap in re.find_iter(content) {
        let pos = cap.start();
        let line = content[..pos].lines().count() + 1;
        let rest = &content[pos..];
        let mut depth = 0;
        let mut i = 0;
        let mut has_else = false;
        let bytes = rest.as_bytes();
        while i < bytes.len() {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let after = &rest[i + 1..].trim_start();
                        if after.starts_with("else") {
                            has_else = true;
                        }
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if !has_else {
            let end = rest[..i.min(120)].to_string();
            results.push((line, end));
        }
    }
    results
}

fn detect_replay_vulnerability(content: &str) -> bool {
    let has_verify = content.contains("verifyProof") || content.contains("verifyTx");
    let has_nonce = content.contains("nonce") || content.contains("nullifier");
    has_verify && !has_nonce
}

fn count_lines_before(content: &str, pos: usize) -> Option<usize> {
    Some(content[..pos].lines().count() + 1)
}
