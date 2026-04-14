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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CapacityAvailableToScedRow {
    #[serde(rename = "DateTime")]
    pub datetime: String,
    #[serde(rename = "HSL Gen")]
    pub hsl_gen: f64,
    #[serde(rename = "LSL Gen")]
    pub lsl_gen: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapacityAvailableToScedSummary {
    pub row_count: usize,
    pub first_datetime: String,
    pub last_datetime: String,
    pub hsl_gen_min: f64,
    pub hsl_gen_max: f64,
    pub hsl_gen_avg: f64,
    pub lsl_gen_min: f64,
    pub lsl_gen_max: f64,
    pub lsl_gen_avg: f64,
    pub estimated_up_flex_avg_mw: f64,
    pub estimated_down_flex_avg_mw: f64,
}

pub fn load_capacity_available_to_sced_csv<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<CapacityAvailableToScedRow>> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    load_capacity_available_to_sced_csv_from_str(&content)
}

pub fn load_capacity_available_to_sced_csv_from_str(
    csv_data: &str,
) -> Result<Vec<CapacityAvailableToScedRow>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_bytes());
    let mut rows = Vec::new();
    for rec in rdr.deserialize() {
        rows.push(rec?);
    }
    Ok(rows)
}

pub fn summarize_capacity_available_to_sced(
    rows: &[CapacityAvailableToScedRow],
) -> CapacityAvailableToScedSummary {
    let row_count = rows.len();
    let first_datetime = rows
        .first()
        .map(|r| r.datetime.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let last_datetime = rows
        .last()
        .map(|r| r.datetime.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let (mut hsl_min, mut hsl_max, mut hsl_sum) = (f64::INFINITY, f64::NEG_INFINITY, 0.0);
    let (mut lsl_min, mut lsl_max, mut lsl_sum) = (f64::INFINITY, f64::NEG_INFINITY, 0.0);

    for row in rows {
        hsl_min = hsl_min.min(row.hsl_gen);
        hsl_max = hsl_max.max(row.hsl_gen);
        hsl_sum += row.hsl_gen;
        lsl_min = lsl_min.min(row.lsl_gen);
        lsl_max = lsl_max.max(row.lsl_gen);
        lsl_sum += row.lsl_gen;
    }

    let denom = if row_count == 0 { 1.0 } else { row_count as f64 };
    let hsl_avg = hsl_sum / denom;
    let lsl_avg = lsl_sum / denom;

    CapacityAvailableToScedSummary {
        row_count,
        first_datetime,
        last_datetime,
        hsl_gen_min: if hsl_min.is_finite() { hsl_min } else { 0.0 },
        hsl_gen_max: if hsl_max.is_finite() { hsl_max } else { 0.0 },
        hsl_gen_avg: hsl_avg,
        lsl_gen_min: if lsl_min.is_finite() { lsl_min } else { 0.0 },
        lsl_gen_max: if lsl_max.is_finite() { lsl_max } else { 0.0 },
        lsl_gen_avg: lsl_avg,
        // In this feed, HSL represents upward room, LSL (often negative) indicates downward room.
        estimated_up_flex_avg_mw: hsl_avg,
        estimated_down_flex_avg_mw: lsl_avg.abs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_summarizes_capacity_feed() {
        let sample = "DateTime,HSL Gen,LSL Gen\n\
2026-04-05 00:00:05-0500,18552,-40133\n\
2026-04-05 00:00:13-0500,18768,-40152\n";
        let rows = load_capacity_available_to_sced_csv_from_str(sample).expect("parse");
        assert_eq!(rows.len(), 2);
        let s = summarize_capacity_available_to_sced(&rows);
        assert_eq!(s.row_count, 2);
        assert!((s.hsl_gen_avg - 18660.0).abs() < 1e-9);
        assert!((s.estimated_down_flex_avg_mw - 40142.5).abs() < 1e-9);
    }
}

