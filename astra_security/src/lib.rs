use serde::{Serialize, Deserialize};

pub mod patterns;
pub mod analyzers;
pub mod report;
pub use report::{report_terminal, report_json, report_html};

use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: String,
    pub category: String,
    pub title: String,
    pub file: String,
    pub line: Option<usize>,
    pub description: String,
    pub snippet: String,
    pub fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub files_scanned: usize,
    pub duration_ms: f64,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
    pub critical: usize,
    pub findings: Vec<Finding>,
}

fn walk_dir(dir: &std::path::Path, findings: &mut Vec<Finding>, files_scanned: &mut usize, verbose: bool) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                if name == "target" || name == ".git" || name == "node_modules" {
                    continue;
                }
                walk_dir(&p, findings, files_scanned, verbose);
            } else if let Some(ext) = p.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "zara" || ext == "rs" || ext == "sol" {
                    if verbose { eprintln!("[{}] scanning...", p.display()); }
                    *files_scanned += 1;
                    if let Ok(code) = std::fs::read_to_string(&p) {
                        let f = analyzers::analyze(&code, &ext, &p.to_string_lossy());
                        findings.extend(f);
                    }
                }
            }
        }
    }
}

pub fn scan_path(path: &str, verbose: bool) -> ScanResult {
    let start = Instant::now();
    let mut findings = Vec::new();
    let mut files_scanned = 0;

    let meta = std::fs::metadata(path);
    if let Ok(m) = meta {
        if m.is_dir() {
            walk_dir(std::path::Path::new(path), &mut findings, &mut files_scanned, verbose);
        } else {
            files_scanned = 1;
            if let Ok(code) = std::fs::read_to_string(path) {
                let ext = path.rsplit('.').next().unwrap_or("").to_string();
                findings = analyzers::analyze(&code, &ext, path);
            }
        }
    }

    let duration = start.elapsed().as_secs_f64() * 1000.0;
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;
    let mut info = 0;
    for f in &findings {
        match f.severity.as_str() {
            "CRITICAL" => critical += 1,
            "HIGH" => high += 1,
            "MEDIUM" => medium += 1,
            "LOW" => low += 1,
            _ => info += 1,
        }
    }

    ScanResult {
        files_scanned,
        duration_ms: duration,
        critical, high, medium, low, info,
        findings,
    }
}
