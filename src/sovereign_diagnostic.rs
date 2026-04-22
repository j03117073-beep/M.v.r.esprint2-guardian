#![deny(unsafe_code)]

use crate::sced_offer_chain::VerifierReport;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde::de::{self, SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SovereignDiagnostic {
    pub version: u8,
    pub dtc: String,
    pub canonical_hash: [u8; 32],
    pub canonical_payload_len: u64,
    pub firmware_hash: [u8; 32],
    #[serde(
        serialize_with = "serialize_signature",
        deserialize_with = "deserialize_signature"
    )]
    pub signature: [u8; 64],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticVerificationOutcome {
    Valid,
    InvalidSignature,
    HashMismatch,
    UnrecognizedFirmware,
}

pub fn derive_dtc(canonical_payload_bytes: &[u8]) -> String {
    if canonical_payload_bytes.is_empty() {
        return "G0999".to_string();
    }
    let hay = String::from_utf8_lossy(canonical_payload_bytes).to_ascii_lowercase();
    if hay.contains("relay") && hay.contains("fault") {
        return "G0122".to_string();
    }
    if hay.contains("frequency") && (hay.contains("violation") || hay.contains("threshold")) {
        return "G0201".to_string();
    }
    "G0000".to_string()
}

pub fn canonical_payload_hash(canonical_payload_bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(canonical_payload_bytes).into()
}

pub fn diagnostic_hash(
    version: u8,
    dtc: &str,
    canonical_hash: &[u8; 32],
    firmware_hash: &[u8; 32],
    canonical_payload_len: u64,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([version]);
    hasher.update(dtc.as_bytes());
    hasher.update(canonical_hash);
    hasher.update(firmware_hash);
    hasher.update(canonical_payload_len.to_le_bytes());
    hasher.finalize().into()
}

pub fn emit_signed_diagnostic(
    canonical_payload_bytes: &[u8],
    firmware_hash: [u8; 32],
    verifier_report: &VerifierReport,
    signing_key: &SigningKey,
) -> Result<SovereignDiagnostic, String> {
    if verifier_report.status != "PASS" {
        return Err("SDA gate refusal: verifier status is not PASS".to_string());
    }
    let version = 1u8;
    let dtc = derive_dtc(canonical_payload_bytes);
    let canonical_hash = canonical_payload_hash(canonical_payload_bytes);
    let d_hash = diagnostic_hash(
        version,
        &dtc,
        &canonical_hash,
        &firmware_hash,
        canonical_payload_bytes.len() as u64,
    );
    let sig = signing_key.sign(&d_hash);
    let signature = sig.to_bytes();
    Ok(SovereignDiagnostic {
        version,
        dtc,
        canonical_hash,
        canonical_payload_len: canonical_payload_bytes.len() as u64,
        firmware_hash,
        signature,
    })
}

pub fn verify_diagnostic(
    diagnostic: &SovereignDiagnostic,
    canonical_payload_bytes: &[u8],
    verifying_key: &VerifyingKey,
    approved_firmware_hashes: &[[u8; 32]],
) -> DiagnosticVerificationOutcome {
    let recomputed_canonical_hash = canonical_payload_hash(canonical_payload_bytes);
    if diagnostic.canonical_hash != recomputed_canonical_hash {
        return DiagnosticVerificationOutcome::HashMismatch;
    }
    if diagnostic.canonical_payload_len != canonical_payload_bytes.len() as u64 {
        return DiagnosticVerificationOutcome::HashMismatch;
    }

    verify_diagnostic_with_canonical_reference(
        diagnostic,
        &recomputed_canonical_hash,
        verifying_key,
        approved_firmware_hashes,
    )
}

pub fn verify_diagnostic_with_canonical_reference(
    diagnostic: &SovereignDiagnostic,
    canonical_hash: &[u8; 32],
    verifying_key: &VerifyingKey,
    approved_firmware_hashes: &[[u8; 32]],
) -> DiagnosticVerificationOutcome {
    if &diagnostic.canonical_hash != canonical_hash {
        return DiagnosticVerificationOutcome::HashMismatch;
    }

    let expected_diag_hash = diagnostic_hash(
        diagnostic.version,
        &diagnostic.dtc,
        &diagnostic.canonical_hash,
        &diagnostic.firmware_hash,
        diagnostic.canonical_payload_len,
    );
    let sig = ed25519_dalek::Signature::from_bytes(&diagnostic.signature);
    if verifying_key.verify(&expected_diag_hash, &sig).is_err() {
        return DiagnosticVerificationOutcome::InvalidSignature;
    }

    if !approved_firmware_hashes
        .iter()
        .any(|h| h == &diagnostic.firmware_hash)
    {
        return DiagnosticVerificationOutcome::UnrecognizedFirmware;
    }

    DiagnosticVerificationOutcome::Valid
}

