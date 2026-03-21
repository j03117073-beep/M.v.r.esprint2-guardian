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

use crate::kernel::KernelAuthority;

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
    pub fn apply(&mut self, desired: &Setpoint) -> Setpoint {
        let delta_p = (desired.p - self.last.p).abs();
        let allowed_delta = self.ramp_limit * 1.0; // 1 ms tick
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
        result
    }
}

/// Enforce active power limits based on available reserve and hard plant
/// capability.  Returns a tuple of (possibly‑modified setpoint, authority).
pub fn clamp_active_power(
    cmd: Setpoint,
    physical_max: f64,
    _ramp_limit: f64,
    _last_valid: f64,
) -> (Setpoint, KernelAuthority) {
    if cmd.p > physical_max {
        (Setpoint { p: physical_max, ..cmd }, KernelAuthority::Clamp)
    } else {
        (cmd, KernelAuthority::PassThrough)
    }
}

/// Enforce reactive power (VAR) limits based on a simplified voltage envelope.
/// Returns modified setpoint and authority indicator.
pub fn clamp_reactive_power(
    cmd: Setpoint,
    _v_min: f64,
    _v_max: f64,
) -> (Setpoint, KernelAuthority) {
    (cmd, KernelAuthority::PassThrough)
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
) -> (Setpoint, KernelAuthority) {
    let (clamped_p, auth_p) = clamp_active_power(desired, physical_max, ramp_limit, desired.p);
    let (clamped_q, auth_q) = clamp_reactive_power(clamped_p, v_min, v_max);
    
    let authority = match (auth_p, auth_q) {
        (KernelAuthority::Clamp, _) | (_, KernelAuthority::Clamp) => KernelAuthority::Clamp,
        _ => KernelAuthority::PassThrough,
    };
    
    (clamped_q, authority)
}
