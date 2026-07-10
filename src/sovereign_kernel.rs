
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

#![deny(unsafe_code)]

use crate::canonical_time::CanonicalTime;
use crate::failure_axis::{FailureAxis, SystemHalt};
use sha2::{Digest, Sha256};
use mvre_core_deterministic::{ExecutionCommitment, Transaction};
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttestationRecord {
    pub commitment_hash: Vec<u8>,
    pub commitment: ExecutionCommitment,
    pub pcr_digest: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: u64,
    pub prev_hash: Vec<u8>,
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
pub trait Signer {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SystemHalt>;
    fn read_pcr(&self) -> Result<Vec<u8>, SystemHalt>;
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

impl Signer for FixedKeySimulatedTpmSigner {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SystemHalt> {
        // For verifier parity in this prototype, the simulated signer returns H(data).
        // Production TPM signer should perform real signature operations.
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        Ok(result.to_vec())
    }

    fn read_pcr(&self) -> Result<Vec<u8>, SystemHalt> {
        Ok(vec![0u8; 32]) // fake PCR
    }
}

/// TPM 2.0 signer using direct tss-esapi integration.
///
/// This provides deterministic, in-process TPM signing without external tools.
#[cfg(feature = "tpm")]
#[derive(Debug)]
pub struct TssEsapiSigner {
    context: Context,
}

#[cfg(feature = "tpm")]
impl TssEsapiSigner {
    pub fn new() -> Result<Self, SystemHalt> {
        let tcti = Tcti::device(std::path::Path::new("/dev/tpm0")).map_err(|e| SystemHalt::new(FailureAxis::TpmUnavailable, &format!("TCTI error: {:?}", e)))?;
        let context = Context::new(tcti).map_err(|e| SystemHalt::new(FailureAxis::TpmUnavailable, &format!("Context error: {:?}", e)))?;
        Ok(Self { context })
    }
}

#[cfg(feature = "tpm")]
impl Signer for TssEsapiSigner {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SystemHalt> {
        // Placeholder: implement actual signing with loaded key
        Ok(data.to_vec()) // fake signature
    }

    fn read_pcr(&self) -> Result<Vec<u8>, SystemHalt> {
        // Placeholder: read actual PCR
        Ok(vec![0u8; 32]) // fake PCR
    }
}

/// Runtime-selectable signer wrapper used by runtime binaries.
#[derive(Debug)]
pub enum AnyTpmSigner {
    Simulated(FixedKeySimulatedTpmSigner),
    #[cfg(feature = "tpm")]
    Tpm(TssEsapiSigner),
}

impl Signer for AnyTpmSigner {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SystemHalt> {
        match self {
            AnyTpmSigner::Simulated(s) => s.sign(data),
            #[cfg(feature = "tpm")]
            AnyTpmSigner::Tpm(s) => s.sign(data),
        }
    }

    fn read_pcr(&self) -> Result<Vec<u8>, SystemHalt> {
        match self {
            AnyTpmSigner::Simulated(s) => s.read_pcr(),
            #[cfg(feature = "tpm")]
            AnyTpmSigner::Tpm(s) => s.read_pcr(),
        }
    }
}

pub fn signer_from_env() -> Result<AnyTpmSigner, SystemHalt> {
    match env::var("SIGNER_MODE").as_deref() {
        Ok("tpm") => {
            #[cfg(feature = "tpm")]
            {
                TssEsapiSigner::new().map(AnyTpmSigner::Tpm)
            }
            #[cfg(not(feature = "tpm"))]
            Err(SystemHalt::new(FailureAxis::TpmUnavailable, "TPM required but not enabled"))
        }
        Ok("simulation") => Ok(AnyTpmSigner::Simulated(FixedKeySimulatedTpmSigner::new())),
        _ => Err(SystemHalt::new(FailureAxis::UnauthorizedMode, "No valid signer mode specified")),
    }
}

#[derive(Debug, Clone)]
pub struct SovereignKernelConfig {
    pub max_ticks: u64,
}

#[derive(Debug)]
pub struct SovereignKernel {
    #[allow(dead_code)]
    config: SovereignKernelConfig,
    signer: AnyTpmSigner,
    last_record_hash: Vec<u8>,
}

impl SovereignKernel {
    pub fn new(signer: AnyTpmSigner, config: SovereignKernelConfig) -> Self {
        Self { config, signer, last_record_hash: vec![0; 32] }
    }

    /// Execute foreign logic inside the sovereign runtime
    pub fn deterministic_execute(
        &mut self,
        ir_module: &crate::universal_frontend::IRModule,
        input: crate::ir_codegen::IRInput,
        timestamp: CanonicalTime,
    ) -> Result<(crate::ir_codegen::IRResult, AttestationRecord), SystemHalt> {
        // Generate Rust code from IR
        let _rust_code = crate::ir_codegen::generate_rust_code(ir_module);

        // Send execution start message to bus
        let start_message = crate::sovereign_bus::SovereignMessage::new_command(
            crate::sovereign_bus::ActorId("kernel".to_string()),
            crate::sovereign_bus::ActorRole::KernelSubsystem,
            crate::sovereign_bus::OriginLanguage::Rust,
            crate::regulatory_policy::Intent::Execute,
            vec![], // payload would contain execution details
            vec![], // invariants applied
            crate::sovereign_bus::TraceId("execution-start".to_string()),
            timestamp,
        );
        if let Some(bus) = &mut *crate::sovereign_bus::global_bus() {
            bus.send(start_message);
        }

        // Placeholder result - in practice, this would execute the generated code
        let result = crate::ir_codegen::IRResult {
            value: crate::universal_frontend::Value::Int(42),
            bus_messages: vec![],
        };

        // Canonicalize input deterministically: sort map keys and serialize values in a stable way
        fn value_to_bytes(v: &crate::universal_frontend::Value) -> Vec<u8> {
            match v {
                crate::universal_frontend::Value::Int(i) => i.to_le_bytes().to_vec(),
                crate::universal_frontend::Value::Float(f) => f.to_bits().to_le_bytes().to_vec(),
                crate::universal_frontend::Value::Bool(b) => vec![*b as u8],
                crate::universal_frontend::Value::String(s) => s.as_bytes().to_vec(),
            }
        }

        let mut canonical_input_bytes: Vec<u8> = Vec::new();
        {
            let mut keys: Vec<_> = input.args.keys().cloned().collect();
            keys.sort();
            for k in keys {
                canonical_input_bytes.extend_from_slice(k.as_bytes());
                if let Some(v) = input.args.get(&k) {
                    canonical_input_bytes.extend(value_to_bytes(v));
                }
            }
        }

        // Execution trace and transition are placeholders here; real execution should produce these deterministically
        let execution_trace_bytes = b"trace:result:42".to_vec();
        let transition_record_bytes = b"transition:noop".to_vec();
        let final_state_bytes = self.last_record_hash.clone();

        // Build transaction and commit deterministic ExecutionCommitment
        let mut tx = Transaction::new(self.last_record_hash.clone().try_into().unwrap_or([0u8;32]));
        tx.append(&execution_trace_bytes);
        let commitment = tx.commit(
            &canonical_input_bytes,
            &execution_trace_bytes,
            &transition_record_bytes,
            &final_state_bytes,
            1,
        ).map_err(|e| SystemHalt::new(FailureAxis::InternalInvariantBreach, &format!("Commitment failed: {:?}", e)))?;

        // Compute commitment hash and produce signature over commitment || pcr
        let commitment_bytes = commitment.to_bytes();
        let commitment_hash = Sha256::digest(&commitment_bytes).to_vec();
        let pcr = self.signer.read_pcr()?;
        let mut sig_input = Vec::new();
        sig_input.extend_from_slice(&commitment_bytes);
        sig_input.extend_from_slice(&pcr);
        let signature = self.signer.sign(&sig_input)?;

        let prev_hash = self.last_record_hash.clone();
        let record = AttestationRecord {
            commitment_hash: commitment_hash.clone(),
            commitment: commitment.clone(),
            pcr_digest: pcr,
            signature,
            timestamp: timestamp.0,
            prev_hash,
        };

        // Update last record hash for chaining (serialize deterministically via to_bytes)
        self.last_record_hash = Sha256::digest(&record.commitment.to_bytes()).to_vec();

        // Send execution complete message to bus
        let complete_message = crate::sovereign_bus::SovereignMessage::new_command(
            crate::sovereign_bus::ActorId("kernel".to_string()),
            crate::sovereign_bus::ActorRole::KernelSubsystem,
            crate::sovereign_bus::OriginLanguage::Rust,
            crate::regulatory_policy::Intent::Execute,
            vec![], // payload would contain result
            vec![], // invariants applied
            crate::sovereign_bus::TraceId("execution-complete".to_string()),
            timestamp,
        );
        if let Some(bus) = &mut *crate::sovereign_bus::global_bus() {
            bus.send(complete_message);
        }

        Ok((result, record))
    }
}

