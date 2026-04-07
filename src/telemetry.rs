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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelemetryClass {
    Scada,
    Pmu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TelemetryTimestamp {
    pub source_timestamp_ms_utc: u64,
    pub arrival_timestamp_ms_utc: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotAlignmentConfig {
    pub scada_skew_window_ms: u64,
    pub pmu_skew_window_ms: u64,
    pub scada_snapshot_width_ms: u64,
}

impl Default for SnapshotAlignmentConfig {
    fn default() -> Self {
        Self {
            scada_skew_window_ms: 2_000,
            pmu_skew_window_ms: 1,
            scada_snapshot_width_ms: 2_000,
        }
    }
}

pub fn effective_timestamp_ms(class: TelemetryClass, ts: TelemetryTimestamp) -> u64 {
    match class {
        TelemetryClass::Scada => ts.arrival_timestamp_ms_utc,
        TelemetryClass::Pmu => ts.source_timestamp_ms_utc,
    }
}

pub fn within_snapshot_skew_window(
    class: TelemetryClass,
    a: TelemetryTimestamp,
    b: TelemetryTimestamp,
    cfg: SnapshotAlignmentConfig,
) -> bool {
    let at = effective_timestamp_ms(class, a) as i128;
    let bt = effective_timestamp_ms(class, b) as i128;
    let skew = (at - bt).unsigned_abs() as u64;
    let limit = match class {
        TelemetryClass::Scada => cfg.scada_skew_window_ms,
        TelemetryClass::Pmu => cfg.pmu_skew_window_ms,
    };
    skew <= limit
}

pub fn scada_snapshot_bucket_start_ms(arrival_timestamp_ms_utc: u64, cfg: SnapshotAlignmentConfig) -> u64 {
    let width = cfg.scada_snapshot_width_ms.max(1);
    (arrival_timestamp_ms_utc / width) * width
}

#[derive(Clone, Debug, PartialEq)]
pub struct DispatchTieOffer {
    pub resource_id: String,
    pub offer_price_per_mwh: f64,
    pub available_mw: f64,
    pub is_online: bool,
    pub low_sustained_limit_mw: f64,
    pub mitigated_offer_cap_per_mwh: Option<f64>,
    pub reserve_locked_mw: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DispatchAllocation {
    pub resource_id: String,
    pub mw: f64,
}

fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

pub fn effective_offer_price(offer: &DispatchTieOffer) -> f64 {
    match offer.mitigated_offer_cap_per_mwh {
        Some(moc) => offer.offer_price_per_mwh.min(moc),
        None => offer.offer_price_per_mwh,
    }
}

pub fn apply_deterministic_epsilon(order: &mut [DispatchTieOffer], epsilon: f64) -> Vec<(String, f64)> {
    order.sort_by(|a, b| a.resource_id.cmp(&b.resource_id));
    order
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let adjusted = effective_offer_price(o) + (epsilon * i as f64);
            (o.resource_id.clone(), round6(adjusted))
        })
        .collect()
}

pub fn allocate_pro_rata_dispatch(required_mw: f64, offers: &[DispatchTieOffer]) -> Vec<DispatchAllocation> {
    if required_mw <= 0.0 || offers.is_empty() {
        return Vec::new();
    }

    let running: Vec<&DispatchTieOffer> = offers
        .iter()
        .filter(|o| o.is_online && o.available_mw > 0.0)
        .collect();

    let mut eligible: Vec<&DispatchTieOffer> = offers.iter().filter(|o| o.available_mw > 0.0).collect();
    let running_total: f64 = running.iter().map(|o| o.available_mw).sum();

    // QSGR-style safeguard: avoid forcing offline starts for tiny requirements that sit below LSL.
    if !running.is_empty()
        && running_total >= required_mw
        && offers
            .iter()
            .any(|o| !o.is_online && required_mw < o.low_sustained_limit_mw)
    {
        eligible = running;
    }

    // RTC tie behavior: keep reserve-capable units available by reducing energy room.
    let weights: Vec<(String, f64)> = eligible
        .iter()
        .map(|o| {
            let energy_room = (o.available_mw - o.reserve_locked_mw).max(0.0);
            (o.resource_id.clone(), energy_room)
        })
        .collect();

    let total_weight: f64 = weights.iter().map(|(_, w)| *w).sum();
    if total_weight <= 0.0 {
        return Vec::new();
    }

    let mut allocations: Vec<DispatchAllocation> = weights
        .iter()
        .map(|(id, w)| DispatchAllocation {
            resource_id: id.clone(),
            mw: round6(required_mw * (*w / total_weight)),
        })
        .collect();

    // Deterministic residue assignment by lexical resource ID.
    let allocated: f64 = allocations.iter().map(|a| a.mw).sum();
    let mut residue = round6(required_mw - allocated);
    allocations.sort_by(|a, b| a.resource_id.cmp(&b.resource_id));
    for a in &mut allocations {
        if residue <= 0.0 {
            break;
        }
        let bump = residue.min(0.000001);
        a.mw = round6(a.mw + bump);
        residue = round6(residue - bump);
    }

    allocations
}

pub fn allocate_pro_rata_curtailment(required_curtailment_mw: f64, offers: &[DispatchTieOffer]) -> Vec<DispatchAllocation> {
    allocate_pro_rata_dispatch(required_curtailment_mw.max(0.0), offers)
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

    #[test]
    fn scada_uses_arrival_time_pmu_uses_source_time() {
        let ts = TelemetryTimestamp {
            source_timestamp_ms_utc: 100,
            arrival_timestamp_ms_utc: 2_000,
        };
        assert_eq!(effective_timestamp_ms(TelemetryClass::Scada, ts), 2_000);
        assert_eq!(effective_timestamp_ms(TelemetryClass::Pmu, ts), 100);
    }

    #[test]
    fn snapshot_skew_windows_follow_scada_and_pmu_assumptions() {
        let cfg = SnapshotAlignmentConfig::default();
        let a = TelemetryTimestamp {
            source_timestamp_ms_utc: 1_000,
            arrival_timestamp_ms_utc: 10_000,
        };
        let b = TelemetryTimestamp {
            source_timestamp_ms_utc: 1_001,
            arrival_timestamp_ms_utc: 11_999,
        };
        assert!(within_snapshot_skew_window(TelemetryClass::Scada, a, b, cfg));
        assert!(within_snapshot_skew_window(TelemetryClass::Pmu, a, b, cfg));
    }

    #[test]
    fn scada_snapshot_bucket_is_two_seconds() {
        let cfg = SnapshotAlignmentConfig::default();
        assert_eq!(scada_snapshot_bucket_start_ms(4_001, cfg), 4_000);
        assert_eq!(scada_snapshot_bucket_start_ms(5_999, cfg), 4_000);
    }

    #[test]
    fn mitigated_offer_cap_and_epsilon_are_deterministic() {
        let mut offers = vec![
            DispatchTieOffer {
                resource_id: "UNIT_B".to_string(),
                offer_price_per_mwh: 30.0,
                available_mw: 20.0,
                is_online: true,
                low_sustained_limit_mw: 10.0,
                mitigated_offer_cap_per_mwh: Some(25.0),
                reserve_locked_mw: 0.0,
            },
            DispatchTieOffer {
                resource_id: "UNIT_A".to_string(),
                offer_price_per_mwh: 30.0,
                available_mw: 20.0,
                is_online: true,
                low_sustained_limit_mw: 10.0,
                mitigated_offer_cap_per_mwh: Some(25.0),
                reserve_locked_mw: 0.0,
            },
        ];
        let adjusted = apply_deterministic_epsilon(&mut offers, 0.0001);
        assert_eq!(adjusted[0], ("UNIT_A".to_string(), 25.0));
        assert_eq!(adjusted[1], ("UNIT_B".to_string(), 25.0001));
    }

    #[test]
    fn pro_rata_dispatch_allocates_by_available_room() {
        let offers = vec![
            DispatchTieOffer {
                resource_id: "U1".to_string(),
                offer_price_per_mwh: 20.0,
                available_mw: 20.0,
                is_online: true,
                low_sustained_limit_mw: 10.0,
                mitigated_offer_cap_per_mwh: None,
                reserve_locked_mw: 0.0,
            },
            DispatchTieOffer {
                resource_id: "U2".to_string(),
                offer_price_per_mwh: 20.0,
                available_mw: 10.0,
                is_online: true,
                low_sustained_limit_mw: 10.0,
                mitigated_offer_cap_per_mwh: None,
                reserve_locked_mw: 0.0,
            },
        ];
        let out = allocate_pro_rata_dispatch(15.0, &offers);
        assert_eq!(out.len(), 2);
        let u1 = out.iter().find(|a| a.resource_id == "U1").unwrap().mw;
        let u2 = out.iter().find(|a| a.resource_id == "U2").unwrap().mw;
        assert_eq!(round6(u1), 10.0);
        assert_eq!(round6(u2), 5.0);
    }

    #[test]
    fn qsgr_guard_avoids_offline_start_for_tiny_requirement() {
        let offers = vec![
            DispatchTieOffer {
                resource_id: "ONLINE".to_string(),
                offer_price_per_mwh: 20.0,
                available_mw: 10.0,
                is_online: true,
                low_sustained_limit_mw: 5.0,
                mitigated_offer_cap_per_mwh: None,
                reserve_locked_mw: 0.0,
            },
            DispatchTieOffer {
                resource_id: "OFFLINE_QSGR".to_string(),
                offer_price_per_mwh: 20.0,
                available_mw: 100.0,
                is_online: false,
                low_sustained_limit_mw: 50.0,
                mitigated_offer_cap_per_mwh: None,
                reserve_locked_mw: 0.0,
            },
        ];
        let out = allocate_pro_rata_dispatch(1.0, &offers);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].resource_id, "ONLINE");
    }

    #[test]
    fn reserve_locked_room_changes_energy_tie_break_in_rtc_style() {
        let offers = vec![
            DispatchTieOffer {
                resource_id: "RESERVE_HEAVY".to_string(),
                offer_price_per_mwh: 20.0,
                available_mw: 20.0,
                is_online: true,
                low_sustained_limit_mw: 5.0,
                mitigated_offer_cap_per_mwh: None,
                reserve_locked_mw: 10.0,
            },
            DispatchTieOffer {
                resource_id: "ENERGY_FREE".to_string(),
                offer_price_per_mwh: 20.0,
                available_mw: 20.0,
                is_online: true,
                low_sustained_limit_mw: 5.0,
                mitigated_offer_cap_per_mwh: None,
                reserve_locked_mw: 0.0,
            },
        ];
        let out = allocate_pro_rata_dispatch(15.0, &offers);
        let heavy = out
            .iter()
            .find(|a| a.resource_id == "RESERVE_HEAVY")
            .unwrap()
            .mw;
        let free = out
            .iter()
            .find(|a| a.resource_id == "ENERGY_FREE")
            .unwrap()
            .mw;
        assert!(free > heavy);
    }
}
