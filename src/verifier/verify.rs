use crate::sovereign_kernel::AttestationRecord as KernelRecord;
use crate::verifier::attestation::AttestationLog;
use crate::verifier::hash;
use crate::verifier::replay;
use sha2::{Digest, Sha256};

pub fn run_from_file(path: &str) -> Result<String, String> {
    let log = AttestationLog::load(path)?;

    for (i, rec) in log.records.iter().enumerate() {
        verify_signature(&KernelRecord {
            decision_hash: rec.decision_hash.clone(),
            pcr_digest: rec.pcr_digest.clone(),
            signature: rec.signature.clone(),
            timestamp: rec.timestamp,
            prev_hash: rec.prev_hash.clone(),
        })?;

        if i > 0 {
            let prev = &log.records[i - 1];
            let mut input = Vec::new();
            input.extend(&prev.signature);
            input.extend(&rec.decision_hash);
            let expected = Sha256::digest(&input).to_vec();
            if expected != rec.prev_hash {
                return Err("VVD-FAIL-02: Tamper detected (chain linkage)".to_string());
            }
        }

        let replay_out = replay::deterministic_replay(rec)?;

        let computed_output_hash = hash::sha256_vec(&replay_out.output);
        if let Some(expected_output_hash) = &rec.output_hash {
            if &computed_output_hash != expected_output_hash {
                return Err("VVD-FAIL-01: Replay mismatch".to_string());
            }
        } else if &computed_output_hash != &rec.decision_hash {
            return Err("VVD-FAIL-01: Replay mismatch".to_string());
        }

        let computed_trace_hash = hash::sha256_vec(&replay_out.trace);
        if let Some(expected_trace_hash) = &rec.trace_hash {
            if &computed_trace_hash != expected_trace_hash {
                return Err("VVD-FAIL-06: Incomplete trace - trace hash mismatch".to_string());
            }
        }

        if let Some(expected_state_hash) = &rec.state_hash {
            if &replay_out.state_hash != expected_state_hash {
                return Err("VVD-FAIL-01: Replay mismatch (state hash)".to_string());
            }
        }

        // If dependency metadata is missing, legacy logs may still be accepted,
        // but richer records should include `deps_hash` for full purity verification.
    }

    Ok(format!("✔ VVD checks passed: {} records", log.records.len()))
}

fn verify_signature(record: &KernelRecord) -> Result<(), String> {
    let mut combined = Vec::new();
    combined.extend(&record.decision_hash);
    combined.extend(&record.pcr_digest);

    let expected = Sha256::digest(&combined).to_vec();

    if expected != record.signature {
        return Err("VVD-FAIL-02: Tamper detected (signature)".to_string());
    }

    Ok(())
}
