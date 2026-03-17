#![deny(unsafe_code)]

use crate::failure_axis::{FailureAxis, SystemHalt};
use std::fmt;

/// The 2026 Policy Violation Matrix
/// Maps physical events to their regulatory triggers and legal citations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyViolation {
    ThermalDerating,
    FrequencyRideThrough,
    CryptoAuthFailure,
    UnknownCondition,
}

impl PolicyViolation {
    pub fn citation(&self) -> &'static str {
        match self {
            Self::ThermalDerating => "TPL-008-1",
            Self::FrequencyRideThrough => "PRC-029-1",
            Self::CryptoAuthFailure => "CIP-012-2",
            Self::UnknownCondition => "UNKNOWN",
        }
    }
}

/// Full legal citation structure: the "Sovereign Trace" ticket
///
/// This structure encodes the regulatory justification for every kernel action.
/// When an auditor asks "Why did you clamp the output?", the kernel points
/// to the corresponding `LegalCitation` with exact standard, date, and tier.
#[derive(Debug, Clone)]
pub struct LegalCitation {
    pub standard: String,
    pub mandate_date: String,
    pub tier_level: u8,
    pub justification: String,
}

impl fmt::Display for LegalCitation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {}",
            self.standard, self.mandate_date, self.justification
        )
    }
}

impl Default for LegalCitation {
    fn default() -> Self {
        Self {
            standard: "DEFAULT".to_string(),
            mandate_date: "2026-01-01".to_string(),
            tier_level: 1,
            justification: "Default legal citation".to_string(),
        }
    }
}

/// Governance Mode: How strictly setpoints are controlled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceMode {
    Normal,
    Degraded,
    EmergencyRateLimit,
    CompleteVeto,
}

impl fmt::Display for GovernanceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GovernanceMode::Normal => write!(f, "Normal"),
            GovernanceMode::Degraded => write!(f, "Degraded"),
            GovernanceMode::EmergencyRateLimit => write!(f, "EmergencyRateLimit"),
            GovernanceMode::CompleteVeto => write!(f, "CompleteVeto"),
        }
    }
}

/// Policy configuration: Temperature and frequency thresholds
#[derive(Debug, Clone)]
pub struct PolicyConfig {
    pub thermal_threshold_c: f32,
    pub freq_deadband_hz: f32,
    pub min_frequency: u8,
    pub max_frequency: u8,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            thermal_threshold_c: 45.0,
            freq_deadband_hz: 0.036,
            min_frequency: 55,
            max_frequency: 65,
        }
    }
}

/// Check if frequency is in the mandatory ride-through zone
pub fn is_in_ride_through_zone(freq: u8, config: &PolicyConfig) -> bool {
    freq >= 58 && freq <= 62
}

/// Apply thermal derating based on ambient temperature
///
/// Formula:
/// - Below 40°C: full power (limit = 1.0)
/// - 40-45°C: linear derating (1% per degree)
/// - Above 45°C: severe derating (capped at 50%)
pub fn apply_thermal_derating(temp_c: f32, config: &PolicyConfig) -> f32 {
    if temp_c < 40.0 {
        1.0
    } else if temp_c < config.thermal_threshold_c {
        1.0 - ((temp_c - 40.0) * 0.01)
    } else {
        0.5
    }
}
