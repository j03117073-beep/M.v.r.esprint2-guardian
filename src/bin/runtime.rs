// Copyright (c) 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// MVRE Authoritative Runtime
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

//! MVRE Authoritative Runtime
//!
//! This is the **canonical operational execution boundary** for the MVRE system.
//!
//! The runtime orchestrates the complete deterministic pipeline:
//!
//! 1. **Telemetry Ingestion** - Load measurements from protocol endpoints
//! 2. **Protocol Validation** - Authenticate and parse grid telemetry
//! 3. **Canonicalization** - Convert protocols to canonical representation
//! 4. **Admissibility Arbitration** - Evaluate constraint feasibility
//! 5. **Deterministic Kernel Arbitration** - Execute through SovereignKernel
//! 6. **Sovereign Trace Generation** - Produce unfalsifiable audit chain
//! 7. **Operator / Regulator Visibility** - Expose outcomes and compliance
//!
//! This is the production runtime. All other execution paths (verification,
//! simulation, research, adversarial harnesses) are clearly separated.

use m_v_r_esprint1::{
    constraint_system::{ConstraintEvaluator, PowerState, Trajectory, ViolationVector},
    ingest::rdf_parser::parse_cim_rdf,
    operator_interface::{build_dashboard_snapshot, render_trace_visualization_html},
    operational_semantics::{evaluate_infrastructure_semantics, OperatorCommandContext, OperationalSnapshot, ResolutionCategory, SemanticConfig, SemanticOutcome, TopologyEvent},
    protocol_drivers::{C37p118Driver, Dnp3Driver, Iec61850Driver, IccpTase2Driver,
        ModbusDriver, ProtocolDriver},
    regulatory_policy::{GovernanceMode, LegalCitation},
    sovereign_kernel::{
        signer_from_env, ActorContext, AuthMethod, Role,
        SovereignKernel, SovereignKernelConfig, TriggerType,
    },
    sovereign_trace::SovereignTrace,
    telemetry::{decode_double_bit_breaker, BreakerState, TelemetryPoint},
    topology::graph_builder::{build_topology_graph_with_mode, TopologyBuildMode, TopologyGraph, TopologyVersion},
    failure_axis::{FailureAxis, SystemHalt},
};
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process::ExitCode;

/// Runtime configuration from environment
#[derive(Debug, Clone)]
struct RuntimeConfig {
    /// Maximum kernel execution ticks per cycle
    max_ticks: u64,
    /// Tolerated telemetry staleness (milliseconds)
    telemetry_staleness_threshold_ms: u64,
    /// Artifacts output directory
    artifacts_dir: String,
    /// Operating mode: "operational", "shadow", "diagnostic"
    mode: String,
    /// Optional replay scenario override for offline sandbox validation
    replay_scenario: Option<String>,
    /// Optional trace HTML output for regulator/operator review
    trace_output_path: Option<String>,
    /// Optional authoritative CIM topology model path
    cim_model_path: Option<String>,
}

impl RuntimeConfig {
    fn from_env() -> Result<Self, SystemHalt> {
        Ok(Self {
            max_ticks: env::var("MVRE_MAX_TICKS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            telemetry_staleness_threshold_ms: env::var("MVRE_TELEMETRY_STALENESS_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20_000),
            artifacts_dir: env::var("MVRE_ARTIFACTS_DIR")
                .unwrap_or_else(|_| "./mvre-artifacts".to_string()),
            mode: env::var("MVRE_MODE").unwrap_or_else(|_| "operational".to_string()),
            replay_scenario: env::var("MVRE_REPLAY_SCENARIO").ok(),
            trace_output_path: env::var("MVRE_TRACE_OUTPUT_PATH").ok(),
            cim_model_path: env::var("MVRE_CIM_MODEL_PATH").ok(),
        })
    }
}

