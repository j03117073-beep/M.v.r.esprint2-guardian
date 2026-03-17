#![deny(unsafe_code)]

use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::tlbss_types::SubstrateNode;

/// Part 2: Logic Trace Proof
/// Canonical deterministic reference for 100-tick compliance
pub struct ComplianceTrace {
    pub input_signal: u8,
    pub mask_constant: u8,
    pub masked_contribution: u64,
    pub l7_threshold: u64,
    pub total_ticks: u64,
    pub expected_final_charge: u64,
}

impl ComplianceTrace {
    pub fn canonical() -> Self {
        // Reference Conditions per Part 2
        let input_signal = 50u8;
        let mask_constant = 0x5A;

        // Masked Contribution: 50 ⊕ 0x5A = 104
        let masked: u8 = input_signal ^ mask_constant;

        Self {
            input_signal,
            mask_constant,
            masked_contribution: masked as u64,
            l7_threshold: 100,
            total_ticks: 100,
            expected_final_charge: 10400,
        }
    }
}

/// Verify that runtime charge matches precomputed deterministic value
pub fn verify_charge_determinism(
    runtime_charge: u64,
    tick_count: u64,
    trace: &ComplianceTrace,
) -> Result<(), SystemHalt> {
    let expected = trace.masked_contribution * tick_count;

    if runtime_charge != expected {
        return Err(SystemHalt::new(
            FailureAxis::InternalInvariantBreach,
            "Charge determinism violation",
        ));
    }

    Ok(())
}

/// Verify that stable_ticks matches tick count
/// (should increment by 1 per tick once charge ≥ threshold)
pub fn verify_stability_counter(
    runtime_stable_ticks: u8,
    tick_count: u64,
) -> Result<(), SystemHalt> {
    // For 100 ticks with charge always ≥ 100 after tick 1,
    // stable_ticks should be min(tick_count, 255) clamped to u8
    let expected_min = if tick_count >= 255 {
        255u8
    } else {
        tick_count as u8
    };

    if runtime_stable_ticks < expected_min {
        return Err(SystemHalt::new(
            FailureAxis::InternalInvariantBreach,
            "Stability counter violation",
        ));
    }

    Ok(())
}

/// Enforce canonical trace over N ticks
/// Returns final verified state or SystemHalt
pub fn enforce_canonical_trace(
    node: &SubstrateNode,
    ticks_executed: u64,
    trace: &ComplianceTrace,
) -> Result<(), SystemHalt> {
    // Verify charge matches deterministic formula
    verify_charge_determinism(node.charge, ticks_executed, trace)?;

    // Verify stability counter advanced
    verify_stability_counter(node.stable_ticks, ticks_executed)?;

    Ok(())
}
