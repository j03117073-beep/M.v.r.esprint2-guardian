use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct HrlyResOutCapRow {
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "HourEnding")]
    pub hour_ending: String,
    #[serde(rename = "TotalResourceMWZoneSouth")]
    pub total_resource_mw_zone_south: f64,
    #[serde(rename = "TotalResourceMWZoneNorth")]
    pub total_resource_mw_zone_north: f64,
    #[serde(rename = "TotalResourceMWZoneWest")]
    pub total_resource_mw_zone_west: f64,
    #[serde(rename = "TotalResourceMWZoneHouston")]
    pub total_resource_mw_zone_houston: f64,
    #[serde(rename = "TotalIRRMWZoneSouth")]
    pub total_irr_mw_zone_south: f64,
    #[serde(rename = "TotalIRRMWZoneNorth")]
    pub total_irr_mw_zone_north: f64,
    #[serde(rename = "TotalIRRMWZoneWest")]
    pub total_irr_mw_zone_west: f64,
    #[serde(rename = "TotalIRRMWZoneHouston")]
    pub total_irr_mw_zone_houston: f64,
    #[serde(rename = "TotalNewEquipResourceMWZoneSouth")]
    pub total_new_equip_resource_mw_zone_south: f64,
    #[serde(rename = "TotalNewEquipResourceMWZoneNorth")]
    pub total_new_equip_resource_mw_zone_north: f64,
    #[serde(rename = "TotalNewEquipResourceMWZoneWest")]
    pub total_new_equip_resource_mw_zone_west: f64,
    #[serde(rename = "TotalNewEquipResourceMWZoneHouston")]
    pub total_new_equip_resource_mw_zone_houston: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HrlyResOutCapSummary {
    pub row_count: usize,
    pub start_date: String,
    pub end_date: String,
    pub hourly_total_peak_mw: f64,
    pub hourly_total_peak_at: String,
    pub zone_resource_totals_mwh: BTreeMap<String, f64>,
    pub zone_irr_totals_mwh: BTreeMap<String, f64>,
    pub zone_new_equipment_totals_mwh: BTreeMap<String, f64>,
}

pub fn load_hrly_res_out_cap_csv<P: AsRef<Path>>(path: P) -> Result<Vec<HrlyResOutCapRow>> {
    let mut file = File::open(path)?;
    let mut csv_data = String::new();
    file.read_to_string(&mut csv_data)?;
    load_hrly_res_out_cap_csv_from_str(&csv_data)
}

pub fn load_hrly_res_out_cap_csv_from_str(csv_data: &str) -> Result<Vec<HrlyResOutCapRow>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_bytes());

    let mut rows = Vec::new();
    for rec in rdr.deserialize() {
        rows.push(rec?);
    }
    Ok(rows)
}

