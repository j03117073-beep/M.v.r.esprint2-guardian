#![deny(unsafe_code)]

use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
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

#[derive(Debug)]
pub enum ParseError {
    MissingColumn(String),
    MissingValue(String),
    InvalidBoolean(String),
    InvalidNumeric(String, String),
    DuplicatePrimaryKey(String, bool, String, String),
    MalformedCsv(String),
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
}

pub fn parse_csv<R: Read>(input: R) -> Result<Vec<ScedResourceOfferRecord>, ParseError> {
    let reader = BufReader::new(input);
    let mut lines = reader.lines();

    let header_line = lines
        .next()
        .ok_or_else(|| ParseError::MalformedCsv("missing header row".to_string()))?
        .map_err(|e| ParseError::MalformedCsv(e.to_string()))?;

    let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();
    if headers.is_empty() {
        return Err(ParseError::MalformedCsv("empty header row".to_string()));
    }

    let mut records = Vec::new();
    for line_result in lines {
        let line = line_result.map_err(|e| ParseError::MalformedCsv(e.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }

        let raw_values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
        if raw_values.len() > headers.len() {
            return Err(ParseError::MalformedCsv(format!(
                "row has {} values but header has {}",
                raw_values.len(),
                headers.len()
            )));
        }

        let mut row = BTreeMap::new();
        for (idx, header) in headers.iter().enumerate() {
            let value = raw_values.get(idx).cloned().unwrap_or_default();
            row.insert(header.clone(), value);
        }

        records.push(record_from_map(&row)?);
    }

    Ok(records)
}

fn record_from_map(row: &BTreeMap<String, String>) -> Result<ScedResourceOfferRecord, ParseError> {
    let scd_timestamp = get_required_string(row, "scd_timestamp")?;
    let repeat_hour_flag = parse_bool(&get_required_string(row, "repeat_hour_flag")?)?;
    let resource_name = get_required_string(row, "resource_name")?;
    let offer_type = get_required_string(row, "offer_type")?;

    let numeric_names = numeric_field_names();
    let mut numeric_values = Vec::with_capacity(48);
    for name in numeric_names {
        numeric_values.push(get_normalized_numeric(row, &name)?);
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

fn get_required_string(row: &BTreeMap<String, String>, key: &str) -> Result<String, ParseError> {
    let value = row
        .get(key)
        .ok_or_else(|| ParseError::MissingColumn(key.to_string()))?
        .trim()
        .to_string();
    if value.is_empty() {
        return Err(ParseError::MissingValue(key.to_string()));
    }
    Ok(value)
}

fn get_normalized_numeric(row: &BTreeMap<String, String>, key: &str) -> Result<String, ParseError> {
    let raw = row
        .get(key)
        .ok_or_else(|| ParseError::MissingColumn(key.to_string()))?
        .trim()
        .to_string();
    if raw.is_empty() || raw.eq_ignore_ascii_case("null") {
        return Ok("0".to_string());
    }
    if raw.parse::<f64>().is_err() {
        return Err(ParseError::InvalidNumeric(key.to_string(), raw));
    }
    Ok(raw)
}

fn parse_bool(raw: &str) -> Result<bool, ParseError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
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

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_record(ts: &str, repeat: bool, resource: &str, offer: &str, first: &str) -> ScedResourceOfferRecord {
        let mut fields = std::array::from_fn::<_, 48, _>(|_| "0".to_string());
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
        let r = mk_record("2026-01-21T23:55:18", false, "7RNCHSLR_UNIT1", "OFFNS", "186.14");
        let serialized = r.canonical_record_string();
        assert!(serialized.starts_with("2026-01-21T23:55:18|false|7RNCHSLR_UNIT1|186.14|"));
        assert!(serialized.ends_with("|OFFNS"));
    }

    #[test]
    fn dst_fallback_uniqueness_uses_repeat_hour_flag() {
        let a = mk_record("2026-11-01T01:30:00", false, "RES_A", "OFFNS", "10");
        let b = mk_record("2026-11-01T01:30:00", true, "RES_A", "OFFNS", "10");
        let chain = build_hash_chain(vec![b, a]).expect("chain should build");
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].key.1, false);
        assert_eq!(chain[1].key.1, true);
    }

    #[test]
    fn numeric_blank_and_null_normalize_to_zero() {
        let csv = "scd_timestamp,repeat_hour_flag,resource_name,offer_type,price1_urs,price1_drs,price1_rrspf,price1_rrsuf,price1_rrsff,price1_ns,price1_ecrs,quantity_mw1,price2_urs,price2_drs,price2_rrspf,price2_rrsuf,price2_rrsff,price2_ns,price2_ecrs,quantity_mw2,price3_urs,price3_drs,price3_rrspf,price3_rrsuf,price3_rrsff,price3_ns,price3_ecrs,quantity_mw3,price4_urs,price4_drs,price4_rrspf,price4_rrsuf,price4_rrsff,price4_ns,price4_ecrs,quantity_mw4,price5_urs,price5_drs,price5_rrspf,price5_rrsuf,price5_rrsff,price5_ns,price5_ecrs,quantity_mw5,price6_urs,price6_drs,price6_rrspf,price6_rrsuf,price6_rrsff,price6_ns,price6_ecrs,quantity_mw6\n\
2026-01-21T23:55:18,false,RES,OFFNS,,null,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0\n";

        let parsed = parse_csv(csv.as_bytes()).expect("parse ok");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].prices_and_quantities[0], "0");
        assert_eq!(parsed[0].prices_and_quantities[1], "0");
    }
}
