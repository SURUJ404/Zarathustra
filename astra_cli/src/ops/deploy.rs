use std::fs;

pub fn exec(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("init", Some(m)) => run_init(m),
        ("verifier", Some(_)) => run_verifier(),
        ("verify", Some(_)) => println!("On-chain verification simulation (not yet implemented)"),
        _ => println!("deploy subcommands: init, verifier, verify"),
    }
}

fn run_init(m: &clap::ArgMatches) {
    let name = m.value_of("name").unwrap_or("project");
    let dir = format!("{}/src", name);
    fs::create_dir_all(&dir).unwrap_or_else(|e| {
        eprintln!("create dir error: {}", e);
        std::process::exit(1);
    });
    let circuit = "def main(field a, field b) -> field {\n    field c = a * b;\n    return c;\n}\n";
    let path = format!("{}/src/main.zara", name);
    fs::write(&path, circuit).unwrap_or_else(|e| {
        eprintln!("write error: {}", e);
        std::process::exit(1);
    });
    println!("Initialized project: {}", name);
    println!("  {} - circuit source", path);
}

/// Verifier export must NOT write a stub contract.
///
/// The old implementation emitted a hardcoded `Verifier.sol` that always
/// returned `true` — a security-critical fake. Real per-circuit export lives in
/// `astra publish -t evm`; until it is implemented we fail loudly instead.
fn run_verifier() {
    eprintln!("verifier export is not implemented yet — use `astra publish -t evm` once available");
    std::process::exit(1);
}
