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

use crate::failure_axis::SystemHalt;
use crate::tlbss_types::SubstrateNode;

#[derive(Debug, Clone, Copy)]
pub struct ZeroState {
    pub value: u32,
}

impl Default for ZeroState {
    fn default() -> Self {
        Self { value: 0 }
    }
}

impl ZeroState {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct TestSuite {
    pub tests: Vec<TestResult>,
}

impl TestSuite {
    pub fn new() -> Self {
        Self { tests: Vec::new() }
    }

    pub fn add_test(&mut self, name: &str, passed: bool, message: Option<String>) {
        self.tests.push(TestResult {
            name: name.to_string(),
            passed,
            message,
        });
    }

    pub fn report(&self) {
        for test in &self.tests {
            if test.passed {
                println!("✓ {}", test.name);
            } else {
                println!("✗ {}", test.name);
                if let Some(msg) = &test.message {
                    println!("  {}", msg);
                }
            }
        }
    }

    pub fn all_passed(&self) -> bool {
        self.tests.iter().all(|t| t.passed)
    }
}

pub fn execute_zero_state_suite() -> TestSuite {
    let mut suite = TestSuite::new();

    // Test 1: ZeroState default
    let zero = ZeroState::default();
    suite.add_test(
        "ZeroState default value",
        zero.value == 0,
        Some("Expected 0, got {}".to_string()),
    );

    // Test 2: ZeroState new
    let zero_new = ZeroState::new(42);
    suite.add_test(
        "ZeroState new value",
        zero_new.value == 42,
        Some("Expected 42, got {}".to_string()),
    );

    // Test 3: SubstrateNode zero state
    let node = SubstrateNode::new(0);
    suite.add_test(
        "SubstrateNode zero charge",
        node.charge == 0,
        Some("Expected 0, got {}".to_string()),
    );
    suite.add_test(
        "SubstrateNode masked signal",
        node.masked_signal == (0 ^ 0x5A),
        Some("Expected 0x5A, got 0x{:02X}".to_string()),
    );
    suite.add_test(
        "SubstrateNode stable ticks",
        node.stable_ticks == 0,
        Some("Expected 0, got {}".to_string()),
    );

    suite
}

pub fn execute_with_first_tick(node: &mut SubstrateNode, sigma: u32) -> Result<TestSuite, SystemHalt> {
    let mut suite = TestSuite::new();

    // Simulate first tick with sigma
    // For now, simple logic: increment charge by sigma, check invariants

    let original_charge = node.charge;
    node.charge = node.charge.saturating_add(sigma as u64);
    node.stable_ticks = node.stable_ticks.saturating_add(1);

    // Validate the node
    match node.validate() {
        Ok(()) => {
            suite.add_test(
                "First tick validation",
                true,
                None,
            );
        }
        Err(halt) => {
            suite.add_test(
                "First tick validation",
                false,
                Some(format!("Halt: {:?}", halt)),
            );
            return Err(halt);
        }
    }

    suite.add_test(
        "Charge increment",
        node.charge == original_charge + sigma as u64,
        Some(format!("Expected {}, got {}", original_charge + sigma as u64, node.charge)),
    );

    suite.add_test(
        "Stable ticks increment",
        node.stable_ticks == 1,
        Some(format!("Expected 1, got {}", node.stable_ticks)),
    );

    Ok(suite)
}
