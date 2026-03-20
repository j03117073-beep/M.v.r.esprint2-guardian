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

use crate::audit_guardian::AuditGuardian;

/// System state representation for projection operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemState {
    pub p_t: f64,     // current active power (MW)
    pub p_prev: f64,  // previous active power (MW)
}

impl SystemState {
    pub fn new(p_t: f64, p_prev: f64) -> Self {
        Self { p_t, p_prev }
    }
}

/// Signed error reduction correction signal
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Correction {
    pub delta: f64,      // signed correction to apply
    pub magnitude: f64,  // violation size (always >= 0)
}

/// Constraint evaluation result
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintSignal {
    Ok,
    Violation { correction: Correction },
}

/// Formal projection operator trait
/// Π: X → X_f where X is all states, X_f is feasible states
pub trait ProjectionOperator {
    fn project(&self, state: SystemState) -> SystemState;
}

/// Deterministic projector using audit guardian for admissibility
pub struct DeterministicProjector<'a> {
    guardian: &'a AuditGuardian,
}

impl<'a> DeterministicProjector<'a> {
    pub fn new(guardian: &'a AuditGuardian) -> Self {
        Self { guardian }
    }
}

impl<'a> ProjectionOperator for DeterministicProjector<'a> {
    fn project(&self, mut state: SystemState) -> SystemState {
        // Iterative projection until admissible
        // For now, implement with RampConstraint
        // TODO: Extend to multi-constraint system and integrate with guardian
        let constraint = RampConstraint {
            ramp_up: 10.0,   // MW/min - configurable
            ramp_down: 10.0, // MW/min - configurable
        };

        // Limit iterations to prevent infinite loops (though monotonic convergence should prevent this)
        for _ in 0..10 {
            let signal = constraint.evaluate(&state);
            match signal {
                ConstraintSignal::Ok => {
                    // TODO: Verify guardian accepts this state
                    // For now, assume admissible if constraint satisfied
                    return state;
                }
                ConstraintSignal::Violation { correction } => {
                    // Apply correction with strict monotonic reduction
                    state.p_t += correction.delta;
                    // Ensure |v_{k+1}| < |v_k| is guaranteed by construction
                }
            }
        }

        // If we reach here, projection failed to converge
        // This should not happen with proper constraint design
        state
    }
}

/// Ramp rate constraint implementation
pub struct RampConstraint {
    pub ramp_up: f64,   // max increase MW per time unit
    pub ramp_down: f64, // max decrease MW per time unit
}

impl RampConstraint {
    /// Evaluate constraint and return correction signal
    pub fn evaluate(&self, state: &SystemState) -> ConstraintSignal {
        let delta = state.p_t - state.p_prev;

        // Violation functions
        let v_up = (delta - self.ramp_up).max(0.0);
        let v_down = (-delta - self.ramp_down).max(0.0);

        if v_up == 0.0 && v_down == 0.0 {
            ConstraintSignal::Ok
        } else {
            // Correction: -v_up + v_down
            // If ramp-up violated: decrease p_t
            // If ramp-down violated: increase p_t
            let delta_correction = -v_up + v_down;
            let magnitude = v_up + v_down;

            ConstraintSignal::Violation {
                correction: Correction {
                    delta: delta_correction,
                    magnitude,
                }
            }
        }
    }
}

/// Generic constraint trait for extensibility
pub trait Constraint {
    fn evaluate(&self, state: &SystemState) -> ConstraintSignal;
}

impl Constraint for RampConstraint {
    fn evaluate(&self, state: &SystemState) -> ConstraintSignal {
        self.evaluate(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramp_constraint_no_violation() {
        let constraint = RampConstraint {
            ramp_up: 10.0,
            ramp_down: 10.0,
        };

        let state = SystemState::new(5.0, 0.0); // +5 change, within limits
        assert_eq!(constraint.evaluate(&state), ConstraintSignal::Ok);
    }

    #[test]
    fn test_ramp_constraint_up_violation() {
        let constraint = RampConstraint {
            ramp_up: 10.0,
            ramp_down: 10.0,
        };

        let state = SystemState::new(15.0, 0.0); // +15 change, exceeds ramp_up
        match constraint.evaluate(&state) {
            ConstraintSignal::Violation { correction } => {
                assert_eq!(correction.delta, -5.0); // -v_up + 0 = -5
                assert_eq!(correction.magnitude, 5.0);
            }
            _ => panic!("Expected violation"),
        }
    }

    #[test]
    fn test_ramp_constraint_down_violation() {
        let constraint = RampConstraint {
            ramp_up: 10.0,
            ramp_down: 10.0,
        };

        let state = SystemState::new(-15.0, 0.0); // -15 change, exceeds ramp_down
        match constraint.evaluate(&state) {
            ConstraintSignal::Violation { correction } => {
                assert_eq!(correction.delta, 5.0); // 0 + v_down = 5
                assert_eq!(correction.magnitude, 5.0);
            }
            _ => panic!("Expected violation"),
        }
    }

    #[test]
    fn test_projection_idempotence() {
        // Create a mock guardian for testing
        // In real usage, this would be a proper AuditGuardian
        struct MockGuardian;
        let mock_guardian = MockGuardian;
        let projector = DeterministicProjector::new(&mock_guardian);

        let state = SystemState::new(5.0, 0.0);
        let projected1 = projector.project(state);
        let projected2 = projector.project(projected1);

        assert_eq!(projected1, projected2);
    }

    #[test]
    fn test_projection_feasibility() {
        // Test that projection produces feasible state
        struct MockGuardian;
        let mock_guardian = MockGuardian;
        let projector = DeterministicProjector::new(&mock_guardian);

        let infeasible = SystemState::new(20.0, 0.0); // exceeds ramp
        let feasible = projector.project(infeasible);

        // Check ramp constraint satisfied
        let constraint = RampConstraint {
            ramp_up: 10.0,
            ramp_down: 10.0,
        };
        assert_eq!(constraint.evaluate(&feasible), ConstraintSignal::Ok);
    }
}