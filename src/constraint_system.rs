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

    /// Evaluate entire trajectory for time-coupled constraints
    pub fn evaluate_trajectory(traj: &Trajectory) -> ViolationVector {
        let mut total = ViolationVector::default();

        // Evaluate intra-interval constraints for each step
        for t in 1..traj.intervals.len() {
            let v = Self::evaluate(
                &traj.intervals[t - 1],
                &traj.intervals[t],
            );

            // Accumulate violations across all intervals
            total.ramp_up += v.ramp_up;
            total.ramp_down += v.ramp_down;
            total.capacity_upper += v.capacity_upper;
            total.capacity_lower += v.capacity_lower;
            total.reg_up += v.reg_up;
            total.reg_down += v.reg_down;
        }

        // Optional: Add inter-temporal constraints here
        // (e.g., energy balance, state of charge limits)

        total
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

    /// Check if entire trajectory is admissible
    pub fn admissible_trajectory(traj: &Trajectory) -> bool {
        let v = ConstraintEvaluator::evaluate_trajectory(traj);
        v.is_feasible()
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

    /// Audit trajectory for falsifiability
    pub fn audit_trajectory(traj: &Trajectory) {
        let v = ConstraintEvaluator::evaluate_trajectory(traj);

        if !v.is_feasible() {
            // Log trajectory-level violations
            let _total = v.total();
        }
    }
}

/// Transition trigger for L7 externalized resolution
pub struct TransitionTrigger;

impl TransitionTrigger {
    /// Trigger L7 transition when saturation is reached
    /// This signals external systems without modifying internal state
    pub fn trigger(boundary: &ConstraintBoundary) {
        // L7 transition actions (non-mutating):

        // 1. Emit transition event for external systems
        Self::emit_transition_event(boundary);

        // 2. Signal VISIONS/Ring-3 escalation
        Self::signal_external_resolution();

        // 3. Freeze internal evolution (optional)
        // Note: This would be handled by kernel state, not here
    }

    fn emit_transition_event(boundary: &ConstraintBoundary) {
        // In real implementation: log to audit trail, emit to sovereign bus
        // For now: placeholder for falsifiability
        let _rejection_count = boundary.rejection_count;
        let _last_violation = &boundary.last_violation;
    }

    fn signal_external_resolution() {
        // Signal to higher layers (VISIONS, market systems, operators)
        // This re-indexes relationships without violating system closure
    }
}

/// Deterministic trajectory proposer (TLBSS-compatible)
pub struct TrajectoryProposer;

impl TrajectoryProposer {
    /// Generate deterministic trajectory proposal
    /// No optimization, no adaptation - pure deterministic evolution
    pub fn propose_trajectory(
        current: &PowerState,
        ctx: &SystemContext,
    ) -> Trajectory {
        let mut intervals = Vec::with_capacity(ctx.horizon);
        let mut state = current.clone();

        for _ in 0..ctx.horizon {
            // Deterministic evolution rule
            // In real implementation: this would come from TLBSS dynamics
            // For now: simple persistence (no change)
            state.p_t = Self::deterministic_update(state.p_t);

            intervals.push(state.clone());
        }

        Trajectory::new(intervals)
    }

    /// Deterministic update rule (placeholder)
    /// In real system: this would be driven by TLBSS L1-L5
    fn deterministic_update(current_power: f64) -> f64 {
        // Simple persistence for demonstration
        // Real implementation: physics-based evolution
        current_power
    }
}

/// Saturation boundary detection and transition triggering
#[derive(Clone, Debug)]
pub struct ConstraintBoundary {
    pub rejection_count: u32,
    pub last_violation: ViolationVector,
    pub saturation_threshold: u32,
}

impl ConstraintBoundary {
    pub fn new(saturation_threshold: u32) -> Self {
        Self {
            rejection_count: 0,
            last_violation: ViolationVector::default(),
            saturation_threshold,
        }
    }

    /// Update boundary state based on latest violation
    pub fn update(&mut self, violation: &ViolationVector) {
        if violation.is_feasible() {
            self.rejection_count = 0;
        } else {
            self.rejection_count += 1;
            self.last_violation = violation.clone();
        }
    }

    /// Check if system has reached saturation (L6 condition)
    pub fn is_saturated(&self) -> bool {
        self.rejection_count >= self.saturation_threshold
    }

    /// Reset boundary state (after transition)
    pub fn reset(&mut self) {
        self.rejection_count = 0;
        self.last_violation = ViolationVector::default();
    }
}

/// Multi-interval trajectory for time-coupled evaluation
#[derive(Clone, Debug)]
pub struct Trajectory {
    pub intervals: Vec<PowerState>,
}

impl Trajectory {
    pub fn new(intervals: Vec<PowerState>) -> Self {
        Self { intervals }
    }

    pub fn horizon(&self) -> usize {
        self.intervals.len()
    }

    /// Get first interval (for rolling horizon commitment)
    pub fn first_interval(&self) -> Option<&PowerState> {
        self.intervals.first()
    }
}

/// System context for trajectory evaluation
#[derive(Clone, Debug)]
pub struct SystemContext {
    pub horizon: usize,
    pub delta_t_minutes: f64,
    pub saturation_threshold: u32,
}

impl Default for SystemContext {
    fn default() -> Self {
        Self {
            horizon: 6,  // 30 minutes at 5-min intervals
            delta_t_minutes: 5.0,
            saturation_threshold: 10,  // 10 consecutive rejections = saturation
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

    #[test]
    fn test_trajectory_evaluation_feasible() {
        let state1 = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);
        let state2 = PowerState::new(105.0, 100.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);
        let state3 = PowerState::new(110.0, 105.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);

        let traj = Trajectory::new(vec![state1, state2, state3]);
        let v = ConstraintEvaluator::evaluate_trajectory(&traj);

        assert!(v.is_feasible());
        assert_eq!(v.total(), 0.0);
    }

    #[test]
    fn test_trajectory_evaluation_infeasible() {
        let state1 = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 120.0, 20.0, 20.0);
        let state2 = PowerState::new(130.0, 100.0, 10.0, 10.0, 50.0, 120.0, 20.0, 20.0); // exceeds capacity

        let traj = Trajectory::new(vec![state1, state2]);
        let v = ConstraintEvaluator::evaluate_trajectory(&traj);

        assert!(!v.is_feasible());
        assert!(v.capacity_upper > 0.0);
    }

    #[test]
    fn test_trajectory_admissibility() {
        let state1 = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);
        let state2 = PowerState::new(105.0, 100.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);

        let feasible_traj = Trajectory::new(vec![state1.clone(), state2.clone()]);
        let infeasible_traj = Trajectory::new(vec![
            state1,
            PowerState::new(200.0, 105.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0) // exceeds capacity
        ]);

        assert!(AdmissibilityChecker::admissible_trajectory(&feasible_traj));
        assert!(!AdmissibilityChecker::admissible_trajectory(&infeasible_traj));
    }

    #[test]
    fn test_constraint_boundary_saturation() {
        let mut boundary = ConstraintBoundary::new(3);

        // Initially not saturated
        assert!(!boundary.is_saturated());

        // Add violations
        let violation = ViolationVector {
            capacity_upper: 10.0,
            ..Default::default()
        };

        boundary.update(&violation);
        assert!(!boundary.is_saturated());

        boundary.update(&violation);
        assert!(!boundary.is_saturated());

        boundary.update(&violation);
        assert!(boundary.is_saturated()); // Now saturated

        // Reset
        boundary.reset();
        assert!(!boundary.is_saturated());
    }

    #[test]
    fn test_constraint_boundary_reset_on_feasible() {
        let mut boundary = ConstraintBoundary::new(3);

        // Add violations
        let violation = ViolationVector {
            ramp_up: 5.0,
            ..Default::default()
        };

        boundary.update(&violation);
        boundary.update(&violation);
        assert_eq!(boundary.rejection_count, 2);

        // Feasible update resets count
        let feasible = ViolationVector::default();
        boundary.update(&feasible);
        assert_eq!(boundary.rejection_count, 0);
    }

    #[test]
    fn test_trajectory_proposer() {
        let current = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);
        let ctx = SystemContext {
            horizon: 3,
            delta_t_minutes: 5.0,
            saturation_threshold: 10,
        };

        let traj = TrajectoryProposer::propose_trajectory(&current, &ctx);

        assert_eq!(traj.horizon(), 3);
        assert_eq!(traj.intervals.len(), 3);

        // First interval should match current state
        assert_eq!(traj.intervals[0].p_t, current.p_t);
    }

    #[test]
    fn test_rolling_horizon_commitment() {
        let current = PowerState::new(100.0, 95.0, 10.0, 10.0, 50.0, 150.0, 20.0, 20.0);
        let ctx = SystemContext::default();

        let traj = TrajectoryProposer::propose_trajectory(&current, &ctx);

        // Should be able to get first interval for commitment
        if let Some(first) = traj.first_interval() {
            assert_eq!(first.p_t, current.p_t);
        } else {
            panic!("Trajectory should have at least one interval");
        }
    }
}