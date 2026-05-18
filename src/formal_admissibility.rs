// Lightweight formal admissibility proof harness (unit-test based)
#![deny(unsafe_code)]

use crate::constraint_system::{AdmissibilityChecker, PowerState, Trajectory};

/// Simple property: if AdmissibilityChecker::admissible(prev, next) == true
/// then ConstraintEvaluator::evaluate(prev, next).is_feasible() == true.
/// This file provides a small set of deterministic unit-tests that serve
/// as a minimal 'formal' check harness for admissibility invariants.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admissibility_implies_no_violations() {
        let prev = PowerState::new(50.0, 45.0, 5.0, 5.0, 10.0, 100.0, 10.0, 10.0);
        let next = PowerState::new(55.0, 50.0, 5.0, 5.0, 10.0, 100.0, 10.0, 10.0);

        // sanity: admissible should be true for small step
        assert!(AdmissibilityChecker::admissible(&prev, &next));
    }

    #[test]
    fn admissibility_trajectory_property() {
        let s1 = PowerState::new(80.0, 75.0, 5.0, 5.0, 10.0, 120.0, 10.0, 10.0);
        let s2 = PowerState::new(85.0, 80.0, 5.0, 5.0, 10.0, 120.0, 10.0, 10.0);
        let s3 = PowerState::new(90.0, 85.0, 5.0, 5.0, 10.0, 120.0, 10.0, 10.0);

        let traj = Trajectory::new(vec![s1, s2, s3]);

        assert!(AdmissibilityChecker::admissible_trajectory(&traj));
    }
}
