//! Backend registry — concrete implementations of [`astra_codegen::Backend`].
//!
//! `ark-groth16` (the community-audited arkworks Groth16 over BLS12-381) is
//! the default. `legacy` keeps the hand-rolled Groth16 reachable behind an
//! explicit flag for reference/education; it is fully honest too — it persists
//! its own keys and proof and verifies that proof.

use std::fs;
use std::path::Path;

use astra_codegen::Backend;
use astra_ir::types::ConstraintSystem;

use crate::groth16 as ark16;
use crate::legacy as legacy_impl;

pub struct ArkGroth16;

pub struct LegacyBackend;

pub fn default_backend() -> Box<dyn Backend> {
    Box::new(ArkGroth16)
}

pub fn registry_names() -> Vec<&'static str> {
    vec!["ark-groth16", "legacy"]
}

pub fn by_name(name: &str) -> Result<Box<dyn Backend>, String> {
    match name {
        "ark" | "groth16" | "ark-groth16" => Ok(Box::new(ArkGroth16)),
        "legacy" => Ok(Box::new(LegacyBackend)),
        other => Err(format!(
            "unknown backend: {} (available: {}; default: ark-groth16)",
            other,
            registry_names().join(", ")
        )),
    }
}

fn ensure_dir(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    Ok(())
}

impl Backend for ArkGroth16 {
    fn name(&self) -> &'static str {
        "ark-groth16"
    }

    fn curve(&self) -> &'static str {
        ark16::DEFAULT_CURVE
    }

    fn setup(&self, cs: &ConstraintSystem, dir: &Path) -> Result<(), String> {
        ensure_dir(dir)?;
        let (pk, vk) = ark16::setup(cs)?;
        ark16::write_pk_json(dir, &pk)?;
        ark16::write_vk_json(dir, &vk)?;
        Ok(())
    }

    fn prove(&self, cs: &ConstraintSystem, dir: &Path) -> Result<(), String> {
        ensure_dir(dir)?;
        let pk_path = dir.join(ark16::PK_FILE);
        if !pk_path.exists() {
            // Convenience: proving without an explicit `setup` derives the key.
            let (pk, vk) = ark16::setup(cs)?;
            ark16::write_pk_json(dir, &pk)?;
            ark16::write_vk_json(dir, &vk)?;
        }
        let pk = ark16::read_pk_json(dir)?;
        let proof = ark16::prove(cs, &pk)?;
        let public = ark16::public_inputs_of(cs);
        ark16::write_proof_json(dir, &proof, &public)?;
        Ok(())
    }

    fn verify(&self, dir: &Path) -> Result<bool, String> {
        ensure_dir(dir)?;
        let vk = ark16::read_vk_json(dir)?;
        let (proof, public) = ark16::read_proof_json(dir)?;
        ark16::verify(&vk, &public, &proof)
    }
}

impl Backend for LegacyBackend {
    fn name(&self) -> &'static str {
        "legacy-groth16"
    }

    fn curve(&self) -> &'static str {
        ark16::DEFAULT_CURVE
    }

    fn setup(&self, cs: &ConstraintSystem, dir: &Path) -> Result<(), String> {
        ensure_dir(dir)?;
        let (pk, vk) = legacy_impl::setup(cs);
        write_json_value(&dir.join(ark16::PK_FILE), &legacy_impl::pk_to_value(&pk))?;
        write_json_value(&dir.join(ark16::VK_FILE), &legacy_impl::vk_to_value(&vk))?;
        Ok(())
    }

    fn prove(&self, cs: &ConstraintSystem, dir: &Path) -> Result<(), String> {
        ensure_dir(dir)?;
        let pk_path = dir.join(ark16::PK_FILE);
        if !pk_path.exists() {
            let (pk, vk) = legacy_impl::setup(cs);
            write_json_value(&pk_path, &legacy_impl::pk_to_value(&pk))?;
            write_json_value(&dir.join(ark16::VK_FILE), &legacy_impl::vk_to_value(&vk))?;
        }
        let pk_value = read_json(&pk_path)?;
        let pk = legacy_impl::pk_from_value(&pk_value)?;
        let proof = legacy_impl::prove(&pk, cs);
        let public = ark16::public_inputs_of(cs);
        write_json_value(
            &dir.join(ark16::PROOF_FILE),
            &legacy_impl::proof_to_value(&proof, &public),
        )?;
        Ok(())
    }

    fn verify(&self, dir: &Path) -> Result<bool, String> {
        ensure_dir(dir)?;
        let vk_value = read_json(&dir.join(ark16::VK_FILE))?;
        let vk = legacy_impl::vk_from_value(&vk_value)?;
        let proof_value = read_json(&dir.join(ark16::PROOF_FILE))?;
        let (proof, public) = legacy_impl::proof_from_value(&proof_value)?;
        Ok(legacy_impl::verify(&vk, &public, &proof))
    }
}

fn write_json_value(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| format!("encode json: {e}"))?;
    fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}
