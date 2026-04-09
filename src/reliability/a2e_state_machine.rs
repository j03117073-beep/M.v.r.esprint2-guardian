#![deny(unsafe_code)]

use crate::reliability_controls::{EnforcementCode, EnforcementEvent};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A2eState {
    Staged,
    Validating,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct A2eRequest {
    pub equipment_id: String,
    pub current_operating_day: String,
    pub energize_date: String,
    pub lookahead_entry_exists: bool,
    pub proposed_post_close_flow_mw: f64,
    pub emergency_rating_2hr_mw: f64,
    pub relay_loadability_mw: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A2ePolicyContext {
    pub validated_cim_equipment_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct A2eDecision {
    pub state: A2eState,
    pub trace: Vec<A2eState>,
    pub a2e_permission_bit: u8,
    pub predicted_threshold_mw: f64,
    pub predicted_flow_mw: f64,
    pub event: Option<EnforcementEvent>,
    pub reason: String,
}

pub fn evaluate_a2e_guard(req: &A2eRequest, ctx: &A2ePolicyContext) -> A2eDecision {
    let mut trace = Vec::new();

    // Stage 1: Planning horizon check (7-day look-ahead artifact already resolved by scheduler).
    if req.lookahead_entry_exists {
        trace.push(A2eState::Staged);
    }

    // Stage 2: Energization is only evaluated on target operating day.
    if req.current_operating_day != req.energize_date {
        return A2eDecision {
            state: A2eState::Staged,
            trace,
            a2e_permission_bit: 0,
            predicted_threshold_mw: 0.0,
            predicted_flow_mw: req.proposed_post_close_flow_mw,
            event: None,
            reason: "awaiting energize_date; remains staged".to_string(),
        };
    }

    trace.push(A2eState::Validating);

    // Stage 3: Topology consistency against validated CIM snapshot.
    if !ctx.validated_cim_equipment_ids.contains(&req.equipment_id) {
        trace.push(A2eState::Rejected);
        return A2eDecision {
            state: A2eState::Rejected,
            trace,
            a2e_permission_bit: 0,
            predicted_threshold_mw: 0.0,
            predicted_flow_mw: req.proposed_post_close_flow_mw,
            event: Some(EnforcementEvent {
                code: EnforcementCode::HaltCip001,
                message: "A2E rejected: topology integrity breach (equipment missing in validated CIM snapshot)".to_string(),
            }),
            reason: "topology consistency check failed".to_string(),
        };
    }

    // Stage 4: Predictive FAC-008 safety check (DC first pass).
    let threshold_125 = 1.25 * req.emergency_rating_2hr_mw;
    let relay_limit = req.relay_loadability_mw.unwrap_or(f64::INFINITY);
    let threshold = threshold_125.min(relay_limit);
    let predicted = req.proposed_post_close_flow_mw.abs();
    if predicted > threshold {
        trace.push(A2eState::Rejected);
        return A2eDecision {
            state: A2eState::Rejected,
            trace,
            a2e_permission_bit: 0,
            predicted_threshold_mw: threshold,
            predicted_flow_mw: predicted,
            event: Some(EnforcementEvent {
                code: EnforcementCode::HaltCip001,
                message: "A2E rejected: predicted post-close flow exceeds FAC relay threshold".to_string(),
            }),
            reason: "predictive safety check failed".to_string(),
        };
    }

    trace.push(A2eState::Approved);
    A2eDecision {
        state: A2eState::Approved,
        trace,
        a2e_permission_bit: 1,
        predicted_threshold_mw: threshold,
        predicted_flow_mw: predicted,
        event: None,
        reason: "A2E approved".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(ids: &[&str]) -> A2ePolicyContext {
        A2ePolicyContext {
            validated_cim_equipment_ids: ids.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn stays_staged_before_energize_day() {
        let req = A2eRequest {
            equipment_id: "L1".to_string(),
            current_operating_day: "2026-03-21".to_string(),
            energize_date: "2026-03-22".to_string(),
            lookahead_entry_exists: true,
            proposed_post_close_flow_mw: 800.0,
            emergency_rating_2hr_mw: 1200.0,
            relay_loadability_mw: None,
        };
        let out = evaluate_a2e_guard(&req, &ctx_with(&["L1"]));
        assert_eq!(out.state, A2eState::Staged);
        assert_eq!(out.a2e_permission_bit, 0);
        assert_eq!(out.trace, vec![A2eState::Staged]);
    }

    #[test]
    fn rejects_on_cim_mismatch_with_haltcip001() {
        let req = A2eRequest {
            equipment_id: "L1".to_string(),
            current_operating_day: "2026-03-22".to_string(),
            energize_date: "2026-03-22".to_string(),
            lookahead_entry_exists: true,
            proposed_post_close_flow_mw: 800.0,
            emergency_rating_2hr_mw: 1200.0,
            relay_loadability_mw: None,
        };
        let out = evaluate_a2e_guard(&req, &ctx_with(&[]));
        assert_eq!(out.state, A2eState::Rejected);
        assert_eq!(out.a2e_permission_bit, 0);
        assert_eq!(
            out.event.as_ref().map(|e| &e.code),
            Some(&EnforcementCode::HaltCip001)
        );
    }

    #[test]
    fn rejects_when_predicted_flow_is_1501_over_1500_threshold() {
        let req = A2eRequest {
            equipment_id: "L1".to_string(),
            current_operating_day: "2026-03-22".to_string(),
            energize_date: "2026-03-22".to_string(),
            lookahead_entry_exists: true,
            proposed_post_close_flow_mw: 1501.0,
            emergency_rating_2hr_mw: 1200.0,
            relay_loadability_mw: None,
        };
        let out = evaluate_a2e_guard(&req, &ctx_with(&["L1"]));
        assert_eq!(out.state, A2eState::Rejected);
        assert_eq!(out.predicted_threshold_mw, 1500.0);
        assert_eq!(out.a2e_permission_bit, 0);
        assert_eq!(
            out.event.as_ref().map(|e| &e.code),
            Some(&EnforcementCode::HaltCip001)
        );
    }

    #[test]
    fn approves_when_all_checks_pass() {
        let req = A2eRequest {
            equipment_id: "L1".to_string(),
            current_operating_day: "2026-03-22".to_string(),
            energize_date: "2026-03-22".to_string(),
            lookahead_entry_exists: true,
            proposed_post_close_flow_mw: 1499.0,
            emergency_rating_2hr_mw: 1200.0,
            relay_loadability_mw: None,
        };
        let out = evaluate_a2e_guard(&req, &ctx_with(&["L1"]));
        assert_eq!(out.state, A2eState::Approved);
        assert_eq!(out.a2e_permission_bit, 1);
        assert!(out.event.is_none());
        assert_eq!(
            out.trace,
            vec![A2eState::Staged, A2eState::Validating, A2eState::Approved]
        );
    }
}

