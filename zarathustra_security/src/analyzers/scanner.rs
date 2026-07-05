use crate::{collect_target_files, Finding, ScanConfig};

pub fn scan_directory(config: &ScanConfig) -> Result<Vec<Finding>, Box<dyn std::error::Error>> {
    let files = collect_target_files(config)?;
    let mut all_findings = Vec::new();

    for file_path in &files {
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "zara" | "rs" => {
                match super::circuit::analyze_circuit(file_path, config.verbose) {
                    Ok(mut findings) => all_findings.append(&mut findings),
                    Err(e) => {
                        if config.verbose {
                            eprintln!("Error scanning {}: {}", file_path.display(), e);
                        }
                    }
                }
            }
            "sol" => {
                match super::solidity::analyze_solidity(file_path, config.verbose) {
                    Ok(mut findings) => all_findings.append(&mut findings),
                    Err(e) => {
                        if config.verbose {
                            eprintln!("Error scanning {}: {}", file_path.display(), e);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    all_findings.sort_by(|a, b| {
        let severity_order = |s: &crate::Severity| -> u8 {
            match s {
                crate::Severity::Critical => 0,
                crate::Severity::High => 1,
                crate::Severity::Medium => 2,
                crate::Severity::Low => 3,
                crate::Severity::Info => 4,
            }
        };
        severity_order(&a.severity).cmp(&severity_order(&b.severity))
    });

    Ok(all_findings)
}
