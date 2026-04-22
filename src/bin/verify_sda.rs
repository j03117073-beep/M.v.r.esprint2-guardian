use m_v_r_esprint1::sced_offer_chain::CanonicalTruthPackage;
use m_v_r_esprint1::sovereign_diagnostic::{
    verify_diagnostic, verify_diagnostic_with_canonical_reference, DiagnosticVerificationOutcome,
    SovereignDiagnostic,
};
use serde::Serialize;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    run()
}

#[derive(Debug, Clone, Copy)]
enum VerificationMode {
    FullPayload,
    SingleFileHash,
    SingleFileEmbedded,
}

#[derive(Debug, Serialize)]
struct VerificationReport<'a> {
    status: &'a str,
    trust: &'a str,
    mode: &'a str,
    dtc: &'a str,
    reason: &'a str,
}

fn run() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        return ExitCode::from(2);
    }

    let result = match args[1].as_str() {
        "full" => verify_full(&args),
        "single" => verify_single_with_hash(&args),
        "single-embedded" => verify_single_embedded(&args),
        _ => {
            print_usage();
            return ExitCode::from(2);
        }
    };

    match result {
        Ok((mode, dtc, outcome)) => {
            let report = build_report(mode, &dtc, outcome);
            print_report(&report);
            if matches!(outcome, DiagnosticVerificationOutcome::Valid) {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(reason) => {
            let report = VerificationReport {
                status: "INVALID",
                trust: "REJECTED",
                mode: "INPUT_ERROR",
                dtc: "N/A",
                reason: &reason,
            };
            print_report(&report);
            ExitCode::from(2)
        }
    }
}

fn verify_full(
    args: &[String],
) -> Result<(VerificationMode, String, DiagnosticVerificationOutcome), String> {
    if args.len() != 6 {
        return Err(
            "Usage: cargo run --bin verify_sda -- full <canonical_truth.json> <diagnostic.bin> <public_key_hex32> <approved_firmware_hash_hex32[,hex32..]>".to_string(),
        );
    }

    let truth: CanonicalTruthPackage = serde_json::from_str(
        &fs::read_to_string(&args[2]).map_err(|e| format!("read truth package failed: {e}"))?,
    )
    .map_err(|e| format!("parse truth package failed: {e}"))?;

    let diagnostic = read_diagnostic(&args[3])?;
    let verifying_key = parse_public_key(&args[4])?;
    let approved = parse_approved_firmware_hashes(&args[5])?;

    let outcome = verify_diagnostic(
        &diagnostic,
        &truth.canonical_payload_bytes,
        &verifying_key,
        &approved,
    );
    Ok((VerificationMode::FullPayload, diagnostic.dtc, outcome))
}

fn verify_single_with_hash(
    args: &[String],
) -> Result<(VerificationMode, String, DiagnosticVerificationOutcome), String> {
    if args.len() != 6 {
        return Err(
            "Usage: cargo run --bin verify_sda -- single <diagnostic.bin> <canonical_hash_hex32> <public_key_hex32> <approved_firmware_hash_hex32[,hex32..]>".to_string(),
        );
    }

    let diagnostic = read_diagnostic(&args[2])?;
    let canonical_hash = parse_hash32_hex(&args[3], "canonical hash")?;
    let verifying_key = parse_public_key(&args[4])?;
    let approved = parse_approved_firmware_hashes(&args[5])?;

    let outcome = verify_diagnostic_with_canonical_reference(
        &diagnostic,
        &canonical_hash,
        &verifying_key,
        &approved,
    );
    Ok((VerificationMode::SingleFileHash, diagnostic.dtc, outcome))
}

fn verify_single_embedded(
    args: &[String],
) -> Result<(VerificationMode, String, DiagnosticVerificationOutcome), String> {
    if args.len() != 5 {
        return Err(
            "Usage: cargo run --bin verify_sda -- single-embedded <diagnostic.bin> <public_key_hex32> <approved_firmware_hash_hex32[,hex32..]>".to_string(),
        );
    }

    let diagnostic = read_diagnostic(&args[2])?;
    let verifying_key = parse_public_key(&args[3])?;
    let approved = parse_approved_firmware_hashes(&args[4])?;

    // Embedded mode validates signature and firmware against the hash already sealed in artifact.
    let outcome = verify_diagnostic_with_canonical_reference(
        &diagnostic,
        &diagnostic.canonical_hash,
        &verifying_key,
        &approved,
    );
    Ok((
        VerificationMode::SingleFileEmbedded,
        diagnostic.dtc,
        outcome,
    ))
}

fn read_diagnostic(path: &str) -> Result<SovereignDiagnostic, String> {
    let diagnostic_bytes = fs::read(path).map_err(|e| format!("read diagnostic failed: {e}"))?;
    let diagnostic: SovereignDiagnostic = bincode::deserialize(&diagnostic_bytes)
        .map_err(|e| format!("decode diagnostic failed: {e}"))?;
    Ok(diagnostic)
}

fn parse_public_key(public_key_hex: &str) -> Result<ed25519_dalek::VerifyingKey, String> {
    let pub_key_bytes =
        hex::decode(public_key_hex).map_err(|e| format!("invalid public key hex: {e}"))?;
    if pub_key_bytes.len() != 32 {
        return Err("public key must be 32 bytes (64 hex chars)".to_string());
    }
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&pub_key_bytes);
    ed25519_dalek::VerifyingKey::from_bytes(&pk)
        .map_err(|e| format!("invalid Ed25519 public key: {e}"))
}

