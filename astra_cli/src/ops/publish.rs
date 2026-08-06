use astra_publish::available_targets;
use std::process;

pub fn exec(matches: &clap::ArgMatches) {
    let target = matches.value_of("target").unwrap_or("evm");
    let available = available_targets();
    if !available.contains(&target) {
        eprintln!(
            "unknown publish target '{}' (available: {})",
            target,
            available.join(", ")
        );
        process::exit(1);
    }
    println!(
        "publish: target '{}' is a compile-time-named skeleton — not yet implemented in v0.0",
        target
    );
    println!("  targets: {}", available.join(", "));
}
