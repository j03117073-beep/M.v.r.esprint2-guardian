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
// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// Deterministic BAL/PRC/FAC/CIP control checks for sprint validation.

#![deny(unsafe_code)]

use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum EnforcementCode {
    HaltCip001,
    Err003,
    Err004,
    ErrBal001,
    ErrFac008,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnforcementEvent {
    pub code: EnforcementCode,
    pub message: String,
}

impl EnforcementEvent {
    fn new(code: EnforcementCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AceInput {
    pub tie_line_actual_mw: f64,
    pub tie_line_schedule_mw: f64,
    pub frequency_hz: f64,
    pub nominal_frequency_hz: f64,
    pub b_factor_mw_per_0p1hz: f64,
}

pub fn compute_ace(input: AceInput) -> f64 {
    let tie_error = input.tie_line_actual_mw - input.tie_line_schedule_mw;
    let hz_delta = input.frequency_hz - input.nominal_frequency_hz;
    let freq_component = 10.0 * input.b_factor_mw_per_0p1hz * hz_delta;
    tie_error - freq_component
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrequencyLimits {
    pub l1_hz: f64,
    pub l2_hz: f64,
}

impl Default for FrequencyLimits {
    fn default() -> Self {
        Self {
            l1_hz: 0.05,
            l2_hz: 0.05,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BalOutcome {
    pub ace_mw: f64,
    pub governor_response_required: bool,
}

pub fn evaluate_bal001(input: AceInput, limits: FrequencyLimits) -> Result<BalOutcome, EnforcementEvent> {
    if input.b_factor_mw_per_0p1hz <= 0.0 {
        return Err(EnforcementEvent::new(
            EnforcementCode::ErrBal001,
            "B-factor must be positive and constant for deterministic validation",
        ));
    }
    let ace = compute_ace(input);
    let delta_hz = (input.frequency_hz - input.nominal_frequency_hz).abs();
    let governor_response_required = delta_hz >= limits.l1_hz || delta_hz >= limits.l2_hz;
    Ok(BalOutcome {
        ace_mw: ace,
        governor_response_required,
    })
}

pub fn validate_constant_b_factor(current: f64, previous: Option<f64>, has_change_log_entry: bool) -> Result<(), EnforcementEvent> {
    if let Some(prev) = previous {
        if (current - prev).abs() > f64::EPSILON && !has_change_log_entry {
            return Err(EnforcementEvent::new(
                EnforcementCode::ErrBal001,
                "B-factor changed without timestamped change-log entry",
            ));
        }
    }
    Ok(())
}

pub fn prc001_ufrt_trip_required(frequency_hz: f64, cycles_below_59p4: u32) -> bool {
    frequency_hz <= 59.4 && cycles_below_59p4 > 9
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RideThroughEnvelope {
    pub min_frequency_hz: f64,
    pub max_frequency_hz: f64,
    pub min_voltage_pu: f64,
    pub max_voltage_pu: f64,
}

pub fn prc024_enforce_envelope(
    envelope: RideThroughEnvelope,
    frequency_hz: f64,
    voltage_pu: f64,
    tripped: bool,
    breaker_open_telemetered: bool,
) -> Result<(), EnforcementEvent> {
    let outside = frequency_hz < envelope.min_frequency_hz
        || frequency_hz > envelope.max_frequency_hz
        || voltage_pu < envelope.min_voltage_pu
        || voltage_pu > envelope.max_voltage_pu;

    if outside && !tripped {
        return Err(EnforcementEvent::new(
            EnforcementCode::Err003,
            "outside PRC-024 ride-through envelope without trip",
        ));
    }

    if outside && tripped && !breaker_open_telemetered {
        return Err(EnforcementEvent::new(
            EnforcementCode::Err004,
            "simulated PRC-024 trip diverges from breaker telemetry",
        ));
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FacLineState {
    pub flow_mva: f64,
    pub normal_rating_mva: f64,
    pub emergency_rating_mva: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FacTracker {
    pub over_emergency_minutes: u32,
    pub unresolved_sced_intervals: u32,
}

pub fn evaluate_fac008(state: FacLineState, tracker: FacTracker) -> Result<(), EnforcementEvent> {
    if state.flow_mva > state.emergency_rating_mva && tracker.over_emergency_minutes > 15 {
        return Err(EnforcementEvent::new(
            EnforcementCode::ErrFac008,
            "flow exceeded emergency rating for more than 15 minutes",
        ));
    }

    if state.flow_mva > state.normal_rating_mva && tracker.unresolved_sced_intervals > 3 {
        return Err(EnforcementEvent::new(
            EnforcementCode::ErrFac008,
            "contingency unresolved after 3 SCED intervals",
        ));
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub enum PacketType {
    Iccp,
    NtpOrPtp,
    Other(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboundPacket {
    pub src_mac: String,
    pub dst_port: u16,
    pub packet_type: PacketType,
    pub writes_base_point: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CipPolicy {
    pub allowed_ports: HashSet<u16>,
    pub allowed_macs: HashSet<String>,
}

impl Default for CipPolicy {
    fn default() -> Self {
        Self {
            allowed_ports: HashSet::from([102, 123]),
            allowed_macs: HashSet::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SecurityOutcome {
    pub halt: bool,
    pub freeze_last_known_good_state: bool,
}

pub fn enforce_cip007(packet: &InboundPacket, policy: &CipPolicy) -> Result<SecurityOutcome, EnforcementEvent> {
    if !policy.allowed_ports.contains(&packet.dst_port) {
        return Err(EnforcementEvent::new(
            EnforcementCode::HaltCip001,
            format!("unauthorized destination port {}", packet.dst_port),
        ));
    }

    if !policy.allowed_macs.is_empty() && !policy.allowed_macs.contains(&packet.src_mac) {
        return Err(EnforcementEvent::new(
            EnforcementCode::HaltCip001,
            format!("unauthorized source MAC {}", packet.src_mac),
        ));
    }

    let type_ok = matches!(
        (&packet.packet_type, packet.dst_port),
        (PacketType::Iccp, 102) | (PacketType::NtpOrPtp, 123)
    );
    if !type_ok {
        return Err(EnforcementEvent::new(
            EnforcementCode::HaltCip001,
            "unexpected packet type for allowed service port",
        ));
    }

    if packet.writes_base_point {
        return Err(EnforcementEvent::new(
            EnforcementCode::HaltCip001,
            "unauthorized write attempt to base point memory",
        ));
    }

    Ok(SecurityOutcome {
        halt: false,
        freeze_last_known_good_state: false,
    })
}

pub fn enforce_cip011_topology_integrity(topology_bytes: &[u8], expected_sha256_hex: &str) -> Result<(), EnforcementEvent> {
    let digest = Sha256::digest(topology_bytes);
    let actual = hex::encode(digest);
    if actual != expected_sha256_hex.to_ascii_lowercase() {
        return Err(EnforcementEvent::new(
            EnforcementCode::HaltCip001,
            "topology checksum mismatch at boot (CIP-011 no-go)",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bal_ace_and_governor_trigger() {
        let input = AceInput {
            tie_line_actual_mw: 1_050.0,
            tie_line_schedule_mw: 1_000.0,
            frequency_hz: 59.94,
            nominal_frequency_hz: 60.0,
            b_factor_mw_per_0p1hz: 100.0,
        };
        let out = evaluate_bal001(input, FrequencyLimits::default()).unwrap();
        assert!(out.governor_response_required);
    }

    #[test]
    fn b_factor_change_without_log_hard_fails() {
        let err = validate_constant_b_factor(101.0, Some(100.0), false).unwrap_err();
        assert_eq!(err.code, EnforcementCode::ErrBal001);
    }

    #[test]
    fn prc_ufrt_trip_logic() {
        assert!(prc001_ufrt_trip_required(59.3, 10));
        assert!(!prc001_ufrt_trip_required(59.5, 10));
    }

    #[test]
    fn prc024_err003_without_trip_outside_envelope() {
        let env = RideThroughEnvelope {
            min_frequency_hz: 59.4,
            max_frequency_hz: 60.6,
            min_voltage_pu: 0.88,
            max_voltage_pu: 1.10,
        };
        let err = prc024_enforce_envelope(env, 58.0, 1.0, false, false).unwrap_err();
        assert_eq!(err.code, EnforcementCode::Err003);
    }

    #[test]
    fn prc024_err004_when_trip_not_seen_in_breaker_telemetry() {
        let env = RideThroughEnvelope {
            min_frequency_hz: 59.4,
            max_frequency_hz: 60.6,
            min_voltage_pu: 0.88,
            max_voltage_pu: 1.10,
        };
        let err = prc024_enforce_envelope(env, 58.9, 1.0, true, false).unwrap_err();
        assert_eq!(err.code, EnforcementCode::Err004);
    }

    #[test]
    fn fac008_hard_fail_paths() {
        let s = FacLineState {
            flow_mva: 1_100.0,
            normal_rating_mva: 900.0,
            emergency_rating_mva: 1_000.0,
        };
        let t = FacTracker {
            over_emergency_minutes: 16,
            unresolved_sced_intervals: 1,
        };
        let err = evaluate_fac008(s, t).unwrap_err();
        assert_eq!(err.code, EnforcementCode::ErrFac008);
    }

    #[test]
    fn cip007_default_deny_blocks_unauthorized_port_and_basepoint_write() {
        let policy = CipPolicy::default();
        let packet = InboundPacket {
            src_mac: "aa:bb:cc:dd:ee:ff".to_string(),
            dst_port: 502,
            packet_type: PacketType::Other("modbus".to_string()),
            writes_base_point: true,
        };
        let err = enforce_cip007(&packet, &policy).unwrap_err();
        assert_eq!(err.code, EnforcementCode::HaltCip001);
    }

    #[test]
    fn cip011_topology_checksum_mismatch_hard_fails() {
        let err = enforce_cip011_topology_integrity(b"topology", "deadbeef").unwrap_err();
        assert_eq!(err.code, EnforcementCode::HaltCip001);
    }
}

