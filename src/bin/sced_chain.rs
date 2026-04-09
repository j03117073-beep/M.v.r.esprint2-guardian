use m_v_r_esprint1::sced_offer_chain::{verify_csv, RecordKey, VerifierReport, VerifyCode};
use std::env;
use std::fs::File;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        return Err(
            "Usage: cargo run --bin sced_chain -- verify <input.csv> [expected_hash] [--records-total <usize>]".to_string(),
        );
    }

    let command = &args[1];
    if command != "verify" {
        return Err("Only 'verify' command is supported.".to_string());
    }

    let input_path = &args[2];
    let mut expected_hash: Option<&str> = None;
    let mut expected_records_total: Option<usize> = None;
    let mut i = 3usize;
    while i < args.len() {
        match args[i].as_str() {
            "--records-total" => {
                let raw = args
                    .get(i + 1)
                    .ok_or_else(|| "--records-total requires a numeric value".to_string())?;
                let parsed = raw.parse::<usize>().map_err(|_| {
                    format!("invalid --records-total value '{}': expected usize", raw)
                })?;
                expected_records_total = Some(parsed);
                i += 2;
            }
            value => {
                if expected_hash.is_none() {
                    expected_hash = Some(value);
                    i += 1;
                } else {
                    return Err(format!("Unexpected argument '{}'", value));
                }
            }
        }
    }

    let file = File::open(input_path)
        .map_err(|e| format!("Failed to open input CSV '{}': {}", input_path, e))?;

    // Initial parse for deterministic record count log.
    let parsed = match m_v_r_esprint1::sced_offer_chain::parse_csv(file) {
        Ok(records) => records,
        Err(err) => {
            let report = VerifierReport {
                status: "FAIL".to_string(),
                records_total: 0,
                records_verified: 0,
                final_chain_hash: "".to_string(),
                expected_final_chain_hash: expected_hash.map(|s| s.to_string()),
                mismatch_index: None,
                errors: vec![map_cli_parse_error(err)],
            };
            println!("[INFO] verifier_start records=0");
            emit_json(&report)?;
            println!(
                "[FAIL] code={} mismatch_index= key={}",
                report.errors[0].code.code_as_str(),
                format_key(&report.errors[0].record_key)
            );
            return Ok(());
        }
    };

    println!("[INFO] verifier_start records={}", parsed.len());
    println!("[INFO] normalized_and_sorted");

    // Run verifier contract from an independently re-opened input stream.
    let file_again = File::open(input_path)
        .map_err(|e| format!("Failed to reopen input CSV '{}': {}", input_path, e))?;
    let report = verify_csv(file_again, expected_hash, expected_records_total);

    println!(
        "[INFO] chain_rebuild_complete final_chain_hash={}",
        report.final_chain_hash
    );

    emit_json(&report)?;

    if report.status == "PASS" {
        println!(
            "[PASS] verification_complete records_verified={}",
            report.records_verified
        );
    } else if let Some(first) = report.errors.first() {
        let idx = report
            .mismatch_index
            .map(|i| i.to_string())
            .unwrap_or_default();
        let key = format_key(&first.record_key);
        println!(
            "[FAIL] code={} mismatch_index={} key={}",
            first.code.code_as_str(),
            idx,
            key
        );
    } else {
        println!("[FAIL] code=UNKNOWN mismatch_index= key=");
    }

    Ok(())
}

fn format_key(key: &RecordKey) -> String {
    format!(
        "({},{},{},{})",
        key.scd_timestamp, key.repeat_hour_flag, key.resource_name, key.offer_type
    )
}

fn emit_json(report: &VerifierReport) -> Result<(), String> {
    let json = serde_json::to_string(report).map_err(|e| format!("JSON encode failed: {}", e))?;
    println!("{json}");
    Ok(())
}

fn map_cli_parse_error(err: m_v_r_esprint1::sced_offer_chain::ParseError) -> m_v_r_esprint1::sced_offer_chain::VerifyError {
    use m_v_r_esprint1::sced_offer_chain::{VerifyError, VerifyCode};
    match err {
        m_v_r_esprint1::sced_offer_chain::ParseError::CsvSchemaMismatch => VerifyError {
            code: VerifyCode::CsvSchemaMismatch,
            message: "CSV schema mismatch".to_string(),
            record_key: RecordKey {
                scd_timestamp: String::new(),
                repeat_hour_flag: false,
                resource_name: String::new(),
                offer_type: String::new(),
            },
        },
        m_v_r_esprint1::sced_offer_chain::ParseError::MissingValue(field) => VerifyError {
            code: VerifyCode::CsvMalformed,
            message: format!("missing value for field '{field}'"),
            record_key: RecordKey {
                scd_timestamp: String::new(),
                repeat_hour_flag: false,
                resource_name: String::new(),
                offer_type: String::new(),
            },
        },
        m_v_r_esprint1::sced_offer_chain::ParseError::InvalidBoolean(v) => VerifyError {
            code: VerifyCode::InvalidBoolean,
            message: format!("invalid boolean '{v}'"),
            record_key: RecordKey {
                scd_timestamp: String::new(),
                repeat_hour_flag: false,
                resource_name: String::new(),
                offer_type: String::new(),
            },
        },
        m_v_r_esprint1::sced_offer_chain::ParseError::InvalidNumeric(field, v) => VerifyError {
            code: VerifyCode::InvalidNumeric,
            message: format!("invalid numeric field '{field}' with value '{v}'"),
            record_key: RecordKey {
                scd_timestamp: String::new(),
                repeat_hour_flag: false,
                resource_name: String::new(),
                offer_type: String::new(),
            },
        },
        m_v_r_esprint1::sced_offer_chain::ParseError::DuplicatePrimaryKey(ts, rep, res, typ) => VerifyError {
            code: VerifyCode::DuplicatePk,
            message: "duplicate primary key detected".to_string(),
            record_key: RecordKey {
                scd_timestamp: ts,
                repeat_hour_flag: rep,
                resource_name: res,
                offer_type: typ,
            },
        },
        m_v_r_esprint1::sced_offer_chain::ParseError::MalformedCsv(msg) => VerifyError {
            code: VerifyCode::CsvMalformed,
            message: msg,
            record_key: RecordKey {
                scd_timestamp: String::new(),
                repeat_hour_flag: false,
                resource_name: String::new(),
                offer_type: String::new(),
            },
        },
    }
}

trait VerifyCodeExt {
    fn code_as_str(&self) -> &'static str;
}

impl VerifyCodeExt for VerifyCode {
    fn code_as_str(&self) -> &'static str {
        match self {
            VerifyCode::DuplicatePk => "DUPLICATE_PK",
            VerifyCode::InvalidNumeric => "INVALID_NUMERIC",
            VerifyCode::InvalidBoolean => "INVALID_BOOLEAN",
            VerifyCode::HashMismatch => "HASH_MISMATCH",
            VerifyCode::CsvSchemaMismatch => "CSV_SCHEMA_MISMATCH",
            VerifyCode::RecordCountMismatch => "RECORD_COUNT_MISMATCH",
            VerifyCode::ChainContinuityBreak => "CHAIN_CONTINUITY_BREAK",
            VerifyCode::CsvMalformed => "CSV_MALFORMED",
        }
    }
}
