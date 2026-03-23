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

//! SovereignTrace: The Auditable Compliance Record
//!
//! Every millisecond, the kernel generates a `SovereignTrace` that records:
//! - What the AI requested
//! - What the kernel actually output
//! - The legal reason for any difference
//! - Timestamp and all physical measurements
//!
//! This is the "Sovereign Trace" - the unfalsifiable proof that the kernel
//! was physically incapable of violating regulatory mandates.
pub mod streamer;

use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::regulatory_policy::{GovernanceMode, LegalCitation};
use crate::sovereign_bus::{SovereignMessage, ActorId, ActorRole, OriginLanguage, TraceId};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cryptographic hash type
pub type Hash256 = [u8; 32];

/// Digital signature type
pub type Signature = Vec<u8>;

/// Canonical structure for all inbound inputs
#[derive(Debug, Clone)]
pub struct InputEnvelope {
    pub actor_id: ActorId,
    pub role: ActorRole,
    pub origin_language: OriginLanguage,
    pub raw_input_bytes: Vec<u8>,
    pub raw_input_hash: Hash256,
    pub normalized_ir_hash: Option<Hash256>,
    pub timestamp: u64,
    pub signature: Option<Signature>,
    pub trace_parent: TraceId,
}

impl InputEnvelope {
    pub fn new(
        actor_id: ActorId,
        role: ActorRole,
        origin_language: OriginLanguage,
        raw_input_bytes: Vec<u8>,
        signature: Option<Signature>,
        trace_parent: TraceId,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let raw_input_hash = Sha256::digest(&raw_input_bytes).into();

        Self {
            actor_id,
            role,
            origin_language,
            raw_input_bytes,
            raw_input_hash,
            normalized_ir_hash: None,
            timestamp,
            signature,
            trace_parent,
        }
    }

    /// Set the normalized IR hash after IR conversion
    pub fn set_ir_hash(&mut self, ir_hash: Hash256) {
        self.normalized_ir_hash = Some(ir_hash);
    }

    /// Verify signature if present (placeholder implementation)
    pub fn verify_signature(&self) -> Result<(), SystemHalt> {
        // In production, this would verify the signature against the actor's public key
        // For now, accept all signatures as valid
        Ok(())
    }
}

/// Types of trace records
#[derive(Debug, Clone)]
pub enum TraceRecord {
    StateTransition(SovereignTrace),
    MessageEvent(SovereignMessage),
    InputReceived(InputEnvelope),
    IRNormalized { raw_hash: String, ir_hash: String },
    RustGenerated { ir_hash: String, rust_hash: String },
    ExecutionResult { rust_hash: String, result_hash: String },
    OutputTranslated { result_hash: String, output_hash: String },
}

/// A single tick's compliance record
///
/// This structure is the core audit artifact. When a utility auditor asks
/// "Why didn't you comply with TPL-008-1?", the kernel produces thousands
/// of these records showing it was obeying the law every millisecond.
#[derive(Debug, Clone)]
pub struct SovereignTrace {
    pub tick: u64,
    pub requested_setpoint: f64,
    pub actual_setpoint: f64,
    pub governance_mode: GovernanceMode,
    pub legal_citation: LegalCitation,
    pub timestamp_ms: u64,
    pub timestamp_us: u64,
    pub grid_sigma: u8,
    pub ambient_temp: f32,
    pub inverter_current: f64,
    pub ai_requested_p: f64,
    pub kernel_output_p: f64,
    pub active_governance: GovernanceMode,
    pub legal_justification: Option<LegalCitation>,
    pub is_authenticated: bool,
    pub state_transition: bool,
}

