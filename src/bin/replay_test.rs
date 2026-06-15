// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// This file is part of the M.V.R.ESPRINT1 Sovereign Execution System.
//
// Replay Equivalence Test Harness — Phase B Certification
//
// This binary enforces Level-3 determinism: any two identical inputs
// to the real kernel must produce byte-identical execution commitments
// and attestation records.
//
// Used as a CI gate to prevent non-deterministic regressions.

use m_v_r_esprint1::testament_audit::DeterminismCertificate;
use m_v_r_esprint1::sovereign_kernel::{AnyTpmSigner, FixedKeySimulatedTpmSigner, SovereignKernel, SovereignKernelConfig, AttestationRecord};
use m_v_r_esprint1::universal_frontend::{IRModule, Value};
use m_v_r_esprint1::ir_codegen::IRInput;
use m_v_r_esprint1::canonical_time::CanonicalTime;
use std::collections::BTreeMap;
use sha2::{Digest, Sha256};

/// Canonical input for deterministic replay
#[derive(Clone, Debug)]
struct CanonicalInput {
    ir_module: IRModule,
    args: BTreeMap<String, String>,
}

/// Result of a single deterministic kernel run
#[derive(Clone, Debug)]
struct KernelRun {
    input_hash: [u8; 32],
    ir_hash: [u8; 32],
    attestation_record: AttestationRecord,
    commitment_bytes: Vec<u8>,
}

/// Execute kernel once with canonical input
fn kernel_execute_once(input: &CanonicalInput, canonical_timestamp: u64) -> Result<KernelRun, String> {
    // Hash canonical input deterministically
    let input_bytes = serde_json::to_vec(&input.args).map_err(|e| format!("Serialize input error: {}", e))?;
    let input_hash = {
        let mut h = [0u8; 32];
        h.copy_from_slice(&Sha256::digest(&input_bytes));
        h
    };

    // Hash IR module
    let ir_bytes = serde_json::to_vec(&input.ir_module).map_err(|e| format!("Serialize IR error: {}", e))?;
    let ir_hash = {
        let mut h = [0u8; 32];
        h.copy_from_slice(&Sha256::digest(&ir_bytes));
        h
    };

    // Initialize signer (use simulated mode for determinism)
    let signer = AnyTpmSigner::Simulated(FixedKeySimulatedTpmSigner::new());

    // Create kernel instance
    let mut kernel = SovereignKernel::new(signer, SovereignKernelConfig { max_ticks: 1000 });

    // Prepare IR input
    let mut ir_input = IRInput {
        args: BTreeMap::new(),
    };
    for (k, v) in &input.args {
        ir_input.args.insert(k.clone(), Value::String(v.clone()));
    }

    // Execute deterministically with canonical timestamp
    let canonical_time = CanonicalTime::from_millis(canonical_timestamp);
    let (_result, attestation) = kernel.deterministic_execute(&input.ir_module, ir_input, canonical_time)
        .map_err(|halt| format!("Kernel execution failed: {:?}", halt))?;

    // Extract commitment bytes from attestation for byte-comparison
    let commitment_bytes = attestation.commitment.to_bytes();

    Ok(KernelRun {
        input_hash,
        ir_hash,
        attestation_record: attestation,
        commitment_bytes,
    })
}

/// Strict replay equivalence test: execute kernel twice with identical input and compare
fn replay_equivalence_test(input: &CanonicalInput, canonical_timestamp: u64) -> Result<(), String> {
    println!("🔄 Running Phase B replay equivalence test...");

    // First kernel execution
    let run_a = kernel_execute_once(input, canonical_timestamp)?;
    println!("  ✓ Run A: input_hash={:02x?}, ir_hash={:02x?}, commitment_len={}", 
             &run_a.input_hash[0..4], 
             &run_a.ir_hash[0..4],
             run_a.commitment_bytes.len());

    // Second kernel execution (identical input)
    let run_b = kernel_execute_once(input, canonical_timestamp)?;
    println!("  ✓ Run B: input_hash={:02x?}, ir_hash={:02x?}, commitment_len={}", 
             &run_b.input_hash[0..4],
             &run_b.ir_hash[0..4],
             run_b.commitment_bytes.len());

    // Compare: input hashes must be identical
    if run_a.input_hash != run_b.input_hash {
        return Err(
            format!(
                "Input hash mismatch:\n  Run A: {:02x?}\n  Run B: {:02x?}",
                run_a.input_hash, run_b.input_hash
            )
        );
    }

    // Compare: IR hashes must be identical
    if run_a.ir_hash != run_b.ir_hash {
        return Err(
            format!(
                "IR hash mismatch:\n  Run A: {:02x?}\n  Run B: {:02x?}",
                run_a.ir_hash, run_b.ir_hash
            )
        );
    }

    // Compare: commitment bytes (canonical execution proof) must be byte-identical
    if run_a.commitment_bytes != run_b.commitment_bytes {
        return Err(
            format!(
                "Commitment bytes mismatch:\n  Run A: {} bytes\n  Run B: {} bytes",
                run_a.commitment_bytes.len(), run_b.commitment_bytes.len()
            )
        );
    }

    // Compare: attestation records must be identical
    if run_a.attestation_record != run_b.attestation_record {
        return Err("Attestation record mismatch".to_string());
    }

    println!("✅ Replay equivalence PASS: both kernel runs produced identical commitments");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MVRE Phase B: Determinism Replay Equivalence Test Suite");
    println!("======================================================\n");

    // Canonical timestamp for all runs (ensures time-independence)
    let canonical_timestamp: u64 = 1000;

    // Test 1: Empty IR module with empty input
    println!("Test 1: Empty IR module with empty input");
    let input_1 = CanonicalInput {
        ir_module: IRModule {
            functions: Vec::new(),
            constants: Vec::new(),
        },
        args: BTreeMap::new(),
    };
    replay_equivalence_test(&input_1, canonical_timestamp)?;
    println!();

    // Test 2: IR with constants
    println!("Test 2: IR module with canonical constants");
    let input_2 = CanonicalInput {
        ir_module: IRModule {
            functions: Vec::new(),
            constants: vec![
                ("CONST_A".to_string(), Value::Int(42)),
                ("CONST_B".to_string(), Value::String("test".to_string())),
            ],
        },
        args: BTreeMap::new(),
    };
    replay_equivalence_test(&input_2, canonical_timestamp)?;
    println!();

    // Test 3: Input with multiple arguments (sorted for determinism)
    println!("Test 3: Input with sorted BTreeMap arguments");
    let mut args_3 = BTreeMap::new();
    args_3.insert("x".to_string(), "100".to_string());
    args_3.insert("a".to_string(), "10".to_string());
    args_3.insert("z".to_string(), "1000".to_string());
    let input_3 = CanonicalInput {
        ir_module: IRModule {
            functions: Vec::new(),
            constants: Vec::new(),
        },
        args: args_3,
    };
    replay_equivalence_test(&input_3, canonical_timestamp)?;
    println!();

    println!("✅ All Phase B replay equivalence tests passed!");
    println!("\n🔬 Level-3 Determinism Certification: PASS");
    println!("The kernel demonstrates byte-identical execution under replay.");
    println!("\nThis checkpoint qualifies for DHM-1 (Deterministic Hardening Milestone-1)");

    Ok(())
}
