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

//! Sovereign Bus — Unified Communication Layer
//!
//! Provides a single, canonical communication channel for ALL actors:
//! humans, AIs, field devices, market interfaces, and kernel subsystems.

use crate::audit_guardian::InvariantId;
use crate::regulatory_policy::Intent;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique identifier for actors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorId(pub String);

/// Role of the actor in the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorRole {
    HumanOperator,
    AiAgent,
    FieldDevice,
    MarketInterface,
    KernelSubsystem,
    ExternalService,
}

/// Origin language of the message
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OriginLanguage {
    Rust,
    Python,
    Javascript,
    CSharp,
    Go,
    HumanInterface,
}

/// Unique identifier for traces
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraceId(pub String);

/// Canonical message type for all communications
#[derive(Debug, Clone)]
pub enum SovereignMessage {
    Command {
        actor_id: ActorId,
        role: ActorRole,
        origin_language: OriginLanguage,
        intent: Intent,
        payload: Vec<u8>,
        invariants_applied: Vec<InvariantId>,
        timestamp: u64,
        trace_parent: TraceId,
    },
    InputEnvelopeReceived(crate::sovereign_trace::InputEnvelope),
}

impl SovereignMessage {
    pub fn new_command(
        actor_id: ActorId,
        role: ActorRole,
        origin_language: OriginLanguage,
        intent: Intent,
        payload: Vec<u8>,
        invariants_applied: Vec<InvariantId>,
        trace_parent: TraceId,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        SovereignMessage::Command {
            actor_id,
            role,
            origin_language,
            intent,
            payload,
            invariants_applied,
            timestamp,
            trace_parent,
        }
    }

    pub fn new_input_envelope(envelope: crate::sovereign_trace::InputEnvelope) -> Self {
        SovereignMessage::InputEnvelopeReceived(envelope)
    }

    /// Compute hash of the message for integrity
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        match self {
            SovereignMessage::Command {
                actor_id,
                role,
                origin_language,
                intent,
                payload,
                timestamp,
                trace_parent,
                ..
            } => {
                hasher.update(actor_id.0.as_bytes());
                hasher.update(format!("{:?}", role).as_bytes());
                hasher.update(format!("{:?}", origin_language).as_bytes());
                hasher.update(format!("{:?}", intent).as_bytes());
                hasher.update(payload);
                hasher.update(timestamp.to_be_bytes());
                hasher.update(trace_parent.0.as_bytes());
            }
            SovereignMessage::InputEnvelopeReceived(envelope) => {
                hasher.update(envelope.actor_id.0.as_bytes());
                hasher.update(format!("{:?}", envelope.role).as_bytes());
                hasher.update(format!("{:?}", envelope.origin_language).as_bytes());
                hasher.update(&envelope.raw_input_bytes);
                hasher.update(&envelope.raw_input_hash);
                hasher.update(envelope.timestamp.to_be_bytes());
                hasher.update(envelope.trace_parent.0.as_bytes());
            }
        }
        format!("{:x}", hasher.finalize())
    }
}

/// The sovereign bus itself
#[derive(Debug)]
pub struct SovereignBus {
    messages: Vec<SovereignMessage>,
}

impl SovereignBus {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Send a message through the bus
    pub fn send(&mut self, message: SovereignMessage) {
        self.messages.push(message);
    }

    /// Receive messages (for subscribers)
    pub fn receive(&self, actor_id: &ActorId) -> Vec<&SovereignMessage> {
        self.messages
            .iter()
            .filter(|msg| {
                match msg {
                    SovereignMessage::Command { actor_id: msg_actor_id, .. } => msg_actor_id == actor_id,
                    SovereignMessage::InputEnvelopeReceived(envelope) => &envelope.actor_id == actor_id,
                }
            })
            .collect()
    }

    /// Get all messages
    pub fn all_messages(&self) -> &[SovereignMessage] {
        &self.messages
    }
}

/// Global bus instance (in production, this would be properly managed)
use std::sync::Mutex;
static GLOBAL_BUS: Mutex<Option<SovereignBus>> = Mutex::new(None);

/// Get reference to global bus
pub fn global_bus() -> std::sync::MutexGuard<'static, Option<SovereignBus>> {
    let mut bus = GLOBAL_BUS.lock().unwrap();
    if bus.is_none() {
        *bus = Some(SovereignBus::new());
    }
    bus
}
