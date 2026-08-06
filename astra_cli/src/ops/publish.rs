use astra_publish::{available_targets, evm_render, jt_from_artifacts};
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

    match target {
        "jt" => publish_jt(),
        "evm" => publish_evm(),
        other => {
            eprintln!(
                "publish: target '{}' is not implemented yet. \
                 `jt` (JSON test vectors) is available now; EVM refuses non-BN254 curves.",
                other
            );
            process::exit(1);
        }
    }
}

fn read_artifacts() -> Result<(serde_json::Value, serde_json::Value), String> {
    let proof_txt = std::fs::read_to_string("proof.json")
        .map_err(|e| format!("cannot read proof.json (run `astra prove prove` first): {e}"))?;
    let vk_txt = std::fs::read_to_string("vk.json")
        .map_err(|e| format!("cannot read vk.json (run `astra prove setup` first): {e}"))?;
    let proof: serde_json::Value =
        serde_json::from_str(&proof_txt).map_err(|e| format!("parse proof.json: {e}"))?;
    let vk: serde_json::Value =
        serde_json::from_str(&vk_txt).map_err(|e| format!("parse vk.json: {e}"))?;
    Ok((proof, vk))
}

fn publish_jt() {
    let (proof, vk) = match read_artifacts() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };
    let vectors = match jt_from_artifacts(&proof, &vk) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };
    let json = serde_json::to_string_pretty(&vectors).unwrap_or_default();
    std::fs::write("zara_test_vectors.json", &json).unwrap_or_else(|e| {
        eprintln!("Error: cannot write zara_test_vectors.json: {e}");
        process::exit(1);
    });
    println!("publish: wrote zara_test_vectors.json");
    println!(
        "  curve:   {}",
        vectors.get("curve").and_then(|x| x.as_str()).unwrap_or("")
    );
    println!(
        "  protocol: {}",
        vectors
            .get("protocol")
            .and_then(|x| x.as_str())
            .unwrap_or("")
    );
}

fn publish_evm() {
    let (proof, _vk) = match read_artifacts() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };
    match evm_render(&proof) {
        Ok(_) => {
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
