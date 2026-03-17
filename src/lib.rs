#![deny(unsafe_code)]

pub mod adversarial_harness;
pub mod compliance;
pub mod drivers;
pub mod failure_axis;
pub mod fiel;
pub mod sp_api;
pub mod testament_audit;
pub mod tlbss_types;
pub mod zero_state;

// new supervisory kernel components
pub mod ai_ingestion_buffer;
pub mod kernel;
pub mod setpoint_guard;
pub mod simulation;

// 2026 Flagship Regulatory Compliance Framework
pub mod audit_guardian;
pub mod deployment_manifest;
pub mod grid_code_templates;
pub mod hal_output;
pub mod interface_discovery;
pub mod operator_interface;
pub mod protocol_drivers;
pub mod recovery;
pub mod regulatory_policy;
pub mod scheduler;
pub mod simulation_harness_core;
pub mod sovereign_kernel;
pub mod sovereign_trace;
pub mod tlbss_integrity_engine;
pub mod visions_core;
