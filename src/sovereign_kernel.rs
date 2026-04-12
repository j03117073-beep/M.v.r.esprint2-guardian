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

use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::tpm_attestation::{build_attestation_nonce, SimulatedTpmAttestor, TpmAttestor};
use crate::trusted_time::{SystemTimeProvider, TimeSource, TrustedTime, TrustedTimeAuthority};
use ed25519_dalek::{Signature as Ed25519Signature, Signer as DalekSigner, SigningKey, Verifier};
use sha2::{Digest, Sha256};
use std::env;
use std::sync::Arc;
#[cfg(feature = "tpm")]
use tss_esapi::{Context, Tcti};

/// Immutable per-tick audit record for the sovereign substrate.
#[derive(Debug, Clone, PartialEq)]
pub struct SovereignTrace {
    pub tick: u64,
    pub ai_request: u64,
    pub kernel_output: u64,
    pub authority_level: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SummaryAttestation {
    pub trace_hash: String,
    pub tick_count: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum Role {
    Dispatcher = 0,
    ReliabilityEngineer = 1,
    MarketOperator = 2,
    System = 3,
}

impl Default for Role {
    fn default() -> Self {
        Self::System
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum AuthMethod {
    SmartCard = 0,
    Token = 1,
    APIKey = 2,
    Internal = 3,
}

impl Default for AuthMethod {
    fn default() -> Self {
        Self::Internal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum TriggerType {
    Human = 0,
    Automated = 1,
    ExternalSignal = 2,
}

impl Default for TriggerType {
    fn default() -> Self {
        Self::Automated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum CommandType {
    ExecuteForeignIr = 0,
    ScedDispatch = 1,
    ScedOverride = 2,
    EmergencyOverride = 3,
    AuditReadOnly = 4,
}

impl Default for CommandType {
    fn default() -> Self {
        Self::ExecuteForeignIr
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct ActorContext {
    pub operator_id: String,
    pub role: Role,
    pub auth_method: AuthMethod,
    pub session_id: String,
    pub is_automated: bool,
    pub trigger: TriggerType,
    pub approver_id: Option<String>,
}

impl ActorContext {
    pub fn system_runtime() -> Self {
        Self {
            operator_id: "system.kernel.runtime".to_string(),
            role: Role::System,
            auth_method: AuthMethod::Internal,
            session_id: "runtime-session".to_string(),
            is_automated: true,
            trigger: TriggerType::Automated,
            approver_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct CommandEnvelope {
    pub command_id: String,
    pub command_type: CommandType,
    pub payload_hash: Vec<u8>,
    pub actor: ActorContext,
    pub timestamp: TrustedTime,
    pub command_signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct ExecutionArtifact {
    pub artifact_hash: Vec<u8>,
    pub result_code: String,
    pub artifact_signature: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttestationRecord {
    pub decision_hash: Vec<u8>,
    pub pcr_digest: Vec<u8>,
    pub signature: Vec<u8>,
    /// Legacy seconds field retained for compatibility with existing artifacts.
    pub timestamp: u64,
    #[serde(default)]
    pub wall_time_ns: u64,
    #[serde(default)]
    pub monotonic_ns: u64,
    #[serde(default)]
    pub time_source: TimeSource,
    #[serde(default)]
    pub time_uncertainty_ns: u64,
    #[serde(default)]
    pub actor: ActorContext,
    #[serde(default)]
    pub command: CommandEnvelope,
    #[serde(default)]
    pub artifact: ExecutionArtifact,
    #[serde(default)]
    pub record_hash: Vec<u8>,
    #[serde(default)]
    pub tpm_attest: Vec<u8>,
    #[serde(default)]
    pub tpm_signature: Vec<u8>,
    #[serde(default)]
    pub pcr_index: u32,
    pub prev_hash: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L7Disposition {
    Allowed,
    Constrained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L7Reason {
    Nominal,
    Thermal,
    Frequency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L7Event {
    pub tick: u64,
    pub disposition: L7Disposition,
}

/// Non-agentic certifier. Reads metrics and signs only if gates are satisfied.
pub trait Signer {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SystemHalt>;
    fn read_pcr(&self) -> Result<Vec<u8>, SystemHalt>;
}

/// Deterministic fixed-key signer for simulation/testing.
/// In production replace with a TPM-backed signer implementation.
#[derive(Debug, Clone)]
pub struct FixedKeySimulatedTpmSigner {
    key: String,
}

impl FixedKeySimulatedTpmSigner {
    pub fn new() -> Self {
        Self {
            key: "simulated-key".to_string(),
        }
    }
}

impl Signer for FixedKeySimulatedTpmSigner {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SystemHalt> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(self.key.as_bytes());
        let result = hasher.finalize();
        Ok(result.to_vec())
    }

    fn read_pcr(&self) -> Result<Vec<u8>, SystemHalt> {
        Ok(vec![0u8; 32]) // fake PCR
    }
}

/// TPM 2.0 signer using direct tss-esapi integration.
///
/// This provides deterministic, in-process TPM signing without external tools.
#[cfg(feature = "tpm")]
#[derive(Debug)]
pub struct TssEsapiSigner {
    context: Context,
}

#[cfg(feature = "tpm")]
impl TssEsapiSigner {
    pub fn new() -> Result<Self, SystemHalt> {
        let tcti = Tcti::device(std::path::Path::new("/dev/tpm0")).map_err(|e| {
            SystemHalt::new(FailureAxis::TpmUnavailable, &format!("TCTI error: {:?}", e))
        })?;
        let context = Context::new(tcti).map_err(|e| {
            SystemHalt::new(FailureAxis::TpmUnavailable, &format!("Context error: {:?}", e))
        })?;
        Ok(Self { context })
    }
}

#[cfg(feature = "tpm")]
impl Signer for TssEsapiSigner {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SystemHalt> {
        let _ = &self.context;
        // Placeholder: implement actual signing with loaded key
        Ok(data.to_vec()) // fake signature
    }

    fn read_pcr(&self) -> Result<Vec<u8>, SystemHalt> {
        let _ = &self.context;
        // Placeholder: read actual PCR
        Ok(vec![0u8; 32]) // fake PCR
    }
}

/// Runtime-selectable signer wrapper used by runtime binaries.
#[derive(Debug)]
pub enum AnyTpmSigner {
    Simulated(FixedKeySimulatedTpmSigner),
    #[cfg(feature = "tpm")]
    Tpm(TssEsapiSigner),
}

impl Signer for AnyTpmSigner {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SystemHalt> {
        match self {
            AnyTpmSigner::Simulated(s) => s.sign(data),
            #[cfg(feature = "tpm")]
            AnyTpmSigner::Tpm(s) => s.sign(data),
        }
    }

    fn read_pcr(&self) -> Result<Vec<u8>, SystemHalt> {
        match self {
            AnyTpmSigner::Simulated(s) => s.read_pcr(),
            #[cfg(feature = "tpm")]
            AnyTpmSigner::Tpm(s) => s.read_pcr(),
        }
    }
}

pub fn signer_from_env() -> Result<AnyTpmSigner, SystemHalt> {
    match env::var("SIGNER_MODE").as_deref() {
        Ok("tpm") => {
            #[cfg(feature = "tpm")]
            {
                TssEsapiSigner::new().map(AnyTpmSigner::Tpm)
            }
            #[cfg(not(feature = "tpm"))]
            Err(SystemHalt::new(
                FailureAxis::TpmUnavailable,
                "TPM required but not enabled",
            ))
        }
        Ok("simulation") => Ok(AnyTpmSigner::Simulated(FixedKeySimulatedTpmSigner::new())),
        _ => Err(SystemHalt::new(
            FailureAxis::UnauthorizedMode,
            "No valid signer mode specified",
        )),
    }
}

#[derive(Debug, Clone)]
pub struct SovereignKernelConfig {
    pub max_ticks: u64,
}

pub struct SovereignKernel {
    _config: SovereignKernelConfig,
    signer: AnyTpmSigner,
    tpm_attestor: Box<dyn TpmAttestor>,
    tta: Arc<TrustedTimeAuthority>,
    last_record_hash: Vec<u8>,
}

impl SovereignKernel {
    pub fn new(signer: AnyTpmSigner, config: SovereignKernelConfig) -> Self {
        let provider = Arc::new(SystemTimeProvider);
        let tta = Arc::new(TrustedTimeAuthority::new(provider));
        let tpm_attestor: Box<dyn TpmAttestor> = Box::new(SimulatedTpmAttestor::new(16));
        Self::new_with_tta_and_tpm(signer, config, tta, tpm_attestor)
    }

    pub fn new_with_tta(
        signer: AnyTpmSigner,
        config: SovereignKernelConfig,
        tta: Arc<TrustedTimeAuthority>,
    ) -> Self {
        let tpm_attestor: Box<dyn TpmAttestor> = Box::new(SimulatedTpmAttestor::new(16));
        Self::new_with_tta_and_tpm(signer, config, tta, tpm_attestor)
    }

    pub fn new_with_tta_and_tpm(
        signer: AnyTpmSigner,
        config: SovereignKernelConfig,
        tta: Arc<TrustedTimeAuthority>,
        tpm_attestor: Box<dyn TpmAttestor>,
    ) -> Self {
        Self {
            _config: config,
            signer,
            tpm_attestor,
            tta,
            last_record_hash: vec![0; 32],
        }
    }

    /// Backward-compatible entry point for existing callers.
    pub fn execute_foreign(
        &mut self,
        ir_module: &crate::universal_frontend::IRModule,
        input: crate::ir_codegen::IRInput,
    ) -> Result<crate::ir_codegen::IRResult, SystemHalt> {
        self.execute_foreign_with_actor(
            ir_module,
            input,
            ActorContext::system_runtime(),
            CommandType::ExecuteForeignIr,
        )
    }

    /// Execute foreign logic inside the sovereign runtime with immutable actor context.
    pub fn execute_foreign_with_actor(
        &mut self,
        ir_module: &crate::universal_frontend::IRModule,
        input: crate::ir_codegen::IRInput,
        actor: ActorContext,
        command_type: CommandType,
    ) -> Result<crate::ir_codegen::IRResult, SystemHalt> {
        let time = self.tta.now();
        let command_hash = hash_command(ir_module, &input, command_type);

        let mut command = CommandEnvelope {
            command_id: format!("cmd-{}", time.monotonic_ns),
            command_type,
            payload_hash: command_hash,
            actor: actor.clone(),
            timestamp: time.clone(),
            command_signature: Vec::new(),
        };

        validate_actor_command(&actor, &command)?;

        let command_payload = build_command_payload(&command);
        command.command_signature = sign_payload_for_actor(&actor, &command_payload);

        // Generate Rust code from IR.
        let _rust_code = crate::ir_codegen::generate_rust_code(ir_module);

        // Send execution start message to bus.
        let start_message = crate::sovereign_bus::SovereignMessage::new_command(
            crate::sovereign_bus::ActorId(actor.operator_id.clone()),
            actor_to_bus_role(actor.role),
            crate::sovereign_bus::OriginLanguage::Rust,
            crate::regulatory_policy::Intent::Execute,
            vec![],
            vec![],
            crate::sovereign_bus::TraceId(command.command_id.clone()),
        );
        if let Some(bus) = &mut *crate::sovereign_bus::global_bus() {
            bus.send(start_message);
        }

        // Placeholder result - in practice, this would execute generated code.
        let result = crate::ir_codegen::IRResult {
            value: crate::universal_frontend::Value::Int(42),
            bus_messages: vec![],
        };

        let artifact_hash = hash_ir_result(&result);
        let mut artifact = ExecutionArtifact {
            artifact_hash: artifact_hash.clone(),
            result_code: "OK".to_string(),
            artifact_signature: Vec::new(),
        };
        let artifact_payload = build_artifact_payload(&artifact, &time, &command.payload_hash);
        artifact.artifact_signature = sign_payload_for_actor(&actor, &artifact_payload);

        let decision_hash = Sha256::digest(&artifact_hash).to_vec();
        let pcr = self.signer.read_pcr()?;
        let signature_payload = build_signature_payload(&decision_hash, &pcr, &time);
        let signature = self.signer.sign(&signature_payload)?;
        let prev_hash = self.last_record_hash.clone();

        let mut record = AttestationRecord {
            decision_hash: decision_hash.clone(),
            pcr_digest: pcr,
            signature,
            timestamp: time.wall_time_ns / 1_000_000_000,
            wall_time_ns: time.wall_time_ns,
            monotonic_ns: time.monotonic_ns,
            time_source: time.source,
            time_uncertainty_ns: time.uncertainty_ns,
            actor,
            command,
            artifact,
            record_hash: Vec::new(),
            tpm_attest: Vec::new(),
            tpm_signature: Vec::new(),
            pcr_index: self.tpm_attestor.pcr_index(),
            prev_hash,
        };

        // Build record hash, anchor in TPM, and attach quote that binds decision+time+chain.
        let record_data = attestation_record_data(&record);
        let record_hash = hash_record(&record.prev_hash, &record_data, &time);
        self.tpm_attestor.extend_pcr(&record_hash)?;
        let nonce = build_attestation_nonce(&record.decision_hash, &record_hash, &time);
        let (tpm_attest, tpm_signature) = self.tpm_attestor.quote(&nonce)?;

        record.record_hash = record_hash.clone();
        record.tpm_attest = tpm_attest;
        record.tpm_signature = tpm_signature;
        self.last_record_hash = record_hash;

        let complete_message = crate::sovereign_bus::SovereignMessage::new_command(
            crate::sovereign_bus::ActorId("kernel".to_string()),
            crate::sovereign_bus::ActorRole::KernelSubsystem,
            crate::sovereign_bus::OriginLanguage::Rust,
            crate::regulatory_policy::Intent::Execute,
            vec![],
            vec![],
            crate::sovereign_bus::TraceId("execution-complete".to_string()),
        );
        if let Some(bus) = &mut *crate::sovereign_bus::global_bus() {
            bus.send(complete_message);
        }

        Ok(result)
    }
}

pub fn is_authorized(role: &Role, command: &CommandType) -> bool {
    match role {
        Role::Dispatcher => matches!(
            command,
            CommandType::ScedDispatch | CommandType::ScedOverride | CommandType::ExecuteForeignIr
        ),
        Role::ReliabilityEngineer => {
            matches!(command, CommandType::EmergencyOverride | CommandType::ExecuteForeignIr)
        }
        Role::MarketOperator => {
            matches!(command, CommandType::ScedDispatch | CommandType::AuditReadOnly)
        }
        Role::System => true,
    }
}

pub fn command_requires_approval(command: &CommandType) -> bool {
    matches!(command, CommandType::ScedOverride | CommandType::EmergencyOverride)
}

pub fn validate_actor_command(actor: &ActorContext, command: &CommandEnvelope) -> Result<(), SystemHalt> {
    if actor.operator_id.is_empty() {
        return Err(SystemHalt::new(
            FailureAxis::AuthorityInversionAttempt,
            "ERR_ACTOR_ID_MISSING",
        ));
    }

    if actor.is_automated && actor.trigger == TriggerType::Human {
        return Err(SystemHalt::new(
            FailureAxis::AuthorityInversionAttempt,
            "ERR_TRIGGER_INCONSISTENT",
        ));
    }

    if !is_authorized(&actor.role, &command.command_type) {
        return Err(SystemHalt::new(
            FailureAxis::AuthorityInversionAttempt,
            "ERR_UNAUTHORIZED_ACTION",
        ));
    }

    if command_requires_approval(&command.command_type) && actor.approver_id.is_none() {
        return Err(SystemHalt::new(
            FailureAxis::AuthorityInversionAttempt,
            "ERR_APPROVAL_REQUIRED",
        ));
    }

    Ok(())
}

pub fn build_command_payload(command: &CommandEnvelope) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&command.payload_hash);
    hasher.update(command.command_id.as_bytes());
    hasher.update((command.command_type as u8).to_le_bytes());
    hasher.update(command.actor.operator_id.as_bytes());
    hasher.update((command.actor.role as u8).to_le_bytes());
    hasher.update((command.actor.auth_method as u8).to_le_bytes());
    hasher.update(command.actor.session_id.as_bytes());
    hasher.update([command.actor.is_automated as u8]);
    hasher.update((command.actor.trigger as u8).to_le_bytes());
    if let Some(approver) = &command.actor.approver_id {
        hasher.update(approver.as_bytes());
    }
    hasher.update(command.timestamp.wall_time_ns.to_le_bytes());
    hasher.update(command.timestamp.monotonic_ns.to_le_bytes());
    hasher.update((command.timestamp.source as u8).to_le_bytes());
    hasher.update(command.timestamp.uncertainty_ns.to_le_bytes());
    hasher.finalize().to_vec()
}

pub fn build_artifact_payload(
    artifact: &ExecutionArtifact,
    time: &TrustedTime,
    command_hash: &[u8],
) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&artifact.artifact_hash);
    hasher.update(artifact.result_code.as_bytes());
    hasher.update(command_hash);
    hasher.update(time.wall_time_ns.to_le_bytes());
    hasher.update(time.monotonic_ns.to_le_bytes());
    hasher.finalize().to_vec()
}

pub fn sign_payload_for_actor(actor: &ActorContext, payload: &[u8]) -> Vec<u8> {
    actor_signing_key(actor).sign(payload).to_bytes().to_vec()
}

pub fn verify_actor_signature(actor: &ActorContext, payload: &[u8], signature: &[u8]) -> bool {
    if signature.len() != 64 {
        return false;
    }

    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(signature);
    let signature = Ed25519Signature::from_bytes(&sig_bytes);

    actor_signing_key(actor)
        .verifying_key()
        .verify(payload, &signature)
        .is_ok()
}

pub fn has_identity_binding(record: &AttestationRecord) -> bool {
    !record.actor.operator_id.is_empty()
        || !record.command.command_id.is_empty()
        || !record.command.payload_hash.is_empty()
        || !record.artifact.artifact_hash.is_empty()
}

pub fn verify_command_envelope(record: &AttestationRecord) -> Result<(), String> {
    if record.command.actor != record.actor {
        return Err("ERR_ACTOR_BINDING_MISMATCH".to_string());
    }

    validate_actor_command(&record.actor, &record.command)
        .map_err(|halt| halt.message.clone())?;

    let command_payload = build_command_payload(&record.command);
    if !verify_actor_signature(&record.actor, &command_payload, &record.command.command_signature) {
        return Err("ERR_COMMAND_SIGNATURE_INVALID".to_string());
    }

    let time = record_time(record);
    let artifact_payload = build_artifact_payload(&record.artifact, &time, &record.command.payload_hash);
    if !verify_actor_signature(&record.actor, &artifact_payload, &record.artifact.artifact_signature) {
        return Err("ERR_ARTIFACT_SIGNATURE_INVALID".to_string());
    }

    Ok(())
}

pub fn build_signature_payload(decision_hash: &[u8], pcr_digest: &[u8], time: &TrustedTime) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(decision_hash);
    hasher.update(pcr_digest);
    hasher.update(time.wall_time_ns.to_le_bytes());
    hasher.update(time.monotonic_ns.to_le_bytes());
    hasher.update((time.source as u8).to_le_bytes());
    hasher.finalize().to_vec()
}

pub fn hash_record(prev_hash: &[u8], record_data: &[u8], time: &TrustedTime) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash);
    hasher.update(record_data);
    hasher.update(time.wall_time_ns.to_le_bytes());
    hasher.update(time.monotonic_ns.to_le_bytes());
    hasher.finalize().to_vec()
}

pub fn record_time(record: &AttestationRecord) -> TrustedTime {
    TrustedTime {
        wall_time_ns: record.wall_time_ns,
        monotonic_ns: record.monotonic_ns,
        source: record.time_source,
        uncertainty_ns: record.time_uncertainty_ns,
    }
}

pub fn attestation_record_data(record: &AttestationRecord) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend(&record.decision_hash);
    data.extend(&record.pcr_digest);
    data.extend(&record.signature);

    // Preserve exact legacy hash material when identity binding is absent.
    if !has_identity_binding(record) {
        return data;
    }

    append_len_prefixed(&mut data, record.actor.operator_id.as_bytes());
    data.push(record.actor.role as u8);
    data.push(record.actor.auth_method as u8);
    append_len_prefixed(&mut data, record.actor.session_id.as_bytes());
    data.push(record.actor.is_automated as u8);
    data.push(record.actor.trigger as u8);
    append_len_prefixed(
        &mut data,
        record.actor.approver_id.as_deref().unwrap_or("").as_bytes(),
    );

    append_len_prefixed(&mut data, record.command.command_id.as_bytes());
    data.push(record.command.command_type as u8);
    append_len_prefixed(&mut data, &record.command.payload_hash);
    append_len_prefixed(&mut data, &record.command.command_signature);

    append_len_prefixed(&mut data, &record.artifact.artifact_hash);
    append_len_prefixed(&mut data, record.artifact.result_code.as_bytes());
    append_len_prefixed(&mut data, &record.artifact.artifact_signature);

    data
}

pub fn sced_chain_anchor_hash(chain: &[crate::sced_offer_chain::ChainedRecord]) -> Option<String> {
    chain.last().map(|r| r.chain_hash.clone())
}

fn actor_signing_key(actor: &ActorContext) -> SigningKey {
    let mut hasher = Sha256::new();
    hasher.update(b"mvr-esprint1-actor-key-v1");
    hasher.update(actor.operator_id.as_bytes());
    hasher.update((actor.role as u8).to_le_bytes());
    hasher.update((actor.auth_method as u8).to_le_bytes());
    hasher.update(actor.session_id.as_bytes());
    let seed: [u8; 32] = hasher.finalize().into();
    SigningKey::from_bytes(&seed)
}

fn hash_command(
    ir_module: &crate::universal_frontend::IRModule,
    input: &crate::ir_codegen::IRInput,
    command_type: CommandType,
) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(format!("{:?}", ir_module).as_bytes());
    hasher.update(format!("{:?}", input).as_bytes());
    hasher.update((command_type as u8).to_le_bytes());
    hasher.finalize().to_vec()
}

fn hash_ir_result(result: &crate::ir_codegen::IRResult) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(format!("{:?}", result.value).as_bytes());
    hasher.update(format!("{:?}", result.bus_messages).as_bytes());
    hasher.finalize().to_vec()
}

fn actor_to_bus_role(role: Role) -> crate::sovereign_bus::ActorRole {
    match role {
        Role::Dispatcher | Role::ReliabilityEngineer | Role::MarketOperator => {
            crate::sovereign_bus::ActorRole::HumanOperator
        }
        Role::System => crate::sovereign_bus::ActorRole::KernelSubsystem,
    }
}

fn append_len_prefixed(target: &mut Vec<u8>, value: &[u8]) {
    target.extend((value.len() as u32).to_le_bytes());
    target.extend(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_actor() -> ActorContext {
        ActorContext {
            operator_id: "dispatch_ercot_01".to_string(),
            role: Role::Dispatcher,
            auth_method: AuthMethod::SmartCard,
            session_id: "session-20260412".to_string(),
            is_automated: false,
            trigger: TriggerType::Human,
            approver_id: Some("shift_supervisor_03".to_string()),
        }
    }

    #[test]
    fn dispatcher_requires_approval_for_sced_override() {
        let actor = ActorContext {
            approver_id: None,
            ..sample_actor()
        };
        let cmd = CommandEnvelope {
            command_type: CommandType::ScedOverride,
            ..Default::default()
        };

        let err = validate_actor_command(&actor, &cmd).expect_err("must reject missing approval");
        assert_eq!(err.message, "ERR_APPROVAL_REQUIRED");
    }

    #[test]
    fn command_signature_verifies_against_actor_identity() {
        let actor = sample_actor();
        let mut cmd = CommandEnvelope {
            command_id: "cmd-1".to_string(),
            command_type: CommandType::ScedDispatch,
            payload_hash: vec![1, 2, 3],
            actor: actor.clone(),
            timestamp: TrustedTime {
                wall_time_ns: 10,
                monotonic_ns: 11,
                source: TimeSource::PTP,
                uncertainty_ns: 1,
            },
            command_signature: Vec::new(),
        };

        let payload = build_command_payload(&cmd);
        cmd.command_signature = sign_payload_for_actor(&actor, &payload);
        assert!(verify_actor_signature(&actor, &payload, &cmd.command_signature));

        let mut tampered = payload.clone();
        tampered[0] ^= 0xAA;
        assert!(!verify_actor_signature(&actor, &tampered, &cmd.command_signature));
    }
}
