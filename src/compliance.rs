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

use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::tlbss_types::SubstrateNode;
use std::collections::HashMap;

/// Part 2: Logic Trace Proof
/// Canonical deterministic reference for 100-tick compliance
pub struct ComplianceTrace {
    pub input_signal: u8,
    pub mask_constant: u8,
    pub masked_contribution: u64,
    pub l7_threshold: u64,
    pub total_ticks: u64,
    pub expected_final_charge: u64,
}

impl ComplianceTrace {
    pub fn canonical() -> Self {
        // Reference Conditions per Part 2
        let input_signal = 50u8;
        let mask_constant = 0x5A;

        // Masked Contribution: 50 ⊕ 0x5A = 104
        let masked: u8 = input_signal ^ mask_constant;

        Self {
            input_signal,
            mask_constant,
            masked_contribution: masked as u64,
            l7_threshold: 100,
            total_ticks: 100,
            expected_final_charge: 10400,
        }
    }
}

/// Verify that runtime charge matches precomputed deterministic value
pub fn verify_charge_determinism(
    runtime_charge: u64,
    tick_count: u64,
    trace: &ComplianceTrace,
) -> Result<(), SystemHalt> {
    let expected = trace.masked_contribution * tick_count;

    if runtime_charge != expected {
        return Err(SystemHalt::new(
            FailureAxis::InternalInvariantBreach,
            "Charge determinism violation",
        ));
    }

    Ok(())
}

/// Verify that stable_ticks matches tick count
/// (should increment by 1 per tick once charge ≥ threshold)
pub fn verify_stability_counter(
    runtime_stable_ticks: u8,
    tick_count: u64,
) -> Result<(), SystemHalt> {
    // For 100 ticks with charge always ≥ 100 after tick 1,
    // stable_ticks should be min(tick_count, 255) clamped to u8
    let expected_min = if tick_count >= 255 {
        255u8
    } else {
        tick_count as u8
    };

    if runtime_stable_ticks < expected_min {
        return Err(SystemHalt::new(
            FailureAxis::InternalInvariantBreach,
            "Stability counter violation",
        ));
    }

    Ok(())
}

/// Enforce canonical trace over N ticks
/// Returns final verified state or SystemHalt
pub fn enforce_canonical_trace(
    node: &SubstrateNode,
    ticks_executed: u64,
    trace: &ComplianceTrace,
) -> Result<(), SystemHalt> {
    // Verify charge matches deterministic formula
    verify_charge_determinism(node.charge, ticks_executed, trace)?;

    // Verify stability counter advanced
    verify_stability_counter(node.stable_ticks, ticks_executed)?;

    Ok(())
}

// DESK v1 — Reg-Up Validation Structures and Function

#[derive(Debug)]
pub enum Constraint {
    Headroom,
    Ramp,
    Qualification,
    Eligibility,
}

// Old structs for v1
#[derive(Debug, Clone)]
pub struct OldResource {
    pub id: String,
    pub status: String,
    pub telemetered_output: f64,
    pub hsl: f64,
    pub lsl: f64,
    pub reg_up_ramp_rate: f64,
    pub reg_up_qualification: f64,
}

#[derive(Debug, Clone)]
pub struct OldState {
    pub resources: Vec<OldResource>,
}

#[derive(Debug, Clone)]
pub struct OldAward {
    pub id: String,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct OldProposal {
    pub reg_up_awards: Vec<OldAward>,
}

// New structs for v2/v3
#[derive(Debug, Clone)]
pub struct Resource {
    pub id: String,
    pub hsl: f64,
    pub lsl: f64,
    pub reg_up_ramp_rate: f64,
    pub reg_down_ramp_rate: f64,
    pub reg_up_qualification: f64,
    pub reg_down_qualification: f64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct State {
    pub resources: Vec<Resource>,
}

#[derive(Debug, Clone)]
pub struct Award {
    pub resource_id: String,
    pub energy_mw: f64,
    pub reg_up: f64,
    pub reg_down: f64,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub awards: Vec<Award>,
}

#[derive(Debug)]
pub struct ResourceResult {
    pub id: String,
    pub status: ValidationStatus,
    pub violations: Vec<String>,
    pub max_deliverable: f64,
}

impl ResourceResult {
    pub fn valid(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: ValidationStatus::Valid,
            violations: vec![],
            max_deliverable: 0.0, // Not used in new logic
        }
    }

