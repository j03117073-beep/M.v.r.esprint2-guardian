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

use std::sync::atomic::{AtomicU32, Ordering};

pub const MAX_GRID_NODES: usize = 256;
const Q16_SCALE: f32 = 65_535.0;

/// Lock-free supervisory resonance coefficient store.
/// P-API reads at frame start; S.P-API writes atomically between frames.
#[derive(Debug)]
pub struct ResonanceStore {
    q16_coefficients: Vec<AtomicU32>,
}

impl ResonanceStore {
    pub fn new(node_count: usize, default_coeff: f32) -> Self {
        let capped = node_count.min(MAX_GRID_NODES);
        let encoded = to_q16(default_coeff);
        let mut q16_coefficients = Vec::with_capacity(capped);
        for _ in 0..capped {
            q16_coefficients.push(AtomicU32::new(encoded));
        }
        Self { q16_coefficients }
    }

    pub fn node_count(&self) -> usize {
        self.q16_coefficients.len()
    }

    pub fn set_node_coefficient(&self, node_idx: usize, coeff: f32) -> bool {
        if node_idx >= self.q16_coefficients.len() {
            return false;
        }
        self.q16_coefficients[node_idx].store(to_q16(coeff), Ordering::Release);
        true
    }

    pub fn read_node_coefficient(&self, node_idx: usize) -> Option<f32> {
        let atom = self.q16_coefficients.get(node_idx)?;
        Some(from_q16(atom.load(Ordering::Acquire)))
    }
}

fn to_q16(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * Q16_SCALE).round() as u32
}

fn from_q16(v: u32) -> f32 {
    (v as f32 / Q16_SCALE).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coefficients_round_trip() {
        let store = ResonanceStore::new(4, 0.4);
        assert!(store.set_node_coefficient(2, 0.75));
        let got = store.read_node_coefficient(2).expect("node");
        assert!((got - 0.75).abs() < 0.01);
    }
}
