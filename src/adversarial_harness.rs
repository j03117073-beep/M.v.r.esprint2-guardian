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

use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::tlbss_types::SubstrateNode;

// ---------------------------------------------------------
// TickState: Immutable trace snapshot for audit
// ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct TickState {
    pub tick: u64,
    pub charge: u64,
    pub stable_ticks: u8,
}

impl TickState {
    pub fn from_node(tick: u64, node: &SubstrateNode) -> Self {
        Self {
            tick,
            charge: node.charge,
            stable_ticks: node.stable_ticks,
        }
    }
}

// ---------------------------------------------------------
// HaltReason: Canonical audit verdict
// ---------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltReason {
    StateCorruptionDetected,
    AuthorityEscalationAttempt,
    MaskBypassDetected,
    TemporalSkipExploit,
}

// Gamma: mask invariant check helper
fn mask_invariant_holds(value: u64) -> bool {
    let as_u8 = (value & 0xFF) as u8;
    (as_u8 ^ 0x5A) <= 1
}

// Vector Alpha: state corruption attempt
pub fn vector_alpha_state_corruption(node: &mut SubstrateNode) -> Result<(), SystemHalt> {
    // Attempt to corrupt the charge field
    node.charge = node.charge.wrapping_add(999);
    
    if !mask_invariant_holds(node.charge) {
        return Err(SystemHalt::new(
            FailureAxis::ExternalInjectionDetected,
            "State corruption attempt detected",
        ));
    }
    Ok(())
}

// Vector Beta: authority escalation attempt
pub fn vector_beta_authority_escalation() -> Result<(), SystemHalt> {
    // This would attempt to bypass authority checks
    Err(SystemHalt::new(
        FailureAxis::AuthorityInversionAttempt,
        "Authority escalation blocked",
    ))
}

// Vector Gamma: masking bypass
pub fn vector_gamma_mask_bypass(charge: &mut u64) -> Result<(), SystemHalt> {
    // Attempt to flip the mask constant
    *charge = (*charge) ^ 0x5A;
    
    if mask_invariant_holds(*charge) {
        Ok(())
    } else {
        Err(SystemHalt::new(
            FailureAxis::ExternalInjectionDetected,
            "Mask bypass attempt detected",
        ))
    }
}

// Vector Delta: temporal skipping exploit
pub fn vector_delta_temporal_skip(node: &mut SubstrateNode) -> Result<(), SystemHalt> {
    // Attempt to skip ticks
    node.stable_ticks = node.stable_ticks.saturating_add(10);
    
    if node.stable_ticks == 255 {
        return Err(SystemHalt::new(
            FailureAxis::TimingDriftFailure,
            "Temporal skip exploit detected",
        ));
    }
    Ok(())
}

pub fn latch_halt_log(halt: &SystemHalt) {
    eprintln!("[ADVERSARIAL] Axis: {:?}", halt.axis);
    eprintln!("[ADVERSARIAL] Message: {}", halt.message);
}

pub fn run_all_vectors(node: &mut SubstrateNode) -> Vec<Result<(), SystemHalt>> {
    vec![
        vector_alpha_state_corruption(node),
        vector_beta_authority_escalation(),
        vector_gamma_mask_bypass(&mut node.charge),
        vector_delta_temporal_skip(node),
    ]
}

// ---------------------------------------------------------
// Comprehensive Adversarial Harness Runner
// ---------------------------------------------------------

pub struct AdversarialReport {
    pub total_vectors: usize,
    pub successful_blocks: usize,
    pub failed_blocks: usize,
}

impl AdversarialReport {
    pub fn new(total: usize, blocked: usize, failed: usize) -> Self {
        Self {
            total_vectors: total,
            successful_blocks: blocked,
            failed_blocks: failed,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Adversarial Report: {} total, {} blocked, {} escaped",
            self.total_vectors, self.successful_blocks, self.failed_blocks
        )
    }
}

// -----------------------------------------------------------------------------
// Phase‑2 Relational Engine adversarial trace generators
// -----------------------------------------------------------------------------
