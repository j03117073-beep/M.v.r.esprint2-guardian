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

use crate::tlbss_integrity_engine::TlbssTickRecord;

/// Unique identifier for invariants
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvariantId(pub String);

#[derive(Debug, Clone, Copy)]
pub struct GuardianDecision {
    pub admissible: bool,
    pub below_threshold: bool,
    pub boundary_saturation: bool,
    pub coherence_saturated: bool,
    pub l7_veto_fire: bool,
    pub l7_veto_tick: Option<u64>,
}

/// Non-agentic boundary certifier.
/// Reads coherence and saturation only; it does not generate commands.
#[derive(Debug, Clone, Copy)]
pub struct AuditGuardian {
    coherence_threshold: f32,
}

impl AuditGuardian {
    pub fn new(coherence_threshold: f32) -> Self {
        Self {
            coherence_threshold,
        }
    }

    pub fn certify(&self, rec: &TlbssTickRecord) -> GuardianDecision {
        self.certify_with_pressure(rec, 1.0)
    }

    /// Axis-4 veto: if saturation is certified and pressure remains high,
    /// fire L7 on the exact certifying tick.
    pub fn certify_with_pressure(
        &self,
        rec: &TlbssTickRecord,
        input_pressure_norm: f32,
    ) -> GuardianDecision {
        let below_threshold = rec.stability_index.l6_coherence < self.coherence_threshold;
        let boundary_saturation = rec.boundary_condition;
        let coherence_saturated = rec.coherence_saturated;
        let pressure_high = input_pressure_norm >= 0.70;
        let delta_s_zero = rec.delta_state == 0;
        let l7_veto_fire = coherence_saturated && delta_s_zero && pressure_high;
        let admissible = !(below_threshold || boundary_saturation || coherence_saturated);

        GuardianDecision {
            admissible,
            below_threshold,
            boundary_saturation,
            coherence_saturated,
            l7_veto_fire,
            l7_veto_tick: if l7_veto_fire { Some(rec.tick) } else { None },
        }
    }
}
