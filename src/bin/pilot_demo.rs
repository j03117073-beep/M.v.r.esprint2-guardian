use m_v_r_esprint1::ir_codegen::IRInput;
use m_v_r_esprint1::sovereign_kernel::{signer_from_env, AttestationRecord, SovereignKernel, SovereignKernelConfig};
use m_v_r_esprint1::universal_frontend::IRModule;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("SIGNER_MODE", "simulation");

    let signer = signer_from_env().map_err(|e| format!("{e:?}"))?;
    let config = SovereignKernelConfig { max_ticks: 100 };
    let mut kernel = SovereignKernel::new(signer, config);

    let ir_module = IRModule {
        functions: vec![],
        constants: vec![],
    };

    let input = IRInput {
        args: HashMap::new(),
    };

    for _ in 0..3 {
        let _ = kernel
            .execute_foreign(&ir_module, input.clone())
            .map_err(|e| format!("{e:?}"))?;
    }

    let records = build_demo_records(3);
    let path = "pilot_attestation_log.json";
    let json = serde_json::to_string_pretty(&records)?;
    fs::write(path, json)?;

    println!("Generated pilot attestation log with {} records", records.len());

    let verifier_output = Command::new("cargo")
        .args(["run", "--bin", "verifier", path])
        .output()?;

    if verifier_output.status.success() {
        print!("{}", String::from_utf8_lossy(&verifier_output.stdout));
    } else {
        eprintln!("{}", String::from_utf8_lossy(&verifier_output.stderr));
    }

    Ok(())
}

fn build_demo_records(count: usize) -> Vec<AttestationRecord> {
    let mut records = Vec::with_capacity(count);
    let mut previous_signature: Option<Vec<u8>> = None;

    for i in 0..count {
        let decision_hash = Sha256::digest(format!("decision-{i}").as_bytes()).to_vec();
        let pcr_digest = vec![0u8; 32];

        let mut signature_input = Vec::new();
        signature_input.extend(&decision_hash);
        signature_input.extend(&pcr_digest);
        let signature = Sha256::digest(&signature_input).to_vec();

        let prev_hash = if let Some(prev_sig) = &previous_signature {
            let mut linkage = Vec::new();
            linkage.extend(prev_sig);
            linkage.extend(&decision_hash);
            Sha256::digest(&linkage).to_vec()
        } else {
            vec![0u8; 32]
        };

        let record = AttestationRecord {
            decision_hash: decision_hash.clone(),
            pcr_digest,
            signature: signature.clone(),
            timestamp: 1_710_000_000 + i as u64,
            prev_hash,
        };

        previous_signature = Some(signature);
        records.push(record);
    }

    records
}
