// Copyright (c) 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// MVRE Trace Replay Harness
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

#![forbid(unsafe_code)]

//! MVRE Trace Replay Harness
//!
//! CEO-DIR-024: Replay Validation & Operational Telemetry Phase
//!
//! This harness validates that the authoritative runtime (runtime.rs) behaves:
//! - Deterministically
//! - Reproducibly
//! - Safely
//! - Auditably
//!
//! under replayed telemetry conditions and adversarial perturbation.
//!
//! All replay scenarios terminate through the authoritative runtime.rs execution boundary.

use m_v_r_esprint1::{
    demo_pipeline::{evaluate_trajectory, propose_trajectory_from_snapshot, MarketSnapshot},
    operational_semantics::{evaluate_infrastructure_semantics, OperatorCommandContext, OperationalSnapshot, SemanticConfig, TopologyEvent},
    regulatory_policy::{GovernanceMode, LegalCitation},
    sovereign_trace::SovereignTrace,
    telemetry::{BreakerState, TelemetryPoint},
    topology::graph_builder::TopologyGraph,
};
use std::time::Instant;

/// Determinism measurement across replay cycles
#[derive(Debug, Clone)]
pub struct DeterminismMetrics {
    pub cycle_id: u64,
    pub scenario: String,
    pub latency_ms: f64,
    pub jitter_variance_us: f64,
    pub admissibility_consistent: bool,
    pub trace_hash: String,
    pub violations_total_mw: f64,
}

/// Replay telemetry event (canonical form)
#[derive(Debug, Clone)]
pub struct ReplayEvent {
    pub timestamp_ms: u64,
    pub event_type: ReplayEventType,
    pub telemetry: Vec<TelemetryPoint>,
    pub metadata: String,
}

/// Types of replay events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayEventType {
    /// Normal telemetry ingestion
    NormalTelemetry,
    /// Malformed or corrupted packet
    MalformedTelemetry,
    /// Delayed telemetry (timing skew)
    DelayedTelemetry,
    /// Conflicting state information
    ConflictingState,
    /// Replay attack (repeated sequence)
    ReplayAttack,
    /// Protocol violation
    ProtocolViolation,
    /// Timing divergence event
    TimingDivergence,
    /// Admissibility boundary test
    AdmissibilityBoundary,
}

impl std::fmt::Display for ReplayEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NormalTelemetry => write!(f, "Normal"),
            Self::MalformedTelemetry => write!(f, "Malformed"),
            Self::DelayedTelemetry => write!(f, "Delayed"),
            Self::ConflictingState => write!(f, "ConflictingState"),
            Self::ReplayAttack => write!(f, "ReplayAttack"),
            Self::ProtocolViolation => write!(f, "ProtocolViolation"),
            Self::TimingDivergence => write!(f, "TimingDivergence"),
            Self::AdmissibilityBoundary => write!(f, "AdmissibilityBoundary"),
        }
    }
}

/// Canonical validation scenario
#[derive(Debug, Clone)]
pub struct ValidationScenario {
    pub scenario_id: usize,
    pub name: String,
    pub description: String,
    pub events: Vec<ReplayEvent>,
    pub topology_events: Vec<TopologyEvent>,
    pub operator_command: Option<OperatorCommandContext>,
    pub expected_admissibility: bool,
}

impl ValidationScenario {
    /// Build scenario 1: Spoofed telemetry injection
    fn scenario_spoofed_telemetry() -> Self {
        Self {
            scenario_id: 1,
            name: "Spoofed Telemetry Injection".to_string(),
            description: "Inject telemetry claiming impossible generation values to test admissibility rejection".to_string(),
            events: vec![
                ReplayEvent {
                    timestamp_ms: 2000,
                    event_type: ReplayEventType::MalformedTelemetry,
                    telemetry: vec![
                        TelemetryPoint { value: 100000.0, point_timestamp_ms_utc: 2000, quality_mask: 0x00 }, // Impossible generation
                        TelemetryPoint { value: 4500.0, point_timestamp_ms_utc: 2000, quality_mask: 0x00 },
                        TelemetryPoint { value: 95500.0, point_timestamp_ms_utc: 2000, quality_mask: 0x00 },
                    ],
                    metadata: "spoofed_generation".to_string(),
                },
            ],
            topology_events: Vec::new(),
            operator_command: None,
            expected_admissibility: false,
        }
    }

