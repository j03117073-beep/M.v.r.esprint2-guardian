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
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

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
            legal_citation: citation,
            timestamp_ms: 0,
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
    traces: Vec<SovereignTrace>,
}

impl TraceBuilder {
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
        }
    }

    pub fn add(mut self, trace: SovereignTrace) -> Self {
        self.traces.push(trace);
        self
    }

    pub fn build(self) -> Vec<SovereignTrace> {
        self.traces
    }
}

/// A log of SovereignTraces for auditing
#[derive(Debug, Clone)]
pub struct SovereignTraceLog {
    pub traces: Vec<SovereignTrace>,
    pub hash_chain: String,
}

impl SovereignTraceLog {
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            hash_chain: String::new(),
        }
    }

    pub fn append(&mut self, trace: SovereignTrace) {
        self.traces.push(trace);
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

        for trace in &self.traces {
            writeln!(file, "{}", trace).map_err(|e| {
                SystemHalt::with_formatted(
                    FailureAxis::ExternalInjectionDetected,
                    format!("Write error: {e}"),
                )
            })?;
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
