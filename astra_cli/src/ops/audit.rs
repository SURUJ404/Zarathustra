use astra_security::scan_path;
use std::process;

pub fn exec(matches: &clap::ArgMatches) {
    let target = matches.value_of("target").unwrap_or(".");
    let verbose = matches.is_present("verbose");
    let format = matches.value_of("format").unwrap_or("terminal");
    let output = matches.value_of("output");
    let deny = matches.value_of("deny");

    let result = scan_path(target, verbose);

    if format == "json" {
        let json = astra_security::report_json(&result);
        if let Some(path) = output {
            std::fs::write(path, &json).unwrap_or_else(|e| { eprintln!("write error: {}", e); });
        } else {
            println!("{}", json);
        }
    } else if format == "html" {
        let html = astra_security::report_html(&result);
        if let Some(path) = output {
            std::fs::write(path, &html).unwrap_or_else(|e| { eprintln!("write error: {}", e); });
        } else {
            println!("{}", html);
        }
    } else {
        astra_security::report_terminal(&result);
    }

    if let Some(level) = deny {
        let has_denied = match level {
            "critical" => result.critical > 0,
            "high" => result.critical > 0 || result.high > 0,
            "medium" => result.critical > 0 || result.high > 0 || result.medium > 0,
            _ => result.critical > 0 || result.high > 0 || result.medium > 0 || result.low > 0 || result.info > 0,
        };
        if has_denied {
            eprintln!("[deny] found findings at '{}' severity or above — aborting", level);
            process::exit(1);
        }
    }
}