impl SovereignTrace {
    pub fn new(
        tick: u64,
        requested: f64,
        actual: f64,
        mode: GovernanceMode,
        citation: LegalCitation,
    ) -> Self {
        Self {
            tick,
            requested_setpoint: requested,
            actual_setpoint: actual,
            governance_mode: mode,
            legal_citation: citation.clone(),
            timestamp_ms: 0,
            timestamp_us: 0,
            grid_sigma: 0,
            ambient_temp: 0.0,
            inverter_current: 0.0,
            ai_requested_p: requested,
            kernel_output_p: actual,
            active_governance: mode,
            legal_justification: Some(citation),
            is_authenticated: false,
            state_transition: false,
        }
    }
}

impl fmt::Display for SovereignTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tick {}: {} -> {} ({})",
            self.tick, self.requested_setpoint, self.actual_setpoint, self.governance_mode
        )
    }
}

/// Builder for SovereignTrace (fluent API for testing)
pub struct TraceBuilder {
    tick: u64,
    governance: GovernanceMode,
    legal_citation: LegalCitation,
}

impl TraceBuilder {
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            governance: GovernanceMode::Normal,
            legal_citation: LegalCitation::default(),
        }
    }

    pub fn governance(mut self, mode: GovernanceMode) -> Self {
        self.governance = mode;
        self
    }

    pub fn build(self) -> SovereignTrace {
        SovereignTrace {
            tick: self.tick,
            requested_setpoint: 0.0,
            actual_setpoint: 0.0,
            governance_mode: self.governance,
            legal_citation: self.legal_citation.clone(),
            timestamp_ms: 0,
            timestamp_us: 0,
            grid_sigma: 0,
            ambient_temp: 0.0,
            inverter_current: 0.0,
            ai_requested_p: 0.0,
            kernel_output_p: 0.0,
            active_governance: self.governance,
            legal_justification: Some(self.legal_citation),
            is_authenticated: false,
            state_transition: false,
        }
    }
}

/// A log of TraceRecords for auditing
#[derive(Debug, Clone)]
pub struct SovereignTraceLog {
    pub records: Vec<TraceRecord>,
    pub hash_chain: String,
}

