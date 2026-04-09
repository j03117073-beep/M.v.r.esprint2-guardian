#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::io::Read;

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowPriceConfig {
    pub parity_tolerance: f64,
    pub congestion_threshold_pct: f64,
    pub min_congestion_shadow_price: f64,
    pub min_halt_shadow_price: f64,
}

impl Default for ShadowPriceConfig {
    fn default() -> Self {
        Self {
            parity_tolerance: 1e-6,
            congestion_threshold_pct: 0.99,
            min_congestion_shadow_price: 0.01,
            min_halt_shadow_price: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowPriceProxyRow {
    pub constraint_id: String,
    pub shadow_price: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowPriceKernelRow {
    pub constraint_id: String,
    pub flow_mw: f64,
    pub limit_mw: f64,
    pub halt_threshold_mw: f64,
    pub shadow_price: f64,
    pub halt_triggered: bool,
    pub battery_energy_available_mw: f64,
    pub battery_ecrs_reserved_mw: f64,
    pub battery_energy_used_mw: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowPriceMismatch {
    pub constraint_id: String,
    pub proxy_shadow_price: f64,
    pub kernel_shadow_price: f64,
    pub abs_error: f64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowPriceReport {
    pub compared: usize,
    pub max_abs_error: f64,
    pub mae: f64,
    pub pass: bool,
    pub mismatches: Vec<ShadowPriceMismatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowPriceCsvError {
    CsvSchemaMismatch,
    CsvMalformed(String),
}

pub fn parse_shadow_proxy_csv<R: Read>(input: R) -> Result<Vec<ShadowPriceProxyRow>, ShadowPriceCsvError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(input);

    let headers = reader
        .headers()
        .map_err(|e| ShadowPriceCsvError::CsvMalformed(e.to_string()))?;
    let found: Vec<String> = headers.iter().map(|h| h.trim().to_string()).collect();
    let expected = vec!["constraint_id".to_string(), "shadow_price".to_string()];
    if found != expected {
        return Err(ShadowPriceCsvError::CsvSchemaMismatch);
    }

    let mut rows = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|e| ShadowPriceCsvError::CsvMalformed(e.to_string()))?;
        if row.len() != 2 {
            return Err(ShadowPriceCsvError::CsvMalformed(
                "row has invalid column count".to_string(),
            ));
        }
        let constraint_id = row[0].trim().to_string();
        let shadow_price = row[1]
            .trim()
            .parse::<f64>()
            .map_err(|_| ShadowPriceCsvError::CsvMalformed("invalid shadow_price".to_string()))?;
        rows.push(ShadowPriceProxyRow {
            constraint_id,
            shadow_price,
        });
    }

    Ok(rows)
}

pub fn verify_shadow_price_parity(
    kernel_rows: &[ShadowPriceKernelRow],
    proxy_rows: &[ShadowPriceProxyRow],
    cfg: &ShadowPriceConfig,
) -> ShadowPriceReport {
    let proxy_map: BTreeMap<String, f64> = proxy_rows
        .iter()
        .map(|r| (r.constraint_id.clone(), r.shadow_price))
        .collect();

    let mut compared = 0usize;
    let mut max_abs_error = 0.0f64;
    let mut abs_error_sum = 0.0f64;
    let mut mismatches = Vec::new();

    for row in kernel_rows {
        let Some(proxy_shadow) = proxy_map.get(&row.constraint_id).copied() else {
            continue;
        };
        compared += 1;

        let abs_error = (row.shadow_price - proxy_shadow).abs();
        abs_error_sum += abs_error;
        if abs_error > max_abs_error {
            max_abs_error = abs_error;
        }
        if abs_error > cfg.parity_tolerance {
            mismatches.push(ShadowPriceMismatch {
                constraint_id: row.constraint_id.clone(),
                proxy_shadow_price: proxy_shadow,
                kernel_shadow_price: row.shadow_price,
                abs_error,
                reason: "parity tolerance exceeded".to_string(),
            });
        }

        // LMP congestion sanity: at >= 99% limit, require a non-trivial shadow price.
        if row.limit_mw > 0.0 && row.flow_mw >= cfg.congestion_threshold_pct * row.limit_mw {
            if row.shadow_price < cfg.min_congestion_shadow_price {
                mismatches.push(ShadowPriceMismatch {
                    constraint_id: row.constraint_id.clone(),
                    proxy_shadow_price: proxy_shadow,
                    kernel_shadow_price: row.shadow_price,
                    abs_error,
                    reason: "congestion threshold crossed without shadow price uplift".to_string(),
                });
            }
        }

        // HALT mapping: if at/over halt threshold or halted, require higher shadow price.
        if row.halt_triggered || row.flow_mw >= row.halt_threshold_mw {
            if row.shadow_price < cfg.min_halt_shadow_price {
                mismatches.push(ShadowPriceMismatch {
                    constraint_id: row.constraint_id.clone(),
                    proxy_shadow_price: proxy_shadow,
                    kernel_shadow_price: row.shadow_price,
                    abs_error,
                    reason: "HALT threshold reached without marginal price escalation".to_string(),
                });
            }
        }

        // Battery ECRS check: energy used must respect reserved ECRS capacity.
        let effective = row.battery_energy_available_mw - row.battery_ecrs_reserved_mw;
        if effective < 0.0 {
            mismatches.push(ShadowPriceMismatch {
                constraint_id: row.constraint_id.clone(),
                proxy_shadow_price: proxy_shadow,
                kernel_shadow_price: row.shadow_price,
                abs_error,
                reason: "battery ECRS reservation exceeds available energy".to_string(),
            });
        } else if row.battery_energy_used_mw > effective + 1e-9 {
            mismatches.push(ShadowPriceMismatch {
                constraint_id: row.constraint_id.clone(),
                proxy_shadow_price: proxy_shadow,
                kernel_shadow_price: row.shadow_price,
                abs_error,
                reason: "battery energy use violates ECRS reservation".to_string(),
            });
        }
    }

    let mae = if compared == 0 {
        0.0
    } else {
        abs_error_sum / compared as f64
    };

    ShadowPriceReport {
        compared,
        max_abs_error,
        mae,
        pass: mismatches.is_empty(),
        mismatches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_parity_and_congestion_checks_pass() {
        let cfg = ShadowPriceConfig::default();
        let proxy = vec![ShadowPriceProxyRow {
            constraint_id: "L1_1500".to_string(),
            shadow_price: 5.0,
        }];
        let kernel = vec![ShadowPriceKernelRow {
            constraint_id: "L1_1500".to_string(),
            flow_mw: 1490.0,
            limit_mw: 1500.0,
            halt_threshold_mw: 1500.0,
            shadow_price: 5.0,
            halt_triggered: false,
            battery_energy_available_mw: 100.0,
            battery_ecrs_reserved_mw: 20.0,
            battery_energy_used_mw: 70.0,
        }];
        let report = verify_shadow_price_parity(&kernel, &proxy, &cfg);
        assert!(report.pass);
    }

    #[test]
    fn flags_halt_threshold_without_price_uplift() {
        let cfg = ShadowPriceConfig {
            min_halt_shadow_price: 10.0,
            ..ShadowPriceConfig::default()
        };
        let proxy = vec![ShadowPriceProxyRow {
            constraint_id: "L1_1500".to_string(),
            shadow_price: 2.0,
        }];
        let kernel = vec![ShadowPriceKernelRow {
            constraint_id: "L1_1500".to_string(),
            flow_mw: 1501.0,
            limit_mw: 1500.0,
            halt_threshold_mw: 1500.0,
            shadow_price: 2.0,
            halt_triggered: true,
            battery_energy_available_mw: 50.0,
            battery_ecrs_reserved_mw: 10.0,
            battery_energy_used_mw: 20.0,
        }];
        let report = verify_shadow_price_parity(&kernel, &proxy, &cfg);
        assert!(!report.pass);
        assert!(report
            .mismatches
            .iter()
            .any(|m| m.reason.contains("HALT threshold")));
    }

    #[test]
    fn flags_battery_ecrs_violation() {
        let cfg = ShadowPriceConfig::default();
        let proxy = vec![ShadowPriceProxyRow {
            constraint_id: "L1_1500".to_string(),
            shadow_price: 1.0,
        }];
        let kernel = vec![ShadowPriceKernelRow {
            constraint_id: "L1_1500".to_string(),
            flow_mw: 1400.0,
            limit_mw: 1500.0,
            halt_threshold_mw: 1500.0,
            shadow_price: 1.0,
            halt_triggered: false,
            battery_energy_available_mw: 30.0,
            battery_ecrs_reserved_mw: 25.0,
            battery_energy_used_mw: 10.0,
        }];
        let report = verify_shadow_price_parity(&kernel, &proxy, &cfg);
        assert!(!report.pass);
        assert!(report
            .mismatches
            .iter()
            .any(|m| m.reason.contains("battery ECRS")));
    }
}

