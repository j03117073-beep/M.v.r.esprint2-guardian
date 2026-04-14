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

use crate::sced_offer_chain::{
    build_hash_chain, parse_csv, sort_records, ChainedRecord, ParseError, ScedResourceOfferRecord,
};
use std::io::Read;

pub const SCHEMA_VERSION: &str = "sced.v1";
pub const HASH_SPEC_VERSION: &str = "sha256.chain.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalBatch {
    pub schema_version: String,
    pub hash_spec_version: String,
    pub records_total: usize,
    pub records: Vec<ScedResourceOfferRecord>,
    pub chain: Vec<ChainedRecord>,
    pub final_chain_hash: String,
}

pub fn ingest_and_canonicalize_csv<R: Read>(input: R) -> Result<CanonicalBatch, ParseError> {
    let mut records = parse_csv(input)?;
    sort_records(&mut records);
    let chain = build_hash_chain(records.clone())?;
    let final_chain_hash = chain
        .last()
        .map(|c| c.chain_hash.clone())
        .unwrap_or_else(|| "0".to_string());

    Ok(CanonicalBatch {
        schema_version: SCHEMA_VERSION.to_string(),
        hash_spec_version: HASH_SPEC_VERSION.to_string(),
        records_total: records.len(),
        records,
        chain,
        final_chain_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_batch_empty_is_genesis() {
        let headers = [
            "scd_timestamp",
            "repeat_hour_flag",
            "resource_name",
            "offer_type",
            "price1_urs",
            "price1_drs",
            "price1_rrspf",
            "price1_rrsuf",
            "price1_rrsff",
            "price1_ns",
            "price1_ecrs",
            "quantity_mw1",
            "price2_urs",
            "price2_drs",
            "price2_rrspf",
            "price2_rrsuf",
            "price2_rrsff",
            "price2_ns",
            "price2_ecrs",
            "quantity_mw2",
            "price3_urs",
            "price3_drs",
            "price3_rrspf",
            "price3_rrsuf",
            "price3_rrsff",
            "price3_ns",
            "price3_ecrs",
            "quantity_mw3",
            "price4_urs",
            "price4_drs",
            "price4_rrspf",
            "price4_rrsuf",
            "price4_rrsff",
            "price4_ns",
            "price4_ecrs",
            "quantity_mw4",
            "price5_urs",
            "price5_drs",
            "price5_rrspf",
            "price5_rrsuf",
            "price5_rrsff",
            "price5_ns",
            "price5_ecrs",
            "quantity_mw5",
            "price6_urs",
            "price6_drs",
            "price6_rrspf",
            "price6_rrsuf",
            "price6_rrsff",
            "price6_ns",
            "price6_ecrs",
            "quantity_mw6",
        ]
        .join(",");
        let csv = format!("{headers}\n");
        let batch = ingest_and_canonicalize_csv(csv.as_bytes()).expect("parse must pass");
        assert_eq!(batch.records_total, 0);
        assert_eq!(batch.final_chain_hash, "0");
    }
}