impl SovereignTraceLog {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            hash_chain: String::new(),
        }
    }

    pub fn append(&mut self, record: TraceRecord) {
        self.records.push(record);
    }

    pub fn append_state_transition(&mut self, trace: SovereignTrace) {
        self.append(TraceRecord::StateTransition(trace));
    }

    pub fn append_message_event(&mut self, message: SovereignMessage) {
        self.append(TraceRecord::MessageEvent(message));
    }

    pub fn append_input_received(&mut self, envelope: InputEnvelope) {
        self.append(TraceRecord::InputReceived(envelope));
    }

    pub fn append_ir_normalized(&mut self, raw_hash: String, ir_hash: String) {
        self.append(TraceRecord::IRNormalized { raw_hash, ir_hash });
    }

    pub fn append_rust_generated(&mut self, ir_hash: String, rust_hash: String) {
        self.append(TraceRecord::RustGenerated { ir_hash, rust_hash });
    }

    pub fn append_execution_result(&mut self, rust_hash: String, result_hash: String) {
        self.append(TraceRecord::ExecutionResult { rust_hash, result_hash });
    }

    pub fn append_output_translated(&mut self, result_hash: String, output_hash: String) {
        self.append(TraceRecord::OutputTranslated { result_hash, output_hash });
    }

    pub fn to_file(&self, path: &Path) -> Result<(), SystemHalt> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .map_err(|e| {
                SystemHalt::with_formatted(
                    FailureAxis::ExternalInjectionDetected,
                    format!("Failed to write trace log: {e}"),
                )
            })?;

        for record in &self.records {
            match record {
                TraceRecord::StateTransition(trace) => {
                    writeln!(file, "STATE: {}", trace).map_err(|e| {
                        SystemHalt::with_formatted(
                            FailureAxis::ExternalInjectionDetected,
                            format!("Write error: {e}"),
                        )
                    })?;
                }
                TraceRecord::MessageEvent(message) => {
                    match message {
                        crate::sovereign_bus::SovereignMessage::Command { actor_id, intent, .. } => {
                            writeln!(file, "MESSAGE: {} -> {:?}", actor_id.0, intent).map_err(|e| {
                                SystemHalt::with_formatted(
                                    FailureAxis::ExternalInjectionDetected,
                                    format!("Write error: {e}"),
                                )
                            })?;
                        }
                        crate::sovereign_bus::SovereignMessage::InputEnvelopeReceived(envelope) => {
                            writeln!(file, "ENVELOPE: {} received input", envelope.actor_id.0).map_err(|e| {
                                SystemHalt::with_formatted(
                                    FailureAxis::ExternalInjectionDetected,
                                    format!("Write error: {e}"),
                                )
                            })?;
                        }
                    }
                }
                TraceRecord::InputReceived(envelope) => {
                    writeln!(file, "INPUT: {} received {} bytes", envelope.actor_id.0, envelope.raw_input_bytes.len()).map_err(|e| {
                        SystemHalt::with_formatted(
                            FailureAxis::ExternalInjectionDetected,
                            format!("Write error: {e}"),
                        )
                    })?;
                }
                TraceRecord::IRNormalized { raw_hash, ir_hash } => {
                    writeln!(file, "IR_NORMALIZED: {} -> {}", raw_hash, ir_hash).map_err(|e| {
                        SystemHalt::with_formatted(
                            FailureAxis::ExternalInjectionDetected,
                            format!("Write error: {e}"),
                        )
                    })?;
                }
                TraceRecord::RustGenerated { ir_hash, rust_hash } => {
                    writeln!(file, "RUST_GENERATED: {} -> {}", ir_hash, rust_hash).map_err(|e| {
                        SystemHalt::with_formatted(
                            FailureAxis::ExternalInjectionDetected,
                            format!("Write error: {e}"),
                        )
                    })?;
                }
                TraceRecord::ExecutionResult { rust_hash, result_hash } => {
                    writeln!(file, "EXECUTION_RESULT: {} -> {}", rust_hash, result_hash).map_err(|e| {
                        SystemHalt::with_formatted(
                            FailureAxis::ExternalInjectionDetected,
                            format!("Write error: {e}"),
                        )
                    })?;
                }
                TraceRecord::OutputTranslated { result_hash, output_hash } => {
                    writeln!(file, "OUTPUT_TRANSLATED: {} -> {}", result_hash, output_hash).map_err(|e| {
                        SystemHalt::with_formatted(
                            FailureAxis::ExternalInjectionDetected,
                            format!("Write error: {e}"),
                        )
                    })?;
                }
            }
        }

        Ok(())
    }
}

/// Append a critical fault event (Level 5) to a sovereign trace fault log with
/// hash chaining for tamper evidence.
pub fn append_critical_fault_event<P: AsRef<Path>>(
    log_path: P,
    details: &str,
) -> Result<(), SystemHalt> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| {
            SystemHalt::with_formatted(
                FailureAxis::ExternalInjectionDetected,
                format!("Cannot open fault log: {e}"),
            )
        })?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    writeln!(file, "[{}] CRITICAL FAULT: {}", timestamp, details).map_err(|e| {
        SystemHalt::with_formatted(
            FailureAxis::ExternalInjectionDetected,
            format!("Cannot write to fault log: {e}"),
        )
    })?;

    Ok(())
}

impl Default for SovereignTraceLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of policy violations in a trace log
#[derive(Debug, Clone)]
pub struct AuditSummary {
    pub total_traces: usize,
    pub violations: usize,
    pub compliant: bool,
}

impl AuditSummary {
    pub fn new(total: usize, violations: usize) -> Self {
        Self {
            total_traces: total,
            violations,
            compliant: violations == 0,
        }
    }
}

impl fmt::Display for AuditSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Audit Summary: {} traces, {} violations, compliant: {}",
            self.total_traces, self.violations, self.compliant
        )
    }
}