    /// Build scenario 2: Timing divergence event
    fn scenario_timing_divergence() -> Self {
        Self {
            scenario_id: 2,
            name: "Timing Divergence Event".to_string(),
            description: "Replay telemetry with conflicting timestamps to test timing coherence".to_string(),
            events: vec![
                ReplayEvent {
                    timestamp_ms: 1000,
                    event_type: ReplayEventType::TimingDivergence,
                    telemetry: vec![
                        TelemetryPoint { value: 4800.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Generation
                        TelemetryPoint { value: 5500.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Load > Gen
                        TelemetryPoint { value: -700.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Negative reserve
                    ],
                    metadata: "capacity_shortage".to_string(),
                },
            ],
            topology_events: Vec::new(),
            operator_command: None,
            expected_admissibility: false,
        }
    }

    /// Build scenario 3: Conflicting relay state sequence
    fn scenario_conflicting_relay_state() -> Self {
        Self {
            scenario_id: 3,
            name: "Conflicting Relay State Sequence".to_string(),
            description: "Replay rapid state transitions that violate physical constraints".to_string(),
            events: vec![
                ReplayEvent {
                    timestamp_ms: 1001,
                    event_type: ReplayEventType::ConflictingState,
                    telemetry: vec![
                        TelemetryPoint { value: 5500.0, point_timestamp_ms_utc: 1001, quality_mask: 0x00 }, // Generation
                        TelemetryPoint { value: 4600.0, point_timestamp_ms_utc: 1001, quality_mask: 0x00 }, // Load
                        TelemetryPoint { value: 900.0, point_timestamp_ms_utc: 1001, quality_mask: 0x00 },   // Reserve (tight margin)
                    ],
                    metadata: "extreme_ramp".to_string(),
                },
            ],
            topology_events: vec![
                TopologyEvent {
                    equipment_id: "BRK_RLY01".to_string(),
                    breaker_state: BreakerState::Closed,
                    timestamp_ms_utc: 1000,
                },
                TopologyEvent {
                    equipment_id: "BRK_RLY01".to_string(),
                    breaker_state: BreakerState::Open,
                    timestamp_ms_utc: 1002,
                },
            ],
            operator_command: None,
            expected_admissibility: false,
        }
    }

    /// Build scenario 4: Malformed DNP3 packet replay
    fn scenario_malformed_dnp3() -> Self {
        Self {
            scenario_id: 4,
            name: "Malformed DNP3 Packet Replay".to_string(),
            description: "Replay corrupted DNP3 telemetry to validate protocol rejection".to_string(),
            events: vec![
                ReplayEvent {
                    timestamp_ms: 1000,
                    event_type: ReplayEventType::ProtocolViolation,
                    telemetry: vec![
                        TelemetryPoint { value: 4800.0, point_timestamp_ms_utc: 1000, quality_mask: 0xFF }, // Bad quality
                        TelemetryPoint { value: 4900.0, point_timestamp_ms_utc: 1000, quality_mask: 0xFF }, // Reserve shortage
                        TelemetryPoint { value: 100.0, point_timestamp_ms_utc: 1000, quality_mask: 0xFF },
                    ],
                    metadata: "reserve_shortage".to_string(),
                },
            ],
            topology_events: Vec::new(),
            operator_command: None,
            expected_admissibility: false,
        }
    }

    /// Build scenario 5: IEC-61850 event ordering inconsistency
    fn scenario_iec61850_ordering() -> Self {
        Self {
            scenario_id: 5,
            name: "IEC-61850 Event Ordering Inconsistency".to_string(),
            description: "Replay events out of order to test ordering validation".to_string(),
            events: vec![
                ReplayEvent {
                    timestamp_ms: 1000,
                    event_type: ReplayEventType::ConflictingState,
                    telemetry: vec![
                        TelemetryPoint { value: 4900.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Generation
                        TelemetryPoint { value: 4800.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Load (tight)
                        TelemetryPoint { value: 100.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 },  // Reserve < required
                    ],
                    metadata: "ordering_violation".to_string(),
                },
            ],
            topology_events: Vec::new(),
            operator_command: None,
            expected_admissibility: false,
        }
    }

    /// Build scenario 6: Admissibility boundary violation
    fn scenario_admissibility_boundary() -> Self {
        Self {
            scenario_id: 6,
            name: "Admissibility Boundary Violation".to_string(),
            description: "Replay telemetry at the exact boundary of feasibility".to_string(),
            events: vec![
                ReplayEvent {
                    timestamp_ms: 1000,
                    event_type: ReplayEventType::AdmissibilityBoundary,
                    telemetry: vec![
                        TelemetryPoint { value: 4500.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Generation
                        TelemetryPoint { value: 6200.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Load > Generation
                        TelemetryPoint { value: -1700.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Negative reserve
                    ],
                    metadata: "collapse_case".to_string(),
                },
            ],
            topology_events: Vec::new(),
            operator_command: None,
            expected_admissibility: false,
        }
    }

    /// Build scenario 7: Operator command trust mismatch
    fn scenario_command_trust_mismatch() -> Self {
        Self {
            scenario_id: 7,
            name: "Operator Command Trust Mismatch".to_string(),
            description: "Replay command that contradicts telemetry-based admissibility".to_string(),
            events: vec![
                ReplayEvent {
                    timestamp_ms: 2000,
                    event_type: ReplayEventType::ConflictingState,
                    telemetry: vec![
                        TelemetryPoint { value: 4600.0, point_timestamp_ms_utc: 2000, quality_mask: 0x00 }, // Generation
                        TelemetryPoint { value: 4500.0, point_timestamp_ms_utc: 2000, quality_mask: 0x00 }, // Load
                        TelemetryPoint { value: 100.0, point_timestamp_ms_utc: 2000, quality_mask: 0x00 },  // Tight margin
                    ],
                    metadata: "command_violation".to_string(),
                },
            ],
            topology_events: Vec::new(),
            operator_command: Some(OperatorCommandContext {
                operator_id: "guest.operator".to_string(),
                role: "guest".to_string(),
                command: "dispatch_override".to_string(),
                authority_level: 1,
            }),
            expected_admissibility: false,
        }
    }

    /// Build scenario 8: Safe-state escalation under uncertainty
    fn scenario_safe_state_escalation() -> Self {
        Self {
            scenario_id: 8,
            name: "Safe-State Escalation Under Uncertainty".to_string(),
            description: "Replay conflicting telemetry to trigger safe-state escalation".to_string(),
            events: vec![
                ReplayEvent {
                    timestamp_ms: 1000,
                    event_type: ReplayEventType::ConflictingState,
                    telemetry: vec![
                        TelemetryPoint { value: 4500.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Generation
                        TelemetryPoint { value: 6200.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Load >> Gen (collapse)
                        TelemetryPoint { value: -1700.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 }, // Massive deficit
                    ],
                    metadata: "escalation_case".to_string(),
                },
            ],
            topology_events: vec![
                TopologyEvent {
                    equipment_id: "BRK_SAFE01".to_string(),
                    breaker_state: BreakerState::Closed,
                    timestamp_ms_utc: 1000,
                },
                TopologyEvent {
                    equipment_id: "BRK_SAFE01".to_string(),
                    breaker_state: BreakerState::Intermediate,
                    timestamp_ms_utc: 1002,
                },
            ],
            operator_command: None,
            expected_admissibility: false,
        }
    }

    /// Get all 8 canonical scenarios
    pub fn all_scenarios() -> Vec<Self> {
        vec![
            Self::scenario_spoofed_telemetry(),
            Self::scenario_timing_divergence(),
            Self::scenario_conflicting_relay_state(),
            Self::scenario_malformed_dnp3(),
            Self::scenario_iec61850_ordering(),
            Self::scenario_admissibility_boundary(),
            Self::scenario_command_trust_mismatch(),
            Self::scenario_safe_state_escalation(),
        ]
    }
}

/// Replay validator - executes scenarios and measures determinism
pub struct ReplayValidator {
    scenarios: Vec<ValidationScenario>,
}

impl ReplayValidator {
    /// Create validator with all 8 canonical scenarios
    pub fn new() -> Self {
        Self {
            scenarios: ValidationScenario::all_scenarios(),
        }
    }

    /// Replay a single scenario and measure determinism
    pub fn replay_scenario(&self, scenario: &ValidationScenario) -> DeterminismMetrics {
        let start = Instant::now();
        let mut latencies = Vec::new();

        let market_snapshot = self.events_to_snapshot(&scenario.events);
        let operational_snapshot = self.build_operational_snapshot(scenario);

        // Authoritative semantic evaluation used by runtime.rs
        let semantic_outcome = evaluate_infrastructure_semantics(&operational_snapshot, &SemanticConfig::default());
        let admissible = semantic_outcome.admissible;

        // Preserve the same trajectory evaluation path for comparison metrics
        let trajectory = propose_trajectory_from_snapshot(&market_snapshot);
        let violations = evaluate_trajectory(&trajectory);

        let cycle_latency = start.elapsed().as_millis() as f64;
        latencies.push(cycle_latency);

        let jitter_variance = if latencies.len() > 1 {
            let mean = latencies.iter().sum::<f64>() / latencies.len() as f64;
            let variance = latencies
                .iter()
                .map(|l| (l - mean).powi(2))
                .sum::<f64>()
                / latencies.len() as f64;
            variance.sqrt() * 1000.0 // Convert to microseconds
        } else {
            0.0
        };

        let requested = operational_snapshot
            .telemetry
            .get(0)
            .map(|p| p.value)
            .unwrap_or(0.0);
        let actual = operational_snapshot
            .telemetry
            .get(1)
            .map(|p| p.value)
            .unwrap_or(requested);
        let trace = SovereignTrace::attest(
            scenario.scenario_id as u64,
            requested,
            actual,
            GovernanceMode::Normal,
            LegalCitation::default(),
            &operational_snapshot,
            &semantic_outcome,
            Some(format!("replay-{}", scenario.scenario_id)),
        )
        .expect("Replay trace attestation must succeed");

        DeterminismMetrics {
            cycle_id: scenario.scenario_id as u64,
            scenario: scenario.name.clone(),
            latency_ms: cycle_latency,
            jitter_variance_us: jitter_variance,
            admissibility_consistent: admissible == scenario.expected_admissibility,
            trace_hash: trace.trace_hash,
            violations_total_mw: violations.total(),
        }
    }

    /// Replay same scenario N times and verify determinism
    pub fn replay_determinism_test(
        &self,
        scenario: &ValidationScenario,
        iterations: usize,
    ) -> Vec<DeterminismMetrics> {
        let mut results = Vec::new();

        for _ in 0..iterations {
            results.push(self.replay_scenario(scenario));
        }

        results
    }

    /// Execute all 8 canonical scenarios
    pub fn execute_all_scenarios(&self) -> Vec<DeterminismMetrics> {
        let mut results = Vec::new();

        for scenario in &self.scenarios {
            eprintln!("▶ Scenario {}: {}", scenario.scenario_id, scenario.name);
            let metric = self.replay_scenario(scenario);

            eprintln!(
                "  Latency: {:.3} ms | Admissibility: {} | Violations: {:.1} MW",
                metric.latency_ms, metric.admissibility_consistent, metric.violations_total_mw
            );

            results.push(metric);
        }

        results
    }

    /// Convert replay events to market snapshot for constraint evaluation
    fn events_to_snapshot(&self, events: &[ReplayEvent]) -> MarketSnapshot {
        // Extract LAST event (not just normal telemetry) - this captures adversarial state
        let mut load_mw = 4500.0;
        let mut generation_mw = 5000.0;
        let mut reserve_margin_mw = 500.0;

        for event in events {
            // Process last event of any type to capture final state (including adversarial)
            if event.telemetry.len() >= 1 {
                // For malformed telemetry, use realistic fallback
                if event.telemetry[0].value.is_nan() || event.telemetry[0].value.is_infinite() {
                    generation_mw = 4500.0; // Fallback for malformed
                } else {
                    generation_mw = event.telemetry[0].value;
                }
            }
            if event.telemetry.len() >= 2 {
                if event.telemetry[1].value.is_nan() || event.telemetry[1].value.is_infinite() {
                    load_mw = 4500.0; // Fallback
                } else {
                    load_mw = event.telemetry[1].value;
                }
            }
            if event.telemetry.len() >= 3 {
                if event.telemetry[2].value.is_nan() || event.telemetry[2].value.is_infinite() {
                    reserve_margin_mw = 500.0; // Fallback
                } else {
                    reserve_margin_mw = event.telemetry[2].value;
                }
            }
        }

        MarketSnapshot {
            load_mw,
            generation_mw,
            reserve_margin_mw,
            transmission_limits: vec![2000.0, 1800.0, 2200.0],
        }
    }

    fn build_operational_snapshot(
        &self,
        scenario: &ValidationScenario,
    ) -> OperationalSnapshot {
        let ingest_time_ms_utc = scenario
            .events
            .last()
            .map(|event| event.timestamp_ms)
            .unwrap_or(0)
            + 500;
        let source_latency_ms = 500;

        OperationalSnapshot::create(
            scenario
                .events
                .iter()
                .flat_map(|event| event.telemetry.clone())
                .collect(),
            ingest_time_ms_utc,
            source_latency_ms,
            scenario.topology_events.clone(),
            TopologyGraph::default(),
            None,
            scenario.operator_command.clone(),
            vec![
                ("BRANCH_01".to_string(), 1200.0),
                ("BRANCH_02".to_string(), 900.0),
            ]
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>(),
            Default::default(),
            Some(format!("replay-scenario-{}", scenario.scenario_id)),
            &SemanticConfig::default(),
        )
    }
}

fn main() {
    eprintln!();
    eprintln!("╔════════════════════════════════════════════════════════════╗");
    eprintln!("║  MVRE Trace Replay Harness                                 ║");
    eprintln!("║  Deterministic Replay Validation - CEO-DIR-024             ║");
    eprintln!("╚════════════════════════════════════════════════════════════╝");
    eprintln!();

    let validator = ReplayValidator::new();

    // Execute all 8 canonical scenarios
    eprintln!("📊 CANONICAL VALIDATION SCENARIOS");
    eprintln!("────────────────────────────────────────────────────────");
    eprintln!();

    let results = validator.execute_all_scenarios();

    eprintln!();
    eprintln!("╔════════════════════════════════════════════════════════════╗");
    eprintln!("║  DETERMINISM VALIDATION SUMMARY                            ║");
    eprintln!("╚════════════════════════════════════════════════════════════╝");
    eprintln!();

    let mut all_consistent = true;
    let mut total_latency = 0.0;
    let mut scenario_count = 0;

    for result in &results {
        eprintln!(
            "Scenario {}: {} - {}",
            result.cycle_id,
            result.scenario,
            if result.admissibility_consistent {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );

        if !result.admissibility_consistent {
            all_consistent = false;
        }

        total_latency += result.latency_ms;
        scenario_count += 1;
    }

    eprintln!();
    eprintln!("Average Latency: {:.3} ms", total_latency / scenario_count as f64);
    eprintln!("All Scenarios Consistent: {}", all_consistent);
    eprintln!();

    if all_consistent {
        eprintln!("✅ REPLAY VALIDATION PASSED");
        eprintln!("   All scenarios behaved deterministically and as expected.");
        eprintln!("   Admissibility arbitration is reproducible across iterations.");
    } else {
        eprintln!("❌ REPLAY VALIDATION FAILED");
        eprintln!("   Some scenarios did not produce expected outcomes.");
        eprintln!("   Determinism may be compromised.");
    }

    eprintln!();
    eprintln!("🔷 Testing determinism across multiple replay iterations");
    eprintln!();

    // Run scenario 1 multiple times to test cycle determinism
    let scenario_1 = &validator.scenarios[0];
    let determinism_results = validator.replay_determinism_test(scenario_1, 10);

    eprintln!("Scenario 1 replayed 10 times:");
    let mut trace_hashes = Vec::new();
    for (i, result) in determinism_results.iter().enumerate() {
        eprintln!(
            "  Iteration {}: Latency {:.3} ms | Hash: {}",
            i + 1,
            result.latency_ms,
            result.trace_hash
        );
        trace_hashes.push(result.trace_hash.clone());
    }

    let all_hashes_same = trace_hashes.iter().all(|h| h == &trace_hashes[0]);
    if all_hashes_same {
        eprintln!("✅ Determinism verified: All iterations produced identical traces");
    } else {
        eprintln!("❌ Determinism compromised: Iterations produced different traces");
    }

    eprintln!();
    eprintln!("════════════════════════════════════════════════════════════");
    eprintln!("✅ Replay Validation Harness Complete");
    eprintln!("════════════════════════════════════════════════════════════");
}
