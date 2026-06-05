use crate::verifier::attestation::AttestationRecord;
use crate::verifier::hash;

pub struct ReplayResult {
    pub output: Vec<u8>,
    pub trace: Vec<u8>,
    pub state_hash: Vec<u8>,
}

/// Pure deterministic execution boundary.
/// This function is intentionally pure and isolated from IO, randomness, and global state.
pub fn deterministic_execute(input: &[u8], deps_hash: &[u8]) -> ReplayResult {
    let mut step1 = Vec::new();
    step1.extend(input);
    step1.extend(deps_hash);
    let node1 = hash::sha256_vec(&step1);

    let mut step2 = Vec::new();
    step2.extend(&node1);
    step2.extend(deps_hash);
    let node2 = hash::sha256_vec(&step2);

    let mut trace = Vec::new();
    trace.extend(&node1);
    trace.extend(&node2);

    let output = hash::sha256_vec(&trace);

    let mut state_input = Vec::new();
    state_input.extend(input);
    state_input.extend(deps_hash);
    state_input.extend(&trace);
    let state_hash = hash::sha256_vec(&state_input);

    ReplayResult {
        output,
        trace,
        state_hash,
    }
}

pub fn deterministic_replay(record: &AttestationRecord) -> Result<ReplayResult, String> {
    let input = record
        .input
        .as_ref()
        .map(Vec::as_slice)
        .unwrap_or_else(|| record.decision_hash.as_slice());
    let deps_hash = record.deps_hash.as_ref().map(Vec::as_slice).unwrap_or_else(|| &[]);

    Ok(deterministic_execute(input, deps_hash))
}
