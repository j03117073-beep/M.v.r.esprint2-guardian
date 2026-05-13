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
use m_v_r_esprint1::ir_codegen::IRInput;
use m_v_r_esprint1::sovereign_kernel::{
    attestation_record_data, build_artifact_payload, build_command_payload, build_signature_payload,
    hash_record, sign_payload_for_actor, signer_from_env, ActorContext, AttestationRecord,
    AuthMethod, CommandEnvelope, CommandType, ExecutionArtifact, Role, SovereignKernel,
    SovereignKernelConfig, TriggerType,
};
use m_v_r_esprint1::tpm_attestation::{build_attestation_nonce, SimulatedTpmAttestor, TpmAttestor};
use m_v_r_esprint1::trusted_time::{TimeSource, TrustedTime};
use m_v_r_esprint1::universal_frontend::IRModule;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("SIGNER_MODE", "simulation");

    let signer = signer_from_env().map_err(|e| format!("{e:?}"))?;
    let config = SovereignKernelConfig { max_ticks: 100 };
    let mut kernel = SovereignKernel::new(signer, config);

    let ir_module = IRModule {
        functions: vec![],
        constants: vec![],
    };

    let input = IRInput {
        args: HashMap::new(),
    };

    for _ in 0..3 {
        let _ = kernel
            .execute_foreign(&ir_module, input.clone())
            .map_err(|e| format!("{e:?}"))?;
    }

    let records = build_demo_records(3);
    let path = "pilot_attestation_log.json";
    let json = serde_json::to_string_pretty(&records)?;
    fs::write(path, json)?;

    println!("Generated pilot attestation log with {} records", records.len());

    let verifier_output = Command::new("cargo")
        .args(["run", "--bin", "verifier", path])
        .output()?;

    if verifier_output.status.success() {
        print!("{}", String::from_utf8_lossy(&verifier_output.stdout));
    } else {
        eprintln!("{}", String::from_utf8_lossy(&verifier_output.stderr));
    }

    Ok(())
}

fn build_demo_records(count: usize) -> Vec<AttestationRecord> {
    let mut records = Vec::with_capacity(count);
    let mut previous_record_hash = vec![0u8; 32];
    let mut previous_time: Option<TrustedTime> = None;
    let mut tpm = SimulatedTpmAttestor::new(16);

    for i in 0..count {
        let decision_hash = Sha256::digest(format!("decision-{i}").as_bytes()).to_vec();
        let pcr_digest = vec![0u8; 32];
        let wall_time_ns = 1_710_000_000_000_000_000u64 + (i as u64 * 1_000_000);
        let monotonic_ns = previous_time
            .as_ref()
            .map(|t| t.monotonic_ns.saturating_add(1_000_000))
            .unwrap_or(wall_time_ns);
        let time = TrustedTime {
            wall_time_ns,
            monotonic_ns,
            source: TimeSource::PTP,
            uncertainty_ns: 1_000,
        };

        let actor = if i == 1 {
            ActorContext {
                operator_id: "dispatch_ercot_01".to_string(),
                role: Role::Dispatcher,
                auth_method: AuthMethod::SmartCard,
                session_id: "session-dispatch-01".to_string(),
                is_automated: false,
                trigger: TriggerType::Human,
                approver_id: Some("shift_supervisor_03".to_string()),
                operator_ack_token: None,
            }
        } else {
            ActorContext {
                operator_id: "system.automation.sced".to_string(),
                role: Role::System,
                auth_method: AuthMethod::Internal,
                session_id: "session-auto-sced".to_string(),
                is_automated: true,
                trigger: TriggerType::Automated,
                approver_id: None,
                operator_ack_token: None,
            }
        };

        let command_type = if i == 1 {
            CommandType::ScedOverride
        } else {
            CommandType::ExecuteForeignIr
        };

        let command_hash = Sha256::digest(format!("command-{i}").as_bytes()).to_vec();
        let mut command = CommandEnvelope {
            command_id: format!("cmd-{i}"),
            command_type,
            payload_hash: command_hash.clone(),
            actor: actor.clone(),
            timestamp: time.clone(),
            command_signature: Vec::new(),
        };
        let command_payload = build_command_payload(&command);
        command.command_signature = sign_payload_for_actor(&actor, &command_payload);

        let mut artifact = ExecutionArtifact {
            artifact_hash: Sha256::digest(format!("artifact-{i}").as_bytes()).to_vec(),
            result_code: "OK".to_string(),
            artifact_signature: Vec::new(),
        };
        let artifact_payload = build_artifact_payload(&artifact, &time, &command_hash);
        artifact.artifact_signature = sign_payload_for_actor(&actor, &artifact_payload);

        let signature_payload = build_signature_payload(&decision_hash, &pcr_digest, &time);
        let mut sig_hasher = Sha256::new();
        sig_hasher.update(&signature_payload);
        sig_hasher.update(b"simulated-key");
        let signature = sig_hasher.finalize().to_vec();

        let prev_hash = previous_record_hash.clone();
        let record = AttestationRecord {
            decision_hash: decision_hash.clone(),
            pcr_digest,
            signature,
            timestamp: wall_time_ns / 1_000_000_000,
            wall_time_ns,
            monotonic_ns,
            time_source: time.source,
            time_uncertainty_ns: time.uncertainty_ns,
            actor,
            command,
            artifact,
            record_hash: Vec::new(),
            tpm_attest: Vec::new(),
            tpm_signature: Vec::new(),
            pcr_index: tpm.pcr_index(),
            prev_hash,
        };

        let mut record = record;
        let record_hash = hash_record(&record.prev_hash, &attestation_record_data(&record), &time);
        tpm.extend_pcr(&record_hash).expect("simulated tpm extend");
        let nonce = build_attestation_nonce(&record.decision_hash, &record_hash, &time);
        let (tpm_attest, tpm_signature) = tpm.quote(&nonce).expect("simulated tpm quote");
        record.record_hash = record_hash.clone();
        record.tpm_attest = tpm_attest;
        record.tpm_signature = tpm_signature;

        previous_record_hash = record_hash;
        previous_time = Some(time);
        records.push(record);
    }

    records
}

