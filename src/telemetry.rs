// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// Deterministic telemetry validation for ERCOT-style ICCP ingestion.

#![deny(unsafe_code)]

use core::fmt;

pub const SCAN_RATE_SECONDS: u64 = 2;
pub const STALE_THRESHOLD_SECONDS: u64 = 10;
pub const MAX_END_TO_END_LATENCY_SECONDS: u64 = 2;

pub const QUALITY_VALID: u8 = 0x00;
pub const QUALITY_HELD: u8 = 0x01;
pub const QUALITY_SUSPECT: u8 = 0x02;
pub const QUALITY_MANUAL: u8 = 0x04;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelemetryQuality {
    Valid,
    Held,
    Suspect,
    Manual,
    Composite(u8),
}

impl TelemetryQuality {
    pub fn from_mask(mask: u8) -> Self {
        match mask {
            QUALITY_VALID => Self::Valid,
            QUALITY_HELD => Self::Held,
            QUALITY_SUSPECT => Self::Suspect,
            QUALITY_MANUAL => Self::Manual,
            other => Self::Composite(other),
        }
    }

    pub fn is_acceptable(self) -> bool {
        matches!(self, TelemetryQuality::Valid)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TelemetryPoint {
    pub value: f64,
    pub point_timestamp_ms_utc: u64,
    pub quality_mask: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TelemetryValidationConfig {
    pub stale_threshold_seconds: u64,
    pub max_end_to_end_latency_seconds: u64,
}

impl Default for TelemetryValidationConfig {
    fn default() -> Self {
        Self {
            stale_threshold_seconds: STALE_THRESHOLD_SECONDS,
            max_end_to_end_latency_seconds: MAX_END_TO_END_LATENCY_SECONDS,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TelemetryIssue {
    Stale { age_ms: u64, threshold_ms: u64 },
    ExcessiveLatency { latency_ms: u64, limit_ms: u64 },
    BadQuality { quality: TelemetryQuality },
}

impl fmt::Display for TelemetryIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelemetryIssue::Stale {
                age_ms,
                threshold_ms,
            } => write!(f, "stale telemetry: age_ms={age_ms} threshold_ms={threshold_ms}"),
            TelemetryIssue::ExcessiveLatency {
                latency_ms,
                limit_ms,
            } => {
                write!(f, "latency exceeded: latency_ms={latency_ms} limit_ms={limit_ms}")
            }
            TelemetryIssue::BadQuality { quality } => write!(f, "bad quality: {quality:?}"),
        }
    }
}

pub fn validate_point(
    point: &TelemetryPoint,
    ingest_time_ms_utc: u64,
    source_to_ercot_latency_ms: u64,
    cfg: TelemetryValidationConfig,
) -> Vec<TelemetryIssue> {
    let mut issues = Vec::new();

    let age_ms = ingest_time_ms_utc.saturating_sub(point.point_timestamp_ms_utc);
    let stale_ms = cfg.stale_threshold_seconds.saturating_mul(1000);
    if age_ms > stale_ms {
        issues.push(TelemetryIssue::Stale {
            age_ms,
            threshold_ms: stale_ms,
        });
    }

    let latency_limit_ms = cfg.max_end_to_end_latency_seconds.saturating_mul(1000);
    if source_to_ercot_latency_ms > latency_limit_ms {
        issues.push(TelemetryIssue::ExcessiveLatency {
            latency_ms: source_to_ercot_latency_ms,
            limit_ms: latency_limit_ms,
        });
    }

    let quality = TelemetryQuality::from_mask(point.quality_mask);
    if !quality.is_acceptable() {
        issues.push(TelemetryIssue::BadQuality { quality });
    }

    issues
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowDirection {
    OutOfBus,
    IntoBus,
}

pub fn validate_bus_oriented_flow_sign(actual_mw: f64, expected_direction: FlowDirection) -> bool {
    match expected_direction {
        FlowDirection::OutOfBus => actual_mw >= 0.0,
        FlowDirection::IntoBus => actual_mw <= 0.0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakerState {
    Intermediate,
    Closed,
    Open,
    BadState,
}

pub fn decode_double_bit_breaker(bits: u8) -> Option<BreakerState> {
    match bits {
        0b00 => Some(BreakerState::Intermediate),
        0b01 => Some(BreakerState::Closed),
        0b10 => Some(BreakerState::Open),
        0b11 => Some(BreakerState::BadState),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScedRampInputs {
    pub current_output_mw: f64,
    pub normal_ramp_rate_up_mw_per_min: f64,
    pub normal_ramp_rate_down_mw_per_min: f64,
    pub hasl_mw: f64,
    pub lasl_mw: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScedRampOutputs {
    pub suramp_mw_per_min: f64,
    pub sdramp_mw_per_min: f64,
    pub hdl_mw: f64,
    pub ldl_mw: f64,
}

pub fn derive_sced_ramps_and_dispatch_limits(input: ScedRampInputs) -> ScedRampOutputs {
    let suramp = input
        .normal_ramp_rate_up_mw_per_min
        .min((input.hasl_mw - input.current_output_mw).max(0.0) / 5.0);
    let sdramp = input
        .normal_ramp_rate_down_mw_per_min
        .min((input.current_output_mw - input.lasl_mw).max(0.0) / 5.0);

    let hdl = input.current_output_mw + (suramp * 5.0);
    let ldl = input.current_output_mw - (sdramp * 5.0);

    ScedRampOutputs {
        suramp_mw_per_min: suramp,
        sdramp_mw_per_min: sdramp,
        hdl_mw: hdl,
        ldl_mw: ldl,
    }
}

pub fn rrs_thermal_cap_ok(rrs_award_mw: f64, hsl_mw: f64) -> bool {
    rrs_award_mw <= (0.20 * hsl_mw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_passes_when_fresh_low_latency_and_valid_quality() {
        let p = TelemetryPoint {
            value: 100.0,
            point_timestamp_ms_utc: 10_000,
            quality_mask: QUALITY_VALID,
        };
        let issues = validate_point(&p, 18_000, 1_200, TelemetryValidationConfig::default());
        assert!(issues.is_empty());
    }

    #[test]
    fn point_fails_when_stale() {
        let p = TelemetryPoint {
            value: 100.0,
            point_timestamp_ms_utc: 10_000,
            quality_mask: QUALITY_VALID,
        };
        let issues = validate_point(&p, 21_001, 500, TelemetryValidationConfig::default());
        assert!(issues.iter().any(|i| matches!(i, TelemetryIssue::Stale { .. })));
    }

    #[test]
    fn point_fails_when_latency_exceeds_two_seconds() {
        let p = TelemetryPoint {
            value: 100.0,
            point_timestamp_ms_utc: 10_000,
            quality_mask: QUALITY_VALID,
        };
        let issues = validate_point(&p, 11_000, 2_500, TelemetryValidationConfig::default());
        assert!(issues
            .iter()
            .any(|i| matches!(i, TelemetryIssue::ExcessiveLatency { .. })));
    }

    #[test]
    fn point_fails_when_quality_not_valid() {
        let p = TelemetryPoint {
            value: 100.0,
            point_timestamp_ms_utc: 10_000,
            quality_mask: QUALITY_SUSPECT,
        };
        let issues = validate_point(&p, 11_000, 500, TelemetryValidationConfig::default());
        assert!(issues
            .iter()
            .any(|i| matches!(i, TelemetryIssue::BadQuality { .. })));
    }

    #[test]
    fn flow_direction_is_bus_oriented() {
        assert!(validate_bus_oriented_flow_sign(10.0, FlowDirection::OutOfBus));
        assert!(validate_bus_oriented_flow_sign(-10.0, FlowDirection::IntoBus));
        assert!(!validate_bus_oriented_flow_sign(-10.0, FlowDirection::OutOfBus));
    }

    #[test]
    fn double_bit_breaker_states_decode() {
        assert_eq!(decode_double_bit_breaker(0b00), Some(BreakerState::Intermediate));
        assert_eq!(decode_double_bit_breaker(0b01), Some(BreakerState::Closed));
        assert_eq!(decode_double_bit_breaker(0b10), Some(BreakerState::Open));
        assert_eq!(decode_double_bit_breaker(0b11), Some(BreakerState::BadState));
    }

    #[test]
    fn sced_ramp_formulas_follow_ercot_shape() {
        let input = ScedRampInputs {
            current_output_mw: 100.0,
            normal_ramp_rate_up_mw_per_min: 8.0,
            normal_ramp_rate_down_mw_per_min: 6.0,
            hasl_mw: 125.0,
            lasl_mw: 90.0,
        };
        let out = derive_sced_ramps_and_dispatch_limits(input);
        assert_eq!(out.suramp_mw_per_min, 5.0);
        assert_eq!(out.sdramp_mw_per_min, 2.0);
        assert_eq!(out.hdl_mw, 125.0);
        assert_eq!(out.ldl_mw, 90.0);
    }

    #[test]
    fn rrs_thermal_cap_enforced() {
        assert!(rrs_thermal_cap_ok(20.0, 100.0));
        assert!(!rrs_thermal_cap_ok(21.0, 100.0));
    }
}
