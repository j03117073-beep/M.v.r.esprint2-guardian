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

//! Audit/guardian logic that passively examines a trace of [`SubstrateNode`] states
//! and produces a `SystemHalt` reason if the trace deviates from the canonical
//! deterministic constitution.  This module is deliberately *observable only*; it
//! does **not** mutate the kernel state or influence execution.

use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::tlbss_types::SubstrateNode;
use sha2::{Digest, Sha256};

/// Canonical hash type for deterministic binding
pub type DeterministicHash = [u8; 32];

/// Canonical trace node representing a causal execution step
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceNode {
    pub index: u64,
    pub parent_hash: DeterministicHash,
    pub instruction_hash: DeterministicHash,
    pub state_diff_hash: DeterministicHash,
    pub scheduler_decision_hash: DeterministicHash,
}

/// First-class execution trace: ordered, immutable, hash-linked
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub nodes: Vec<TraceNode>,
    pub root_hash: DeterministicHash,
}

impl ExecutionTrace {
    /// Compute trace root hash from ordered trace nodes
    pub fn compute_root(nodes: &[TraceNode]) -> DeterministicHash {
        let mut hasher = Sha256::new();
        for node in nodes {
            hasher.update(node.index.to_le_bytes());
            hasher.update(&node.parent_hash);
            hasher.update(&node.instruction_hash);
            hasher.update(&node.state_diff_hash);
            hasher.update(&node.scheduler_decision_hash);
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&hasher.finalize());
        out
    }

    pub fn new(nodes: Vec<TraceNode>) -> Self {
        let root_hash = Self::compute_root(&nodes);
        Self { nodes, root_hash }
    }
}

/// Cryptographic certificate binding input → execution path → state → output
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeterminismCertificate {
    pub input_hash: DeterministicHash,
    pub ir_hash: DeterministicHash,
    pub execution_path_hash: DeterministicHash,
    pub scheduler_decision_chain_hash: DeterministicHash,
    pub trace_root_hash: DeterministicHash,
    pub state_root_hash: DeterministicHash,
    pub final_state_hash: DeterministicHash,
    pub binary_hash: DeterministicHash,
    pub environment_hash: DeterministicHash,
}

impl DeterminismCertificate {
    /// Emit certificate: all hashes must be canonical and reproducible
    pub fn commit(
        input_hash: DeterministicHash,
        ir_hash: DeterministicHash,
        execution_path_hash: DeterministicHash,
        scheduler_decision_chain_hash: DeterministicHash,
        trace: &ExecutionTrace,
        state_root_hash: DeterministicHash,
        final_state_hash: DeterministicHash,
        binary_hash: DeterministicHash,
        environment_hash: DeterministicHash,
    ) -> Self {
        Self {
            input_hash,
            ir_hash,
            execution_path_hash,
            scheduler_decision_chain_hash,
            trace_root_hash: trace.root_hash,
            state_root_hash,
            final_state_hash,
            binary_hash,
            environment_hash,
        }
    }

    /// Canonical byte representation for signing
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 * 9);
        out.extend_from_slice(&self.input_hash);
        out.extend_from_slice(&self.ir_hash);
        out.extend_from_slice(&self.execution_path_hash);
        out.extend_from_slice(&self.scheduler_decision_chain_hash);
        out.extend_from_slice(&self.trace_root_hash);
        out.extend_from_slice(&self.state_root_hash);
        out.extend_from_slice(&self.final_state_hash);
        out.extend_from_slice(&self.binary_hash);
        out.extend_from_slice(&self.environment_hash);
        out
    }
}

/// Passive audit structure used by the adversarial harness and, eventually, the
/// hardened runtime guardian.  The values here are the same constants used in
/// the canonical trace specification:
///
/// * `coherence_threshold` (τ) – minimum acceptable coherence score; traces with
///   a lower value are considered corrupted and mapped to a failure axis.
/// * `canonical_multiplier` – the masked contribution per tick (104 in the
///   standard run).  Used to compute expected charge increments.
#[derive(Debug, Clone)]
pub struct TestamentAudit {
    pub coherence_threshold: f64,
    pub canonical_multiplier: u64,
}

impl TestamentAudit {
    /// Create a new, default audit instance with the values used in the
    /// original specification (τ = 0.7, multiplier = 104).
    pub fn new() -> Self {
        Self {
            coherence_threshold: 0.7,
            canonical_multiplier: 104,
        }
    }

    /// Compute a very simple coherence metric over the supplied trace.  The
    /// implementation used here is intentionally lightweight: it sums the
    /// absolute difference between actual charge and the expected charge (based
    /// on the canonical multiplier), then normalises that value to the range
    /// `[0.0,1.0]`.  A perfectly deterministic trace therefore yields a score of
    /// `1.0` and any deviation reduces the score proportionally.
    fn compute_coherence(&self, trace: &[SubstrateNode]) -> f64 {
        if trace.is_empty() {
            return 0.0;
        }

        let mut total_deviation = 0u64;
        for (i, node) in trace.iter().enumerate() {
            let tick = (i + 1) as u64;
            let expected = self.canonical_multiplier * tick;
            let deviation = if node.charge > expected {
                node.charge - expected
            } else {
                expected - node.charge
            };
            total_deviation = total_deviation.saturating_add(deviation);
        }

        let normalized = (total_deviation as f64) / (trace.len() as f64);
        (1.0 - (normalized / 10000.0)).max(0.0).min(1.0)
    }

    /// Walk the trace and attempt to classify the first anomaly we see.  The
    /// returned `SystemHalt` uses one of the five canonical axes.  If the trace
    /// is clean the method returns `None`.
    pub fn evaluate(&self, trace: &[SubstrateNode]) -> Option<SystemHalt> {
        let coherence = self.compute_coherence(trace);
        
        if coherence < self.coherence_threshold {
            return Some(SystemHalt::new(
                FailureAxis::InternalInvariantBreach,
                "Trace coherence below threshold",
            ));
        }

        None
    }
}