pub fn default_simulation_signing_key() -> SigningKey {
    let digest: [u8; 32] = Sha256::digest(b"mvre-sda-device-key-v1").into();
    SigningKey::from_bytes(&digest)
}

fn serialize_signature<S>(value: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut tup = serializer.serialize_tuple(64)?;
    for b in value {
        tup.serialize_element(b)?;
    }
    tup.end()
}

fn deserialize_signature<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct SigVisitor;
    impl<'de> Visitor<'de> for SigVisitor {
        type Value = [u8; 64];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an array with exactly 64 u8 elements")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut out = [0u8; 64];
            for (i, slot) in out.iter_mut().enumerate() {
                *slot = seq
                    .next_element::<u8>()?
                    .ok_or_else(|| de::Error::invalid_length(i, &self))?;
            }
            Ok(out)
        }
    }
    deserializer.deserialize_tuple(64, SigVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sced_offer_chain::{compute_canonical_truth, verify_truth_package};

    fn canonical_header() -> String {
        let mut cols = vec![
            "scd_timestamp".to_string(),
            "repeat_hour_flag".to_string(),
            "resource_name".to_string(),
            "offer_type".to_string(),
        ];
        for b in 1..=6 {
            cols.push(format!("price{b}_urs"));
            cols.push(format!("price{b}_drs"));
            cols.push(format!("price{b}_rrspf"));
            cols.push(format!("price{b}_rrsuf"));
            cols.push(format!("price{b}_rrsff"));
            cols.push(format!("price{b}_ns"));
            cols.push(format!("price{b}_ecrs"));
            cols.push(format!("quantity_mw{b}"));
        }
        cols.join(",")
    }

    fn one_row_truth() -> crate::sced_offer_chain::CanonicalTruthPackage {
        let csv = format!(
            "{header}\n{row}\n",
            header = canonical_header(),
            row = [
                "2026-01-21T23:55:18",
                "false",
                "RES",
                "OFFNS",
                &vec!["1"; 48].join(",")
            ]
            .join(",")
        );
        compute_canonical_truth(csv.as_bytes()).expect("truth package must build")
    }

    #[test]
    fn byte_mutation_invalidates_diagnostic_verification() {
        let truth = one_row_truth();
        let report = verify_truth_package(&truth, Some(&truth.final_chain_hash));
        let sk = default_simulation_signing_key();
        let vk = sk.verifying_key();
        let fw = canonical_payload_hash(b"firmware-v1");
        let diag = emit_signed_diagnostic(&truth.canonical_payload_bytes, fw, &report, &sk)
            .expect("diagnostic should emit");

        let mut tampered = truth.canonical_payload_bytes.clone();
        tampered[0] ^= 0x01;
        let out = verify_diagnostic(&diag, &tampered, &vk, &[fw]);
        assert_eq!(out, DiagnosticVerificationOutcome::HashMismatch);
    }

    #[test]
    fn dtc_is_stable_for_identical_payload() {
        let truth = one_row_truth();
        let a = derive_dtc(&truth.canonical_payload_bytes);
        let b = derive_dtc(&truth.canonical_payload_bytes);
        assert_eq!(a, b);
    }

    #[test]
    fn verifier_gate_blocks_non_pass_diagnostic_emit() {
        let truth = one_row_truth();
        let mut report = verify_truth_package(&truth, Some(&truth.final_chain_hash));
        report.status = "FAIL".to_string();
        let sk = default_simulation_signing_key();
        let fw = canonical_payload_hash(b"firmware-v1");
        let res = emit_signed_diagnostic(&truth.canonical_payload_bytes, fw, &report, &sk);
        assert!(res.is_err());
    }
}
