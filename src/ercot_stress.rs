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

use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

/// ERCOT supply and demand telemetry with the most valuable fields for
/// adaptive governance.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SupplyDemandRow {
    #[serde(rename = "TimestampMsUtc")]
    pub timestamp_ms_utc: u64,
    #[serde(rename = "Demand")]
    pub demand_mw: f64,
    #[serde(rename = "Demand Forecast")]
    pub demand_forecast_mw: f64,
    #[serde(rename = "Committed Capacity")]
    pub committed_capacity_mw: f64,
    #[serde(rename = "Available Capacity")]
    pub available_capacity_mw: f64,
    #[serde(rename = "Available Seasonal Capacity")]
    pub available_seasonal_capacity_mw: f64,
}

/// A deterministic snapshot of reserve margin state from the ERCOT supply
/// and demand feed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReserveMarginSnapshot {
    pub demand_mw: f64,
    pub demand_forecast_6d_mw: f64,
    pub committed_capacity_mw: f64,
    pub available_capacity_mw: f64,
    pub available_seasonal_capacity_mw: f64,
    pub timestamp_ms_utc: u64,
}

/// Discrete stress levels for reserve-aware governance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StressState {
    Normal,
    Tight,
    Emergency,
    CollapseRisk,
}

impl SupplyDemandRow {
    pub fn to_snapshot(&self) -> ReserveMarginSnapshot {
        ReserveMarginSnapshot {
            demand_mw: self.demand_mw,
            demand_forecast_6d_mw: self.demand_forecast_mw,
            committed_capacity_mw: self.committed_capacity_mw,
            available_capacity_mw: self.available_capacity_mw,
            available_seasonal_capacity_mw: self.available_seasonal_capacity_mw,
            timestamp_ms_utc: self.timestamp_ms_utc,
        }
    }
}

impl ReserveMarginSnapshot {
    pub fn reserve_margin_mw(&self) -> f64 {
        self.available_capacity_mw - self.demand_mw
    }

    pub fn forecast_reserve_margin_mw(&self) -> f64 {
        self.available_capacity_mw - self.demand_forecast_6d_mw
    }

    pub fn seasonal_reserve_margin_mw(&self) -> f64 {
        self.available_seasonal_capacity_mw - self.demand_mw
    }

    pub fn normalized_stress_level(&self) -> f64 {
        let available = self.available_capacity_mw.max(1.0);
        let margin = self.reserve_margin_mw();

        if margin <= 0.0 {
            1.0
        } else {
            ((available - margin) / available).clamp(0.0, 1.0)
        }
    }

    pub fn stress_state(&self) -> StressState {
        classify_stress(self.reserve_margin_mw(), self.available_capacity_mw)
    }

    pub fn forecast_stress_state(&self) -> StressState {
        classify_stress(self.forecast_reserve_margin_mw(), self.available_capacity_mw)
    }
}

fn classify_stress(reserve_margin_mw: f64, available_capacity_mw: f64) -> StressState {
    let capacity = available_capacity_mw.max(1.0);

    if reserve_margin_mw <= 0.0 {
        return StressState::CollapseRisk;
    }

    let stress_level = ((capacity - reserve_margin_mw) / capacity).clamp(0.0, 1.0);

    if stress_level >= 0.70 {
        StressState::Emergency
    } else if stress_level >= 0.40 {
        StressState::Tight
    } else {
        StressState::Normal
    }
}

/// Demand acceleration computed from two time-ordered snapshots.
pub fn demand_acceleration_mw_per_s(
    previous: &ReserveMarginSnapshot,
    current: &ReserveMarginSnapshot,
) -> Option<f64> {
    let dt_ms = current
        .timestamp_ms_utc
        .saturating_sub(previous.timestamp_ms_utc);
    let dt_s = dt_ms as f64 / 1_000.0;

    if dt_s <= 0.0 {
        return None;
    }

    Some((current.demand_mw - previous.demand_mw) / dt_s)
}

/// Compute an adaptive ramp limit factor based on current stress and demand
/// acceleration. This function makes the control surface explicit for the
/// governor.
pub fn adaptive_ramp_factor(
    stress_state: StressState,
    demand_acceleration_mw_per_s: f64,
) -> f64 {
    let base = match stress_state {
        StressState::Normal => 1.0,
        StressState::Tight => 0.70,
        StressState::Emergency => 0.45,
        StressState::CollapseRisk => 0.20,
    };

    let acceleration_penalty = (demand_acceleration_mw_per_s / 1_000.0).clamp(0.0, 0.25);
    (base - acceleration_penalty).clamp(0.10, 1.0)
}

pub fn load_supply_demand_csv<P: AsRef<Path>>(path: P) -> Result<Vec<SupplyDemandRow>> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    load_supply_demand_csv_from_str(&content)
}

pub fn load_supply_demand_csv_from_str(csv_data: &str) -> Result<Vec<SupplyDemandRow>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_bytes());
    let mut rows = Vec::new();
    for rec in rdr.deserialize() {
        rows.push(rec?);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserve_margin_snapshot_classifies_stress() {
        let snapshot = ReserveMarginSnapshot {
            demand_mw: 900.0,
            demand_forecast_6d_mw: 950.0,
            committed_capacity_mw: 2500.0,
            available_capacity_mw: 1200.0,
            available_seasonal_capacity_mw: 1300.0,
            timestamp_ms_utc: 1_700_000_000_000,
        };

        assert_eq!(snapshot.reserve_margin_mw(), 300.0);
        assert_eq!(snapshot.normalized_stress_level(), 0.75);
        assert_eq!(snapshot.stress_state(), StressState::Emergency);
        assert_eq!(snapshot.forecast_stress_state(), StressState::Emergency);
    }

    #[test]
    fn demand_acceleration_is_computed() {
        let previous = ReserveMarginSnapshot {
            demand_mw: 850.0,
            demand_forecast_6d_mw: 900.0,
            committed_capacity_mw: 2500.0,
            available_capacity_mw: 1200.0,
            available_seasonal_capacity_mw: 1300.0,
            timestamp_ms_utc: 1_700_000_000_000,
        };
        let current = ReserveMarginSnapshot {
            demand_mw: 880.0,
            demand_forecast_6d_mw: 920.0,
            committed_capacity_mw: 2500.0,
            available_capacity_mw: 1200.0,
            available_seasonal_capacity_mw: 1300.0,
            timestamp_ms_utc: 1_700_000_000_500,
        };

        let accel = demand_acceleration_mw_per_s(&previous, &current).unwrap();
        assert!((accel - 60.0).abs() < 1e-9);
    }

    #[test]
    fn adaptive_ramp_factor_tightens_on_acceleration() {
        let factor = adaptive_ramp_factor(StressState::Tight, 300.0);
        assert!((factor - 0.45).abs() < 1e-9);
    }
}
