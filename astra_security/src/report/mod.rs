use crate::ScanResult;

pub fn report_terminal(result: &ScanResult) {
    println!();
    println!("=== Security Scan Complete ===");
    println!("Files scanned: {}", result.files_scanned);
    println!("Scan duration: {:.2}ms", result.duration_ms);
    println!();
    println!("Findings by severity:");
    if result.critical > 0 { println!("  CRITICAL: {}", result.critical); }
    if result.high > 0 { println!("  HIGH:     {}", result.high); }
    if result.medium > 0 { println!("  MEDIUM:   {}", result.medium); }
    if result.low > 0 { println!("  LOW:      {}", result.low); }
    if result.info > 0 { println!("  INFO:     {}", result.info); }
    println!("  Total:    {}", result.findings.len());
    println!();

    if !result.findings.is_empty() {
        println!("--- Detailed Findings ---");
        for (i, f) in result.findings.iter().enumerate() {
            println!();
            println!("[{}/{}]", i + 1, result.findings.len());
            println!("  Severity:    {}", f.severity);
            println!("  Category:    {}", f.category);
            println!("  Title:       {}", f.title);
            println!("  File:        {}", f.file);
            if let Some(line) = f.line {
                println!("  Line:        {}", line);
            }
            println!("  Description: {}", f.description);
            println!("  Snippet:     {}", f.snippet);
            println!("  Fix:         {}", f.fix);
        }
    }
}

pub fn report_json(result: &ScanResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string())
}

pub fn report_html(result: &ScanResult) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Astra Security Report</title>
<style>
body {{ font-family: sans-serif; margin: 2em; }}
h1 {{ color: #333; }}
.finding {{ border: 1px solid #ddd; border-radius: 8px; padding: 1em; margin: 1em 0; }}
.critical {{ border-left: 4px solid #d32f2f; }}
.high {{ border-left: 4px solid #f57c00; }}
.medium {{ border-left: 4px solid #fbc02d; }}
.low {{ border-left: 4px solid #7cb342; }}
.info {{ border-left: 4px solid #90a4ae; }}
.severity {{ font-weight: bold; }}
.summary {{ margin: 1em 0; }}
</style></head><body>
<h1>Astra Security Scan Report</h1>
<div class="summary">
<p>Files scanned: {}</p>
<p>Scan duration: {:.2}ms</p>
</div>
<div class="summary">
<h3>Findings by severity</h3>
<ul>"#,
        result.files_scanned, result.duration_ms
    );

    if result.critical > 0 {
        html.push_str(&format!("<li>CRITICAL: {}</li>", result.critical));
    }
    if result.high > 0 {
        html.push_str(&format!("<li>HIGH: {}</li>", result.high));
    }
    if result.medium > 0 {
        html.push_str(&format!("<li>MEDIUM: {}</li>", result.medium));
    }
    if result.low > 0 {
        html.push_str(&format!("<li>LOW: {}</li>", result.low));
    }
    if result.info > 0 {
        html.push_str(&format!("<li>INFO: {}</li>", result.info));
    }
    html.push_str(&format!("<li>Total: {}</li></ul></div>", result.findings.len()));

    for f in &result.findings {
        let cls = f.severity.to_lowercase();
        let line_info = match f.line {
            Some(l) => format!(" (line {})", l),
            None => String::new(),
        };
        html.push_str(&format!(
            r#"<div class="finding {}">
<h3>{}: {}</h3>
<p><span class="severity">{}</span> - {} {}</p>
<p><strong>File:</strong> {}{}</p>
<p><strong>Description:</strong> {}</p>
<pre><code>{}</code></pre>
<p><strong>Fix:</strong> {}</p>
</div>"#,
            cls, f.title, f.severity, f.severity, f.category, line_info,
            f.file, line_info, f.description, f.snippet, f.fix
        ));
    }

    html.push_str("</body></html>");
    html
}
