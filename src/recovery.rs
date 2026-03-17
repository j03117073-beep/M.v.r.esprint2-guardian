//! Sovereign Recovery Protocol: Hysteresis-based restoration from Degraded/Emergency states
//!
//! When the kernel detects a 2026 regulatory violation (TPL-008-1, PRC-029-1, CIP-012-2),
//! it transitions to a safe state (clamping/veto). Once the environment returns to compliance,
//! this module safely restores normal operation using hysteresis to prevent chatter.
//!
//! Restoration Thresholds (Stable Ticks Required):
//! - TPL-008-1 (Thermal): 5,000 ticks (5 sec @ 1 kHz)
//! - PRC-029-1 (Ride-Through): 3,000 ticks (3 sec @ 1 kHz)
//! - CIP-012-2 (Crypto): 100 ticks (100 ms @ 1 kHz)

use crate::regulatory_policy::GovernanceMode;
use std::fmt;

/// Recovery testament: Audit ticket proving state restoration was compliant
#[derive(Debug, Clone)]
pub struct RecoveryTestament {
    pub mandate: String,
    pub violation_tick: u64,
    pub recovery_tick: u64,
    pub stable_ticks: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryStatus {
    Nominal,
    InRecovery,
    Recovered,
}

impl fmt::Display for RecoveryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryStatus::Nominal => write!(f, "Nominal"),
            RecoveryStatus::InRecovery => write!(f, "InRecovery"),
            RecoveryStatus::Recovered => write!(f, "Recovered"),
        }
    }
}

/// Recovery configuration: Threshold setpoints for each 2026 mandate
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub tpl008_thermal_stable_ticks: u64,
    pub prc029_freq_stable_ticks: u64,
    pub cip012_crypto_stable_ticks: u64,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            tpl008_thermal_stable_ticks: 5000,
            prc029_freq_stable_ticks: 3000,
            cip012_crypto_stable_ticks: 100,
        }
    }
}

/// Recovery state tracker
#[derive(Debug, Clone)]
pub struct RecoveryState {
    pub status: RecoveryStatus,
    pub violation_mandate: Option<String>,
    pub stable_tick_count: u64,
    pub entered_at: u64,
}

impl Default for RecoveryState {
    fn default() -> Self {
        Self {
            status: RecoveryStatus::Nominal,
            violation_mandate: None,
            stable_tick_count: 0,
            entered_at: 0,
        }
    }
}

/// Check if environment satisfies 2026 compliance envelopes
pub fn check_2026_envelopes(grid_freq: u8, ambient_temp: f32) -> bool {
    grid_freq >= 59 && grid_freq <= 61 && ambient_temp < 45.0
}

/// Evaluate state recovery (called once per kernel tick)
pub fn evaluate_state_recovery(
    recovery: &mut RecoveryState,
    config: &RecoveryConfig,
    grid_freq: u8,
    ambient_temp: f32,
    timestamp_us: u64,
) -> Option<RecoveryTestament> {
    if recovery.status == RecoveryStatus::Nominal {
        return None;
    }

    if check_2026_envelopes(grid_freq, ambient_temp) {
        recovery.stable_tick_count += 1;

        let threshold = match recovery.violation_mandate.as_deref() {
            Some("TPL-008-1") => config.tpl008_thermal_stable_ticks,
            Some("PRC-029-1") => config.prc029_freq_stable_ticks,
            Some("CIP-012-2") => config.cip012_crypto_stable_ticks,
            _ => 1000,
        };

        if recovery.stable_tick_count >= threshold {
            recovery.status = RecoveryStatus::Recovered;
            return Some(RecoveryTestament {
                mandate: recovery.violation_mandate.clone().unwrap_or_default(),
                violation_tick: recovery.entered_at,
                recovery_tick: timestamp_us,
                stable_ticks: recovery.stable_tick_count,
            });
        }
    } else {
        recovery.stable_tick_count = 0;
    }

    None
}

/// Enter recovery: Mark that a mandate was violated and start stabilization window
pub fn enter_recovery(recovery: &mut RecoveryState, mandate: &str, timestamp_us: u64) {
    recovery.status = RecoveryStatus::InRecovery;
    recovery.violation_mandate = Some(mandate.to_string());
    recovery.stable_tick_count = 0;
    recovery.entered_at = timestamp_us;
}

/// Check if currently in recovery state
pub fn in_recovery(recovery: &RecoveryState) -> bool {
    recovery.status == RecoveryStatus::InRecovery
}

/// Generate narrative recovery report (for audit/compliance documentation)
pub fn recovery_narrative(testament: &RecoveryTestament) -> String {
    format!(
        "Recovery from {}: {} ticks in recovery, stabilized at tick {}",
        testament.mandate, testament.stable_ticks, testament.recovery_tick
    )
}