pub fn summarize_hrly_res_out_cap(rows: &[HrlyResOutCapRow]) -> HrlyResOutCapSummary {
    let start_date = rows
        .first()
        .map(|r| r.date.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let end_date = rows
        .last()
        .map(|r| r.date.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let mut zone_resource_totals_mwh = BTreeMap::from([
        ("South".to_string(), 0.0),
        ("North".to_string(), 0.0),
        ("West".to_string(), 0.0),
        ("Houston".to_string(), 0.0),
    ]);
    let mut zone_irr_totals_mwh = zone_resource_totals_mwh.clone();
    let mut zone_new_equipment_totals_mwh = zone_resource_totals_mwh.clone();

    let mut hourly_total_peak_mw = 0.0_f64;
    let mut hourly_total_peak_at = "n/a".to_string();

    for row in rows {
        let resource_hourly = row.total_resource_mw_zone_south
            + row.total_resource_mw_zone_north
            + row.total_resource_mw_zone_west
            + row.total_resource_mw_zone_houston;
        let irr_hourly = row.total_irr_mw_zone_south
            + row.total_irr_mw_zone_north
            + row.total_irr_mw_zone_west
            + row.total_irr_mw_zone_houston;
        let new_equipment_hourly = row.total_new_equip_resource_mw_zone_south
            + row.total_new_equip_resource_mw_zone_north
            + row.total_new_equip_resource_mw_zone_west
            + row.total_new_equip_resource_mw_zone_houston;
        let total_hourly = resource_hourly + irr_hourly + new_equipment_hourly;

        if total_hourly > hourly_total_peak_mw {
            hourly_total_peak_mw = total_hourly;
            hourly_total_peak_at = format!("{} HE{}", row.date, row.hour_ending);
        }

        *zone_resource_totals_mwh
            .entry("South".to_string())
            .or_insert(0.0) += row.total_resource_mw_zone_south;
        *zone_resource_totals_mwh
            .entry("North".to_string())
            .or_insert(0.0) += row.total_resource_mw_zone_north;
        *zone_resource_totals_mwh
            .entry("West".to_string())
            .or_insert(0.0) += row.total_resource_mw_zone_west;
        *zone_resource_totals_mwh
            .entry("Houston".to_string())
            .or_insert(0.0) += row.total_resource_mw_zone_houston;

        *zone_irr_totals_mwh.entry("South".to_string()).or_insert(0.0) += row.total_irr_mw_zone_south;
        *zone_irr_totals_mwh.entry("North".to_string()).or_insert(0.0) += row.total_irr_mw_zone_north;
        *zone_irr_totals_mwh.entry("West".to_string()).or_insert(0.0) += row.total_irr_mw_zone_west;
        *zone_irr_totals_mwh
            .entry("Houston".to_string())
            .or_insert(0.0) += row.total_irr_mw_zone_houston;

        *zone_new_equipment_totals_mwh
            .entry("South".to_string())
            .or_insert(0.0) += row.total_new_equip_resource_mw_zone_south;
        *zone_new_equipment_totals_mwh
            .entry("North".to_string())
            .or_insert(0.0) += row.total_new_equip_resource_mw_zone_north;
        *zone_new_equipment_totals_mwh
            .entry("West".to_string())
            .or_insert(0.0) += row.total_new_equip_resource_mw_zone_west;
        *zone_new_equipment_totals_mwh
            .entry("Houston".to_string())
            .or_insert(0.0) += row.total_new_equip_resource_mw_zone_houston;
    }

    HrlyResOutCapSummary {
        row_count: rows.len(),
        start_date,
        end_date,
        hourly_total_peak_mw,
        hourly_total_peak_at,
        zone_resource_totals_mwh,
        zone_irr_totals_mwh,
        zone_new_equipment_totals_mwh,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_summarizes_sample() {
        let sample = "Date,HourEnding,TotalResourceMWZoneSouth,TotalResourceMWZoneNorth,TotalResourceMWZoneWest,TotalResourceMWZoneHouston,TotalIRRMWZoneSouth,TotalIRRMWZoneNorth,TotalIRRMWZoneWest,TotalIRRMWZoneHouston,TotalNewEquipResourceMWZoneSouth,TotalNewEquipResourceMWZoneNorth,TotalNewEquipResourceMWZoneWest,TotalNewEquipResourceMWZoneHouston\n\
04/05/2026,01,10,20,30,40,1,2,3,4,5,6,7,8\n\
04/05/2026,02,11,21,31,41,1,2,3,4,5,6,7,8\n";
        let rows = load_hrly_res_out_cap_csv_from_str(sample).expect("parse");
        assert_eq!(rows.len(), 2);
        let summary = summarize_hrly_res_out_cap(&rows);
        assert_eq!(summary.row_count, 2);
        assert_eq!(summary.start_date, "04/05/2026");
        assert_eq!(summary.end_date, "04/05/2026");
        assert!((summary.hourly_total_peak_mw - 140.0).abs() < 1e-9);
        assert_eq!(summary.hourly_total_peak_at, "04/05/2026 HE02");
        assert_eq!(summary.zone_resource_totals_mwh.get("South"), Some(&21.0));
        assert_eq!(summary.zone_irr_totals_mwh.get("Houston"), Some(&8.0));
        assert_eq!(
            summary.zone_new_equipment_totals_mwh.get("West"),
            Some(&14.0)
        );
    }
}
