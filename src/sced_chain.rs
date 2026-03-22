use anyhow::{anyhow, bail, Context, Result};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScedResourceOfferRecord {
    pub scd_timestamp: String,
    pub repeat_hour_flag: bool,
    pub resource_name: String,
    pub offer_type: String,
    pub prices_and_quantities: [String; 48],
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ChainedRecord {
    pub scd_timestamp: String,
    pub repeat_hour_flag: bool,
    pub resource_name: String,
    pub offer_type: String,
    pub record_hash: String,
    pub chain_hash: String,
}

impl ScedResourceOfferRecord {
    pub fn from_csv_row(headers: &StringRecord, row: &StringRecord) -> Result<Self> {
        let scd_timestamp = required_string(headers, row, "scd_timestamp")?;
        let repeat_hour_flag = parse_bool(&required_string(headers, row, "repeat_hour_flag")?)?;
        let resource_name = required_string(headers, row, "resource_name")?;
        let offer_type = required_string(headers, row, "offer_type")?;

        let mut values = Vec::with_capacity(48);
        for block in 1..=6 {
            values.push(required_numeric(headers, row, &format!("price{}_urs", block))?);
            values.push(required_numeric(headers, row, &format!("price{}_drs", block))?);
            values.push(required_numeric(headers, row, &format!("price{}_rrspf", block))?);
            values.push(required_numeric(headers, row, &format!("price{}_rrsuf", block))?);
            values.push(required_numeric(headers, row, &format!("price{}_rrsff", block))?);
            values.push(required_numeric(headers, row, &format!("price{}_ns", block))?);
            values.push(required_numeric(headers, row, &format!("price{}_ecrs", block))?);
            values.push(required_numeric(headers, row, &format!("quantity_mw{}", block))?);
        }

        let prices_and_quantities: [String; 48] = values
            .try_into()
            .map_err(|_| anyhow!("expected 48 numeric fields from block 1..6"))?;

        Ok(Self {
            scd_timestamp,
            repeat_hour_flag,
            resource_name,
            offer_type,
            prices_and_quantities,
        })
    }

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

pub fn read_records_from_csv(path: &Path) -> Result<Vec<ScedResourceOfferRecord>> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut reader = ReaderBuilder::new().trim(csv::Trim::None).from_reader(file);
    let headers = reader
        .headers()
        .context("failed reading CSV headers")?
        .clone();

    let mut out = Vec::new();
    for (idx, row) in reader.records().enumerate() {
        let row = row.with_context(|| format!("failed reading row {}", idx + 2))?;
        let parsed = ScedResourceOfferRecord::from_csv_row(&headers, &row)
            .with_context(|| format!("failed parsing row {}", idx + 2))?;
        out.push(parsed);
    }
    Ok(out)
}

pub fn build_hash_chain(mut records: Vec<ScedResourceOfferRecord>) -> Result<Vec<ChainedRecord>> {
    records.sort_by(compare_records_for_chain);

    ensure_unique_primary_keys(&records)?;

    let mut out = Vec::with_capacity(records.len());
    let mut previous_chain_hash = String::from("0");

    for record in records {
        let record_hash = record.record_hash();
        let chain_input = format!("{}|{}", previous_chain_hash, record_hash);
        let chain_hash = sha256_hex(&chain_input);

        out.push(ChainedRecord {
            scd_timestamp: record.scd_timestamp,
            repeat_hour_flag: record.repeat_hour_flag,
            resource_name: record.resource_name,
            offer_type: record.offer_type,
            record_hash,
            chain_hash: chain_hash.clone(),
        });

        previous_chain_hash = chain_hash;
    }

    Ok(out)
}

pub fn write_chain_csv(path: &Path, records: &[ChainedRecord]) -> Result<()> {
    let file = File::create(path).with_context(|| format!("failed creating {}", path.display()))?;
    let mut writer = WriterBuilder::new().from_writer(file);

    for r in records {
        writer.serialize(r)?;
    }
    writer.flush()?;
    Ok(())
}

fn ensure_unique_primary_keys(records: &[ScedResourceOfferRecord]) -> Result<()> {
    for pair in records.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        if a.primary_key() == b.primary_key() {
            bail!(
                "duplicate primary key detected: ({}, {}, {}, {})",
                a.scd_timestamp,
                a.repeat_hour_flag,
                a.resource_name,
                a.offer_type
            );
        }
    }
    Ok(())
}

fn compare_records_for_chain(a: &ScedResourceOfferRecord, b: &ScedResourceOfferRecord) -> Ordering {
    a.scd_timestamp
        .cmp(&b.scd_timestamp)
        .then(a.repeat_hour_flag.cmp(&b.repeat_hour_flag))
        .then(a.resource_name.cmp(&b.resource_name))
        .then(a.offer_type.cmp(&b.offer_type))
}

fn required_string(headers: &StringRecord, row: &StringRecord, key: &str) -> Result<String> {
    let idx = headers
        .iter()
        .position(|h| h == key)
        .ok_or_else(|| anyhow!("missing required column '{}'", key))?;

    let value = row
        .get(idx)
        .ok_or_else(|| anyhow!("missing value for column '{}'", key))?;

    Ok(value.trim().to_string())
}

fn required_numeric(headers: &StringRecord, row: &StringRecord, key: &str) -> Result<String> {
    let raw = required_string(headers, row, key)?;
    if raw.is_empty() {
        return Ok(String::from("0"));
    }

    raw.parse::<f64>()
        .with_context(|| format!("column '{}' expects numeric value, got '{}'", key, raw))?;

    Ok(raw)
}

fn parse_bool(raw: &str) -> Result<bool> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => bail!("invalid boolean '{}'; expected true/false or 1/0", raw),
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(
        ts: &str,
        repeat: bool,
        resource: &str,
        offer: &str,
        first_price: &str,
    ) -> ScedResourceOfferRecord {
        let mut fields = std::array::from_fn::<_, 48, _>(|_| "0".to_string());
        fields[0] = first_price.to_string();
        ScedResourceOfferRecord {
            scd_timestamp: ts.to_string(),
            repeat_hour_flag: repeat,
            resource_name: resource.to_string(),
            offer_type: offer.to_string(),
            prices_and_quantities: fields,
        }
    }

    #[test]
    fn deterministic_sort_and_chain() {
        let a = mk("2026-11-01T01:30:00", true, "RES1", "OFFNS", "10");
        let b = mk("2026-11-01T01:30:00", false, "RES1", "OFFNS", "20");

        let chain = build_hash_chain(vec![a.clone(), b.clone()]).expect("chain should build");

        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].repeat_hour_flag, false);
        assert_eq!(chain[1].repeat_hour_flag, true);

        let rerun = build_hash_chain(vec![a, b]).expect("chain should build");
        assert_eq!(chain, rerun);
    }

    #[test]
    fn duplicate_primary_key_is_rejected() {
        let a = mk("2026-01-21T23:55:18", false, "7RNCHSLR_UNIT1", "OFFNS", "1");
        let b = mk("2026-01-21T23:55:18", false, "7RNCHSLR_UNIT1", "OFFNS", "2");
        let err = build_hash_chain(vec![a, b]).expect_err("must reject duplicate key");
        assert!(err.to_string().contains("duplicate primary key"));
    }
}
