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

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::failure_axis::{FailureAxis, SystemHalt};

/// Lock-free microsecond clock adapter for 1 kHz frame stamps.
/// Monotonicity is guaranteed per process by CAS-clamping.
#[derive(Debug, Default)]
pub struct PtpClock {
    last_us: AtomicU64,
}

impl PtpClock {
    pub fn new() -> Self {
        Self {
            last_us: AtomicU64::new(0),
        }
    }

    /// Returns a non-decreasing microsecond timestamp.
    pub fn read_micros(&self) -> Result<u64, SystemHalt> {
        let observed = wall_clock_micros()?;
        let mut prev = self.last_us.load(Ordering::Relaxed);
        loop {
            let next = observed.max(prev.saturating_add(1));
            match self.last_us.compare_exchange_weak(
                prev,
                next,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(next),
                Err(actual) => prev = actual,
            }
        }
    }
}

fn wall_clock_micros() -> Result<u64, SystemHalt> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .map_err(|_e| SystemHalt::new(FailureAxis::TimingDriftFailure, "Clock read failed"))
}
