// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// Deterministic normal/degraded/emergency path matrix for sprint validation.

#![deny(unsafe_code)]

use crate::reliability_controls::{
    evaluate_fac008, prc001_ufrt_trip_required, CipPolicy, EnforcementCode, FacLineState, FacTracker, InboundPacket,
};
use crate::telemetry::{validate_point, TelemetryPoint, TelemetryValidationConfig};

#[derive(Clone, Debug, PartialEq)]
pub enum MatrixCode {
    Err001StaleSubstitute,
    Halt0x0A,
    Halt0x0B,
    Exec0x0C,
}

#[derive(Clone, Debug, PartialEq)]
pub enum KernelLogicAction {
    ExecuteSced,
    BufferAlign,
    UseStateEstimator,
    Prc024Trip,
    KernelPanicHalt,
    PanicDispatch,
    LoadShed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutputState {
    BalancedGridOptimizedCost,
    BalancedGridLatencyMasked,
    ValidatedEstimateWarnIssued,
    ResourceOfflineOrLoadShed,
    SecureLockdown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatrixOutcome {
    pub action: KernelLogicAction,
    pub output_state: OutputState,
    pub code: Option<MatrixCode>,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalPathInput {
    pub sum_generation_mw: f64,
    pub net_interchanges_mw: f64,
    pub load_mw: f64,
    pub losses_mw: f64,
    pub current_output_mw: f64,
    pub lsl_mw: f64,
    pub hsl_mw: f64,
    pub requested_delta_mw: f64,
    pub suramp_mw_limit: f64,
    pub sdramp_mw_limit: f64,
    pub gps_delta_ms: u64,
}

pub fn evaluate_normal_path(input: NormalPathInput) -> Result<MatrixOutcome, MatrixOutcome> {
    if input.gps_delta_ms > 50 {
        return Err(MatrixOutcome {
            action: KernelLogicAction::KernelPanicHalt,
            output_state: OutputState::SecureLockdown,
            code: Some(MatrixCode::Halt0x0A),
            message: "time invariant violated: gps delta exceeded 50ms".to_string(),
        });
    }

    let balance = (input.sum_generation_mw + input.net_interchanges_mw) - (input.load_mw + input.losses_mw);
    if balance.abs() > 0.001 {
        return Err(MatrixOutcome {
            action: KernelLogicAction::KernelPanicHalt,
            output_state: OutputState::SecureLockdown,
            code: None,
            message: "power balance mismatch in steady-state dispatch".to_string(),
        });
    }

    if input.current_output_mw < input.lsl_mw || input.current_output_mw > input.hsl_mw {
        return Err(MatrixOutcome {
            action: KernelLogicAction::KernelPanicHalt,
            output_state: OutputState::SecureLockdown,
            code: None,
            message: "resource output violated LSL/HSL bounds".to_string(),
        });
    }

    if input.requested_delta_mw > input.suramp_mw_limit || -input.requested_delta_mw > input.sdramp_mw_limit {
        return Err(MatrixOutcome {
            action: KernelLogicAction::KernelPanicHalt,
            output_state: OutputState::SecureLockdown,
            code: None,
            message: "physics violation: requested base-point exceeds SURAMP/SDRAMP".to_string(),
        });
    }

    let action = if input.gps_delta_ms <= 2_000 {
        KernelLogicAction::BufferAlign
    } else {
        KernelLogicAction::ExecuteSced
    };

    Ok(MatrixOutcome {
        action,
        output_state: OutputState::BalancedGridLatencyMasked,
        code: None,
        message: "normal path dispatch validated".to_string(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DegradedPathInput {
    pub telemetry: TelemetryPoint,
    pub ingest_time_ms_utc: u64,
    pub source_to_ercot_latency_ms: u64,
    pub last_good_value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DegradedPathResult {
    pub substitute_value_mw: f64,
    pub outcome: MatrixOutcome,
}

pub fn evaluate_degraded_path(input: DegradedPathInput) -> DegradedPathResult {
    let issues = validate_point(
        &input.telemetry,
        input.ingest_time_ms_utc,
        input.source_to_ercot_latency_ms,
        TelemetryValidationConfig::default(),
    );

    if issues.is_empty() {
        return DegradedPathResult {
            substitute_value_mw: input.telemetry.value,
            outcome: MatrixOutcome {
                action: KernelLogicAction::ExecuteSced,
                output_state: OutputState::BalancedGridOptimizedCost,
                code: None,
                message: "telemetry remained valid".to_string(),
            },
        };
    }

    // Deterministic degraded behavior: substitute last-good value and continue loop.
    DegradedPathResult {
        substitute_value_mw: input.last_good_value,
        outcome: MatrixOutcome {
            action: KernelLogicAction::UseStateEstimator,
            output_state: OutputState::ValidatedEstimateWarnIssued,
            code: Some(MatrixCode::Err001StaleSubstitute),
            message: "degraded path using last-good substitute (ERR_001)".to_string(),
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmergencyPhysicsInput {
    pub frequency_hz: f64,
    pub cycles_below_59p4: u32,
    pub line_state: FacLineState,
    pub fac_tracker: FacTracker,
    pub can_cover_with_rrs: bool,
}

pub fn evaluate_emergency_physics(input: EmergencyPhysicsInput) -> Option<MatrixOutcome> {
    if prc001_ufrt_trip_required(input.frequency_hz, input.cycles_below_59p4) {
        let action = if input.can_cover_with_rrs {
            KernelLogicAction::PanicDispatch
        } else {
            KernelLogicAction::LoadShed
        };
        let code = if input.can_cover_with_rrs {
            None
        } else {
            Some(MatrixCode::Exec0x0C)
        };
        return Some(MatrixOutcome {
            action,
            output_state: OutputState::ResourceOfflineOrLoadShed,
            code,
            message: "UFRT threshold exceeded; emergency mitigation required".to_string(),
        });
    }

    if let Err(err) = evaluate_fac008(input.line_state, input.fac_tracker) {
        if err.code == EnforcementCode::ErrFac008 {
            let action = if input.can_cover_with_rrs {
                KernelLogicAction::PanicDispatch
            } else {
                KernelLogicAction::LoadShed
            };
            let code = if input.can_cover_with_rrs {
                None
            } else {
                Some(MatrixCode::Exec0x0C)
            };
            return Some(MatrixOutcome {
                action,
                output_state: OutputState::ResourceOfflineOrLoadShed,
                code,
                message: "FAC-008 emergency thermal violation".to_string(),
            });
        }
    }

    None
}

pub fn evaluate_emergency_cyber(packet: &InboundPacket, policy: &CipPolicy) -> Option<MatrixOutcome> {
    if crate::reliability_controls::enforce_cip007(packet, policy).is_err() {
        return Some(MatrixOutcome {
            action: KernelLogicAction::KernelPanicHalt,
            output_state: OutputState::SecureLockdown,
            code: Some(MatrixCode::Halt0x0B),
            message: "sovereignty breach attempt blocked; fail-safe freeze engaged".to_string(),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reliability_controls::PacketType;
    use crate::telemetry::{QUALITY_HELD, QUALITY_VALID};
    use std::collections::HashSet;

    #[test]
    fn normal_path_valid_case_passes() {
        let input = NormalPathInput {
            sum_generation_mw: 1_100.0,
            net_interchanges_mw: 50.0,
            load_mw: 1_130.0,
            losses_mw: 20.0,
            current_output_mw: 100.0,
            lsl_mw: 80.0,
            hsl_mw: 120.0,
            requested_delta_mw: 5.0,
            suramp_mw_limit: 10.0,
            sdramp_mw_limit: 10.0,
            gps_delta_ms: 10,
        };
        let out = evaluate_normal_path(input).unwrap();
        assert!(matches!(out.action, KernelLogicAction::BufferAlign));
    }

    #[test]
    fn normal_path_time_invariant_violation_halts_0x0a() {
        let input = NormalPathInput {
            sum_generation_mw: 100.0,
            net_interchanges_mw: 0.0,
            load_mw: 100.0,
            losses_mw: 0.0,
            current_output_mw: 100.0,
            lsl_mw: 50.0,
            hsl_mw: 150.0,
            requested_delta_mw: 0.0,
            suramp_mw_limit: 10.0,
            sdramp_mw_limit: 10.0,
            gps_delta_ms: 51,
        };
        let err = evaluate_normal_path(input).unwrap_err();
        assert_eq!(err.code, Some(MatrixCode::Halt0x0A));
    }

    #[test]
    fn degraded_path_uses_last_good_value_and_warns_err001() {
        let degraded = DegradedPathInput {
            telemetry: TelemetryPoint {
                value: 123.0,
                point_timestamp_ms_utc: 1_000,
                quality_mask: QUALITY_HELD,
            },
            ingest_time_ms_utc: 20_000,
            source_to_ercot_latency_ms: 3_000,
            last_good_value: 110.0,
        };
        let out = evaluate_degraded_path(degraded);
        assert_eq!(out.substitute_value_mw, 110.0);
        assert_eq!(out.outcome.code, Some(MatrixCode::Err001StaleSubstitute));
    }

    #[test]
    fn degraded_path_valid_telemetry_executes_normally() {
        let degraded = DegradedPathInput {
            telemetry: TelemetryPoint {
                value: 123.0,
                point_timestamp_ms_utc: 10_000,
                quality_mask: QUALITY_VALID,
            },
            ingest_time_ms_utc: 11_000,
            source_to_ercot_latency_ms: 500,
            last_good_value: 110.0,
        };
        let out = evaluate_degraded_path(degraded);
        assert_eq!(out.substitute_value_mw, 123.0);
        assert_eq!(out.outcome.code, None);
    }

    #[test]
    fn emergency_physics_load_shed_exec_0x0c_when_reserves_exhausted() {
        let input = EmergencyPhysicsInput {
            frequency_hz: 59.3,
            cycles_below_59p4: 12,
            line_state: FacLineState {
                flow_mva: 800.0,
                normal_rating_mva: 900.0,
                emergency_rating_mva: 1_000.0,
            },
            fac_tracker: FacTracker {
                over_emergency_minutes: 0,
                unresolved_sced_intervals: 0,
            },
            can_cover_with_rrs: false,
        };
        let out = evaluate_emergency_physics(input).unwrap();
        assert_eq!(out.code, Some(MatrixCode::Exec0x0C));
        assert!(matches!(out.action, KernelLogicAction::LoadShed));
    }

    #[test]
    fn emergency_cyber_halts_0x0b() {
        let policy = CipPolicy {
            allowed_ports: HashSet::from([102, 123]),
            allowed_macs: HashSet::new(),
        };
        let packet = InboundPacket {
            src_mac: "ff:ff:ff:ff:ff:ff".to_string(),
            dst_port: 102,
            packet_type: PacketType::Iccp,
            writes_base_point: true,
        };
        let out = evaluate_emergency_cyber(&packet, &policy).unwrap();
        assert_eq!(out.code, Some(MatrixCode::Halt0x0B));
    }
}
