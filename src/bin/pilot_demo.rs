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

use m_v_r_esprint1::sovereign_kernel::{signer_from_env, AttestationRecord, SovereignKernel, SovereignKernelConfig};
use m_v_r_esprint1::universal_frontend::IRModule;
use m_v_r_esprint1::ir_codegen::IRInput;
use std::env;
use std::fs::File;
use std::io::Write;
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set simulation mode
    env::set_var("SIGNER_MODE", "simulation");

    // Create kernel
    let signer = signer_from_env(.map_err(|e| format!("{:?}", e))?;
    let config = SovereignKernelConfig { max_ticks: 100 };
    let mut kernel = SovereignKernel::new(signer, config);

    // Simulate multiple decisions (e.g., frequency responses)
    let mut records = Vec::new();

    for i in 0..3 {
        // Dummy IR module and input
        let ir_module = IRModule { /* placeholder */ };
        let input = IRInput { /* placeholder */ };

        // Execute and capture record (in real impl, extract from kernel)
        let _result = kernel.execute_foreign(&ir_module, input.map_err(|e| format!("{:?}", e))?;

        // For demo, create a sample record (in practice, kernel would expose records)
        let record = AttestationRecord {
            decision_hash: vec![i as u8; 32],
            pcr_digest: vec![0; 32],
            signature: vec![i as u8 + 1; 32],
            timestamp: 1710000000 + i,
            prev_hash: if i == 0 { vec![0; 32] } else { vec![(i-1) as u8; 32] },
        };
        records.push(record);
    }

    // Save to file
    let mut file = File::create("pilot_attestation_log.json".map_err(|e| format!("{:?}", e))?;
    for record in &records {
        writeln!(file, "{}", serde_json::to_string(record)?.map_err(|e| format!("{:?}", e))?;
    }

    println!("Generated pilot attestation log with {} records", records.len());

    // Run verifier
    let verifier_output = std::process::Command::new("cargo")
        .args(&["run", "--bin", "verifier", "pilot_attestation_log.json"])
        .output(.map_err(|e| format!("{:?}", e))?;

    if verifier_output.status.success() {
        println!("Verification successful!");
    } else {
        println!("Verification failed: {}", String::from_utf8_lossy(&verifier_output.stderr));
    }

    Ok(())
}