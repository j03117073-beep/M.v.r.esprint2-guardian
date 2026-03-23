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

use crate::audit_guardian::GuardianDecision;
use crate::hal_output::OutputCommand;
use crate::tlbss_integrity_engine::TlbssTickRecord;

#[derive(Debug, Clone)]
pub struct VisionsDecision {
    pub allow_dispatch: bool,
    pub externalize_to_entity_c: bool,
    pub command: OutputCommand,
}

/// Active-control gate. Emits commands only when guardian certifies admissible.
#[derive(Debug, Clone, Copy, Default)]
pub struct VisionsCore;

impl VisionsCore {
    pub fn new() -> Self {
        Self
    }

    pub fn route(
        &self,
        tick: u64,
        rec: &TlbssTickRecord,
        guardian: GuardianDecision,
    ) -> VisionsDecision {
        let externalize_to_entity_c = rec.boundary_condition
            && rec.coherence_saturated
            && rec.dimensional_transition.is_some();
        let allow_dispatch = guardian.admissible && !externalize_to_entity_c;

        let command = if allow_dispatch {
            OutputCommand {
                tick,
                state_vector: rec.state.as_array(),
                coherence_metric: rec.stability_index.l6_coherence,
                safe_state: false,
            }
        } else {
            OutputCommand {
                tick,
                state_vector: [0, 0, 0],
                coherence_metric: rec.stability_index.l6_coherence,
                safe_state: true,
            }
        };

        VisionsDecision {
            allow_dispatch,
            externalize_to_entity_c,
            command,
        }
    }
}
