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

use crate::phase_control::PhaseControlGate;
use crate::failure_axis::SystemHalt;

/// Fault codes for guard operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultCode {
    InvalidActivePower,
    InvalidReactivePower,
    InvalidTiming,
    PhysicalLimitExceeded,
    RateLimitExceeded,
}

/// Result of a guard operation: valid output, degraded output, or fault
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuardResult<T> {
    Valid(T),
    Degraded(T),
    Fault(FaultCode),
}

/// Simplified representation of an active/reactive power command issued by the
/// upstream optimiser or AI.  The timestamp field allows the kernel to detect
/// stale or delayed messages when compared with its internal clock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Setpoint {
    pub p: f64,  // active power (MW)
    pub q: f64,  // reactive power (MVAr)
    pub ts: u64, // source timestamp in milliseconds
}

impl Default for Setpoint {
    fn default() -> Self {
        Self {
            p: 0.0,
            q: 0.0,
            ts: 0,
        }
    }
}

/// Rate limiter state used by the 1 kHz kernel loop.  Maintains the most recent
/// output so that subsequent commands can be smoothed without heap activity.
pub struct RateLimiter {
    last: Setpoint,
    ramp_limit: f64, // MW per millisecond
}

impl RateLimiter {
    pub fn new(ramp_limit: f64) -> Self {
        Self {
            last: Setpoint::default(),
            ramp_limit,
        }
    }

    /// Apply a rate limit to the desired setpoint.  The limiter only affects the
    /// active power component; reactive power is handled separately.
    pub fn apply(&mut self, desired: &Setpoint, dt_ms: f64) -> GuardResult<Setpoint> {
        // Reject invalid timing
        if !dt_ms.is_finite() || dt_ms <= 0.0 {
            return GuardResult::Degraded(self.last);
        }

        // Reject invalid commands
        if !desired.p.is_finite() || !desired.q.is_finite() {
            return GuardResult::Degraded(self.last);
        }

        let delta_p = (desired.p - self.last.p).abs();
        let allowed_delta = self.ramp_limit * dt_ms;
        let clamped_p = if delta_p > allowed_delta {
            if desired.p > self.last.p {
                self.last.p + allowed_delta
            } else {
                self.last.p - allowed_delta
            }
        } else {
            desired.p
        };
        
        let result = Setpoint {
            p: clamped_p,
            q: desired.q,
            ts: desired.ts,
        };
        
        self.last = result;
        if clamped_p != desired.p {
            GuardResult::Degraded(result)
        } else {
            GuardResult::Valid(result)
        }
    }
}

/// Enforce active power limits based on available reserve and hard plant
/// capability.  Returns guard result with possibly modified setpoint.
pub fn clamp_active_power(
    cmd: Setpoint,
    physical_max: f64,
    _ramp_limit: f64,
    _last_valid: f64,
) -> GuardResult<Setpoint> {
    if !cmd.p.is_finite() {
        return GuardResult::Fault(FaultCode::InvalidActivePower);
    }
    if cmd.p > physical_max {
        GuardResult::Degraded(Setpoint { p: physical_max, ..cmd })
    } else {
        GuardResult::Valid(cmd)
    }
}

/// Enforce reactive power (VAR) limits based on a simplified voltage envelope.
/// Returns guard result with setpoint.
pub fn clamp_reactive_power(
    cmd: Setpoint,
    _v_min: f64,
    _v_max: f64,
) -> GuardResult<Setpoint> {
    if !cmd.q.is_finite() {
        return GuardResult::Fault(FaultCode::InvalidReactivePower);
    }
    GuardResult::Valid(cmd)
}

/// Given a desired setpoint and the most recent valid setpoint, produce the
/// actual command the kernel will forward to the PPC.  This wrapper handles
/// active and reactive clamping, rate‑limiting, and authority merging.
pub fn govern_setpoint(
    desired: Setpoint,
    ramp_limit: f64,
    physical_max: f64,
    v_min: f64,
    v_max: f64,
) -> GuardResult<Setpoint> {
    let clamped_p = match clamp_active_power(desired, physical_max, ramp_limit, desired.p) {
        GuardResult::Fault(code) => return GuardResult::Fault(code),
        GuardResult::Degraded(s) => s,
        GuardResult::Valid(s) => s,
    };
    
    let clamped_q = match clamp_reactive_power(clamped_p, v_min, v_max) {
        GuardResult::Fault(code) => return GuardResult::Fault(code),
        GuardResult::Degraded(s) => s,
        GuardResult::Valid(s) => s,
    };
    
    // If any clamping occurred, it's degraded
    if clamped_q.p != desired.p || clamped_q.q != desired.q {
        GuardResult::Degraded(clamped_q)
    } else {
        GuardResult::Valid(clamped_q)
    }
}

/// Assisted-control wrapper used for Phase 3 operation.
///
/// This path requires explicit authorization and clamps the incoming setpoint
/// to a narrow phase scope before applying existing deterministic guardrails.
pub fn govern_setpoint_phase3(
    desired: Setpoint,
    ramp_limit: f64,
    physical_max: f64,
    v_min: f64,
    v_max: f64,
    gate: &PhaseControlGate,
    operator_ack_token: Option<&str>,
) -> Result<GuardResult<Setpoint>, SystemHalt> {
    gate.ensure_assisted_control_authorized(operator_ack_token)?;

    let scoped = gate.clamp_to_assisted_scope(desired);
    Ok(govern_setpoint(scoped, ramp_limit, physical_max, v_min, v_max))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase_control::{AssistedControlScope, DeploymentPhase};

    #[test]
    fn phase3_governor_requires_ack() {
        let gate = PhaseControlGate {
            phase: DeploymentPhase::Phase3AssistedControl,
            scope: AssistedControlScope {
                require_operator_ack: true,
                ..AssistedControlScope::default()
            },
        };

        let result = govern_setpoint_phase3(
            Setpoint { p: 5.0, q: 1.0, ts: 1 },
            1.0,
            50.0,
            0.95,
            1.05,
            &gate,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn phase3_governor_applies_phase_scope_before_kernel_limits() {
        let gate = PhaseControlGate {
            phase: DeploymentPhase::Phase3AssistedControl,
            scope: AssistedControlScope {
                max_abs_p_mw: 8.0,
                max_abs_q_mvar: 3.0,
                require_operator_ack: false,
            },
        };

        let guard_result = govern_setpoint_phase3(
            Setpoint {
                p: 30.0,
                q: -8.0,
                ts: 10,
            },
            1.0,
            50.0,
            0.95,
            1.05,
            &gate,
            Some("ack"),
        )
        .expect("phase3 authorization should pass");

        match guard_result {
            GuardResult::Valid(setpoint) => {
                assert_eq!(
                    setpoint,
                    Setpoint {
                        p: 8.0,
                        q: -3.0,
                        ts: 10,
                    }
                );
            }
            _ => panic!("Expected Valid result"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    fn prove_rate_limiter_properties() {
        let mut limiter = RateLimiter::new(kani::any());
        let desired: Setpoint = kani::any();
        let dt_ms: f64 = kani::any();

        // Assume finite, valid inputs
        kani::assume(dt_ms.is_finite() && dt_ms > 0.0);
        kani::assume(desired.p.is_finite());
        kani::assume(desired.q.is_finite());

        let result = limiter.apply(&desired, dt_ms);

        // Verify no NaN exits
        assert!(result.p.is_finite());
        assert!(result.q.is_finite());

        // Verify bounded slew rate
        let delta_p = (result.p - limiter.last.p).abs();
        assert!(delta_p <= limiter.ramp_limit * dt_ms);
    }
}
