#![deny(unsafe_code)]

use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::tlbss_types::BinaryState;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::env;

/// Immutable per-tick audit record for the sovereign substrate.
#[derive(Debug, Clone, PartialEq)]
pub struct SovereignTrace {
    pub tick: u64,
    pub ai_request: u64,
    pub kernel_output: u64,
    pub authority_level: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SummaryAttestation {
    pub trace_hash: String,
    pub tick_count: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L7Disposition {
    Allowed,
    Constrained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L7Reason {
    Nominal,
    Thermal,
    Frequency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L7Event {
    pub tick: u64,
    pub disposition: L7Disposition,
}

/// Non-agentic certifier. Reads metrics and signs only if gates are satisfied.
pub trait TpmSigner {
    fn sign(&self, data: &[u8]) -> Result<String, SystemHalt>;
}

/// Deterministic fixed-key signer for simulation/testing.
/// In production replace with a TPM-backed signer implementation.
#[derive(Debug, Clone)]
pub struct FixedKeySimulatedTpmSigner {
    key: String,
}

impl FixedKeySimulatedTpmSigner {
    pub fn new() -> Self {
        Self {
            key: "simulated-key".to_string(),
        }
    }
}

impl TpmSigner for FixedKeySimulatedTpmSigner {
    fn sign(&self, data: &[u8]) -> Result<String, SystemHalt> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(self.key.as_bytes());
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
}

/// Runtime-selectable signer wrapper used by runtime binaries.
#[derive(Debug, Clone)]
pub enum AnyTpmSigner {
    Simulated(FixedKeySimulatedTpmSigner),
}

impl TpmSigner for AnyTpmSigner {
    fn sign(&self, data: &[u8]) -> Result<String, SystemHalt> {
        match self {
            AnyTpmSigner::Simulated(s) => s.sign(data),
        }
    }
}

/// TPM 2.0 signer skeleton that delegates to `tpm2_sign` from tpm2-tools.
///
/// This is intentionally lightweight and designed for integration wiring. It
/// requires a pre-provisioned non-exportable key context in the TPM.
#[cfg(feature = "tpm2-tools")]
#[derive(Debug, Clone)]
pub struct Tpm2ToolsSigner {
    key_context: String,
}

#[cfg(feature = "tpm2-tools")]
impl Tpm2ToolsSigner {
    pub fn new(key_context: String) -> Self {
        Self { key_context }
    }
}
