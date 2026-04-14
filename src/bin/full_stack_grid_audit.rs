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
use m_v_r_esprint1::economics::shadow_prices::{
    build_shadow_price_chain, parse_proxy_snapshot_csv, ShadowPriceChainRecord,
};
use m_v_r_esprint1::topology::ybus::{parse_sparse_ybus_csv, ybus_decision_hash};
use serde::Serialize;
use std::env;
use std::fs::File;

#[derive(Debug, Serialize)]
struct FullStackAuditArtifact {
    ri04_decision_hash_hex: String,
    shadow_records_total: usize,
    shadow_final_chain_hash_hex: String,
    shadow_chain_records: Vec<ShadowPriceChainRecord>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        return Err(
            "Usage: cargo run --bin full_stack_grid_audit -- <shadow_proxy_csv> (--ri04-hash <hex32> | --ri04-ybus-csv <ybus.csv>) [output_json]"
                .into(),
        );
    }

    let shadow_path = &args[1];
    let (ri04_hash, next_idx) = match args.get(2).map(String::as_str) {
        Some("--ri04-hash") => {
            let hex = args.get(3).ok_or("missing value for --ri04-hash")?;
            (decode_hash_hex(hex)?, 4usize)
        }
        Some("--ri04-ybus-csv") => {
            let ybus_path = args.get(3).ok_or("missing value for --ri04-ybus-csv")?;
            let file = File::open(ybus_path)?;
            let ybus = parse_sparse_ybus_csv(file).map_err(|e| format!("{e:?}"))?;
            (ybus_decision_hash(&ybus), 4usize)
        }
        _ => {
            return Err("Expected --ri04-hash or --ri04-ybus-csv".into());
        }
    };

    let output_json = args
        .get(next_idx)
        .cloned()
        .unwrap_or_else(|| "artifacts/full_stack_grid_audit.json".to_string());

    let shadow_file = File::open(shadow_path)?;
    let shadow_rows = parse_proxy_snapshot_csv(shadow_file).map_err(|e| format!("{e:?}"))?;
    let chain_report = build_shadow_price_chain(&shadow_rows, &ri04_hash);

    let artifact = FullStackAuditArtifact {
        ri04_decision_hash_hex: chain_report.ri04_decision_hash_hex.clone(),
        shadow_records_total: chain_report.records_total,
        shadow_final_chain_hash_hex: chain_report.final_chain_hash_hex.clone(),
        shadow_chain_records: chain_report.records.clone(),
    };

    if let Some(parent) = std::path::Path::new(&output_json).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(&output_json, serde_json::to_string_pretty(&artifact)?)?;

    println!("RI04 decision hash: {}", chain_report.ri04_decision_hash_hex);
    println!(
        "RI18 shadow chain final hash: {} (records={})",
        chain_report.final_chain_hash_hex, chain_report.records_total
    );
    println!("Artifact: {}", output_json);

    Ok(())
}

fn decode_hash_hex(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = hex::decode(input)?;
    if bytes.len() != 32 {
        return Err("RI04 hash must be 32 bytes (64 hex chars)".into());
    }
    Ok(bytes)
}

