use serde::{Deserialize, Serialize};

/// Minimal attestation record used by the verifier core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationRecord {
    pub index: Option<u64>,
    pub input: Option<Vec<u8>>,
    pub deps_hash: Option<Vec<u8>>,
    pub output_hash: Option<Vec<u8>>,
    pub trace_hash: Option<Vec<u8>>,
    pub state_hash: Option<Vec<u8>>,
    pub decision_hash: Vec<u8>,
    pub pcr_digest: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: u64,
    pub prev_hash: Vec<u8>,
}
