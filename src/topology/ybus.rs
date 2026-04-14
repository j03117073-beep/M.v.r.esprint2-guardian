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
#![deny(unsafe_code)]

use crate::ingest::rdf_parser::{EquipmentKind, FixedDec9};
use crate::topology::graph_builder::{BranchEdge, TopologyGraph};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

#[derive(Debug, Clone, PartialEq)]
pub struct YbusConfig {
    pub zero_impedance_epsilon: f64,
    pub zib_penalty_conductance: f64,
}

impl Default for YbusConfig {
    fn default() -> Self {
        Self {
            zero_impedance_epsilon: 1e-12,
            zib_penalty_conductance: 1_000_000.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SparseYbusEntry {
    pub row_bus: String,
    pub col_bus: String,
    pub g_pu: f64,
    pub b_pu: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SparseYbus {
    pub bus_order: Vec<String>,
    pub entries: Vec<SparseYbusEntry>,
    pub zib_penalty_branches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YbusMismatch {
    pub row_bus: String,
    pub col_bus: String,
    pub expected_g_pu: f64,
    pub expected_b_pu: f64,
    pub actual_g_pu: f64,
    pub actual_b_pu: f64,
    pub abs_error: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParityDiffReport {
    pub tolerance: f64,
    pub compared_entries: usize,
    pub max_abs_error: f64,
    pub mae: f64,
    pub pass: bool,
    pub mismatches: Vec<YbusMismatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementProxyRow {
    pub element_id: String,
    pub bus_a: String,
    pub bus_b: String,
    pub g_pu: f64,
    pub b_pu: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SparseYbusCsvError {
    CsvSchemaMismatch,
    CsvMalformed(String),
}

pub fn build_sparse_ybus(graph: &TopologyGraph, cfg: &YbusConfig) -> SparseYbus {
    let bus_order: Vec<String> = graph.buses.iter().map(|b| b.bus_id.clone()).collect();
    let mut acc: BTreeMap<(String, String), Cx> = BTreeMap::new();
    let mut zib_penalty_branches = Vec::new();

    for br in &graph.branches {
        stamp_branch(br, cfg, &mut acc, &mut zib_penalty_branches);
    }

    let entries = acc
        .into_iter()
        .filter(|(_, v)| v.re.abs() > 0.0 || v.im.abs() > 0.0)
        .map(|((row_bus, col_bus), v)| SparseYbusEntry {
            row_bus,
            col_bus,
            g_pu: v.re,
            b_pu: v.im,
        })
        .collect();

    SparseYbus {
        bus_order,
        entries,
        zib_penalty_branches,
    }
}

pub fn compare_sparse_ybus(
    expected: &SparseYbus,
    actual: &SparseYbus,
    tolerance: f64,
) -> ParityDiffReport {
    let expected_map = to_map(expected);
    let actual_map = to_map(actual);

    let mut keys = BTreeSet::new();
    keys.extend(expected_map.keys().cloned());
    keys.extend(actual_map.keys().cloned());

    let mut compared_entries = 0usize;
    let mut max_abs_error = 0.0f64;
    let mut abs_error_sum = 0.0f64;
    let mut mismatches = Vec::new();

    for (row_bus, col_bus) in keys {
        let ev = expected_map
            .get(&(row_bus.clone(), col_bus.clone()))
            .copied()
            .unwrap_or_default();
        let av = actual_map
            .get(&(row_bus.clone(), col_bus.clone()))
            .copied()
            .unwrap_or_default();

        let err = (ev.re - av.re).abs().max((ev.im - av.im).abs());
        compared_entries += 1;
        abs_error_sum += err;
        if err > max_abs_error {
            max_abs_error = err;
        }
        if err > tolerance {
            mismatches.push(YbusMismatch {
                row_bus,
                col_bus,
                expected_g_pu: ev.re,
                expected_b_pu: ev.im,
                actual_g_pu: av.re,
                actual_b_pu: av.im,
                abs_error: err,
            });
        }
    }

    let mae = if compared_entries == 0 {
        0.0
    } else {
        abs_error_sum / compared_entries as f64
    };

    ParityDiffReport {
        tolerance,
        compared_entries,
        max_abs_error,
        mae,
        pass: mismatches.is_empty(),
        mismatches,
    }
}

pub fn parse_sparse_ybus_csv<R: Read>(input: R) -> Result<SparseYbus, SparseYbusCsvError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(input);

    let headers = reader
        .headers()
        .map_err(|e| SparseYbusCsvError::CsvMalformed(e.to_string()))?;
    let found: Vec<String> = headers.iter().map(|h| h.trim().to_string()).collect();
    let expected = vec![
        "row_bus".to_string(),
        "col_bus".to_string(),
        "g_pu".to_string(),
        "b_pu".to_string(),
    ];
    if found != expected {
        return Err(SparseYbusCsvError::CsvSchemaMismatch);
    }

    let mut entries = Vec::new();
    let mut buses = BTreeSet::new();
    for row in reader.records() {
        let row = row.map_err(|e| SparseYbusCsvError::CsvMalformed(e.to_string()))?;
        if row.len() != 4 {
            return Err(SparseYbusCsvError::CsvMalformed(
                "row has invalid column count".to_string(),
            ));
        }
        let row_bus = row[0].trim().to_string();
        let col_bus = row[1].trim().to_string();
        let g_pu = row[2]
            .trim()
            .parse::<f64>()
            .map_err(|_| SparseYbusCsvError::CsvMalformed("invalid g_pu".to_string()))?;
        let b_pu = row[3]
            .trim()
            .parse::<f64>()
            .map_err(|_| SparseYbusCsvError::CsvMalformed("invalid b_pu".to_string()))?;

        buses.insert(row_bus.clone());
        buses.insert(col_bus.clone());
        entries.push(SparseYbusEntry {
            row_bus,
            col_bus,
            g_pu,
            b_pu,
        });
    }

    Ok(SparseYbus {
        bus_order: buses.into_iter().collect(),
        entries,
        zib_penalty_branches: Vec::new(),
    })
}

pub fn compare_element_proxy_rows(
    expected: &[ElementProxyRow],
    actual: &[ElementProxyRow],
    tolerance: f64,
) -> ParityDiffReport {
    let mut expected_map = BTreeMap::new();
    for r in expected {
        expected_map.insert(
            (r.element_id.clone(), r.bus_a.clone(), r.bus_b.clone()),
            (r.g_pu, r.b_pu),
        );
    }
    let mut actual_map = BTreeMap::new();
    for r in actual {
        actual_map.insert(
            (r.element_id.clone(), r.bus_a.clone(), r.bus_b.clone()),
            (r.g_pu, r.b_pu),
        );
    }

    let mut keys = BTreeSet::new();
    keys.extend(expected_map.keys().cloned());
    keys.extend(actual_map.keys().cloned());

    let mut compared_entries = 0usize;
    let mut max_abs_error = 0.0f64;
    let mut abs_error_sum = 0.0f64;
    let mut mismatches = Vec::new();

    for (element_id, bus_a, bus_b) in keys {
        let (eg, eb) = expected_map
            .get(&(element_id.clone(), bus_a.clone(), bus_b.clone()))
            .copied()
            .unwrap_or((0.0, 0.0));
        let (ag, ab) = actual_map
            .get(&(element_id.clone(), bus_a.clone(), bus_b.clone()))
            .copied()
            .unwrap_or((0.0, 0.0));

        let err = (eg - ag).abs().max((eb - ab).abs());
        compared_entries += 1;
        abs_error_sum += err;
        if err > max_abs_error {
            max_abs_error = err;
        }
        if err > tolerance {
            mismatches.push(YbusMismatch {
                row_bus: format!("{element_id}:{bus_a}"),
                col_bus: bus_b.clone(),
                expected_g_pu: eg,
                expected_b_pu: eb,
                actual_g_pu: ag,
                actual_b_pu: ab,
                abs_error: err,
            });
        }
    }

    let mae = if compared_entries == 0 {
        0.0
    } else {
        abs_error_sum / compared_entries as f64
    };

    ParityDiffReport {
        tolerance,
        compared_entries,
        max_abs_error,
        mae,
        pass: mismatches.is_empty(),
        mismatches,
    }
}

/// Deterministic decision hash for RI_04 Ybus outputs.
///
/// This hash is stable across platforms and is intended to seed downstream
/// cryptographic audit chains (for example RI_18 shadow-price chaining).
pub fn ybus_decision_hash(ybus: &SparseYbus) -> Vec<u8> {
    let mut entries = ybus.entries.clone();
    entries.sort_by(|a, b| {
        a.row_bus
            .cmp(&b.row_bus)
            .then_with(|| a.col_bus.cmp(&b.col_bus))
    });

    let mut hasher = Sha256::new();
    hasher.update(b"RI04_YBUS_DECISION_V1");

    for bus in &ybus.bus_order {
        hasher.update((bus.len() as u32).to_le_bytes());
        hasher.update(bus.as_bytes());
    }

    for e in entries {
        hasher.update((e.row_bus.len() as u32).to_le_bytes());
        hasher.update(e.row_bus.as_bytes());
        hasher.update((e.col_bus.len() as u32).to_le_bytes());
        hasher.update(e.col_bus.as_bytes());
        hasher.update(format!("{:.12}", e.g_pu).as_bytes());
        hasher.update(format!("{:.12}", e.b_pu).as_bytes());
    }

    for zib in &ybus.zib_penalty_branches {
        hasher.update((zib.len() as u32).to_le_bytes());
        hasher.update(zib.as_bytes());
    }

    hasher.finalize().to_vec()
}

pub fn derive_branch_series_row(branch: &BranchEdge, cfg: &YbusConfig) -> ElementProxyRow {
    let r = branch.r.as_ref().map(fixed_to_f64).unwrap_or(0.0);
    let x = branch.x.as_ref().map(fixed_to_f64).unwrap_or(0.0);
    let z_mag_sq = r * r + x * x;
    let (g, b) = if z_mag_sq <= cfg.zero_impedance_epsilon {
        (cfg.zib_penalty_conductance, 0.0)
    } else {
        (r / z_mag_sq, -x / z_mag_sq)
    };
    ElementProxyRow {
        element_id: branch.equipment_id.clone(),
        bus_a: branch.from_bus.clone(),
        bus_b: branch.to_bus.clone(),
        g_pu: g,
        b_pu: b,
    }
}

pub fn derive_shunt_row(element_id: &str, bus: &str, b_pu: f64) -> ElementProxyRow {
    ElementProxyRow {
        element_id: element_id.to_string(),
        bus_a: bus.to_string(),
        bus_b: "N/A".to_string(),
        g_pu: 0.0,
        b_pu,
    }
}

pub fn derive_switch_penalty_row(element_id: &str, bus_a: &str, bus_b: &str, cfg: &YbusConfig) -> ElementProxyRow {
    ElementProxyRow {
        element_id: element_id.to_string(),
        bus_a: bus_a.to_string(),
        bus_b: bus_b.to_string(),
        g_pu: cfg.zib_penalty_conductance,
        b_pu: 0.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Cx {
    re: f64,
    im: f64,
}

impl Cx {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
}

fn to_map(s: &SparseYbus) -> BTreeMap<(String, String), Cx> {
    let mut map = BTreeMap::new();
    for e in &s.entries {
        map.insert(
            (e.row_bus.clone(), e.col_bus.clone()),
            Cx {
                re: e.g_pu,
                im: e.b_pu,
            },
        );
    }
    map
}

fn stamp_branch(
    br: &BranchEdge,
    cfg: &YbusConfig,
    acc: &mut BTreeMap<(String, String), Cx>,
    zib_penalty_branches: &mut Vec<String>,
) {
    if br.from_bus == br.to_bus {
        return;
    }

    let r = br.r.as_ref().map(fixed_to_f64).unwrap_or(0.0);
    let x = br.x.as_ref().map(fixed_to_f64).unwrap_or(0.0);
    let bch = br.bch.as_ref().map(fixed_to_f64).unwrap_or(0.0);
    let z_mag_sq = r * r + x * x;

    let y_series = if z_mag_sq <= cfg.zero_impedance_epsilon {
        zib_penalty_branches.push(br.equipment_id.clone());
        Cx::new(cfg.zib_penalty_conductance, 0.0)
    } else {
        Cx::new(r / z_mag_sq, -x / z_mag_sq)
    };

    // Half-line charging on both terminal diagonals.
    let y_shunt_half = Cx::new(0.0, bch / 2.0);

    add(acc, &br.from_bus, &br.from_bus, y_series);
    add(acc, &br.to_bus, &br.to_bus, y_series);
    add(acc, &br.from_bus, &br.from_bus, y_shunt_half);
    add(acc, &br.to_bus, &br.to_bus, y_shunt_half);

    add(
        acc,
        &br.from_bus,
        &br.to_bus,
        Cx::new(-y_series.re, -y_series.im),
    );
    add(
        acc,
        &br.to_bus,
        &br.from_bus,
        Cx::new(-y_series.re, -y_series.im),
    );
}

fn add(acc: &mut BTreeMap<(String, String), Cx>, row: &str, col: &str, value: Cx) {
    let key = (row.to_string(), col.to_string());
    let entry = acc.entry(key).or_default();
    entry.re += value.re;
    entry.im += value.im;
}

fn fixed_to_f64(v: &FixedDec9) -> f64 {
    (v.0 as f64) / (FixedDec9::SCALE as f64)
}

#[allow(dead_code)]
fn is_network_branch(kind: EquipmentKind) -> bool {
    matches!(
        kind,
        EquipmentKind::AcLineSegment | EquipmentKind::PowerTransformer | EquipmentKind::SeriesCompensator
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::rdf_parser::EquipmentKind;
    use crate::topology::graph_builder::{BranchEdge, ElectricalBus, TopologyGraph};

    fn fx(v: f64) -> FixedDec9 {
        FixedDec9::from_str(&format!("{v:.9}")).expect("fixed")
    }

    fn one_branch_graph(
        from_bus: &str,
        to_bus: &str,
        r: f64,
        x: f64,
        bch: Option<f64>,
    ) -> TopologyGraph {
        TopologyGraph {
            buses: vec![
                ElectricalBus {
                    bus_id: from_bus.to_string(),
                    connectivity_nodes: vec!["A".to_string()],
                },
                ElectricalBus {
                    bus_id: to_bus.to_string(),
                    connectivity_nodes: vec!["B".to_string()],
                },
            ],
            branches: vec![BranchEdge {
                equipment_id: "L1".to_string(),
                kind: EquipmentKind::AcLineSegment,
                from_bus: from_bus.to_string(),
                to_bus: to_bus.to_string(),
                r: Some(fx(r)),
                x: Some(fx(x)),
                bch: bch.map(fx),
            }],
            divergences: Vec::new(),
        }
    }

    #[test]
    fn stamps_series_and_half_shunt() {
        let graph = one_branch_graph("BUS::A", "BUS::B", 0.01, 0.1, Some(0.02));
        let y = build_sparse_ybus(&graph, &YbusConfig::default());
        let map = to_map(&y);

        // y = 1/(0.01+j0.1) = 0.9900990099 - j9.900990099
        // diagonal imag includes +bch/2 = +0.01
        let aa = map
            .get(&("BUS::A".to_string(), "BUS::A".to_string()))
            .copied()
            .expect("AA");
        let ab = map
            .get(&("BUS::A".to_string(), "BUS::B".to_string()))
            .copied()
            .expect("AB");

        assert!((aa.re - 0.9900990099).abs() < 1e-9);
        assert!((aa.im - (-9.8909900990)).abs() < 1e-9);
        assert!((ab.re - (-0.9900990099)).abs() < 1e-9);
        assert!((ab.im - 9.9009900990).abs() < 1e-9);
    }

    #[test]
    fn zero_impedance_branch_uses_penalty_conductance() {
        let graph = one_branch_graph("BUS::A", "BUS::B", 0.0, 0.0, None);
        let y = build_sparse_ybus(&graph, &YbusConfig::default());
        let map = to_map(&y);
        let aa = map
            .get(&("BUS::A".to_string(), "BUS::A".to_string()))
            .copied()
            .expect("AA");
        assert!((aa.re - 1_000_000.0).abs() < 1e-9);
        assert_eq!(y.zib_penalty_branches, vec!["L1".to_string()]);
    }

    #[test]
    fn parity_diff_hard_fails_over_threshold() {
        let graph = one_branch_graph("BUS::A", "BUS::B", 0.01, 0.1, Some(0.02));
        let actual = build_sparse_ybus(&graph, &YbusConfig::default());
        let mut expected = actual.clone();
        expected.entries[0].g_pu += 5e-8;

        let pass = compare_sparse_ybus(&expected, &actual, 1e-7);
        assert!(pass.pass);
        assert!(pass.max_abs_error <= 1e-7);

        expected.entries[0].g_pu += 2e-7;
        let fail = compare_sparse_ybus(&expected, &actual, 1e-7);
        assert!(!fail.pass);
        assert!(fail.max_abs_error > 1e-7);
    }

    #[test]
    fn march22_proxy_snapshot_hits_1e_7_mark() {
        let cfg = YbusConfig::default();
        let graph = one_branch_graph("BUS::CN_A+CN_B", "BUS::CN_C", 0.01, 0.1, Some(0.02));
        let l1_actual = derive_branch_series_row(&graph.branches[0], &cfg);
        let brk_actual = derive_switch_penalty_row("BRK_1", "CN_A", "CN_B", &cfg);
        let shunt_actual = derive_shunt_row("SHUNT_1", "BUS::CN_C", 0.05);
        let actual = vec![l1_actual, brk_actual, shunt_actual];

        let expected = vec![
            ElementProxyRow {
                element_id: "L1".to_string(),
                bus_a: "BUS::CN_A+CN_B".to_string(),
                bus_b: "BUS::CN_C".to_string(),
                g_pu: 0.99009901,
                b_pu: -9.90099010,
            },
            ElementProxyRow {
                element_id: "BRK_1".to_string(),
                bus_a: "CN_A".to_string(),
                bus_b: "CN_B".to_string(),
                g_pu: 1_000_000.0,
                b_pu: 0.0,
            },
            ElementProxyRow {
                element_id: "SHUNT_1".to_string(),
                bus_a: "BUS::CN_C".to_string(),
                bus_b: "N/A".to_string(),
                g_pu: 0.0,
                b_pu: 0.05,
            },
        ];

        let report = compare_element_proxy_rows(&expected, &actual, 1e-7);
        assert!(report.pass);
        assert!(report.mae < 1e-7);
        assert!(report.max_abs_error < 1e-7);
    }

    #[test]
    fn ybus_decision_hash_is_deterministic_for_reordered_entries() {
        let a = SparseYbus {
            bus_order: vec!["B".to_string(), "A".to_string()],
            entries: vec![
                SparseYbusEntry {
                    row_bus: "B".to_string(),
                    col_bus: "A".to_string(),
                    g_pu: -1.0,
                    b_pu: 9.0,
                },
                SparseYbusEntry {
                    row_bus: "A".to_string(),
                    col_bus: "A".to_string(),
                    g_pu: 1.0,
                    b_pu: -9.0,
                },
            ],
            zib_penalty_branches: vec!["SW1".to_string()],
        };

        let mut b = a.clone();
        b.entries.reverse();

        assert_eq!(ybus_decision_hash(&a), ybus_decision_hash(&b));
    }
}

