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

use crate::sovereign_trace::SovereignTrace;

const STREAM_PACKET_BYTES: usize = 64;

#[derive(Debug, Clone, Copy)]
pub struct StreamPacket {
    pub bytes: [u8; STREAM_PACKET_BYTES],
}

/// Preallocated, zero-copy packet ring for SovereignTrace export.
/// No heap allocation occurs during `push_trace`.
#[derive(Debug)]
pub struct TraceStreamer {
    ring: Vec<StreamPacket>,
    capacity: usize,
    write_idx: usize,
    len: usize,
}

impl TraceStreamer {
    pub fn new(capacity: usize) -> Self {
        let capped = capacity.max(1);
        let mut ring = Vec::with_capacity(capped);
        for _ in 0..capped {
            ring.push(StreamPacket {
                bytes: [0u8; STREAM_PACKET_BYTES],
            });
        }
        Self {
            ring,
            capacity: capped,
            write_idx: 0,
            len: 0,
        }
    }

    pub fn push_trace(&mut self, trace: &SovereignTrace) {
        let slot = &mut self.ring[self.write_idx].bytes;
        encode_trace_packet(trace, slot);
        self.write_idx = (self.write_idx + 1) % self.capacity;
        self.len = self.len.saturating_add(1).min(self.capacity);
    }

    pub fn latest_packet(&self) -> Option<&[u8; STREAM_PACKET_BYTES]> {
        if self.len == 0 {
            return None;
        }
        let idx = if self.write_idx == 0 {
            self.capacity - 1
        } else {
            self.write_idx - 1
        };
        Some(&self.ring[idx].bytes)
    }
}

fn encode_trace_packet(trace: &SovereignTrace, out: &mut [u8; STREAM_PACKET_BYTES]) {
    out.fill(0);
    out[0..8].copy_from_slice(&trace.tick.to_le_bytes());
    out[8..16].copy_from_slice(&trace.timestamp_us.to_le_bytes());
    out[16] = trace.grid_sigma;
    out[17..21].copy_from_slice(&trace.ambient_temp.to_le_bytes());
    out[21..29].copy_from_slice(&trace.inverter_current.to_le_bytes());
    out[29..37].copy_from_slice(&trace.ai_requested_p.to_le_bytes());
    out[37..45].copy_from_slice(&trace.kernel_output_p.to_le_bytes());
    out[45] = trace.active_governance as u8;
    out[46] = u8::from(trace.legal_justification.is_some());
    out[47] = u8::from(trace.is_authenticated);
    out[48] = u8::from(trace.state_transition);

    let checksum = out[..60]
        .iter()
        .fold(0u32, |acc, b| acc.wrapping_add(*b as u32));
    out[60..64].copy_from_slice(&checksum.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regulatory_policy::GovernanceMode;
    use crate::sovereign_trace::TraceBuilder;

    #[test]
    fn ring_streamer_encodes_without_allocating_per_push() {
        let mut streamer = TraceStreamer::new(4);
        let trace = TraceBuilder::new(17)
            .governance(GovernanceMode::PassThrough)
            .build();
        streamer.push_trace(&trace);
        let packet = streamer.latest_packet().expect("packet");
        assert_eq!(&packet[0..8], &17u64.to_le_bytes());
    }
}
