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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionType {
    NormalAccumulation,
    L7TransitionCandidate,
}

pub struct StateSentinel;

impl StateSentinel {
    pub fn validate_pre_write(node: &SubstrateNode, cycle_index: u64) -> Result<(), SystemHalt> {
        Self::check_mask_invariant(node)?;
        Self::check_sequence_invariant(cycle_index)?;
        Self::check_boundary_invariant(node)?;
        Ok(())
    }

    fn check_mask_invariant(node: &SubstrateNode) -> Result<(), SystemHalt> {
        let unmasked = node.masked_signal ^ 0x5A;
        if unmasked > 1 {
            return Err(SystemHalt::new(
                FailureAxis::InternalInvariantBreach,
                "Mask invariant violated",
            ));
        }
        Ok(())
    }

    fn check_sequence_invariant(cycle_index: u64) -> Result<(), SystemHalt> {
        if cycle_index > u64::MAX / 2 {
            return Err(SystemHalt::new(
                FailureAxis::TimingDriftFailure,
                "Sequence overflow detected",
            ));
        }
        Ok(())
    }

    fn check_boundary_invariant(node: &SubstrateNode) -> Result<(), SystemHalt> {
        if node.charge == u64::MAX {
            return Err(SystemHalt::new(
                FailureAxis::InternalInvariantBreach,
                "Charge overflow",
            ));
        }
        Ok(())
    }
}

pub fn classify_transition(node: &SubstrateNode) -> TransitionType {
    if node.stable_ticks >= 3 {
        TransitionType::L7TransitionCandidate
    } else {
        TransitionType::NormalAccumulation
    }
}

// in real‑time context, logging to disk is forbidden.  Instead we print
// to stderr; an out‑of‑band collector thread may capture and persist these
// messages when the system is not in its 1 kHz loop.

pub fn latch_halt_log(halt: &SystemHalt) {
    eprintln!("[HALT] Axis={:?} Message={}", halt.axis, halt.message);
}

pub fn execute_tick(
    node: &mut SubstrateNode,
    cycle_index: u64,
    signal: u8,
) -> Result<(), SystemHalt> {
    let transition = classify_transition(node);

    StateSentinel::validate_pre_write(node, cycle_index)?;

    // Apply bit-masking law: C = σ ⊕ 0x5A
    let masked_contribution = (signal ^ 0x5A) as u64;

    match transition {
        TransitionType::NormalAccumulation => {
            node.charge = node.charge.saturating_add(masked_contribution);
            if node.charge >= 100 {
                node.stable_ticks = node.stable_ticks.saturating_add(1);
            }
        }
        TransitionType::L7TransitionCandidate => {
            node.charge = node.charge.saturating_add(masked_contribution);
            node.stable_ticks = node.stable_ticks.saturating_add(1);
        }
    }

    Ok(())
}
