// Copyright (c) 2026 OBINNA JAMES EJIOFOR
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

use crate::topology::graph_builder::TopologyGraph;
use std::collections::{BTreeMap, BTreeSet};

pub const HALT_0X0C: &str = "HALT_0x0C";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BranchRelayProfile {
    pub normal_rating_mw: f64,
    pub emergency_rating_2hr_mw: f64,
    pub relay_loadability_mw: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelayViolation {
    pub equipment_id: String,
    pub flow_mw: f64,
    pub threshold_mw: f64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelayEnforcementOutcome {
    pub halt_sced_cycle: bool,
    pub code: Option<String>,
    pub message: String,
    pub violations: Vec<RelayViolation>,
    pub tripped_branch_ids: Vec<String>,
    pub tripped_topology: TopologyGraph,
}

pub fn evaluate_relay_hard_fail_dc(
    graph: &TopologyGraph,
    branch_flow_mw: &BTreeMap<String, f64>,
    profiles: &BTreeMap<String, BranchRelayProfile>,
) -> RelayEnforcementOutcome {
    let mut violations = Vec::new();
    let mut tripped = BTreeSet::new();

    for branch in &graph.branches {
        let Some(profile) = profiles.get(&branch.equipment_id) else {
            continue;
        };
        let flow = branch_flow_mw
            .get(&branch.equipment_id)
            .copied()
            .unwrap_or(0.0)
            .abs();

        let threshold_125 = 1.25 * profile.emergency_rating_2hr_mw;
        let relay_limit = profile.relay_loadability_mw.unwrap_or(f64::INFINITY);
        let threshold = threshold_125.min(relay_limit);

        if flow > threshold {
            violations.push(RelayViolation {
                equipment_id: branch.equipment_id.clone(),
                flow_mw: flow,
                threshold_mw: threshold,
                reason: if threshold_125 <= relay_limit {
                    "flow exceeded 125% of 2-hour emergency rating".to_string()
                } else {
                    "flow exceeded relay loadability rating".to_string()
                },
            });
            tripped.insert(branch.equipment_id.clone());
        }
    }

    let mut tripped_topology = graph.clone();
    tripped_topology
        .branches
        .retain(|b| !tripped.contains(&b.equipment_id));

    let tripped_branch_ids: Vec<String> = tripped.into_iter().collect();
    if violations.is_empty() {
        RelayEnforcementOutcome {
            halt_sced_cycle: false,
            code: None,
            message: "relay loadability constraints satisfied".to_string(),
            violations,
            tripped_branch_ids,
            tripped_topology,
        }
    } else {
        RelayEnforcementOutcome {
            halt_sced_cycle: true,
            code: Some(HALT_0X0C.to_string()),
            message: "FAC-008 relay hard-fail; SCED cycle halted and violating branches tripped"
                .to_string(),
            violations,
            tripped_branch_ids,
            tripped_topology,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::rdf_parser::EquipmentKind;
    use crate::topology::graph_builder::{BranchEdge, ElectricalBus, TopologyGraph};

    fn mk_graph() -> TopologyGraph {
        TopologyGraph {
            buses: vec![
                ElectricalBus {
                    bus_id: "BUS::CN_A+CN_B".to_string(),
                    connectivity_nodes: vec!["CN_A".to_string(), "CN_B".to_string()],
                },
                ElectricalBus {
                    bus_id: "BUS::CN_C".to_string(),
                    connectivity_nodes: vec!["CN_C".to_string()],
                },
            ],
            branches: vec![BranchEdge {
                equipment_id: "L1".to_string(),
                kind: EquipmentKind::AcLineSegment,
                from_bus: "BUS::CN_A+CN_B".to_string(),
                to_bus: "BUS::CN_C".to_string(),
                r: None,
                x: None,
                bch: None,
            }],
            divergences: vec![],
        }
    }

    #[test]
    fn halts_and_trips_when_flow_exceeds_125_percent_emergency_rating() {
        let graph = mk_graph();
        let mut flows = BTreeMap::new();
        flows.insert("L1".to_string(), 1501.0);
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "L1".to_string(),
            BranchRelayProfile {
                normal_rating_mw: 1000.0,
                emergency_rating_2hr_mw: 1200.0,
                relay_loadability_mw: None,
            },
        );

        let out = evaluate_relay_hard_fail_dc(&graph, &flows, &profiles);
        assert!(out.halt_sced_cycle);
        assert_eq!(out.code.as_deref(), Some(HALT_0X0C));
        assert_eq!(out.tripped_branch_ids, vec!["L1".to_string()]);
        assert!(out.tripped_topology.branches.is_empty());
        assert_eq!(out.violations.len(), 1);
        assert_eq!(out.violations[0].threshold_mw, 1500.0);
    }

    #[test]
    fn does_not_halt_at_exact_125_percent_boundary() {
        let graph = mk_graph();
        let mut flows = BTreeMap::new();
        flows.insert("L1".to_string(), 1500.0);
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "L1".to_string(),
            BranchRelayProfile {
                normal_rating_mw: 1000.0,
                emergency_rating_2hr_mw: 1200.0,
                relay_loadability_mw: None,
            },
        );

        let out = evaluate_relay_hard_fail_dc(&graph, &flows, &profiles);
        assert!(!out.halt_sced_cycle);
        assert!(out.code.is_none());
        assert_eq!(out.tripped_topology.branches.len(), 1);
    }
}


