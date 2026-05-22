// ERCOT dataset ingestion and canonical telemetry normalization
#![deny(unsafe_code)]

use crate::telemetry::TelemetryPoint;
use chrono::NaiveDateTime;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Deterministic SHA256 hashing for telemetry normalization
fn sha256_normalize(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hex::encode(hasher.finalize())
}

/// Parse ERCOT system-wide demand CSV into canonical TelemetryPoints.
/// Expects CSV with header: "DateTime","Current Forecast","Actual Hourly Avg.",...
pub fn parse_system_wide_demand(path: &str) -> Result<Vec<TelemetryPoint>, String> {
    let file = File::open(path).map_err(|e| format!("open failed: {}", e))?;
    let reader = BufReader::new(file);

    let mut points = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("read failed: {}", e))?;
        if i == 0 {
            continue; // header
        }
        if line.trim().is_empty() {
            continue;
        }
        // Use csv reader per-line to respect quoting; keep reader in scope
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(line.as_bytes());
        if let Some(Ok(record)) = rdr.records().next() {
            let dt = record.get(0).unwrap_or("");
            let actual = record.get(2).unwrap_or("");
            // parse timestamp
            let ts_ms = NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S")
                .map(|d| d.and_utc().timestamp_millis() as u64)
                .unwrap_or(0);

            // parse value (may be empty)
            let value = if actual.trim().is_empty() {
                0.0
            } else {
                actual.trim().parse::<f64>().unwrap_or(0.0)
            };

            points.push(TelemetryPoint {
                value,
                point_timestamp_ms_utc: ts_ms,
                quality_mask: crate::telemetry::QUALITY_VALID,
            });
        }
    }

    Ok(points)
}

/// Canonical telemetry normalization pipeline.
/// Ensures deterministic ordering, provenance tracking, and immutable identity generation.
#[derive(Debug, Clone)]
pub struct CanonicalTelemetryStream {
    pub points: Vec<TelemetryPoint>,
    pub provenance_hash: String,
    pub stream_identity: String,
}

impl CanonicalTelemetryStream {
    /// Normalize raw ERCOT telemetry into a governed telemetry stream.
    /// Guarantees:
    /// - deterministic ordering by timestamp
    /// - immutable provenance lineage
    /// - deterministic stream identity
    pub fn normalize(mut points: Vec<TelemetryPoint>, source_path: &str) -> Self {
        // Sort by timestamp for deterministic ordering
        points.sort_by_key(|p| p.point_timestamp_ms_utc);

        // Compute provenance hash from source path and point count
        let provenance_payload = format!("source={}|count={}|first_ts={}|last_ts={}",
            source_path,
            points.len(),
            points.first().map(|p| p.point_timestamp_ms_utc).unwrap_or(0),
            points.last().map(|p| p.point_timestamp_ms_utc).unwrap_or(0),
        );
        let provenance_hash = sha256_normalize(&provenance_payload);

        // Compute stream identity from all points
        let mut identity_payload = String::new();
        for point in &points {
            identity_payload.push_str(&format!(
                "{:.9}|{}|{};",
                point.value,
                point.point_timestamp_ms_utc,
                point.quality_mask
            ));
        }
        let stream_identity = sha256_normalize(&identity_payload);

        Self {
            points,
            provenance_hash,
            stream_identity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_csv() {
        let path = "Grid and Market Conditions/system-wide-demand.csv";
        let pts = parse_system_wide_demand(path).expect("parse");
        assert!(!pts.is_empty());
    }

    #[test]
    fn normalizes_deterministically() {
        let pts = vec![
            TelemetryPoint { value: 100.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 },
            TelemetryPoint { value: 200.0, point_timestamp_ms_utc: 2000, quality_mask: 0x00 },
        ];
        let stream1 = CanonicalTelemetryStream::normalize(pts.clone(), "test_source");
        let stream2 = CanonicalTelemetryStream::normalize(pts, "test_source");
        
        // Deterministic: same inputs produce identical identity
        assert_eq!(stream1.stream_identity, stream2.stream_identity);
        assert_eq!(stream1.provenance_hash, stream2.provenance_hash);
    }

    #[test]
    fn sorts_by_timestamp() {
        let pts = vec![
            TelemetryPoint { value: 100.0, point_timestamp_ms_utc: 3000, quality_mask: 0x00 },
            TelemetryPoint { value: 200.0, point_timestamp_ms_utc: 1000, quality_mask: 0x00 },
            TelemetryPoint { value: 150.0, point_timestamp_ms_utc: 2000, quality_mask: 0x00 },
        ];
        let stream = CanonicalTelemetryStream::normalize(pts, "test_source");
        
        // Verify sorted by timestamp
        assert_eq!(stream.points[0].point_timestamp_ms_utc, 1000);
        assert_eq!(stream.points[1].point_timestamp_ms_utc, 2000);
        assert_eq!(stream.points[2].point_timestamp_ms_utc, 3000);
    }
}