fn parse_hash32_hex(input: &str, label: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(input).map_err(|e| format!("invalid {label} hex: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!("{label} must be 32 bytes (64 hex chars)"));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

fn parse_approved_firmware_hashes(input: &str) -> Result<Vec<[u8; 32]>, String> {
    let mut approved = Vec::new();
    for token in input.split(',').filter(|s| !s.trim().is_empty()) {
        approved.push(parse_hash32_hex(
            token.trim(),
            &format!("firmware hash '{token}'"),
        )?);
    }
    Ok(approved)
}

fn build_report<'a>(
    mode: VerificationMode,
    dtc: &'a str,
    outcome: DiagnosticVerificationOutcome,
) -> VerificationReport<'a> {
    let mode_str = match mode {
        VerificationMode::FullPayload => "FULL_PAYLOAD",
        VerificationMode::SingleFileHash => "SINGLE_FILE_HASH",
        VerificationMode::SingleFileEmbedded => "SINGLE_FILE_EMBEDDED",
    };

    match outcome {
        DiagnosticVerificationOutcome::Valid => VerificationReport {
            status: "VALID",
            trust: "VERIFIED",
            mode: mode_str,
            dtc,
            reason: "NONE",
        },
        DiagnosticVerificationOutcome::InvalidSignature => VerificationReport {
            status: "INVALID",
            trust: "REJECTED",
            mode: mode_str,
            dtc,
            reason: "INVALID_SIGNATURE",
        },
        DiagnosticVerificationOutcome::HashMismatch => VerificationReport {
            status: "INVALID",
            trust: "REJECTED",
            mode: mode_str,
            dtc,
            reason: "HASH_MISMATCH",
        },
        DiagnosticVerificationOutcome::UnrecognizedFirmware => VerificationReport {
            status: "INVALID",
            trust: "REJECTED",
            mode: mode_str,
            dtc,
            reason: "UNRECOGNIZED_FIRMWARE",
        },
    }
}

fn print_report(report: &VerificationReport<'_>) {
    println!("STATUS: {}", report.status);
    println!("TRUST: {}", report.trust);
    println!("MODE: {}", report.mode);
    println!("DTC: {}", report.dtc);
    println!("REASON: {}", report.reason);
    let json = serde_json::to_string(report).unwrap_or_else(|_| {
        "{\"status\":\"INVALID\",\"reason\":\"REPORT_SERIALIZE_FAILED\"}".to_string()
    });
    println!("RESULT_JSON: {json}");
}

fn print_usage() {
    println!("STATUS: INVALID");
    println!("TRUST: REJECTED");
    println!("MODE: INPUT_ERROR");
    println!("DTC: N/A");
    println!("REASON: USAGE");
    println!("RESULT_JSON: {{\"status\":\"INVALID\",\"reason\":\"USAGE\"}}");
    println!("USAGE_FULL: cargo run --bin verify_sda -- full <canonical_truth.json> <diagnostic.bin> <public_key_hex32> <approved_firmware_hash_hex32[,hex32..]>");
    println!("USAGE_SINGLE: cargo run --bin verify_sda -- single <diagnostic.bin> <canonical_hash_hex32> <public_key_hex32> <approved_firmware_hash_hex32[,hex32..]>");
    println!("USAGE_SINGLE_EMBEDDED: cargo run --bin verify_sda -- single-embedded <diagnostic.bin> <public_key_hex32> <approved_firmware_hash_hex32[,hex32..]>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_mapping_is_unambiguous() {
        let ok = build_report(
            VerificationMode::SingleFileHash,
            "G0000",
            DiagnosticVerificationOutcome::Valid,
        );
        assert_eq!(ok.status, "VALID");
        assert_eq!(ok.trust, "VERIFIED");
        assert_eq!(ok.reason, "NONE");

        let bad = build_report(
            VerificationMode::SingleFileHash,
            "G0901",
            DiagnosticVerificationOutcome::UnrecognizedFirmware,
        );
        assert_eq!(bad.status, "INVALID");
        assert_eq!(bad.reason, "UNRECOGNIZED_FIRMWARE");
    }

    #[test]
    fn parse_hash32_hex_enforces_exact_length() {
        let err = parse_hash32_hex("aa", "canonical hash").expect_err("short hash must fail");
        assert!(err.contains("32 bytes"));
    }
}
