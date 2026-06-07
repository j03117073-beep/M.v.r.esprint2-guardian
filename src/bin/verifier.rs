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

use m_v_r_esprint1::sovereign_kernel::AttestationRecord;
use sha2::{Digest, Sha256};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("provide log file path");

    let data = fs::read_to_string(&path)?;

    let records: Vec<AttestationRecord> = serde_json::from_str(&data)?;

    verify_chain(&records)?;

    println!("✔ Chain verified: {} records valid", records.len());

    Ok(())
}

fn verify_chain(records: &[AttestationRecord]) -> Result<(), String> {
    for i in 0..records.len() {
        let record = &records[i];

        // 1. Verify signature
        verify_signature(record)?;

        // 2. Verify hash linkage
        if i > 0 {
            let prev = &records[i - 1];
            // Expect prev_hash to be the hash of the previous commitment bytes
            let expected = Sha256::digest(&prev.commitment.to_bytes());
            if expected.to_vec() != record.prev_hash {
                return Err(format!("Chain broken at index {}", i));
            }
        } else {
            // First record should have prev_hash as zeros
            if record.prev_hash != vec![0; 32] {
                return Err("First record prev_hash not zero".into());
            }
        }

        // 3. Monotonic timestamp check
        if i > 0 && record.timestamp < records[i - 1].timestamp {
            return Err("Timestamp ordering violated".into());
        }
    }

    Ok(())
}

fn verify_signature(record: &AttestationRecord) -> Result<(), String> {
    let mut combined = Vec::new();
    combined.extend(record.commitment.to_bytes());
    combined.extend(&record.pcr_digest);

    let expected = Sha256::digest(&combined);

    if expected.to_vec() != record.signature {
        return Err("Invalid signature".into());
    }

    Ok(())
}