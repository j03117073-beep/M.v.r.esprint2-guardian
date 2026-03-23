#![deny(unsafe_code)]

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScedResourceOfferRecord {
    pub scd_timestamp: String,
    pub repeat_hour_flag: bool,
    pub resource_name: String,
    pub offer_type: String,
    pub prices_and_quantities: [String; 48],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainedRecord {
    pub key: (String, bool, String, String),
    pub record_hash: String,
    pub chain_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    CsvSchemaMismatch,
    MissingValue(String),
    InvalidBoolean(String),
    InvalidNumeric(String, String),
    DuplicatePrimaryKey(String, bool, String, String),
    MalformedCsv(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerifyCode {
    DuplicatePk,
    InvalidNumeric,
    InvalidBoolean,
    HashMismatch,
    CsvSchemaMismatch,
    RecordCountMismatch,
    ChainContinuityBreak,
    CsvMalformed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordKey {
    pub scd_timestamp: String,
    pub repeat_hour_flag: bool,
    pub resource_name: String,
    pub offer_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyError {
    pub code: VerifyCode,
    pub message: String,
    pub record_key: Option<RecordKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifierReport {
    pub status: String,
    pub records_total: usize,
    pub records_verified: usize,
    pub final_chain_hash: String,
    pub expected_final_chain_hash: Option<String>,
    pub mismatch_index: Option<usize>,
    pub errors: Vec<VerifyError>,
}

fn numeric_field_names() -> Vec<String> {
    let mut names = Vec::with_capacity(48);
    for block in 1..=6 {
        names.push(format!("price{}_urs", block));
        names.push(format!("price{}_drs", block));
        names.push(format!("price{}_rrspf", block));
        names.push(format!("price{}_rrsuf", block));
        names.push(format!("price{}_rrsff", block));
        names.push(format!("price{}_ns", block));
        names.push(format!("price{}_ecrs", block));
        names.push(format!("quantity_mw{}", block));
    }
    names
}

fn expected_headers() -> Vec<String> {
    let mut headers = vec![
        "scd_timestamp".to_string(),
        "repeat_hour_flag".to_string(),
        "resource_name".to_string(),
        "offer_type".to_string(),
    ];
    headers.extend(numeric_field_names());
    headers
}

impl ScedResourceOfferRecord {
    pub fn primary_key(&self) -> (&str, bool, &str, &str) {
        (
            &self.scd_timestamp,
            self.repeat_hour_flag,
            &self.resource_name,
            &self.offer_type,
        )
    }

    pub fn canonical_record_string(&self) -> String {
        let mut fields = Vec::with_capacity(52);
        fields.push(self.scd_timestamp.clone());
        fields.push(self.repeat_hour_flag.to_string());
        fields.push(self.resource_name.clone());
        fields.extend(self.prices_and_quantities.iter().cloned());
        fields.push(self.offer_type.clone());
        fields.join("|")
    }

    pub fn record_hash(&self) -> String {
        sha256_hex(&self.canonical_record_string())
    }

    fn key_owned(&self) -> RecordKey {
        RecordKey {
            scd_timestamp: self.scd_timestamp.clone(),
            repeat_hour_flag: self.repeat_hour_flag,
            resource_name: self.resource_name.clone(),
            offer_type: self.offer_type.clone(),
        }
    }
}

pub fn parse_csv<R: Read>(input: R) -> Result<Vec<ScedResourceOfferRecord>, ParseError> {
    let reader = BufReader::new(input);
    let mut lines = reader.lines();

    let header_line = lines
        .next()
        .ok_or_else(|| ParseError::MalformedCsv("missing header row".to_string()))?
        .map_err(|e| ParseError::MalformedCsv(e.to_string()))?;

    let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();
    if headers != expected_headers() {
        return Err(ParseError::CsvSchemaMismatch);
    }

    let mut records = Vec::new();
    for line_result in lines {
        let line = line_result.map_err(|e| ParseError::MalformedCsv(e.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }

        let raw_values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
        if raw_values.len() != headers.len() {
            return Err(ParseError::MalformedCsv(format!(
                "row has {} values but header has {}",
                raw_values.len(),
                headers.len()
            )));
        }

        records.push(record_from_values(&raw_values)?);
    }

    Ok(records)
}

fn record_from_values(values: &[String]) -> Result<ScedResourceOfferRecord, ParseError> {
    let scd_timestamp = required_string(values, 0, "scd_timestamp")?;
    let repeat_hour_flag = parse_bool(&required_string(values, 1, "repeat_hour_flag")?)?;
    let resource_name = required_string(values, 2, "resource_name")?;
    let offer_type = required_string(values, 3, "offer_type")?;

    let mut numeric_values = Vec::with_capacity(48);
    for (offset, key) in numeric_field_names().iter().enumerate() {
        numeric_values.push(normalize_numeric(&values[4 + offset], key)?);
    }

    let prices_and_quantities = numeric_values
        .try_into()
        .map_err(|_| ParseError::MalformedCsv("expected 48 numeric values".to_string()))?;

    Ok(ScedResourceOfferRecord {
        scd_timestamp,
        repeat_hour_flag,
        resource_name,
        offer_type,
        prices_and_quantities,
    })
}

fn required_string(values: &[String], idx: usize, key: &str) -> Result<String, ParseError> {
    let value = values
        .get(idx)
        .ok_or_else(|| ParseError::MissingValue(key.to_string()))?
        .trim()
        .to_string();
    if value.is_empty() {
        return Err(ParseError::MissingValue(key.to_string()));
    }
    Ok(value)
}

fn normalize_numeric(raw: &str, key: &str) -> Result<String, ParseError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        return Ok("0.000000".to_string());
    }

    let parsed = trimmed
        .parse::<f64>()
        .map_err(|_| ParseError::InvalidNumeric(key.to_string(), trimmed.to_string()))?;

    Ok(format!("{parsed:.6}"))
}

fn parse_bool(raw: &str) -> Result<bool, ParseError> {
    match raw.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ParseError::InvalidBoolean(raw.to_string())),
    }
}

pub fn sort_records(records: &mut [ScedResourceOfferRecord]) {
    records.sort_by(compare_records);
}

fn compare_records(a: &ScedResourceOfferRecord, b: &ScedResourceOfferRecord) -> Ordering {
    a.scd_timestamp
        .cmp(&b.scd_timestamp)
        .then(a.repeat_hour_flag.cmp(&b.repeat_hour_flag))
        .then(a.resource_name.cmp(&b.resource_name))
        .then(a.offer_type.cmp(&b.offer_type))
}

pub fn build_hash_chain(mut records: Vec<ScedResourceOfferRecord>) -> Result<Vec<ChainedRecord>, ParseError> {
    sort_records(&mut records);

    let mut seen = HashSet::new();
    for r in &records {
        let key = (
            r.scd_timestamp.clone(),
            r.repeat_hour_flag,
            r.resource_name.clone(),
            r.offer_type.clone(),
        );
        if !seen.insert(key.clone()) {
            return Err(ParseError::DuplicatePrimaryKey(key.0, key.1, key.2, key.3));
        }
    }

    let mut out = Vec::with_capacity(records.len());
    let mut previous_chain_hash = "0".to_string();

    for r in records {
        let record_hash = r.record_hash();
        let chain_hash = sha256_hex(&format!("{}|{}", previous_chain_hash, record_hash));
        previous_chain_hash = chain_hash.clone();

        out.push(ChainedRecord {
            key: (
                r.scd_timestamp,
                r.repeat_hour_flag,
                r.resource_name,
                r.offer_type,
            ),
            record_hash,
            chain_hash,
        });
    }

    Ok(out)
}

pub fn verify_records(
    records: Vec<ScedResourceOfferRecord>,
    expected_final_chain_hash: Option<&str>,
    expected_records_total: Option<usize>,
) -> VerifierReport {
    let records_total = records.len();

    if records_total == 0 {
        let mut errors = Vec::new();
        let mut status = "PASS".to_string();
        if let Some(expected_count) = expected_records_total {
            if expected_count != 0 {
                status = "FAIL".to_string();
                errors.push(VerifyError {
                    code: VerifyCode::RecordCountMismatch,
                    message: format!("expected records_total={expected_count}, got 0"),
                    record_key: None,
                });
            }
        }
        if let Some(expected_hash) = expected_final_chain_hash {
            if expected_hash != "0" {
                status = "FAIL".to_string();
                errors.push(VerifyError {
                    code: VerifyCode::HashMismatch,
                    message: format!("expected final_chain_hash={expected_hash}, got 0"),
                    record_key: None,
                });
            }
        }

        return VerifierReport {
            status,
            records_total: 0,
            records_verified: if errors.is_empty() { 0 } else { 0 },
            final_chain_hash: "0".to_string(),
            expected_final_chain_hash: expected_final_chain_hash.map(|s| s.to_string()),
            mismatch_index: None,
            errors,
        };
    }

    let chain = match build_hash_chain(records) {
        Ok(v) => v,
        Err(err) => {
            return VerifierReport {
                status: "FAIL".to_string(),
                records_total,
                records_verified: 0,
                final_chain_hash: "".to_string(),
                expected_final_chain_hash: expected_final_chain_hash.map(|s| s.to_string()),
                mismatch_index: None,
                errors: vec![map_parse_error(err)],
            }
        }
    };

    let mut errors = Vec::new();
    let mut mismatch_index = None;

    if let Some(expected_count) = expected_records_total {
        if expected_count != chain.len() {
            errors.push(VerifyError {
                code: VerifyCode::RecordCountMismatch,
                message: format!("expected records_total={expected_count}, got {}", chain.len()),
                record_key: None,
            });
        }
    }

    // Explicit chain continuity validation (0-based index after deterministic sort)
    let mut prev = "0".to_string();
    for (idx, item) in chain.iter().enumerate() {
        let expected_link = sha256_hex(&format!("{}|{}", prev, item.record_hash));
        if expected_link != item.chain_hash {
            mismatch_index = Some(idx);
            errors.push(VerifyError {
                code: VerifyCode::ChainContinuityBreak,
                message: "chain continuity broken".to_string(),
                record_key: Some(RecordKey {
                    scd_timestamp: item.key.0.clone(),
                    repeat_hour_flag: item.key.1,
                    resource_name: item.key.2.clone(),
                    offer_type: item.key.3.clone(),
                }),
            });
            break;
        }
        prev = item.chain_hash.clone();
    }

    let final_chain_hash = chain
        .last()
        .map(|r| r.chain_hash.clone())
        .unwrap_or_else(|| "0".to_string());

    if mismatch_index.is_none() {
        if let Some(expected_hash) = expected_final_chain_hash {
            if expected_hash != final_chain_hash {
                mismatch_index = Some(chain.len().saturating_sub(1));
                errors.push(VerifyError {
                    code: VerifyCode::HashMismatch,
                    message: format!("expected final_chain_hash={expected_hash}, got {final_chain_hash}"),
                    record_key: chain.last().map(|last| RecordKey {
                        scd_timestamp: last.key.0.clone(),
                        repeat_hour_flag: last.key.1,
                        resource_name: last.key.2.clone(),
                        offer_type: last.key.3.clone(),
                    }),
                });
            }
        }
    }

    VerifierReport {
        status: if errors.is_empty() {
            "PASS".to_string()
        } else {
            "FAIL".to_string()
        },
        records_total,
        records_verified: if errors.is_empty() { chain.len() } else { 0 },
        final_chain_hash,
        expected_final_chain_hash: expected_final_chain_hash.map(|s| s.to_string()),
        mismatch_index,
        errors,
    }
}

fn map_parse_error(err: ParseError) -> VerifyError {
    match err {
        ParseError::CsvSchemaMismatch => VerifyError {
            code: VerifyCode::CsvSchemaMismatch,
            message: "CSV schema mismatch".to_string(),
            record_key: None,
        },
        ParseError::MissingValue(field) => VerifyError {
            code: VerifyCode::CsvMalformed,
            message: format!("missing value for field '{field}'"),
            record_key: None,
        },
        ParseError::InvalidBoolean(v) => VerifyError {
            code: VerifyCode::InvalidBoolean,
            message: format!("invalid boolean '{v}'"),
            record_key: None,
        },
        ParseError::InvalidNumeric(field, v) => VerifyError {
            code: VerifyCode::InvalidNumeric,
            message: format!("invalid numeric field '{field}' with value '{v}'"),
            record_key: None,
        },
        ParseError::DuplicatePrimaryKey(ts, rep, res, typ) => VerifyError {
            code: VerifyCode::DuplicatePk,
            message: "duplicate primary key detected".to_string(),
            record_key: Some(RecordKey {
                scd_timestamp: ts,
                repeat_hour_flag: rep,
                resource_name: res,
                offer_type: typ,
            }),
        },
        ParseError::MalformedCsv(msg) => VerifyError {
            code: VerifyCode::CsvMalformed,
            message: msg,
            record_key: None,
        },
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_record(ts: &str, repeat: bool, resource: &str, offer: &str, first: &str) -> ScedResourceOfferRecord {
        let mut fields = std::array::from_fn::<_, 48, _>(|_| "0.000000".to_string());
        fields[0] = first.to_string();
        ScedResourceOfferRecord {
            scd_timestamp: ts.to_string(),
            repeat_hour_flag: repeat,
            resource_name: resource.to_string(),
            offer_type: offer.to_string(),
            prices_and_quantities: fields,
        }
    }

    #[test]
    fn canonical_order_is_fixed() {
        let r = mk_record("2026-01-21T23:55:18", false, "7RNCHSLR_UNIT1", "OFFNS", "186.140000");
        let serialized = r.canonical_record_string();
        assert!(serialized.starts_with("2026-01-21T23:55:18|false|7RNCHSLR_UNIT1|186.140000|"));
        assert!(serialized.ends_with("|OFFNS"));
    }

    #[test]
    fn schema_order_mismatch_fails() {
        let csv = "repeat_hour_flag,scd_timestamp,resource_name,offer_type\nfalse,2026-01-21T23:55:18,RES,OFFNS\n";
        let err = parse_csv(csv.as_bytes()).expect_err("must fail schema mismatch");
        assert_eq!(err, ParseError::CsvSchemaMismatch);
    }

    #[test]
    fn bool_is_strict_lowercase_only() {
        let csv = format!(
            "{}\n{}\n",
            expected_headers().join(","),
            [
                "2026-01-21T23:55:18",
                "TRUE",
                "RES",
                "OFFNS",
                &vec!["0"; 48].join(",")
            ]
            .join(",")
        );
        let err = parse_csv(csv.as_bytes()).expect_err("must reject TRUE");
        assert_eq!(err, ParseError::InvalidBoolean("TRUE".to_string()));
    }

    #[test]
    fn numeric_normalization_is_canonical_fixed_6() {
        let header = expected_headers().join(",");
        let mut vals = vec![
            "2026-01-21T23:55:18".to_string(),
            "false".to_string(),
            "RES".to_string(),
            "OFFNS".to_string(),
        ];
        vals.push("1".to_string());
        vals.push("1.0".to_string());
        vals.extend(vec!["0".to_string(); 46]);
        let csv = format!("{}\n{}\n", header, vals.join(","));

        let parsed = parse_csv(csv.as_bytes()).expect("parse ok");
        assert_eq!(parsed[0].prices_and_quantities[0], "1.000000");
        assert_eq!(parsed[0].prices_and_quantities[1], "1.000000");
    }

    #[test]
    fn dst_fallback_uniqueness_uses_repeat_hour_flag() {
        let a = mk_record("2026-11-01T01:30:00", false, "RES_A", "OFFNS", "10.000000");
        let b = mk_record("2026-11-01T01:30:00", true, "RES_A", "OFFNS", "10.000000");
        let chain = build_hash_chain(vec![b, a]).expect("chain should build");
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].key.1, false);
        assert_eq!(chain[1].key.1, true);
    }

    #[test]
    fn empty_file_behavior_passes_with_genesis_hash() {
        let report = verify_records(vec![], Some("0"), Some(0));
        assert_eq!(report.status, "PASS");
        assert_eq!(report.final_chain_hash, "0");
        assert_eq!(report.records_verified, 0);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn record_count_binding_enforced() {
        let r = mk_record("2026-01-21T23:55:18", false, "RES", "OFFNS", "1.000000");
        let report = verify_records(vec![r], None, Some(2));
        assert_eq!(report.status, "FAIL");
        assert_eq!(report.errors[0].code, VerifyCode::RecordCountMismatch);
    }

    #[test]
    fn mismatch_index_is_zero_based_after_sort() {
        let r = mk_record("2026-01-21T23:55:18", false, "RES", "OFFNS", "1.000000");
        let report = verify_records(vec![r], Some("not-a-real-hash"), Some(1));
        assert_eq!(report.status, "FAIL");
        assert_eq!(report.mismatch_index, Some(0));
        assert_eq!(report.errors[0].code, VerifyCode::HashMismatch);
    }
}
