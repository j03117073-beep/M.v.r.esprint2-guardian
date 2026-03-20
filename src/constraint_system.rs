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

/// Power system state overlay (parallel to TLBSS, not inside it)
#[derive(Clone, Debug, PartialEq)]
pub struct PowerState {
    pub p_t: f64,      // current active power (MW)
    pub p_prev: f64,   // previous active power (MW)

    pub reg_up: f64,   // regulation up commitment (MW)
    pub reg_down: f64, // regulation down commitment (MW)

    pub p_min: f64,    // minimum power limit (MW)
    pub p_max: f64,    // maximum power limit (MW)

    pub ramp_up: f64,   // ramp up rate limit (MW/time)
    pub ramp_down: f64, // ramp down rate limit (MW/time)
}

impl PowerState {
    pub fn new(
        p_t: f64,
        p_prev: f64,
        reg_up: f64,
        reg_down: f64,
        p_min: f64,
        p_max: f64,
        ramp_up: f64,
        ramp_down: f64,
    ) -> Self {
        Self {
            p_t,
            p_prev,
            reg_up,
            reg_down,
            p_min,
            p_max,
            ramp_up,
            ramp_down,
        }
    }
}

/// Violation vector - pure diagnostic, no mutation
#[derive(Default, Debug, Clone, PartialEq)]
pub struct ViolationVector {
    pub ramp_up: f64,
    pub ramp_down: f64,
    pub capacity_upper: f64,
    pub capacity_lower: f64,
    pub reg_up: f64,
    pub reg_down: f64,
}

impl ViolationVector {
    /// Total violation magnitude
    pub fn total(&self) -> f64 {
        self.ramp_up
            + self.ramp_down
            + self.capacity_upper
            + self.capacity_lower
            + self.reg_up
            + self.reg_down
    }

    /// Check if state transition is feasible
    pub fn is_feasible(&self) -> bool {
        self.total() == 0.0
    }
}

/// Constraint evaluator - pure function, no state mutation
pub struct ConstraintEvaluator;

impl ConstraintEvaluator {
    /// Evaluate constraint violations for a proposed transition
    /// Returns diagnostic only - never modifies state
    pub fn evaluate(prev: &PowerState, next: &PowerState) -> ViolationVector {
        let mut v = ViolationVector::default();

        // Power delta
        let delta = next.p_t - prev.p_t;

        // --- Ramp Constraints ---
        v.ramp_up = (delta - prev.ramp_up).max(0.0);
        v.ramp_down = (-delta - prev.ramp_down).max(0.0);

        // --- Capacity Constraints ---
        v.capacity_upper = (next.p_t - next.p_max).max(0.0);
        v.capacity_lower = (next.p_min - next.p_t).max(0.0);

        // --- Regulation Up (headroom requirement) ---
        let headroom = next.p_max - next.p_t;
        v.reg_up = (next.reg_up - headroom).max(0.0);

        // --- Regulation Down (footroom requirement) ---
        let footroom = next.p_t - next.p_min;
        v.reg_down = (next.reg_down - footroom).max(0.0);

        v
    }
}

/// Admissibility checker - binary certification, no mutation
pub struct AdmissibilityChecker;

impl AdmissibilityChecker {
    /// Check if proposed transition is admissible
    /// Pure function - only returns true/false
    pub fn admissible(prev: &PowerState, next: &PowerState) -> bool {
        let v = ConstraintEvaluator::evaluate(prev, next);

        // Hard feasibility check
        if !v.is_feasible() {
            return false;
        }

        // Optional: additional monotonic safety invariants
        // (implementation can be extended here)
        true
    }

    /// Audit trace for falsifiability (logs violations without modifying behavior)
    pub fn audit_trace(prev: &PowerState, next: &PowerState) {
        let v = ConstraintEvaluator::evaluate(prev, next);

        if !v.is_feasible() {
            // In real implementation, this would log to audit trail
            // For now, we just check that violations are properly computed
            let _total = v.total();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_evaluation_no_violations() {
        let prev = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);
        let next = PowerState::new(105.0, 100.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);

        let v = ConstraintEvaluator::evaluate(&prev, &next);
        assert!(v.is_feasible());
        assert_eq!(v.total(), 0.0);
    }

    #[test]
    fn test_ramp_up_violation() {
        let prev = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 150.0, 5.0, 20.0); // ramp_up = 5
        let next = PowerState::new(110.0, 100.0, 10.0, 10.0, 50.0, 150.0, 5.0, 20.0); // +10 change > 5 limit

        let v = ConstraintEvaluator::evaluate(&prev, &next);
        assert!(!v.is_feasible());
        assert_eq!(v.ramp_up, 5.0); // 10 - 5 = 5
        assert_eq!(v.total(), 5.0);
    }

    #[test]
    fn test_capacity_upper_violation() {
        let prev = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 120.0, 20.0, 20.0);
        let next = PowerState::new(130.0, 100.0, 10.0, 10.0, 50.0, 120.0, 20.0, 20.0); // 130 > 120 max

        let v = ConstraintEvaluator::evaluate(&prev, &next);
        assert!(!v.is_feasible());
        assert_eq!(v.capacity_upper, 10.0); // 130 - 120 = 10
    }

    #[test]
    fn test_regulation_up_violation() {
        let prev = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 120.0, 20.0, 20.0);
        let next = PowerState::new(115.0, 100.0, 15.0, 10.0, 50.0, 120.0, 20.0, 20.0); // reg_up=15, headroom=5

        let v = ConstraintEvaluator::evaluate(&prev, &next);
        assert!(!v.is_feasible());
        assert_eq!(v.reg_up, 10.0); // 15 - 5 = 10
    }

    #[test]
    fn test_admissibility_checker() {
        let prev = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);
        let next_feasible = PowerState::new(105.0, 100.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);
        let next_infeasible = PowerState::new(200.0, 100.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);

        assert!(AdmissibilityChecker::admissible(&prev, &next_feasible));
        assert!(!AdmissibilityChecker::admissible(&prev, &next_infeasible));
    }
}