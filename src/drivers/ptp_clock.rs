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

use crate::trusted_time::{TimeProvider, TimeSource, TrustedTime};

pub struct PtpClock;

impl TimeProvider for PtpClock {
    fn now_raw(&self) -> TrustedTime {
        // TODO: replace with a real PTP/NIC hardware clock read.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time error");
        let ns = now.as_nanos().min(u64::MAX as u128) as u64;

        TrustedTime {
            wall_time_ns: ns,
            monotonic_ns: ns,
            source: TimeSource::PTP,
            uncertainty_ns: 1_000, // placeholder 1us target envelope
        }
    }
}
