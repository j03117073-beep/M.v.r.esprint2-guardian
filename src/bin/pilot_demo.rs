// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// This file is part of the M.V.R.ESPRINT1 Sovereign Execution System,
// including TLBSS geometry, the Universal Execution Layer, the
// Deterministic IR, Rust Codegen Pipeline, SovereignBus, and the
// Cryptographic Audit Chain.
//
// No part of this file, its algorithms, structures, or designs may be
// copied, reproduced, modified, distributed, published, sublicensed,
// reverse-engineered, or used to create derivative works without the
// express written permission of OBINNA JAMES EJIOFOR.
//
// This software contains proprietary trade secrets and confidential
// intellectual property. Unauthorized use is strictly prohibited.

use m_v_r_esprint1::sovereign_kernel::{signer_from_env, SovereignKernel, SovereignKernelConfig, Signer};
use m_v_r_esprint1::verifier::attestation::AttestationRecord;
use m_v_r_esprint1::verifier::replay;
use m_v_r_esprint1::universal_frontend::IRModule;
use m_v_r_esprint1::ir_codegen::IRInput;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set simulation mode
    env::set_var("SIGNER_MODE", "simulation");

    // Create kernel
    let signer = signer_from_env().map_err(|e| format!("{:?}", e))?;
    let config = SovereignKernelConfig { max_ticks: 100 };
    let mut kernel = SovereignKernel::new(signer, config);
    let pcr_signer = signer_from_env().map_err(|e| format!("{:?}", e))?;

    // Simulate multiple decisions (e.g., frequency responses)
    let mut records: Vec<AttestationRecord> = Vec::new();

    for i in 0..3 {
        // Dummy IR module, input, and execution path.
        let ir_module = IRModule {
            functions: Vec::new(),
            constants: Vec::new(),
        };
        let input = IRInput {
            args: std::collections::HashMap::new(),
        };

        let _result = kernel.execute_foreign(&ir_module, input)
            .map_err(|e| format!("{:?}", e))?;

        let input_bytes = format!("pilot-input-{}", i).into_bytes();
        let deps = b"depA:1.0,depB:2.0".to_vec();
        let deps_hash = Sha256::digest(&deps).to_vec();

        let replay_result = replay::deterministic_execute(&input_bytes, &deps_hash);
        let output_hash = Sha256::digest(&replay_result.output).to_vec();
        let trace_hash = Sha256::digest(&replay_result.trace).to_vec();
        let state_hash = replay_result.state_hash.clone();

        let pcr_digest = pcr_signer.read_pcr().map_err(|e| format!("{:?}", e))?;

        let mut combined = Vec::new();
        combined.extend(&output_hash);
        combined.extend(&pcr_digest);
        let signature = Sha256::digest(&combined).to_vec();

        let prev_hash = if i == 0 {
            vec![0; 32]
        } else {
            let prev = &records[i - 1];
            let mut prev_combined = Vec::new();
            prev_combined.extend(&prev.signature);
            prev_combined.extend(&output_hash);
            Sha256::digest(&prev_combined).to_vec()
        };

        records.push(AttestationRecord {
            index: Some(i as u64),
            input: Some(input_bytes),
            deps_hash: Some(deps_hash),
            output_hash: Some(output_hash.clone()),
            trace_hash: Some(trace_hash),
            state_hash: Some(state_hash),
            decision_hash: output_hash,
            pcr_digest,
            signature,
            timestamp: 1710000000u64 + i as u64,
            prev_hash,
        });
    }

    // Save the whole log as a JSON array so verifier can parse it correctly.
    let file = File::create("pilot_attestation_log.json")
        .map_err(|e| format!("{:?}", e))?;
    serde_json::to_writer_pretty(file, &records)
        .map_err(|e| format!("{:?}", e))?;

    println!("Generated pilot attestation log with {} records", records.len());
    println!("Run `cargo run --bin verifier pilot_attestation_log.json` to validate the generated attestation chain.");

    Ok(())
}