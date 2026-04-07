use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MoraResourceRow {
    pub report_month: String,
    pub category: String,
    pub row_number: u32,
    pub unit_name: String,
    pub inr: String,
    pub unit_code: String,
    pub county: String,
    pub fuel: String,
    pub zone: String,
    pub in_service_year: String,
    pub installed_capacity_mw: String,
    pub mora_capacity_mw: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoraSummary {
    pub report_month: String,
    pub row_count: usize,
    pub total_installed_capacity_mw: f64,
    pub total_mora_capacity_mw: f64,
    pub reserve_ratio: f64,
    pub by_fuel_mora_mw: BTreeMap<String, f64>,
    pub by_zone_mora_mw: BTreeMap<String, f64>,
    pub by_category_mora_mw: BTreeMap<String, f64>,
}

pub fn load_mora_resource_details<P: AsRef<Path>>(path: P) -> Result<Vec<MoraResourceRow>> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    load_mora_resource_details_from_str(&content)
}

pub fn load_mora_resource_details_from_str(csv_data: &str) -> Result<Vec<MoraResourceRow>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_bytes());
    let mut rows = Vec::new();
    for result in rdr.deserialize() {
        rows.push(result?);
    }
    Ok(rows)
}

pub fn summarize_mora(rows: &[MoraResourceRow]) -> MoraSummary {
    let report_month = rows
        .first()
        .map(|r| r.report_month.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let mut total_installed_capacity_mw = 0.0_f64;
    let mut total_mora_capacity_mw = 0.0_f64;
    let mut by_fuel_mora_mw = BTreeMap::new();
    let mut by_zone_mora_mw = BTreeMap::new();
    let mut by_category_mora_mw = BTreeMap::new();

    for row in rows {
        let installed = parse_mw(&row.installed_capacity_mw);
        let mora = parse_mw(&row.mora_capacity_mw);

        total_installed_capacity_mw += installed;
        total_mora_capacity_mw += mora;
        *by_fuel_mora_mw.entry(row.fuel.clone()).or_insert(0.0) += mora;
        *by_zone_mora_mw.entry(row.zone.clone()).or_insert(0.0) += mora;
        *by_category_mora_mw
            .entry(row.category.clone())
            .or_insert(0.0) += mora;
    }

    let reserve_ratio = if total_installed_capacity_mw > 0.0 {
        total_mora_capacity_mw / total_installed_capacity_mw
    } else {
        0.0
    };

    MoraSummary {
        report_month,
        row_count: rows.len(),
        total_installed_capacity_mw,
        total_mora_capacity_mw,
        reserve_ratio,
        by_fuel_mora_mw,
        by_zone_mora_mw,
        by_category_mora_mw,
    }
}

fn parse_mw(raw: &str) -> f64 {
    raw.trim().parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_summarizes_small_csv() {
        let csv_data = "report_month,category,row_number,unit_name,inr,unit_code,county,fuel,zone,in_service_year,installed_capacity_mw,mora_capacity_mw\n\
2026-06,Operational Resources (Thermal),4,UNIT_A,,CODE_A,COUNTY_A,NUCLEAR,NORTH,1990,100,95\n\
2026-06,Operational Resources (Thermal),5,UNIT_B,,CODE_B,COUNTY_B,COAL,SOUTH,1991,200,150\n";

        let rows = load_mora_resource_details_from_str(csv_data).expect("csv parse");
        assert_eq!(rows.len(), 2);

        let summary = summarize_mora(&rows);
        assert_eq!(summary.report_month, "2026-06");
        assert_eq!(summary.row_count, 2);
        assert!((summary.total_installed_capacity_mw - 300.0).abs() < 1e-9);
        assert!((summary.total_mora_capacity_mw - 245.0).abs() < 1e-9);
        assert!((summary.reserve_ratio - (245.0 / 300.0)).abs() < 1e-9);
        assert_eq!(summary.by_fuel_mora_mw.get("NUCLEAR"), Some(&95.0));
        assert_eq!(summary.by_zone_mora_mw.get("SOUTH"), Some(&150.0));
    }
}
