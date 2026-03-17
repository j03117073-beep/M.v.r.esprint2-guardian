#![deny(unsafe_code)]

//! Audit/guardian logic that passively examines a trace of [`SubstrateNode`] states
//! and produces a `SystemHalt` reason if the trace deviates from the canonical
//! deterministic constitution.  This module is deliberately *observable only*; it
//! does **not** mutate the kernel state or influence execution.

use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::tlbss_types::SubstrateNode;

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
