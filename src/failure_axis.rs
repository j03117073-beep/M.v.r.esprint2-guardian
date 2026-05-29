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


/* Failure axis registry for deterministic halts */

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureAxis {
    InternalInvariantBreach,
    ExternalInjectionDetected,
    TimingDriftFailure,
    AuthorityInversionAttempt,

    // Additional axes introduced for audit/hardening
    Reference,
    Feedback,
    Coupling,
    Resolution,
    Axiom6_7Misalignment,
    TpmUnavailable,
    UnauthorizedMode,
}

#[derive(Debug, Clone)]
pub struct SystemHalt {
    pub axis: FailureAxis,
    pub message: String,
}

impl SystemHalt {
    pub fn new(axis: FailureAxis, message: &str) -> Self {
        Self {
            axis,
            message: message.to_string(),
        }
    }

    pub fn with_formatted(axis: FailureAxis, message: String) -> Self {
        Self { axis, message }
    }
}
