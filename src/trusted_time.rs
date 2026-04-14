// Copyright (c) 2026 OBINNA JAMES EJIOFOR
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
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum TimeSource {
    PTP = 0,
    GPS = 1,
    NTP = 2,
    LOCAL = 3,
}

impl Default for TimeSource {
    fn default() -> Self {
        Self::LOCAL
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct TrustedTime {
    pub wall_time_ns: u64,
    pub monotonic_ns: u64,
    pub source: TimeSource,
    pub uncertainty_ns: u64,
}

pub trait TimeProvider: Send + Sync {
    fn now_raw(&self) -> TrustedTime;
}

/// Default fallback provider (replace with hardened PTP/GPS provider in production deployment).
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn now_raw(&self) -> TrustedTime {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards");
        let ns = now.as_nanos().min(u64::MAX as u128) as u64;

        TrustedTime {
            wall_time_ns: ns,
            monotonic_ns: ns,
            source: TimeSource::LOCAL,
            uncertainty_ns: 1_000_000, // 1 ms fallback uncertainty
        }
    }
}

pub struct TrustedTimeAuthority {
    provider: Arc<dyn TimeProvider>,
    last_monotonic: AtomicU64,
}

impl TrustedTimeAuthority {
    pub fn new(provider: Arc<dyn TimeProvider>) -> Self {
        Self {
            provider,
            last_monotonic: AtomicU64::new(0),
        }
    }

    pub fn now(&self) -> TrustedTime {
        let raw = self.provider.now_raw();
        let mut prev = self.last_monotonic.load(Ordering::Relaxed);

        loop {
            let monotonic = if raw.monotonic_ns <= prev {
                prev.saturating_add(1)
            } else {
                raw.monotonic_ns
            };

            match self.last_monotonic.compare_exchange_weak(
                prev,
                monotonic,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return TrustedTime {
                        wall_time_ns: raw.wall_time_ns,
                        monotonic_ns: monotonic,
                        source: raw.source,
                        uncertainty_ns: raw.uncertainty_ns,
                    };
                }
                Err(actual) => prev = actual,
            }
        }
    }
}

