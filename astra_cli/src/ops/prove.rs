use astra_core::{compile, setup, prove, verify, validate_constraints, scalar_from_dec_str, scalar_display};
use bls12_381::Scalar;
use ff::Field;

pub fn exec(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("compile", Some(m)) => run_compile(m),
        ("setup", Some(m)) => run_setup(m),
        ("prove", Some(m)) => run_prove(m),
        ("verify", Some(m)) => run_verify(m),
        _ => println!("prove subcommands: compile, setup, prove, verify"),
    }
}

fn parse_inputs(s: &str) -> Vec<Scalar> {
    s.split(',')
        .filter(|x| !x.is_empty())
        .map(|x| scalar_from_dec_str(x.trim()).unwrap_or(Scalar::ZERO))
        .collect()
}

fn read_source(m: &clap::ArgMatches) -> Result<String, String> {
    let path = m.value_of("input").ok_or("missing input argument")?;
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path, e))
}

fn show_cs(cs: &astra_core::ConstraintSystem) {
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
    let source = read_source(m).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
    let public = parse_inputs(m.value_of("public").unwrap_or(""));
    let private = parse_inputs(m.value_of("private").unwrap_or(""));
    let cs = compile(&source, &public, &private).unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });
    show_cs(&cs);
}

fn run_setup(m: &clap::ArgMatches) {
    let source = read_source(m).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
    let public = parse_inputs(m.value_of("public").unwrap_or(""));
    let private = parse_inputs(m.value_of("private").unwrap_or(""));
    let cs = compile(&source, &public, &private).unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });
    let (_pk, _vk) = setup(&cs);
    println!("=== Trusted Setup Complete ===");
    println!("  CRS generated for {} constraints", cs.num_constraints);
}

fn run_prove(m: &clap::ArgMatches) {
    let source = read_source(m).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
    let public = parse_inputs(m.value_of("public").unwrap_or(""));
    let private = parse_inputs(m.value_of("private").unwrap_or(""));
    let cs = compile(&source, &public, &private).unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });
    if let Err(e) = validate_constraints(&cs) {
        eprintln!("Error: {} — cannot generate proof", e);
        std::process::exit(1);
    }
    let (pk, _vk) = setup(&cs);
    let proof = prove(&pk, &cs);

    println!("=== Proof Generated ===");
    println!("  A: {:?}", proof.a);
    println!("  B: {:?}", proof.b);
    println!("  C: {:?}", proof.c);

    let proof_json = serde_json::json!({
        "a": format!("{:?}", proof.a),
        "b": format!("{:?}", proof.b),
        "c": format!("{:?}", proof.c),
    });
    if let Ok(json) = serde_json::to_string_pretty(&proof_json) {
        if std::fs::write("proof.json", &json).is_ok() {
            println!("  Saved to proof.json");
        }
    }
}

fn run_verify(m: &clap::ArgMatches) {
    let source = read_source(m).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
    let public = parse_inputs(m.value_of("public").unwrap_or(""));
    let private = parse_inputs(m.value_of("private").unwrap_or(""));
    let cs = compile(&source, &public, &private).unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });
    if let Err(e) = validate_constraints(&cs) {
        eprintln!("Error: {} — cannot verify", e);
        std::process::exit(1);
    }
    let (pk, vk) = setup(&cs);
    let proof = prove(&pk, &cs);

    let result = verify(&vk, &public, &proof);

    println!("=== Verification ===");
    if result {
        println!("  ✓ PROOF VERIFIED");
    } else {
        println!("  ✗ PROOF REJECTED");
    }
}
