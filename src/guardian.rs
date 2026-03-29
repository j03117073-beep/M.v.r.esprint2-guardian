#![deny(unsafe_code)]

use crate::sced_offer_chain::{verify_records, VerifierReport};
use crate::sprint1::CanonicalBatch;

#[derive(Debug, Default)]
pub struct GuardianAuditor;

impl GuardianAuditor {
    pub fn verify_batch(&self, batch: &CanonicalBatch) -> VerifierReport {
        verify_records(
            batch.records.clone(),
            Some(&batch.final_chain_hash),
            Some(batch.records_total),
        )
    }

    pub fn verify_batch_against(
        &self,
        batch: &CanonicalBatch,
        expected_final_chain_hash: Option<&str>,
        expected_records_total: Option<usize>,
    ) -> VerifierReport {
        verify_records(
            batch.records.clone(),
            expected_final_chain_hash,
            expected_records_total,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprint1::ingest_and_canonicalize_csv;

    #[test]
    fn guardian_replay_matches_sprint1_hash() {
        let header = [
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
        let row = [
            "2026-03-22T18:05:00",
            "false",
            "7RNCHSLR_UNIT1",
            "OFFNS",
            "1",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "100",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
            "0",
        ]
        .join(",");
        let csv = format!("{header}\n{row}\n");

        let batch = ingest_and_canonicalize_csv(csv.as_bytes()).expect("batch must build");
        let guardian = GuardianAuditor;
        let report = guardian.verify_batch(&batch);

        assert_eq!(report.status, "PASS");
        assert_eq!(report.final_chain_hash, batch.final_chain_hash);
    }
}

