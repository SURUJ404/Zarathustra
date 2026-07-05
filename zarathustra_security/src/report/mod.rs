use crate::Finding;

pub struct SecurityReport {
    pub findings: Vec<Finding>,
    pub scan_time: std::time::Duration,
    pub files_scanned: usize,
}

impl SecurityReport {
    pub fn new(
        findings: Vec<Finding>,
        scan_time: std::time::Duration,
        files_scanned: usize,
    ) -> Self {
        Self {
            findings,
            scan_time,
            files_scanned,
        }
    }

    pub fn print_summary(&self) {
        let critical = self
            .findings
            .iter()
            .filter(|f| matches!(f.severity, crate::Severity::Critical))
            .count();
        let high = self
            .findings
            .iter()
            .filter(|f| matches!(f.severity, crate::Severity::High))
            .count();
        let medium = self
            .findings
            .iter()
            .filter(|f| matches!(f.severity, crate::Severity::Medium))
            .count();
        let low = self
            .findings
            .iter()
            .filter(|f| matches!(f.severity, crate::Severity::Low))
            .count();
        let info = self
            .findings
            .iter()
            .filter(|f| matches!(f.severity, crate::Severity::Info))
            .count();

        println!("\n=== Security Scan Complete ===");
        println!("Files scanned: {}", self.files_scanned);
        println!("Scan duration: {:.2?}", self.scan_time);
        println!("\nFindings by severity:");
        if critical > 0 {
            println!("  CRITICAL: {}", critical);
        }
        if high > 0 {
            println!("  HIGH:     {}", high);
        }
        if medium > 0 {
            println!("  MEDIUM:   {}", medium);
        }
        if low > 0 {
            println!("  LOW:      {}", low);
        }
        if info > 0 {
            println!("  INFO:     {}", info);
        }
        println!("  Total:    {}", self.findings.len());
    }

    pub fn print_detailed(&self) {
        self.print_summary();
        if self.findings.is_empty() {
            println!("\nNo vulnerabilities found.");
            return;
        }
        println!("\n--- Detailed Findings ---");
        for (i, finding) in self.findings.iter().enumerate() {
            println!("\n[{}/{}]", i + 1, self.findings.len());
            println!("  Severity:    {}", finding.severity);
            println!("  Category:    {}", finding.category);
            println!("  Title:       {}", finding.title);
            println!("  File:        {}", finding.file_path);
            if let Some(line) = finding.line_number {
                println!("  Line:        {}", line);
            }
            println!("  Description: {}", finding.description);
            if let Some(ref snippet) = finding.snippet {
                println!("  Snippet:     {}", snippet);
            }
            println!("  Fix:         {}", finding.recommendation);
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.findings)
    }

    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html><html><head><title>Security Audit Report</title>");
        html.push_str("<style>body{font-family:sans-serif;margin:40px;background:#0d1117;color:#c9d1d9}");
        html.push_str("table{border-collapse:collapse;width:100%}");
        html.push_str("th,td{padding:12px;text-align:left;border-bottom:1px solid #30363d}");
        html.push_str("th{background:#161b22}.critical{color:#f85149}.high{color:#d29922}");
        html.push_str(".medium{color:#58a6ff}.low{color:#8b949e}.info{color:#6e7681}</style></head><body>");
        html.push_str(&format!(
            "<h1>Security Audit Report</h1>"
        ));
        html.push_str(&format!(
            "<p>Files scanned: {} | Duration: {:.2?} | Total findings: {}</p>",
            self.files_scanned,
            self.scan_time,
            self.findings.len()
        ));

        html.push_str("<table><tr><th>Severity</th><th>Category</th><th>Title</th><th>File</th><th>Line</th></tr>");
        for finding in &self.findings {
            let severity_class = format!("{}", finding.severity).to_lowercase();
            html.push_str(&format!(
                "<tr><td class=\"{}\">{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                severity_class,
                finding.severity,
                finding.category,
                finding.title,
                finding.file_path,
                finding.line_number.map_or("-".into(), |l| l.to_string())
            ));
        }
        html.push_str("</table></body></html>");
        html
    }
}

pub fn format_finding(finding: &Finding, ansi: bool) -> String {
    let severity_tag = if ansi {
        match finding.severity {
            crate::Severity::Critical => "\x1b[31m[CRITICAL]\x1b[0m",
            crate::Severity::High => "\x1b[33m[HIGH]\x1b[0m",
            crate::Severity::Medium => "\x1b[34m[MEDIUM]\x1b[0m",
            crate::Severity::Low => "\x1b[36m[LOW]\x1b[0m",
            crate::Severity::Info => "\x1b[90m[INFO]\x1b[0m",
        }
    } else {
        &format!("[{}]", finding.severity)
    };

    format!(
        "{} {}: {} ({}:{})",
        severity_tag, finding.category, finding.title, finding.file_path,
        finding.line_number.map_or("-".into(), |l| l.to_string())
    )
}
