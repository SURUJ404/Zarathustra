pub mod analyzers;
pub mod patterns;
pub mod report;

pub use analyzers::*;
pub use patterns::*;
pub use report::*;

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub target_paths: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub max_file_size: u64,
    pub verbose: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            target_paths: vec![".".to_string()],
            excluded_paths: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
            ],
            max_file_size: 1_048_576,
            verbose: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub category: String,
    pub title: String,
    pub description: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub snippet: Option<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

pub fn collect_target_files(
    config: &ScanConfig,
) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for target in &config.target_paths {
        let path = Path::new(target);
        if !path.exists() {
            continue;
        }

        if path.is_file() {
            if is_scanable_file(path) {
                files.push(path.to_path_buf());
            }
            continue;
        }

        let walker = walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                !config.excluded_paths.contains(&name.to_string())
            });

        for entry in walker.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() && is_scanable_file(entry_path) {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.len() <= config.max_file_size {
                        files.push(entry_path.to_path_buf());
                    }
                }
            }
        }
    }

    Ok(files)
}

fn is_scanable_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("zara") | Some("sol") | Some("rs") | Some("json") | Some("yaml") | Some("yml") => true,
        _ => false,
    }
}

pub fn run_security_scan(
    config: &ScanConfig,
) -> Result<Vec<Finding>, Box<dyn std::error::Error>> {
    let files = collect_target_files(config)?;
    let mut findings = Vec::new();

    for file_path in &files {
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "zara" | "rs" => {
                findings.extend(analyzers::circuit::analyze_circuit(
                    file_path,
                    config.verbose,
                )?);
            }
            "sol" => {
                findings.extend(analyzers::solidity::analyze_solidity(
                    file_path,
                    config.verbose,
                )?);
            }
            _ => {}
        }
    }

    Ok(findings)
}
