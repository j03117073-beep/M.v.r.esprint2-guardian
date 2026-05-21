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

#![deny(unsafe_code)]

use crate::reliability::relay_logic::{evaluate_relay_hard_fail_dc, BranchRelayProfile};
use crate::telemetry::{decode_double_bit_breaker, validate_point, BreakerState, TelemetryPoint,
    TelemetryValidationConfig, TelemetryClass, TelemetryTimestamp, SnapshotAlignmentConfig,
    within_snapshot_skew_window};
use crate::canonical_core::hash::sha256_hex;
use crate::topology::graph_builder::{TopologyGraph, TopologyVersion};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalSource {
    Dnp3,
    Iec61850,
    Pmu,
    Topology,
    OperatorCommand,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementAction {
    Reject,
    QStateIsolation,
    OperatorReview,
    PreserveTrace,
    Accept,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionCategory {
    Informational,
    Advisory,
    Suspicious,
    Incoherent,
    NonAdmissible,
    InfrastructureImpossible,
    SovereignViolation,
}

#[derive(Debug, Clone)]
pub struct SemanticSpecification {
    pub name: String,
    pub version: String,
    pub identity_hash: String,
    pub constraint_lineage_hash: String,
}

impl SemanticSpecification {
    pub fn current() -> Self {
        let name = "MVRE Operational Semantics".to_string();
        let version = "1.0.0".to_string();
        let constraint_lineage = [
            "temporal_coherence",
            "event_ordering",
            "contradictory_state",
            "breaker_sequencing",
            "topology_consistency",
            "operator_authority",
            "relay_transition_legality",
        ]
        .join("|");
        let identity_payload = format!("{}|{}|{}", name, version, constraint_lineage);
        let identity_hash = sha256_hex(&identity_payload);
        let constraint_lineage_hash = sha256_hex(&constraint_lineage);

        Self {
            name,
            version,
            identity_hash,
            constraint_lineage_hash,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticViolation {
    pub source: OperationalSource,
    pub constraint: String,
    pub reason: String,
    pub action: EnforcementAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticOutcome {
    pub admissible: bool,
    pub violations: Vec<SemanticViolation>,
    pub resolution: EnforcementAction,
    pub resolution_category: ResolutionCategory,
}

#[derive(Debug, Clone)]
pub struct OperatorCommandContext {
    pub operator_id: String,
    pub role: String,
    pub command: String,
    pub authority_level: u8,
}

#[derive(Debug, Clone)]
pub struct TopologyEvent {
    pub equipment_id: String,
    pub breaker_state: BreakerState,
    pub timestamp_ms_utc: u64,
}

#[derive(Debug, Clone)]
pub struct OperationalSnapshot {
    pub telemetry: Vec<TelemetryPoint>,
    pub ingest_time_ms_utc: u64,
    pub source_latency_ms: u64,
    pub topology_events: Vec<TopologyEvent>,
    pub topology_graph: TopologyGraph,
    pub topology_version: Option<crate::topology::graph_builder::TopologyVersion>,
    pub topology_identity: String,
    pub topology_lineage_hash: Option<String>,
    pub semantic_spec_identity: String,
    pub semantic_spec_version: String,
    pub semantic_configuration_hash: String,
    pub constraint_lineage_hash: String,
    pub telemetry_provenance_hash: String,
    pub operator_command_context_hash: Option<String>,
    pub timing_identity: String,
    pub replay_equivalence_metadata: Option<String>,
    pub snapshot_identity: String,
    pub operator_command: Option<OperatorCommandContext>,
    pub branch_flow_mw: BTreeMap<String, f64>,
    pub relay_profiles: BTreeMap<String, BranchRelayProfile>,
}

#[derive(Debug, Clone)]
pub struct SemanticConfig {
    pub stale_threshold_ms: u64,
    pub max_latency_ms: u64,
    pub skew_window_ms: u64,
    pub authority_threshold: u8,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            stale_threshold_ms: 20_000,
            max_latency_ms: 2_000,
            skew_window_ms: 2_000,
            authority_threshold: 5,
        }
    }
}

impl SemanticConfig {
    pub fn identity_hash(&self) -> String {
        let payload = format!(
            "{}|{}|{}|{}",
            self.stale_threshold_ms,
            self.max_latency_ms,
            self.skew_window_ms,
            self.authority_threshold
        );
        sha256_hex(&payload)
    }
}

impl OperationalSnapshot {
    pub fn create(
        telemetry: Vec<TelemetryPoint>,
        ingest_time_ms_utc: u64,
        source_latency_ms: u64,
        topology_events: Vec<TopologyEvent>,
        topology_graph: TopologyGraph,
        topology_version: Option<TopologyVersion>,
        operator_command: Option<OperatorCommandContext>,
        branch_flow_mw: BTreeMap<String, f64>,
        relay_profiles: BTreeMap<String, BranchRelayProfile>,
        replay_equivalence_metadata: Option<String>,
        semantic_config: &SemanticConfig,
    ) -> Self {
        let topology_identity = topology_graph.canonical_identity();
        let topology_lineage_hash = topology_version
            .as_ref()
            .and_then(|version| version.lineage_hash.clone());

        let semantic_spec = SemanticSpecification::current();
        let semantic_spec_identity = semantic_spec.identity_hash.clone();
        let semantic_spec_version = semantic_spec.version.clone();
        let semantic_configuration_hash = semantic_config.identity_hash();
        let constraint_lineage_hash = semantic_spec.constraint_lineage_hash.clone();

        let telemetry_provenance_hash = {
            let mut provenance = String::new();
            for point in &telemetry {
                write!(
                    &mut provenance,
                    "{:.9}|{}|{};",
                    point.value,
                    point.point_timestamp_ms_utc,
                    point.quality_mask
                )
                .unwrap();
            }
            sha256_hex(&provenance)
        };

        let operator_command_context_hash = operator_command.as_ref().map(|command| {
            let payload = format!(
                "{}|{}|{}|{}",
                command.operator_id,
                command.role,
                command.command,
                command.authority_level,
            );
            sha256_hex(&payload)
        });

        let timing_identity = {
            let payload = format!("{}|{}", ingest_time_ms_utc, source_latency_ms);
            sha256_hex(&payload)
        };

        let snapshot_identity = {
            let mut payload = String::new();
            write!(
                &mut payload,
                "topology_identity={}|topology_lineage_hash={}|semantic_spec_identity={}|semantic_spec_version={}|semantic_configuration_hash={}|constraint_lineage_hash={}|telemetry_provenance_hash={}|operator_command_context_hash={}|timing_identity={}|replay_equivalence_metadata={}|ingest_time_ms_utc={}|source_latency_ms={}",
                topology_identity,
                topology_lineage_hash.clone().unwrap_or_else(|| "NONE".to_string()),
                semantic_spec_identity,
                semantic_spec_version,
                semantic_configuration_hash,
                constraint_lineage_hash,
                telemetry_provenance_hash,
                operator_command_context_hash.clone().unwrap_or_else(|| "NONE".to_string()),
                timing_identity,
                replay_equivalence_metadata.clone().unwrap_or_else(|| "NONE".to_string()),
                ingest_time_ms_utc,
                source_latency_ms,
            )
            .unwrap();
            sha256_hex(&payload)
        };

        OperationalSnapshot {
            telemetry,
            ingest_time_ms_utc,
            source_latency_ms,
            topology_events,
            topology_graph,
            topology_version,
            topology_identity,
            topology_lineage_hash,
            semantic_spec_identity,
            semantic_spec_version,
            semantic_configuration_hash,
            constraint_lineage_hash,
            telemetry_provenance_hash,
            operator_command_context_hash,
            timing_identity,
            replay_equivalence_metadata,
            snapshot_identity,
            operator_command,
            branch_flow_mw,
            relay_profiles,
        }
    }
}

impl SemanticViolation {
    fn new(
        source: OperationalSource,
        constraint: impl Into<String>,
        reason: impl Into<String>,
        action: EnforcementAction,
    ) -> Self {
        Self {
            source,
            constraint: constraint.into(),
            reason: reason.into(),
            action,
        }
    }
}

pub fn evaluate_infrastructure_semantics(
    snapshot: &OperationalSnapshot,
    config: &SemanticConfig,
) -> SemanticOutcome {
    let mut violations = Vec::new();

    violations.extend(validate_timing_coherence(snapshot, config));
    violations.extend(validate_contradictory_state(snapshot));
    violations.extend(validate_breaker_sequencing(snapshot, config));
    violations.extend(validate_topology_coherence(snapshot, config));
    violations.extend(validate_operator_authority(snapshot, config));
    violations.extend(validate_relay_transitions(snapshot));

    let resolution = if violations.is_empty() {
        EnforcementAction::Accept
    } else if violations.iter().any(|v| matches!(v.action, EnforcementAction::OperatorReview)) {
        EnforcementAction::OperatorReview
    } else if violations.iter().any(|v| matches!(v.action, EnforcementAction::QStateIsolation)) {
        EnforcementAction::QStateIsolation
    } else if violations.iter().any(|v| matches!(v.action, EnforcementAction::PreserveTrace)) {
        EnforcementAction::PreserveTrace
    } else {
        EnforcementAction::Reject
    };

    let resolution_category = if violations.is_empty() {
        ResolutionCategory::Informational
    } else if violations.iter().any(|v| matches!(v.action, EnforcementAction::Reject)) {
        ResolutionCategory::NonAdmissible
    } else if violations.iter().any(|v| matches!(v.action, EnforcementAction::QStateIsolation)) {
        ResolutionCategory::Incoherent
    } else if violations.iter().any(|v| matches!(v.action, EnforcementAction::OperatorReview)) {
        ResolutionCategory::Suspicious
    } else if violations.iter().any(|v| matches!(v.action, EnforcementAction::PreserveTrace)) {
        ResolutionCategory::Advisory
    } else {
        ResolutionCategory::Suspicious
    };

    let resolution_category = if violations.iter().any(|v| v.constraint == "topology_consistency") {
        ResolutionCategory::InfrastructureImpossible
    } else {
        resolution_category
    };

    SemanticOutcome {
        admissible: violations.is_empty(),
        violations,
        resolution,
        resolution_category,
    }
}

fn validate_timing_coherence(
    snapshot: &OperationalSnapshot,
    config: &SemanticConfig,
) -> Vec<SemanticViolation> {
    let mut results = Vec::new();
    let tv_config = TelemetryValidationConfig {
        stale_threshold_seconds: config.stale_threshold_ms / 1000,
        max_end_to_end_latency_seconds: config.max_latency_ms / 1000,
    };

    for point in &snapshot.telemetry {
        let issues = validate_point(point, snapshot.ingest_time_ms_utc, snapshot.source_latency_ms, tv_config);
        for issue in issues {
            let reason = format!("Telemetry coherence failure: {issue}");
            results.push(SemanticViolation::new(
                OperationalSource::Dnp3,
                "temporal_coherence",
                reason,
                EnforcementAction::Reject,
            ));
        }
    }

    if snapshot.telemetry.len() >= 2 {
        let timestamps: Vec<u64> = snapshot
            .telemetry
            .iter()
            .map(|p| p.point_timestamp_ms_utc)
            .collect();
        let min_ts = *timestamps.iter().min().unwrap_or(&0);
        let max_ts = *timestamps.iter().max().unwrap_or(&0);
        if max_ts.saturating_sub(min_ts) > config.skew_window_ms {
            results.push(SemanticViolation::new(
                OperationalSource::Iec61850,
                "event_ordering",
                format!(
                    "Telemetry ordering divergence exceeds {}ms window (delta {}ms)",
                    config.skew_window_ms,
                    max_ts.saturating_sub(min_ts)
                ),
                EnforcementAction::QStateIsolation,
            ));
        }
    }

    results
}

fn validate_contradictory_state(snapshot: &OperationalSnapshot) -> Vec<SemanticViolation> {
    let mut results = Vec::new();
    if snapshot.telemetry.len() >= 3 {
        let load = snapshot.telemetry[0].value;
        let generation = snapshot.telemetry[1].value;
        let reserve = snapshot.telemetry[2].value;

        if load > generation && reserve >= 0.0 {
            results.push(SemanticViolation::new(
                OperationalSource::Unknown,
                "contradictory_state",
                format!(
                    "Load ({:.1} MW) exceeds generation ({:.1} MW) but reserve remains non-negative ({:.1} MW)",
                    load, generation, reserve
                ),
                EnforcementAction::Reject,
            ));
        }

        if reserve < 0.0 && generation >= load {
            results.push(SemanticViolation::new(
                OperationalSource::Unknown,
                "contradictory_state",
                format!(
                    "Reserve is negative ({:.1} MW) while generation ({:.1} MW) meets load ({:.1} MW)",
                    reserve, generation, load
                ),
                EnforcementAction::Reject,
            ));
        }
    }
    results
}

fn validate_breaker_sequencing(
    snapshot: &OperationalSnapshot,
    config: &SemanticConfig,
) -> Vec<SemanticViolation> {
    let mut results = Vec::new();
    let mut events_by_equipment: BTreeMap<String, Vec<&TopologyEvent>> = BTreeMap::new();

    for event in &snapshot.topology_events {
        events_by_equipment
            .entry(event.equipment_id.clone())
            .or_default()
            .push(event);
    }

    for (eq_id, events) in events_by_equipment {
        if events.len() < 2 {
            continue;
        }

        let mut closed_count = 0;
        let mut open_count = 0;
        let mut last_ts = 0;

        for event in events {
            match event.breaker_state {
                BreakerState::Closed => closed_count += 1,
                BreakerState::Open => open_count += 1,
                BreakerState::Intermediate | BreakerState::BadState => {
                    results.push(SemanticViolation::new(
                        OperationalSource::Topology,
                        "breaker_sequencing",
                        format!(
                            "Breaker {} reported intermediate or bad state at {}ms",
                            eq_id, event.timestamp_ms_utc
                        ),
                        EnforcementAction::QStateIsolation,
                    ));
                }
            }
            if last_ts > 0 && event.timestamp_ms_utc.saturating_sub(last_ts) < config.skew_window_ms {
                results.push(SemanticViolation::new(
                    OperationalSource::Topology,
                    "breaker_timing",
                    format!(
                        "Breaker {} toggled state too quickly: {}ms between events",
                        eq_id,
                        event.timestamp_ms_utc.saturating_sub(last_ts)
                    ),
                    EnforcementAction::QStateIsolation,
                ));
            }
            last_ts = event.timestamp_ms_utc;
        }

        if closed_count > 0 && open_count > 0 {
            results.push(SemanticViolation::new(
                OperationalSource::Topology,
                "topology_consistency",
                format!(
                    "Breaker {} has conflicting closed/open reports", eq_id),
                EnforcementAction::QStateIsolation,
            ));
        }
    }

    results
}

fn validate_topology_coherence(
    snapshot: &OperationalSnapshot,
    config: &SemanticConfig,
) -> Vec<SemanticViolation> {
    let mut results = Vec::new();

    if snapshot.topology_graph.buses.is_empty()
        && snapshot.topology_graph.branches.is_empty()
        && snapshot.topology_version.is_none()
    {
        results.push(SemanticViolation::new(
            OperationalSource::Topology,
            "authoritative_topology_missing",
            "No authoritative CIM-derived topology was available for this snapshot",
            EnforcementAction::Reject,
        ));
    }

    let mut events_by_equipment: BTreeMap<String, Vec<&TopologyEvent>> = BTreeMap::new();

    for event in &snapshot.topology_events {
        events_by_equipment
            .entry(event.equipment_id.clone())
            .or_default()
            .push(event);
    }

    for (eq_id, mut events) in events_by_equipment {
        events.sort_unstable_by_key(|event| event.timestamp_ms_utc);
        let mut last_state: Option<BreakerState> = None;
        let mut last_ts: Option<u64> = None;

        for event in events {
            if let Some(previous_state) = last_state {
                if event.timestamp_ms_utc < last_ts.unwrap_or(0) {
                    results.push(SemanticViolation::new(
                        OperationalSource::Topology,
                        "causal_regression",
                        format!(
                            "Topology event {} regression: {}ms after {}",
                            eq_id, event.timestamp_ms_utc, last_ts.unwrap_or(0)
                        ),
                        EnforcementAction::Reject,
                    ));
                }
                if previous_state != event.breaker_state
                    && event.timestamp_ms_utc.saturating_sub(last_ts.unwrap_or(0)) < config.skew_window_ms
                {
                    results.push(SemanticViolation::new(
                        OperationalSource::Topology,
                        "breaker_timing",
                        format!(
                            "Topology event {} toggled too quickly: {}ms between events",
                            eq_id,
                            event.timestamp_ms_utc.saturating_sub(last_ts.unwrap_or(0))
                        ),
                        EnforcementAction::QStateIsolation,
                    ));
                }
                if previous_state != event.breaker_state && event.timestamp_ms_utc == last_ts.unwrap_or(0) {
                    results.push(SemanticViolation::new(
                        OperationalSource::Topology,
                        "topology_consistency",
                        format!(
                            "Topology event {} reported conflicting states at same timestamp {}",
                            eq_id, event.timestamp_ms_utc
                        ),
                        EnforcementAction::QStateIsolation,
                    ));
                }
            }
            last_state = Some(event.breaker_state);
            last_ts = Some(event.timestamp_ms_utc);
        }
    }

    if !snapshot.topology_graph.divergences.is_empty() {
        for divergence in &snapshot.topology_graph.divergences {
            results.push(SemanticViolation::new(
                OperationalSource::Topology,
                "topology_divergence",
                format!(
                    "Topology divergence detected for {}: modeled_closed={} telemetered_closed={}",
                    divergence.equipment_id, divergence.modeled_closed, divergence.telemetered_closed
                ),
                EnforcementAction::Reject,
            ));
        }
    }

    results
}

fn validate_operator_authority(
    snapshot: &OperationalSnapshot,
    config: &SemanticConfig,
) -> Vec<SemanticViolation> {
    let mut results = Vec::new();
    if let Some(command) = &snapshot.operator_command {
        if command.authority_level < config.authority_threshold {
            results.push(SemanticViolation::new(
                OperationalSource::OperatorCommand,
                "operator_authority",
                format!(
                    "Operator '{}' role '{}' lacks authority level {} for command {}",
                    command.operator_id, command.role, command.authority_level, command.command
                ),
                EnforcementAction::OperatorReview,
            ));
        }
        if command.role.to_lowercase() == "guest" {
            results.push(SemanticViolation::new(
                OperationalSource::OperatorCommand,
                "operator_authority",
                format!(
                    "Unauthorized command {} from guest operator {}",
                    command.command, command.operator_id
                ),
                EnforcementAction::Reject,
            ));
        }
    }
    results
}

fn validate_relay_transitions(snapshot: &OperationalSnapshot) -> Vec<SemanticViolation> {
    let mut results = Vec::new();
    if snapshot.branch_flow_mw.is_empty() || snapshot.relay_profiles.is_empty() {
        return results;
    }

    let outcome = evaluate_relay_hard_fail_dc(
        &snapshot.topology_graph,
        &snapshot.branch_flow_mw,
        &snapshot.relay_profiles,
    );

    if outcome.halt_sced_cycle {
        for violation in outcome.violations {
            results.push(SemanticViolation::new(
                OperationalSource::Topology,
                "relay_transition_legality",
                format!(
                    "Relay {} exceeded threshold {:.1} MW with flow {:.1} MW: {}",
                    violation.equipment_id, violation.threshold_mw, violation.flow_mw, violation.reason
                ),
                EnforcementAction::Reject,
            ));
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{TelemetryPoint, QUALITY_VALID};

    #[test]
    fn rejects_stale_telemetry() {
        let snapshot = OperationalSnapshot::create(
            vec![TelemetryPoint {
                value: 1000.0,
                point_timestamp_ms_utc: 1000,
                quality_mask: QUALITY_VALID,
            }],
            30_000,
            0,
            Vec::new(),
            TopologyGraph::default(),
            None,
            None,
            BTreeMap::new(),
            BTreeMap::new(),
            None,
            &SemanticConfig::default(),
        );
        let config = SemanticConfig::default();
        let outcome = evaluate_infrastructure_semantics(&snapshot, &config);
        assert!(!outcome.admissible);
        assert!(outcome.violations.iter().any(|v| v.constraint == "temporal_coherence"));
    }
}
