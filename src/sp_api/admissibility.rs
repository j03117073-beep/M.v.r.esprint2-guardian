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

const MAX_NODES: usize = 256;
const WORDS: usize = MAX_NODES / 64;

#[derive(Debug, Clone, Copy)]
pub struct GlobalAdmissibilitySnapshot {
    pub active_words: [u64; WORDS],
    pub admissible_words: [u64; WORDS],
    pub global_green: bool,
}

/// Bitmask matrix for O(1) global health checks.
#[derive(Debug, Clone)]
pub struct GlobalAdmissibilityMatrix {
    active_words: [u64; WORDS],
    admissible_words: [u64; WORDS],
    global_green: bool,
}

impl GlobalAdmissibilityMatrix {
    pub fn new() -> Self {
        Self {
            active_words: [0; WORDS],
            admissible_words: [0; WORDS],
            global_green: true,
        }
    }

    pub fn register_node(&mut self, node_idx: usize) -> bool {
        let Some((word, bit)) = split_index(node_idx) else {
            return false;
        };
        let mask = 1u64 << bit;
        self.active_words[word] |= mask;
        self.admissible_words[word] |= mask;
        self.global_green = self.active_words == self.admissible_words;
        true
    }

    pub fn update_node(&mut self, node_idx: usize, admissible: bool) -> bool {
        let Some((word, bit)) = split_index(node_idx) else {
            return false;
        };
        let mask = 1u64 << bit;
        if (self.active_words[word] & mask) == 0 {
            return false;
        }
        if admissible {
            self.admissible_words[word] |= mask;
        } else {
            self.admissible_words[word] &= !mask;
        }
        self.global_green = self.active_words == self.admissible_words;
        true
    }

    pub fn global_green(&self) -> bool {
        self.global_green
    }

    pub fn snapshot(&self) -> GlobalAdmissibilitySnapshot {
        GlobalAdmissibilitySnapshot {
            active_words: self.active_words,
            admissible_words: self.admissible_words,
            global_green: self.global_green,
        }
    }
}

fn split_index(node_idx: usize) -> Option<(usize, usize)> {
    if node_idx >= MAX_NODES {
        return None;
    }
    Some((node_idx / 64, node_idx % 64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_status_flips_from_single_node() {
        let mut matrix = GlobalAdmissibilityMatrix::new();
        assert!(matrix.register_node(7));
        assert!(matrix.global_green());
        assert!(matrix.update_node(7, false));
        assert!(!matrix.global_green());
        assert!(matrix.update_node(7, true));
        assert!(matrix.global_green());
    }
}