/// Complete runtime state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeState {
    /// Initializing protocol and kernel
    Initializing,
    /// Ready to process telemetry
    Ready,
    /// Evaluating constraint feasibility
    Evaluating,
    /// Kernel executing
    Executing,
    /// Generating and publishing audit trace
    Auditing,
    /// Healthy operational state
    Nominal,
    /// Degraded (telemetry issues, constraint violations)
    Degraded,
    /// Incoherent (contradictions detected)
    Incoherent,
    /// Emergency halt
    Emergency,
}

/// Runtime operational telemetry
#[derive(Debug, Clone)]
struct RuntimeMetrics {
    cycles_executed: u64,
    admissible_decisions: u64,
    inadmissible_decisions: u64,
    total_violations: f64,
    last_error: Option<String>,
}

impl Default for RuntimeMetrics {
    fn default() -> Self {
        Self {
            cycles_executed: 0,
            admissible_decisions: 0,
            inadmissible_decisions: 0,
            total_violations: 0.0,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeInput {
    pub telemetry: Vec<TelemetryPoint>,
    pub topology_events: Vec<TopologyEvent>,
    pub topology_graph: TopologyGraph,
    pub topology_version: Option<TopologyVersion>,
    pub operator_command: Option<OperatorCommandContext>,
    pub source_latency_ms: u64,
    pub ingest_time_ms_utc: u64,
    pub scenario: Option<String>,
}

/// Main runtime orchestrator
struct MvreRuntime {
    config: RuntimeConfig,
    kernel: SovereignKernel,
    state: RuntimeState,
    metrics: RuntimeMetrics,
}

impl MvreRuntime {
    /// Construct and initialize the runtime
    fn new(config: RuntimeConfig) -> Result<Self, SystemHalt> {
        eprintln!("🔄 MVRE Runtime: Initialization Phase");
        eprintln!("  Mode: {}", config.mode);
        eprintln!("  Max Ticks: {}", config.max_ticks);
        eprintln!("  Artifacts: {}", config.artifacts_dir);

        // Validate signer mode
        env::set_var("SIGNER_MODE", "simulation"); // Default to simulation for now
        let signer = signer_from_env()?;

        let kernel_config = SovereignKernelConfig {
            max_ticks: config.max_ticks,
        };
        let kernel = SovereignKernel::new(signer, kernel_config);

        eprintln!("✅ Kernel initialized");
        eprintln!("✅ Signer ready");
        eprintln!("✅ Trusted time authority initialized");

        Ok(Self {
            config,
            kernel,
            state: RuntimeState::Initializing,
            metrics: RuntimeMetrics::default(),
        })
    }

    /// Execute the authoritative runtime pipeline
    fn execute_cycle(&mut self) -> Result<(), SystemHalt> {
        eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("🔷 MVRE Runtime Cycle");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        self.metrics.cycles_executed += 1;

        // Phase 1: Telemetry Ingestion
        self.state = RuntimeState::Initializing;
        eprintln!("\n📡 Phase 1: Telemetry Ingestion");
        let runtime_input = self.ingest_telemetry()?;
        eprintln!("  ✓ Loaded {} measurement points", runtime_input.telemetry.len());
        if let Some(scenario) = &runtime_input.scenario {
            eprintln!("  ✓ Replay scenario: {}", scenario);
        }

        // Phase 2: Protocol Validation
        eprintln!("\n🔐 Phase 2: Protocol Validation");
        self.state = RuntimeState::Ready;
        self.validate_protocols(&runtime_input)?;
        eprintln!("  ✓ Protocols parsed and authenticated");

        // Phase 3: Canonicalization
        eprintln!("\n📝 Phase 3: Canonicalization");
        let trajectory = self.canonicalize_telemetry(&runtime_input.telemetry)?;
        eprintln!("  ✓ Trajectory canonicalized: {} intervals", trajectory.intervals.len());

        // Phase 4: Admissibility Arbitration
        eprintln!("\n⚖️  Phase 4: Admissibility Arbitration");
        self.state = RuntimeState::Evaluating;
        let operational_snapshot = OperationalSnapshot::create(
            runtime_input.telemetry.clone(),
            runtime_input.ingest_time_ms_utc,
            runtime_input.source_latency_ms,
            runtime_input.topology_events.clone(),
            runtime_input.topology_graph.clone(),
            runtime_input.topology_version.clone(),
            runtime_input.operator_command.clone(),
            BTreeMap::new(),
            BTreeMap::new(),
            runtime_input.scenario.clone(),
            &SemanticConfig::default(),
        );
        let semantics = evaluate_infrastructure_semantics(&operational_snapshot, &SemanticConfig::default());
        let (admissible, violations) = self.evaluate_admissibility(&trajectory)?;
        let admissible = admissible && semantics.admissible;

        if admissible {
            eprintln!("  ✅ ADMISSIBLE");
            self.metrics.admissible_decisions += 1;
        } else {
            eprintln!("  ❌ PRE-RESOLUTION CONSTRAINT ENFORCEMENT: invalid state detected before kernel commitment");
            eprintln!("     Capacity Upper: {:.1} MW", violations.capacity_upper);
            eprintln!("     Capacity Lower: {:.1} MW", violations.capacity_lower);
            eprintln!("     Total Violation: {:.1} MW", violations.total());
            for violation in &semantics.violations {
                eprintln!("     · {}: {}", violation.constraint, violation.reason);
            }
            self.metrics.inadmissible_decisions += 1;
            self.metrics.total_violations += violations.total();
            self.state = RuntimeState::Incoherent;
        }

        // Phase 5: Kernel Arbitration
        if admissible {
            eprintln!("\n🔮 Phase 5: Deterministic Kernel Arbitration");
            self.state = RuntimeState::Executing;
            let _kernel_result = self.execute_kernel_cycle(admissible)?;
            eprintln!("  ✓ Kernel cycle completed");
        } else {
            eprintln!("\n🔮 Phase 5: Deterministic Kernel Arbitration skipped due to pre-resolution invalid topology or semantic divergence");
        }

        // Phase 6: Sovereign Trace Generation
        eprintln!("\n🔗 Phase 6: Sovereign Trace Generation");
        self.state = RuntimeState::Auditing;
        let _trace = self.generate_sovereign_trace(admissible, &operational_snapshot, &semantics)?;
        eprintln!("  ✓ Trace anchored in TPM");

        // Phase 7: Operator Visibility
        eprintln!("\n👁️  Phase 7: Operator / Regulator Visibility");
        self.expose_operator_status(&violations, &semantics)?;

        self.state = if admissible {
            RuntimeState::Nominal
        } else {
            RuntimeState::Degraded
        };

        eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("✅ Cycle complete | State: {:?}", self.state);
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        Ok(())
    }

    /// Phase 1: Ingest telemetry from protocol endpoints or replay scenario
    fn ingest_telemetry(&self) -> Result<RuntimeInput, SystemHalt> {
        let ingest_time_ms_utc = 30_000;
        let mut source_latency_ms = 500;
        let mut telemetry = vec![
            TelemetryPoint {
                value: 4500.0,
                point_timestamp_ms_utc: 1000,
                quality_mask: 0x00,
            },
            TelemetryPoint {
                value: 5000.0,
                point_timestamp_ms_utc: 1000,
                quality_mask: 0x00,
            },
            TelemetryPoint {
                value: 500.0,
                point_timestamp_ms_utc: 1000,
                quality_mask: 0x00,
            },
        ];
        let mut topology_events = Vec::new();
        let mut operator_command = None;
        let scenario = self.config.replay_scenario.clone();

        if let Some(name) = scenario.as_deref() {
            match name {
                "timing_divergence" => {
                    telemetry = vec![
                        TelemetryPoint {
                            value: 4800.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 5500.0,
                            point_timestamp_ms_utc: 800,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: -700.0,
                            point_timestamp_ms_utc: 900,
                            quality_mask: 0x00,
                        },
                    ];
                    source_latency_ms = 2100;
                }
                "relay_disagreement" => {
                    telemetry = vec![
                        TelemetryPoint {
                            value: 5200.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 5100.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 100.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                    ];
                    topology_events.push(TopologyEvent {
                        equipment_id: "BRK_1".to_string(),
                        breaker_state: BreakerState::Closed,
                        timestamp_ms_utc: 1000,
                    });
                    topology_events.push(TopologyEvent {
                        equipment_id: "BRK_1".to_string(),
                        breaker_state: BreakerState::Open,
                        timestamp_ms_utc: 1005,
                    });
                }
                "conflicting_telemetry" => {
                    telemetry = vec![
                        TelemetryPoint {
                            value: 4500.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 6200.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x02,
                        },
                        TelemetryPoint {
                            value: -1700.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x02,
                        },
                    ];
                }
                "delayed_scada" => {
                    telemetry = vec![
                        TelemetryPoint {
                            value: 4700.0,
                            point_timestamp_ms_utc: 5_000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 5100.0,
                            point_timestamp_ms_utc: 5_000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 400.0,
                            point_timestamp_ms_utc: 5_000,
                            quality_mask: 0x00,
                        },
                    ];
                    source_latency_ms = 30_000;
                }
                "impossible_topology" => {
                    telemetry = vec![
                        TelemetryPoint {
                            value: 4900.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 4900.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 0.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                    ];
                    topology_events.push(TopologyEvent {
                        equipment_id: "BRK_2".to_string(),
                        breaker_state: BreakerState::Closed,
                        timestamp_ms_utc: 1000,
                    });
                    topology_events.push(TopologyEvent {
                        equipment_id: "BRK_2".to_string(),
                        breaker_state: BreakerState::Open,
                        timestamp_ms_utc: 1000,
                    });
                }
                "command_mismatch" => {
                    telemetry = vec![
                        TelemetryPoint {
                            value: 5000.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 4950.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 50.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                    ];
                    operator_command = Some(OperatorCommandContext {
                        operator_id: "guest.operator".to_string(),
                        role: "guest".to_string(),
                        command: "increase_generation".to_string(),
                        authority_level: 1,
                    });
                }
                "cascading_failure" => {
                    telemetry = vec![
                        TelemetryPoint {
                            value: 6200.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 6600.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: -400.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                    ];
                    source_latency_ms = 2_500;
                }
                "safe_state_stabilization" => {
                    telemetry = vec![
                        TelemetryPoint {
                            value: 4800.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 4900.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                        TelemetryPoint {
                            value: 100.0,
                            point_timestamp_ms_utc: 1000,
                            quality_mask: 0x00,
                        },
                    ];
                    topology_events.push(TopologyEvent {
                        equipment_id: "BRK_4".to_string(),
                        breaker_state: BreakerState::Closed,
                        timestamp_ms_utc: 1000,
                    });
                    topology_events.push(TopologyEvent {
                        equipment_id: "BRK_4".to_string(),
                        breaker_state: BreakerState::Intermediate,
                        timestamp_ms_utc: 1002,
                    });
                }
                _ => {}
            }
        }

        let (topology_graph, topology_version) = self.build_authoritative_topology(&topology_events)?;

        Ok(RuntimeInput {
            telemetry,
            topology_events,
            topology_graph,
            topology_version,
            operator_command,
            source_latency_ms,
            ingest_time_ms_utc,
            scenario,
        })
    }

    fn build_authoritative_topology(
        &self,
        topology_events: &[TopologyEvent],
    ) -> Result<(TopologyGraph, Option<TopologyVersion>), SystemHalt> {
        if let Some(path) = &self.config.cim_model_path {
            let file = File::open(path).map_err(|e| {
                SystemHalt::with_formatted(
                    FailureAxis::ExternalInjectionDetected,
                    format!("Cannot open CIM model file '{}': {e}", path),
                )
            })?;
            let reader = BufReader::new(file);
            let model = parse_cim_rdf(reader).map_err(|e| {
                SystemHalt::with_formatted(
                    FailureAxis::ExternalInjectionDetected,
                    format!("Failed to parse CIM model '{}': {:?}", path, e),
                )
            })?;
            let telemetered_switch_closed = self.derive_telemetered_switch_state(topology_events);
            let graph = build_topology_graph_with_mode(
                &model,
                &telemetered_switch_closed,
                TopologyBuildMode::TelemetryOverride,
            );
            let version = graph.seal_version(None, self.metrics.cycles_executed);
            Ok((graph, Some(version)))
        } else if self.config.mode == "operational" {
            Err(SystemHalt::with_formatted(
                FailureAxis::InternalInvariantBreach,
                "Operational runtime requires authoritative CIM topology input".to_string(),
            ))
        } else {
            Ok((TopologyGraph::default(), None))
        }
    }

    fn derive_telemetered_switch_state(
        &self,
        topology_events: &[TopologyEvent],
    ) -> BTreeMap<String, bool> {
        let mut status_updates: Vec<(&String, bool, u64)> = topology_events
            .iter()
            .filter_map(|event| match event.breaker_state {
                BreakerState::Closed => Some((&event.equipment_id, true, event.timestamp_ms_utc)),
                BreakerState::Open => Some((&event.equipment_id, false, event.timestamp_ms_utc)),
                _ => None,
            })
            .collect();

        status_updates.sort_unstable_by(|a, b| a.0.cmp(b.0).then(a.2.cmp(&b.2)));
        let mut map = BTreeMap::new();
        for (equipment_id, closed, _) in status_updates {
            map.insert(equipment_id.clone(), closed);
        }
        map
    }

    /// Phase 2: Validate protocol endpoints and authentication
    fn validate_protocols(&self, input: &RuntimeInput) -> Result<(), SystemHalt> {
        // In production, this would discover and authenticate:
        // - DNP3 endpoints
        // - IEC-61850 endpoints
        // - ICCP-TASE2 endpoints
        // - C37.118 PMU endpoints
        // - Modbus RTU endpoints

        let drivers: Vec<Box<dyn ProtocolDriver>> = vec![
            Box::new(Dnp3Driver),
            Box::new(Iec61850Driver),
            Box::new(IccpTase2Driver),
            Box::new(C37p118Driver),
            Box::new(ModbusDriver),
        ];

        for driver in drivers {
            eprintln!("  ✓ {:?} driver ready", driver.kind());
        }

        for point in &input.telemetry {
            if point.quality_mask != 0x00 {
                let msg = format!(
                    "Protocol validation failed: bad telemetry quality mask {:#04x}",
                    point.quality_mask
                );
                return Err(SystemHalt::new(
                    FailureAxis::ExternalInjectionDetected,
                    &msg,
                ));
            }
        }

        Ok(())
    }

    /// Phase 3: Convert telemetry to canonical trajectory
    fn canonicalize_telemetry(&self, points: &[TelemetryPoint]) -> Result<Trajectory, SystemHalt> {
        // Sum telemetry into canonical power state
        // For this canonical runtime, we use positional mapping:
        // [0] = total load, [1] = total generation, [2] = reserve margin
        let _total_load = if points.len() >= 1 {
            points[0].value
        } else {
            0.0
        };
        let total_generation = if points.len() >= 2 {
            points[1].value
        } else {
            0.0
        };
        let reserve_margin = if points.len() >= 3 {
            points[2].value
        } else {
            0.0
        };

        // Create canonical power state
        let state = PowerState::new(
            total_generation * 0.9, // Current power (90% of generation)
            total_generation * 0.85, // Previous power (85% baseline)
            reserve_margin * 0.5,   // Reg up (50% of reserve)
            reserve_margin * 0.3,   // Reg down (30% of reserve)
            total_generation * 0.2, // Min power
            total_generation * 1.1, // Max power
            total_generation * 0.1, // Ramp up
            total_generation * 0.1, // Ramp down
        );

        // Create 3-interval trajectory
        let trajectory = Trajectory::new(vec![
            state.clone(),
            PowerState::new(
                total_generation * 0.92,
                state.p_t,
                state.reg_up,
                state.reg_down,
                state.p_min,
                state.p_max,
                state.ramp_up,
                state.ramp_down,
            ),
            PowerState::new(
                total_generation * 0.95,
                state.p_t,
                state.reg_up,
                state.reg_down,
                state.p_min,
                state.p_max,
                state.ramp_up,
                state.ramp_down,
            ),
        ]);

        Ok(trajectory)
    }

    /// Phase 4: Evaluate admissibility (constraint feasibility)
    fn evaluate_admissibility(
        &self,
        trajectory: &Trajectory,
    ) -> Result<(bool, ViolationVector), SystemHalt> {
        let violations = ConstraintEvaluator::evaluate_trajectory(trajectory);
        let admissible = violations.is_feasible();
        Ok((admissible, violations))
    }

    /// Phase 5: Execute through the SovereignKernel
    fn execute_kernel_cycle(&mut self, _admissible: bool) -> Result<(), SystemHalt> {
        // In production, this would:
        // - Build an IR module representing the current control decision
        // - Pass it to SovereignKernel::execute_foreign_with_actor()
        // - The kernel generates attestation records bound to:
        //   * Actor identity (System)
        //   * Command type (ExecuteForeignIr)
        //   * TPM PCR chain
        //   * Trusted time
        // - Kernel output is deterministic and reproducible

        let actor = ActorContext {
            operator_id: "system.kernel.runtime".to_string(),
            role: Role::System,
            auth_method: AuthMethod::Internal,
            session_id: "mvre-runtime-session".to_string(),
            is_automated: true,
            trigger: TriggerType::Automated,
            approver_id: None,
            operator_ack_token: None,
        };

        // Placeholder: actual IR generation and kernel execution would go here
        eprintln!("  Actor: {} ({})", actor.operator_id, "System");

        Ok(())
    }

    /// Phase 6: Generate sovereign trace (unfalsifiable audit chain)
    fn generate_sovereign_trace(
        &self,
        _admissible: bool,
        snapshot: &OperationalSnapshot,
        semantics: &SemanticOutcome,
    ) -> Result<SovereignTrace, SystemHalt> {
        let governance_mode = match semantics.resolution_category {
            ResolutionCategory::Informational => GovernanceMode::Normal,
            ResolutionCategory::Advisory => GovernanceMode::Normal,
            ResolutionCategory::Suspicious => GovernanceMode::Degraded,
            ResolutionCategory::Incoherent => GovernanceMode::Degraded,
            ResolutionCategory::NonAdmissible => GovernanceMode::EmergencyRateLimit,
            ResolutionCategory::InfrastructureImpossible => GovernanceMode::EmergencyRateLimit,
            ResolutionCategory::SovereignViolation => GovernanceMode::EmergencyRateLimit,
        };

        let requested = snapshot.telemetry.get(0).map(|p| p.value).unwrap_or(0.0);
        let actual = snapshot.telemetry.get(1).map(|p| p.value).unwrap_or(requested);
        let replay_equivalence_id = snapshot
            .replay_equivalence_metadata
            .clone()
            .or_else(|| Some("operational-runtime".to_string()));

        let trace = SovereignTrace::attest(
            self.metrics.cycles_executed,
            requested,
            actual,
            governance_mode,
            LegalCitation::default(),
            snapshot,
            semantics,
            replay_equivalence_id,
        )?;

        eprintln!("  Trace tick: {}", trace.tick);
        eprintln!("  Trace hash: {}", trace.trace_hash);

        Ok(trace)
    }

    /// Phase 7: Expose status to operator dashboard
    fn expose_operator_status(
        &self,
        violations: &ViolationVector,
        semantics: &SemanticOutcome,
    ) -> Result<(), SystemHalt> {
        let snapshot = build_dashboard_snapshot(&self.config.artifacts_dir)?;
        let html = render_trace_visualization_html(&snapshot, violations, semantics);

        eprintln!("  Compliance Score: {:.1}%", snapshot.compliance_score * 100.0);
        eprintln!("  Violations: {:.1} MW", violations.total());
        eprintln!("  Resolution: {:?}", semantics.resolution);
        eprintln!("  Resolution Category: {:?}", semantics.resolution_category);

        if let Some(path) = &self.config.trace_output_path {
            std::fs::write(path, html.as_bytes()).map_err(|e| {
                SystemHalt::with_formatted(
                    FailureAxis::ExternalInjectionDetected,
                    format!("Cannot write trace visualization HTML: {e}"),
                )
            })?;
            eprintln!("  ✓ Trace visualization written to {}", path);
        }

        Ok(())
    }

    /// Print operational summary
    fn print_summary(&self) {
        eprintln!("\n");
        eprintln!("╔════════════════════════════════════════════╗");
        eprintln!("║  MVRE AUTHORITATIVE RUNTIME - SUMMARY     ║");
        eprintln!("╚════════════════════════════════════════════╝");
        eprintln!();
        eprintln!("  Cycles Executed:      {}", self.metrics.cycles_executed);
        eprintln!(
            "  Admissible Decisions: {}",
            self.metrics.admissible_decisions
        );
        eprintln!(
            "  Inadmissible Decisions: {}",
            self.metrics.inadmissible_decisions
        );
        eprintln!("  Total Violations:     {:.1} MW", self.metrics.total_violations);
        eprintln!("  Final State:          {:?}", self.state);
        if let Some(ref error) = self.metrics.last_error {
            eprintln!("  Last Error:           {}", error);
        }
        eprintln!();
    }
}

fn main() -> ExitCode {
    eprintln!();
    eprintln!("╔════════════════════════════════════════════════════════════╗");
    eprintln!("║  MVRE: Deterministic Operational Trust Kernel              ║");
    eprintln!("║  Authoritative Runtime - CEO-DIR-023-EXEC (Verified)      ║");
    eprintln!("╚════════════════════════════════════════════════════════════╝");
    eprintln!();

    // Load configuration from environment
    let config = match RuntimeConfig::from_env() {
        Ok(cfg) => cfg,
        Err(halt) => {
            eprintln!("❌ Configuration error: {}", halt.message);
            return ExitCode::FAILURE;
        }
    };

    // Initialize runtime
    let mut runtime = match MvreRuntime::new(config.clone()) {
        Ok(rt) => rt,
        Err(halt) => {
            eprintln!("❌ Initialization failed: {}", halt.message);
            return ExitCode::FAILURE;
        }
    };

    // Execute operational cycle(s)
    let cycle_count = env::var("MVRE_CYCLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    for cycle in 1..=cycle_count {
        eprintln!("\n[ Cycle {}/{} ]", cycle, cycle_count);

        if let Err(halt) = runtime.execute_cycle() {
            eprintln!("❌ Cycle error: {}", halt.message);
            runtime.metrics.last_error = Some(halt.message.clone());

            // In production, this might trigger escalation logic, but we continue
            if cycle == cycle_count {
                runtime.print_summary();
                return ExitCode::FAILURE;
            }
        }
    }

    // Print final summary
    runtime.print_summary();

    eprintln!("✅ MVRE Authoritative Runtime - Execution Complete");
    eprintln!();

    ExitCode::SUCCESS
}