    pub fn invalid(id: &str, violation: &str) -> Self {
        Self {
            id: id.to_string(),
            status: ValidationStatus::Invalid,
            violations: vec![violation.to_string()],
            max_deliverable: 0.0,
        }
    }
}

#[derive(Debug)]
pub enum ValidationStatus {
    Valid,
    Invalid,
}

#[derive(Debug)]
pub struct SystemResult {
    pub total_awarded: f64,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub results: Vec<ResourceResult>,
    pub system: SystemResult,
}

pub fn validate_reg_up(state: OldState, proposal: OldProposal) -> ValidationResult {
    let mut map: HashMap<String, OldResource> = HashMap::new();
    for resource in state.resources {
        map.insert(resource.id.clone(), resource);
    }

    let mut results = Vec::new();
    let mut total_awarded = 0.0;

    for award in proposal.reg_up_awards {
        let mut violations = Vec::new();
        let mut status = ValidationStatus::Valid;
        let max_deliverable;

        if let Some(resource) = map.get(&award.id) {
            // Rule 2: Eligibility
            let allowed_statuses = ["ON", "ONRUC", "ONEMR", "ONOPTOUT"];
            if !allowed_statuses.contains(&resource.status.as_str()) {
                if award.value != 0.0 {
                    violations.push("Resource not eligible".to_string());
                    status = ValidationStatus::Invalid;
                }
            }

            // Rule 3: Non-Negative
            if award.value < 0.0 {
                violations.push("Negative award".to_string());
                status = ValidationStatus::Invalid;
            }

            // Compute remaining_up
            let remaining_up = resource.hsl - resource.telemetered_output;

            // Rule 4: Headroom
            if remaining_up < 0.0 {
                violations.push("No upward capacity".to_string());
                status = ValidationStatus::Invalid;
            }

            // Ramp limited
            let ramp_limited = resource.reg_up_ramp_rate * 5.0;

            // Max deliverable
            max_deliverable = remaining_up.min(ramp_limited).min(resource.reg_up_qualification);

            // Rule 5: Feasibility
            if award.value > max_deliverable {
                violations.push("Exceeds deliverable capability".to_string());
                status = ValidationStatus::Invalid;
            }

            // If valid, add to total
            if matches!(status, ValidationStatus::Valid) {
                total_awarded += award.value;
            }
        } else {
            // Rule 1: Unknown resource
            violations.push("Unknown resource".to_string());
            status = ValidationStatus::Invalid;
            max_deliverable = 0.0;
        }

        results.push(ResourceResult {
            id: award.id.clone(),
            status,
            violations,
            max_deliverable,
        });
    }

    ValidationResult {
        results,
        system: SystemResult { total_awarded },
    }
}

// DESK v2 — Co-Optimized Validation Function

pub fn validate_cooptimized(
    resources: &HashMap<String, Resource>,
    awards: &[Award],
    delta_t_minutes: f64,
) -> Vec<ResourceResult> {
    let mut results = Vec::new();

    for award in awards {
        let resource = match resources.get(&award.resource_id) {
            Some(r) => r,
            None => {
                results.push(ResourceResult::invalid(&award.resource_id, "Unknown resource"));
                continue;
            }
        };

        // Eligibility
        let allowed_statuses = ["ON", "ONRUC", "ONEMR", "ONOPTOUT"];
        if !allowed_statuses.contains(&resource.status.as_str()) {
            if award.energy_mw != 0.0 || award.reg_up != 0.0 || award.reg_down != 0.0 {
                results.push(ResourceResult::invalid(&award.resource_id, "Ineligible resource with non-zero award"));
            } else {
                results.push(ResourceResult::valid(&award.resource_id));
            }
            continue;
        }

        let p = award.energy_mw;

        // --- HEADROOM CALCULATIONS ---
        let upward_headroom = resource.hsl - p;
        let downward_headroom = p - resource.lsl;

        if upward_headroom < 0.0 {
            results.push(ResourceResult::invalid(&award.resource_id, "Energy exceeds HSL"));
            continue;
        }

        if downward_headroom < 0.0 {
            results.push(ResourceResult::invalid(&award.resource_id, "Energy below LSL"));
            continue;
        }

        // --- RAMP LIMITS ---
        let ramp_up_limit = resource.reg_up_ramp_rate * delta_t_minutes;
        let ramp_down_limit = resource.reg_down_ramp_rate * delta_t_minutes;

        let max_reg_up = upward_headroom
            .min(ramp_up_limit)
            .min(resource.reg_up_qualification);

        let max_reg_down = downward_headroom
            .min(ramp_down_limit)
            .min(resource.reg_down_qualification);

        // --- VALIDATION ---
        if award.reg_up < 0.0 || award.reg_down < 0.0 {
            results.push(ResourceResult::invalid(&award.resource_id, "Negative regulation award"));
            continue;
        }

        if award.reg_up > max_reg_up {
            results.push(ResourceResult::invalid(&award.resource_id, "Reg-Up exceeds capability"));
            continue;
        }

        if award.reg_down > max_reg_down {
            results.push(ResourceResult::invalid(&award.resource_id, "Reg-Down exceeds capability"));
            continue;
        }

        // --- RAMP SHARING (CRITICAL) ---
        let total_ramp_needed = award.reg_up + award.reg_down;
        let ramp_cap = ramp_up_limit.min(ramp_down_limit);

        if total_ramp_needed > ramp_cap {
            results.push(ResourceResult::invalid(&award.resource_id, "Combined regulation exceeds ramp capability"));
            continue;
        }

        results.push(ResourceResult::valid(&award.resource_id));
    }

    results
}

// DESK v3 — Deterministic Co-Optimization Solver

pub struct Cost {
    pub energy: f64,
    pub reg_up: f64,
    pub reg_down: f64,
}

pub struct Dispatch {
    pub resource_id: String,
    pub energy: f64,
    pub reg_up: f64,
    pub reg_down: f64,
}

pub fn solve_dispatch(
    resources: &HashMap<String, Resource>,
    costs: &HashMap<String, Cost>,
    demand: f64,
    reg_up_req: f64,
    reg_down_req: f64,
    delta_t: f64,
) -> Vec<Dispatch> {
    let mut dispatch = initialize_zero(resources);

    allocate_energy(&mut dispatch, resources, costs, demand);
    allocate_reg_up(&mut dispatch, resources, costs, reg_up_req, delta_t);
    allocate_reg_down(&mut dispatch, resources, costs, reg_down_req, delta_t);

    enforce_coupling(&mut dispatch, resources, delta_t);

    dispatch
}

pub fn validate_sced_offer_chain(
    records: Vec<crate::sced_offer_chain::ScedResourceOfferRecord>,
) -> Result<Vec<crate::sced_offer_chain::ChainedRecord>, crate::sced_offer_chain::ParseError> {
    crate::sced_offer_chain::build_hash_chain(records)
}

fn initialize_zero(resources: &HashMap<String, Resource>) -> Vec<Dispatch> {
    resources.keys().map(|id| Dispatch {
        resource_id: id.clone(),
        energy: 0.0,
        reg_up: 0.0,
        reg_down: 0.0,
    }).collect()
}

fn allocate_energy(
    dispatch: &mut Vec<Dispatch>,
    resources: &HashMap<String, Resource>,
    costs: &HashMap<String, Cost>,
    demand: f64,
) {
    let mut remaining = demand;

    let mut order: Vec<_> = dispatch.iter_mut().collect();

    order.sort_by(|a, b| {
        costs[&a.resource_id]
            .energy
            .partial_cmp(&costs[&b.resource_id].energy)
            .unwrap()
    });

    for d in order {
        if remaining <= 0.0 {
            break;
        }

        let r = &resources[&d.resource_id];

        let max_energy = r.hsl;
        let alloc = remaining.min(max_energy);

        d.energy = alloc;
        remaining -= alloc;
    }
}

fn allocate_reg_up(
    dispatch: &mut Vec<Dispatch>,
    resources: &HashMap<String, Resource>,
    costs: &HashMap<String, Cost>,
    requirement: f64,
    delta_t: f64,
) {
    let mut remaining = requirement;

    let mut order: Vec<_> = dispatch.iter_mut().collect();

    order.sort_by(|a, b| {
        costs[&a.resource_id]
            .reg_up
            .partial_cmp(&costs[&b.resource_id].reg_up)
            .unwrap()
    });

    for d in order {
        if remaining <= 0.0 {
            break;
        }

        let r = &resources[&d.resource_id];

        let headroom = r.hsl - d.energy;
        let ramp = r.reg_up_ramp_rate * delta_t;

        let max = headroom
            .min(ramp)
            .min(r.reg_up_qualification);

        let alloc = remaining.min(max);

        d.reg_up = alloc;
        remaining -= alloc;
    }
}

fn allocate_reg_down(
    dispatch: &mut Vec<Dispatch>,
    resources: &HashMap<String, Resource>,
    costs: &HashMap<String, Cost>,
    requirement: f64,
    delta_t: f64,
) {
    let mut remaining = requirement;

    let mut order: Vec<_> = dispatch.iter_mut().collect();

    order.sort_by(|a, b| {
        costs[&a.resource_id]
            .reg_down
            .partial_cmp(&costs[&b.resource_id].reg_down)
            .unwrap()
    });

    for d in order {
        if remaining <= 0.0 {
            break;
        }

        let r = &resources[&d.resource_id];

        let footroom = d.energy - r.lsl;
        let ramp = r.reg_down_ramp_rate * delta_t;

        let max = footroom
            .min(ramp)
            .min(r.reg_down_qualification);

        let alloc = remaining.min(max);

        d.reg_down = alloc;
        remaining -= alloc;
    }
}

fn enforce_coupling(
    dispatch: &mut Vec<Dispatch>,
    resources: &HashMap<String, Resource>,
    delta_t: f64,
) {
    for d in dispatch.iter_mut() {
        let r = &resources[&d.resource_id];

        let ramp_cap = (r.reg_up_ramp_rate * delta_t)
            .min(r.reg_down_ramp_rate * delta_t);

        let total = d.reg_up + d.reg_down;

        if total > ramp_cap {
            let scale = ramp_cap / total;

            d.reg_up *= scale;
            d.reg_down *= scale;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_case() {
        let state = OldState {
            resources: vec![OldResource {
                id: "res1".to_string(),
                status: "ON".to_string(),
                telemetered_output: 80.0,
                hsl: 100.0,
                lsl: 0.0,
                reg_up_ramp_rate: 2.0,
                reg_up_qualification: 15.0,
            }],
        };
        let proposal = OldProposal {
            reg_up_awards: vec![OldAward {
                id: "res1".to_string(),
                value: 10.0,
            }],
        };
        let result = validate_reg_up(state, proposal);
        assert_eq!(result.results.len(), 1);
        assert!(matches!(result.results[0].status, ValidationStatus::Valid));
        assert_eq!(result.results[0].violations.len(), 0);
        assert_eq!(result.results[0].max_deliverable, 10.0); // min(20, 10, 15) = 10
        assert_eq!(result.system.total_awarded, 10.0);
    }

    #[test]
    fn test_ramp_violation() {
        let state = OldState {
            resources: vec![OldResource {
                id: "res1".to_string(),
                status: "ON".to_string(),
                telemetered_output: 80.0,
                hsl: 100.0,
                lsl: 0.0,
                reg_up_ramp_rate: 1.0, // ramp_limited = 5.0
                reg_up_qualification: 15.0,
            }],
        };
        let proposal = OldProposal {
            reg_up_awards: vec![OldAward {
                id: "res1".to_string(),
                value: 10.0,
            }],
        };
        let result = validate_reg_up(state, proposal);
        assert_eq!(result.results.len(), 1);
        assert!(matches!(result.results[0].status, ValidationStatus::Invalid));
        assert_eq!(result.results[0].violations, vec!["Exceeds deliverable capability"]);
        assert_eq!(result.results[0].max_deliverable, 5.0); // min(20, 5, 15) = 5
        assert_eq!(result.system.total_awarded, 0.0);
    }

    #[test]
    fn test_headroom_violation() {
        let state = State {
            resources: vec![Resource {
                id: "res1".to_string(),
                status: "ON".to_string(),
                telemetered_output: 105.0, // hsl = 100, remaining_up = -5
                hsl: 100.0,
                lsl: 0.0,
                reg_up_ramp_rate: 2.0,
                reg_up_qualification: 15.0,
            }],
        };
        let proposal = Proposal {
            reg_up_awards: vec![Award {
                id: "res1".to_string(),
                value: 10.0,
            }],
        };
        let result = validate_reg_up(state, proposal);
        assert_eq!(result.results.len(), 1);
        assert!(matches!(result.results[0].status, ValidationStatus::Invalid));
        assert_eq!(result.results[0].violations, vec!["No upward capacity".to_string(), "Exceeds deliverable capability".to_string()]);
        assert_eq!(result.results[0].max_deliverable, -5.0); // but since remaining_up < 0, and min with negative
        assert_eq!(result.system.total_awarded, 0.0);
    }

    #[test]
    fn test_ineligible_resource() {
        let state = State {
            resources: vec![Resource {
                id: "res1".to_string(),
                status: "OFF".to_string(),
                telemetered_output: 80.0,
                hsl: 100.0,
                lsl: 0.0,
                reg_up_ramp_rate: 2.0,
                reg_up_qualification: 15.0,
            }],
        };
        let proposal = Proposal {
            reg_up_awards: vec![Award {
                id: "res1".to_string(),
                value: 5.0,
            }],
        };
        let result = validate_reg_up(state, proposal);
        assert_eq!(result.results.len(), 1);
        assert!(matches!(result.results[0].status, ValidationStatus::Invalid));
        assert_eq!(result.results[0].violations, vec!["Resource not eligible"]);
        assert_eq!(result.results[0].max_deliverable, 10.0);
        assert_eq!(result.system.total_awarded, 0.0);
    }

    #[test]
    fn test_unknown_resource() {
        let state = State {
            resources: vec![Resource {
                id: "res1".to_string(),
                status: "ON".to_string(),
                telemetered_output: 80.0,
                hsl: 100.0,
                lsl: 0.0,
                reg_up_ramp_rate: 2.0,
                reg_up_qualification: 15.0,
            }],
        };
        let proposal = Proposal {
            reg_up_awards: vec![Award {
                id: "res2".to_string(),
                value: 5.0,
            }],
        };
        let result = validate_reg_up(state, proposal);
        assert_eq!(result.results.len(), 1);
        assert!(matches!(result.results[0].status, ValidationStatus::Invalid));
        assert_eq!(result.results[0].violations, vec!["Unknown resource"]);
        assert_eq!(result.results[0].max_deliverable, 0.0);
        assert_eq!(result.system.total_awarded, 0.0);
    }

    #[test]
    fn test_negative_award() {
        let state = State {
            resources: vec![Resource {
                id: "res1".to_string(),
                status: "ON".to_string(),
                telemetered_output: 80.0,
                hsl: 100.0,
                lsl: 0.0,
                reg_up_ramp_rate: 2.0,
                reg_up_qualification: 15.0,
            }],
        };
        let proposal = Proposal {
            reg_up_awards: vec![Award {
                id: "res1".to_string(),
                value: -5.0,
            }],
        };
        let result = validate_reg_up(state, proposal);
        assert_eq!(result.results.len(), 1);
        assert!(matches!(result.results[0].status, ValidationStatus::Invalid));
        assert_eq!(result.results[0].violations, vec!["Negative award"]);
        assert_eq!(result.results[0].max_deliverable, 10.0);
        assert_eq!(result.system.total_awarded, 0.0);
    }

    #[test]
    fn test_ineligible_with_zero_award() {
        let state = State {
            resources: vec![Resource {
                id: "res1".to_string(),
                status: "OFF".to_string(),
                telemetered_output: 80.0,
                hsl: 100.0,
                lsl: 0.0,
                reg_up_ramp_rate: 2.0,
                reg_up_qualification: 15.0,
            }],
        };
        let proposal = Proposal {
            reg_up_awards: vec![Award {
                id: "res1".to_string(),
                value: 0.0,
            }],
        };
        let result = validate_reg_up(state, proposal);
        assert_eq!(result.results.len(), 1);
        assert!(matches!(result.results[0].status, ValidationStatus::Valid));
        assert_eq!(result.results[0].violations.len(), 0);
        assert_eq!(result.results[0].max_deliverable, 10.0);
        assert_eq!(result.system.total_awarded, 0.0);
    }
}
