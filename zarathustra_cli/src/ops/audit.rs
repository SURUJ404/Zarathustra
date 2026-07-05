use clap::{App, Arg, SubCommand};
use std::time::Instant;
use zarathustra_security::report::SecurityReport;
use zarathustra_security::analyzers::scanner;
use zarathustra_security::ScanConfig;

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name("audit")
        .about("Security audit for circuits and smart contracts")
        .arg(
            Arg::with_name("target")
                .help("Target file or directory to scan")
                .default_value(".")
                .index(1),
        )
        .arg(
            Arg::with_name("format")
                .long("format")
                .short("f")
                .help("Output format (terminal, json, html)")
                .default_value("terminal")
                .possible_values(&["terminal", "json", "html"]),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .takes_value(true)
                .help("Output file path (for json/html formats)"),
        )
        .arg(
            Arg::with_name("exclude")
                .long("exclude")
                .short("e")
                .help("Exclude directory pattern")
                .multiple(true),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .help("Verbose output"),
        )
}

pub fn exec(matches: &clap::ArgMatches) -> Result<(), String> {
    let target = matches.value_of("target").unwrap_or(".");
    let format = matches.value_of("format").unwrap_or("terminal");
    let verbose = matches.is_present("verbose");
    let excludes: Vec<String> = matches
        .values_of("exclude")
        .map(|v| v.map(|s| s.to_string()).collect())
        .unwrap_or_default();

    let mut config = ScanConfig::default();
    config.target_paths = vec![target.to_string()];
    config.excluded_paths.extend(excludes);
    config.verbose = verbose;

    let start = Instant::now();
    println!("Scanning {} for vulnerabilities...", target);

    let findings = scanner::scan_directory(&config).map_err(|e| format!("Scan error: {}", e))?;

    let duration = start.elapsed();
    let files_scanned = zarathustra_security::collect_target_files(&config)
        .map(|f| f.len())
        .unwrap_or(0);

    let report = SecurityReport::new(findings, duration, files_scanned);

    match format {
        "json" => {
            let json = report.to_json().map_err(|e| format!("JSON error: {}", e))?;
            if let Some(path) = matches.value_of("output") {
                std::fs::write(path, &json)
                    .map_err(|e| format!("Write error: {}", e))?;
                println!("Report saved to {}", path);
            } else {
                println!("{}", json);
            }
        }
        "html" => {
            let html = report.to_html();
            if let Some(path) = matches.value_of("output") {
                std::fs::write(path, &html)
                    .map_err(|e| format!("Write error: {}", e))?;
                println!("Report saved to {}", path);
            } else {
                println!("{}", html);
            }
        }
        _ => {
            report.print_detailed();
        }
    }

    Ok(())
}
