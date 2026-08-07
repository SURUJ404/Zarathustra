use serde::{Deserialize, Serialize};

pub mod analyzers;
pub mod patterns;
pub mod report;
pub use report::{report_html, report_json, report_terminal};

use std::time::Instant;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AllowList {
    /// Findings to exempt from the scan. Each entry suppresses findings in a
    /// specific file that match the given (optional) category/title/line.
    #[serde(default)]
    pub allow: Vec<AllowEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllowEntry {
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub line: Option<usize>,
}

impl AllowEntry {
    fn matches(&self, f: &Finding) -> bool {
        if let Some(file) = &self.file {
            let needle = file.replace('\\', "/");
            let hay = f.file.replace('\\', "/");
            if !hay.ends_with(needle.as_str()) {
                return false;
            }
        }
        if let Some(category) = &self.category {
            if f.category != *category {
                return false;
            }
        }
        if let Some(title) = &self.title {
            if f.title != *title {
                return false;
            }
        }
        if let Some(line) = self.line {
            if f.line != Some(line) {
                return false;
            }
        }
        true
    }
}

impl AllowList {
    fn is_allowed(&self, f: &Finding) -> bool {
        // An entry with no matchers at all would exempt everything — guard.
        let specific = self.allow.iter().any(|e| {
            e.file.is_some() || e.category.is_some() || e.title.is_some() || e.line.is_some()
        });
        if !specific || self.allow.is_empty() {
            return false;
        }
        self.allow.iter().any(|e| e.matches(f))
    }
}

impl AllowList {
    pub fn from_config<S: AsRef<std::path::Path>>(path: S) -> Option<AllowList> {
        let p = path.as_ref();
        if !p.exists() {
            return None;
        }
        let text = std::fs::read_to_string(p).ok()?;
        serde_json::from_str(&text).ok()
    }
}

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

fn walk_dir(
    dir: &std::path::Path,
    findings: &mut Vec<Finding>,
    files_scanned: &mut usize,
    verbose: bool,
) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if name == "target" || name == ".git" || name == "node_modules" {
                    continue;
                }
                walk_dir(&p, findings, files_scanned, verbose);
            } else if let Some(ext) = p.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "zara" || ext == "rs" || ext == "sol" {
                    if verbose {
                        eprintln!("[{}] scanning...", p.display());
                    }
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
            walk_dir(
                std::path::Path::new(path),
                &mut findings,
                &mut files_scanned,
                verbose,
            );
        } else {
            files_scanned = 1;
            if let Ok(code) = std::fs::read_to_string(path) {
                let ext = path.rsplit('.').next().unwrap_or("").to_string();
                findings = analyzers::analyze(&code, &ext, path);
            }
        }
    }

    // Apply the allow-list (if any) so sanctioned findings — e.g. deliberate
    // `unsafe` in the WASM ABI glue — don't block deploys.
    if let Some(allow) = AllowList::from_config(".astra-audit.json") {
        findings.retain(|f| !allow.is_allowed(f));
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
        critical,
        high,
        medium,
        low,
        info,
        findings,
    }
}
