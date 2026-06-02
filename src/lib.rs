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

pub mod adversarial_harness;
pub mod compliance;
pub mod constraint_system;
pub mod demo_pipeline;
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
pub mod audit_ticket;
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

// Universal Execution + Audit Layer
pub mod universal_frontend;
pub mod ir_codegen;
pub mod sovereign_bus;
pub mod ir_backends;
pub mod crypto_pipeline;
