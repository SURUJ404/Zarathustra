use astra_security::scan_path;

pub fn exec(matches: &clap::ArgMatches) {
    let target = matches.value_of("target").unwrap_or(".");
    let verbose = matches.is_present("verbose");
    let format = matches.value_of("format").unwrap_or("terminal");
    let output = matches.value_of("output");

    let findings = scan_path(target, verbose);

    if format == "json" {
        let json = astra_security::report_json(&findings);
        if let Some(path) = output {
            std::fs::write(path, &json).unwrap_or_else(|e| { eprintln!("write error: {}", e); });
        } else {
            println!("{}", json);
        }
    } else if format == "html" {
        let html = astra_security::report_html(&findings);
        if let Some(path) = output {
            std::fs::write(path, &html).unwrap_or_else(|e| { eprintln!("write error: {}", e); });
        } else {
            println!("{}", html);
        }
    } else {
        astra_security::report_terminal(&findings);
    }
}
