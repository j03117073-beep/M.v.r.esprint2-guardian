// mvre_core_deterministic: Core deterministic execution primitives
// Provides ExecutionCommitment, DeterministicExecutable trait, Transaction scratchpad,
// and deterministic hashing helpers.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ExecutionCommitment {
    pub parent_state_hash: [u8; 32],
    pub canonical_input_hash: [u8; 32],
    pub execution_trace_hash: [u8; 32],
    pub transition_hash: [u8; 32],
    pub final_state_hash: [u8; 32],
    pub state_version: u64,
}

impl ExecutionCommitment {
    /// Deterministic canonical byte representation of the commitment.
    /// Fields are concatenated in struct declaration order using big-endian
    /// or fixed-width encodings to ensure cross-platform byte identity.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 * 5 + 8);
        out.extend_from_slice(&self.parent_state_hash);
        out.extend_from_slice(&self.canonical_input_hash);
        out.extend_from_slice(&self.execution_trace_hash);
        out.extend_from_slice(&self.transition_hash);
        out.extend_from_slice(&self.final_state_hash);
        out.extend_from_slice(&self.state_version.to_le_bytes());
        out
    }
}

pub trait DeterministicExecutable {
    fn execute(&self, tx: &mut Transaction) -> Result<(), MVREError>;
}

#[derive(Debug)]
pub enum MVREError {
    ExecutionFailure(String),
    HashingError,
    InvalidState(String),
    Abort(String),
}

pub struct Transaction {
    pub scratchpad: Vec<u8>,
    pub parent_state_hash: [u8; 32],
    pub committed: bool,
}

impl Transaction {
    pub fn new(parent_state_hash: [u8; 32]) -> Self {
        Self { scratchpad: Vec::new(), parent_state_hash, committed: false }
    }

    pub fn append(&mut self, data: &[u8]) {
        self.scratchpad.extend_from_slice(data);
    }

    pub fn rollback(&mut self) {
        self.scratchpad.clear();
        self.committed = false;
    }

    /// Commit produces an ExecutionCommitment by hashing the provided components.
    /// This function is intentionally deterministic and avoids any external
    /// entropy or system time calls.
    pub fn commit(
        mut self,
        canonical_input: &[u8],
        execution_trace: &[u8],
        transition_record: &[u8],
        final_state: &[u8],
        state_version: u64,
    ) -> Result<ExecutionCommitment, MVREError> {
        if self.committed {
            return Err(MVREError::InvalidState("already committed".to_string()));
        }

        // Compute deterministic hashes
        let canonical_input_hash = sha256_fixed(canonical_input);
        let execution_trace_hash = sha256_fixed(execution_trace);
        let transition_hash = sha256_fixed(transition_record);
        let final_state_hash = sha256_fixed(final_state);

        let commitment = ExecutionCommitment {
            parent_state_hash: self.parent_state_hash,
            canonical_input_hash,
            execution_trace_hash,
            transition_hash,
            final_state_hash,
            state_version,
        };

        // Optionally extend scratchpad with commitment bytes (deterministic)
        if let Ok(bytes) = serde_json::to_vec(&commitment) {
            self.scratchpad.extend_from_slice(&bytes);
        } else {
            return Err(MVREError::HashingError);
        }

        self.committed = true;
        Ok(commitment)
    }
}

fn sha256_fixed(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let res = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&res);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_is_deterministic() {
        let parent = [0u8; 32];
        let mut tx = Transaction::new(parent);
        tx.append(b"step1");
        let c1 = tx
            .commit(b"input", b"trace", b"transition", b"final", 1)
            .expect("commit should succeed");

        // Recreate second transaction with same inputs
        let mut tx2 = Transaction::new(parent);
        tx2.append(b"step1");
        let c2 = tx2
            .commit(b"input", b"trace", b"transition", b"final", 1)
            .expect("commit should succeed");

        assert_eq!(c1, c2);
    }
}
