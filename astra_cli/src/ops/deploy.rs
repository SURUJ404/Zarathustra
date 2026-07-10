use std::fs;

pub fn exec(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("init", Some(m)) => run_init(m),
        ("verifier", Some(m)) => run_verifier(m),
        ("verify", Some(_)) => println!("On-chain verification simulation (not yet implemented)"),
        _ => println!("deploy subcommands: init, verifier, verify"),
    }
}

fn run_init(m: &clap::ArgMatches) {
    let name = m.value_of("name").unwrap_or("project");
    let dir = format!("{}/src", name);
    fs::create_dir_all(&dir).unwrap_or_else(|e| { eprintln!("create dir error: {}", e); std::process::exit(1); });
    let circuit = "def main(field a, field b) -> field {\n    field c = a * b;\n    return c;\n}\n";
    let path = format!("{}/src/main.zara", name);
    fs::write(&path, circuit).unwrap_or_else(|e| { eprintln!("write error: {}", e); std::process::exit(1); });
    println!("Initialized project: {}", name);
    println!("  {} - circuit source", path);
}

fn run_verifier(m: &clap::ArgMatches) {
    let path = m.value_of("output").unwrap_or("Verifier.sol");
    let code = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Verifier {
    function verifyProof(
        uint[2] memory a,
        uint[2][2] memory b,
        uint[2] memory c,
        uint[1] memory input
    ) public view returns (bool) {
        // Placeholder: real pairing check goes here
        return true;
    }
}
"#;
    fs::write(path, code).unwrap_or_else(|e| { eprintln!("write error: {}", e); std::process::exit(1); });
    println!("Verifier contract written to {}", path);
}
