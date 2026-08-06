use std::path::Path;

use astra_codegen::Backend;
use astra_codegen::{compile, validate_constraints};
use astra_ir::types::{scalar_display, scalar_from_dec_str, ConstraintSystem};
use astra_prover::backend;
use bls12_381::Scalar;

pub fn exec(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("compile", Some(m)) => run_compile(m),
        ("setup", Some(m)) => run_setup(m),
        ("prove", Some(m)) => run_prove(m),
        ("verify", Some(m)) => run_verify(m),
        _ => println!("prove subcommands: compile, setup, prove, verify"),
    }
}

fn parse_inputs(s: &str) -> Result<Vec<Scalar>, String> {
    s.split(',')
        .filter(|x| !x.is_empty())
        .map(|x| scalar_from_dec_str(x.trim()))
        .collect()
}

fn read_source(m: &clap::ArgMatches) -> Result<String, String> {
    let path = m.value_of("input").ok_or("missing input argument")?;
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path, e))
}

fn backend_for(m: &clap::ArgMatches) -> Box<dyn Backend> {
    let name = m.value_of("backend").unwrap_or("");
    if name.is_empty() {
        return backend::default_backend();
    }
    match backend::by_name(name) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn compile_or_exit(m: &clap::ArgMatches) -> ConstraintSystem {
    let source = read_source(m).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    let public = parse_inputs(m.value_of("public").unwrap_or("")).unwrap_or_else(|e| {
        eprintln!("Error: invalid public input: {}", e);
        std::process::exit(1);
    });
    let private = parse_inputs(m.value_of("private").unwrap_or("")).unwrap_or_else(|e| {
        eprintln!("Error: invalid private input: {}", e);
        std::process::exit(1);
    });
    let cs = compile(&source, &public, &private).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });
    if let Err(e) = validate_constraints(&cs) {
        eprintln!("Error: {} — circuit is not satisfied", e);
        std::process::exit(1);
    }
    cs
}

fn show_cs(cs: &ConstraintSystem) {
    println!("=== Circuit ===");
    println!("  Variables:   {}", cs.num_variables);
    println!("  Public:      {}", cs.num_public);
    println!("  Private:     {}", cs.num_private);
    println!("  Constraints: {}", cs.num_constraints);
    let w = &cs.witness;
    let labels = ["~one", "a", "b", "c"];
    for (i, val) in w.iter().enumerate() {
        let lbl = labels.get(i).unwrap_or(&"");
        println!("    w[{}] {} = {}", i, lbl, scalar_display(val));
    }
}

fn run_compile(m: &clap::ArgMatches) {
    let cs = compile_or_exit(m);
    show_cs(&cs);
}

fn run_setup(m: &clap::ArgMatches) {
    let cs = compile_or_exit(m);
    let b = backend_for(m);
    let dir = Path::new(".");
    b.setup(&cs, dir).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });
    println!("=== Trusted Setup Complete ===");
    println!("  Backend:     {}", b.name());
    println!("  Curve:       {}", b.curve());
    println!("  CRS generated for {} constraints", cs.num_constraints);
    println!("  Wrote pk.json, vk.json");
}

fn run_prove(m: &clap::ArgMatches) {
    let cs = compile_or_exit(m);
    let b = backend_for(m);
    let dir = Path::new(".");
    b.prove(&cs, dir).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    println!("=== Proof Generated ===");
    println!("  Backend:     {}", b.name());
    println!("  Curve:       {}", b.curve());
    let proof_path = b.proof_path(dir);
    match proof_summary(&proof_path) {
        Some((a, bstr, cstr, public)) => {
            println!("  A:           {}", a);
            println!("  B:           {}", bstr);
            println!("  C:           {}", cstr);
            println!("  Public:      {}", public.join(", "));
            println!("  Saved to {}", proof_path.display());
        }
        None => println!("  Saved to {}", proof_path.display()),
    }
}

fn proof_summary(path: &Path) -> Option<(String, String, String, Vec<String>)> {
    let text = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let a = v
        .get("a")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let b = v
        .get("b")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let c = v
        .get("c")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let public = v
        .get("public")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    Some((a, b, c, public))
}

fn read_bound_public(dir: &Path) -> Vec<Scalar> {
    let text = match std::fs::read_to_string(dir.join("proof.json")) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let v: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    v.get("public")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.as_str())
                .filter_map(|s| scalar_from_dec_str(s).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn run_verify(m: &clap::ArgMatches) {
    let b = backend_for(m);
    let dir = Path::new(".");
    let proof_path = b.proof_path(dir);
    if !proof_path.exists() {
        eprintln!(
            "Error: {} not found — run `astra prove prove` (with the same source) first",
            proof_path.display()
        );
        std::process::exit(1);
    }

    let bound = read_bound_public(dir);
    let provided = parse_inputs(m.value_of("public").unwrap_or("")).unwrap_or_else(|e| {
        eprintln!("Error: invalid public input: {}", e);
        std::process::exit(1);
    });
    if !provided.is_empty() && provided != bound {
        eprintln!(
            "warning: provided public inputs differ from the inputs bound in proof.json; \
             verifying the proof against the bound inputs"
        );
    }

    let result = b.verify(dir).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    println!("=== Verification ===");
    println!("  Backend:     {}", b.name());
    println!("  Proof:       {}", proof_path.display());
    if result {
        println!("  ✓ PROOF VERIFIED");
    } else {
        println!("  ✗ PROOF REJECTED");
        std::process::exit(1);
    }
}
