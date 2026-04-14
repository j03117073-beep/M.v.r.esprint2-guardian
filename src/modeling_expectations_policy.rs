// Copyright (c) 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// This file is part of the M.V.R.ESPRINT1 Sovereign Execution System,
// including TLBSS geometry, the Universal Execution Layer, the
// Deterministic IR, Rust Codegen Pipeline, SovereignBus, and the
// Cryptographic Audit Chain.
//
// No part of this file, its algorithms, structures, or designs may be
// copied, reproduced, modified, distributed, published, sublicensed,
// reverse-engineered, or used to create derivative works without the
// express written permission of OBINNA JAMES EJIOFOR.
//
// This software contains proprietary trade secrets and confidential
// intellectual property. Unauthorized use is strictly prohibited.
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ModelingExpectationsPolicy {
    pub document: PolicyDocument,
    pub submission_channels: Vec<SubmissionChannelRule>,
    pub rarf_rules: RarfRules,
    pub interim_update_rules: InterimUpdateRules,
    pub contingency_rules: ContingencyRules,
    pub relay_loadability_rules: RelayLoadabilityRules,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PolicyDocument {
    pub title: String,
    pub version: String,
    pub source_file: String,
    pub extracted_text_file: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SubmissionChannelRule {
    pub participant: String,
    pub domain: String,
    pub required_channel: String,
    pub source_excerpt: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RarfRules {
    pub failed_validation_rejected: bool,
    pub source_excerpt: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct InterimUpdateRules {
    pub accepted_reasons: Vec<String>,
    pub discretionary_acceptance: bool,
    pub source_excerpt: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ContingencyRules {
    pub contingency_component_flag_managed_by_submitter: bool,
    pub source_excerpt: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RelayLoadabilityRules {
    pub applies_to_equipment: Vec<String>,
    pub co_owned_uses_most_conservative_value: bool,
    pub not_monitored_by_relay_sentinel_value: u32,
    pub validation: RelayValidationRules,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RelayValidationRules {
    pub must_be_greater_than_static_15_min: bool,
    pub must_be_greater_than_dynamic_20f_15_min_if_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionEnvelope {
    pub participant: String,
    pub domain: String,
    pub channel: String,
    pub is_interim_update: bool,
    pub interim_reason: Option<String>,
    pub rarf_validation_passed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelayLoadabilityInput {
    pub equipment_type: String,
    pub relay_loadability_rating: f64,
    pub static_15_min_rating: f64,
    pub dynamic_20f_15_min_rating: Option<f64>,
    pub is_co_owned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyViolation {
    pub code: &'static str,
    pub message: String,
}

pub fn load_policy_file<P: AsRef<Path>>(path: P) -> Result<ModelingExpectationsPolicy> {
    let mut f = File::open(path)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    let policy: ModelingExpectationsPolicy = serde_json::from_str(&content)?;
    Ok(policy)
}

pub fn validate_submission(
    policy: &ModelingExpectationsPolicy,
    envelope: &SubmissionEnvelope,
) -> Vec<PolicyViolation> {
    let mut violations = Vec::new();

    if let Some(rule) = policy
        .submission_channels
        .iter()
        .find(|r| r.participant == envelope.participant && r.domain == envelope.domain)
    {
        if rule.required_channel != envelope.channel {
            violations.push(PolicyViolation {
                code: "CHANNEL_MISMATCH",
                message: format!(
                    "{} {} must use channel {}, got {}",
                    envelope.participant, envelope.domain, rule.required_channel, envelope.channel
                ),
            });
        }
    }

    if envelope.participant == "RE"
        && envelope.domain == "network_model_change"
        && envelope.channel == "RARF"
        && policy.rarf_rules.failed_validation_rejected
        && envelope.rarf_validation_passed == Some(false)
    {
        violations.push(PolicyViolation {
            code: "RARF_VALIDATION_FAILED",
            message: "RARF submission failed validation and must be rejected".to_string(),
        });
    }

    if envelope.is_interim_update {
        match envelope.interim_reason.as_deref() {
            Some(reason)
                if policy
                    .interim_update_rules
                    .accepted_reasons
                    .iter()
                    .any(|r| r == reason) => {}
            _ => violations.push(PolicyViolation {
                code: "INTERIM_REASON_NOT_ALLOWED",
                message: "Interim update reason not in accepted policy reasons".to_string(),
            }),
        }
    }

    violations
}

pub fn validate_relay_loadability(
    policy: &ModelingExpectationsPolicy,
    input: &RelayLoadabilityInput,
) -> Vec<PolicyViolation> {
    let mut violations = Vec::new();

    if !policy
        .relay_loadability_rules
        .applies_to_equipment
        .iter()
        .any(|eq| eq == &input.equipment_type)
    {
        return violations;
    }

    if policy
        .relay_loadability_rules
        .validation
        .must_be_greater_than_static_15_min
        && input.relay_loadability_rating <= input.static_15_min_rating
    {
        violations.push(PolicyViolation {
            code: "RELAY_NOT_GT_STATIC15",
            message: "Relay loadability must be greater than static 15-minute rating".to_string(),
        });
    }

    if policy
        .relay_loadability_rules
        .validation
        .must_be_greater_than_dynamic_20f_15_min_if_present
    {
        if let Some(dynamic_rating) = input.dynamic_20f_15_min_rating {
            if input.relay_loadability_rating <= dynamic_rating {
                violations.push(PolicyViolation {
                    code: "RELAY_NOT_GT_DYNAMIC20F",
                    message: "Relay loadability must be greater than dynamic 20F 15-minute rating"
                        .to_string(),
                });
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy_from_embedded() -> ModelingExpectationsPolicy {
        let raw = include_str!("../data/modeling_expectations_7_8_policy.json");
        serde_json::from_str(raw).expect("embedded policy json should parse")
    }

    #[test]
    fn rejects_wrong_submission_channel() {
        let policy = policy_from_embedded();
        let env = SubmissionEnvelope {
            participant: "QSE".to_string(),
            domain: "telemetry_change".to_string(),
            channel: "NOMCR".to_string(),
            is_interim_update: false,
            interim_reason: None,
            rarf_validation_passed: None,
        };
        let violations = validate_submission(&policy, &env);
        assert!(violations.iter().any(|v| v.code == "CHANNEL_MISMATCH"));
    }

    #[test]
    fn rejects_failed_rarf_validation() {
        let policy = policy_from_embedded();
        let env = SubmissionEnvelope {
            participant: "RE".to_string(),
            domain: "network_model_change".to_string(),
            channel: "RARF".to_string(),
            is_interim_update: false,
            interim_reason: None,
            rarf_validation_passed: Some(false),
        };
        let violations = validate_submission(&policy, &env);
        assert!(violations
            .iter()
            .any(|v| v.code == "RARF_VALIDATION_FAILED"));
    }

    #[test]
    fn validates_relay_thresholds() {
        let policy = policy_from_embedded();
        let input = RelayLoadabilityInput {
            equipment_type: "line".to_string(),
            relay_loadability_rating: 99.0,
            static_15_min_rating: 100.0,
            dynamic_20f_15_min_rating: Some(98.0),
            is_co_owned: false,
        };
        let violations = validate_relay_loadability(&policy, &input);
        assert!(violations.iter().any(|v| v.code == "RELAY_NOT_GT_STATIC15"));
        assert!(!violations
            .iter()
            .any(|v| v.code == "RELAY_NOT_GT_DYNAMIC20F"));
    }
}

