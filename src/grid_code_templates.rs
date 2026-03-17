#![deny(unsafe_code)]

use crate::failure_axis::{FailureAxis, SystemHalt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridCodeTemplate {
    pub name: &'static str,
    pub ramp_rate_mw_per_ms: f32,
    pub droop_percent: f32,
    pub freq_deadband_hz: f32,
}

pub fn list_templates() -> Vec<GridCodeTemplate> {
    vec![
        GridCodeTemplate {
            name: "CAISO",
            ramp_rate_mw_per_ms: 0.50,
            droop_percent: 5.0,
            freq_deadband_hz: 0.036,
        },
        GridCodeTemplate {
            name: "LUMA",
            ramp_rate_mw_per_ms: 0.35,
            droop_percent: 4.0,
            freq_deadband_hz: 0.030,
        },
        GridCodeTemplate {
            name: "Hawaii-HECO",
            ramp_rate_mw_per_ms: 0.25,
            droop_percent: 3.5,
            freq_deadband_hz: 0.025,
        },
    ]
}

pub fn select_template(name: &str) -> Result<GridCodeTemplate, SystemHalt> {
    list_templates()
        .into_iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| {
            SystemHalt::with_formatted(
                FailureAxis::ExternalInjectionDetected,
                format!("Unknown grid template: {name}"),
            )
        })
}

pub fn template_to_env_file(template: GridCodeTemplate) -> String {
    format!(
        "GRID_TEMPLATE={}\nRAMP_RATE_MW_PER_MS={:.3}\nDROOP_PERCENT={:.2}\nFREQ_DEADBAND_HZ={:.3}\n",
        template.name, template.ramp_rate_mw_per_ms, template.droop_percent, template.freq_deadband_hz
    )
}
