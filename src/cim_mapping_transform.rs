use std::collections::BTreeMap;

use crate::cim_mapping_data::{MappingEntry, MAPPING_DATA};
use crate::cim_mapping_rules::{derive_migration_actions, DerModelCategory, MappingAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformContext {
    pub limit_set_is_rating: bool,
    pub der_category_hint: Option<DerModelCategory>,
}

impl Default for TransformContext {
    fn default() -> Self {
        Self {
            limit_set_is_rating: false,
            der_category_hint: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformationIssue {
    pub source_attr: String,
    pub target_attr: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformResult {
    pub target_type: String,
    pub fields: BTreeMap<String, String>,
    pub issues: Vec<TransformationIssue>,
}

pub fn transform_record(
    cim10_type: &str,
    source_fields: &BTreeMap<String, String>,
    context: &TransformContext,
) -> TransformResult {
    let candidates: Vec<&MappingEntry> = MAPPING_DATA
        .iter()
        .filter(|entry| entry.instance_type_cim10 == Some(cim10_type))
        .collect();

    let chosen_target_type = choose_target_type(&candidates, context);
    let mut fields = BTreeMap::new();
    let mut issues = Vec::new();

    for entry in candidates {
        if entry.instance_type_cim16 != Some(chosen_target_type.as_str()) {
            continue;
        }

        let source_attr = entry.attr_name_cim10.unwrap_or_default();
        let target_attr = entry.attr_name_cim16.unwrap_or_default();
        if source_attr.is_empty() || target_attr.is_empty() {
            continue;
        }

        let actions = derive_migration_actions(entry);
        if actions.contains(&MappingAction::SkipUnmappedField) {
            continue;
        }

        let mut applied = false;
        if let Some(value) = source_fields.get(source_attr) {
            fields.insert(target_attr.to_string(), value.clone());
            applied = true;
        } else if actions.contains(&MappingAction::ApplyDefaultValue) {
            if let Some(default_value) =
                parse_default_value(entry.mapping_notes.unwrap_or_default())
            {
                fields.insert(target_attr.to_string(), default_value);
                applied = true;
            }
        }

        if actions.contains(&MappingAction::ManualReview) {
            issues.push(TransformationIssue {
                source_attr: source_attr.to_string(),
                target_attr: target_attr.to_string(),
                reason: entry
                    .mapping_notes
                    .unwrap_or("Manual review required")
                    .to_string(),
            });
        }

        if actions.contains(&MappingAction::RemapTypeConditionally) && !applied {
            issues.push(TransformationIssue {
                source_attr: source_attr.to_string(),
                target_attr: target_attr.to_string(),
                reason: "Conditional remap requires additional context".to_string(),
            });
        }
    }

    TransformResult {
        target_type: chosen_target_type,
        fields,
        issues,
    }
}

fn choose_target_type(candidates: &[&MappingEntry], context: &TransformContext) -> String {
    let analog_limit_note = candidates.iter().any(|entry| {
        entry
            .mapping_notes
            .unwrap_or_default()
            .contains("AnalogLimit changed to ApparentPowerLimit")
    });

    if analog_limit_note && context.limit_set_is_rating {
        return "ApparentPowerLimit".to_string();
    }

    if let Some(first) = candidates.iter().find_map(|e| e.instance_type_cim16) {
        return first.to_string();
    }

    "UnknownTargetType".to_string()
}

fn parse_default_value(notes: &str) -> Option<String> {
    let lower = notes.to_ascii_lowercase();
    if !lower.contains("all set to") {
        return None;
    }
    let marker = "all set to";
    let idx = lower.find(marker)?;
    let raw = notes[idx + marker.len()..].trim();
    if raw.is_empty() {
        return None;
    }
    Some(raw.trim_matches('.').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transforms_simple_attribute_copy() {
        let mut source = BTreeMap::new();
        source.insert("byPass".to_string(), "true".to_string());

        let out = transform_record("AnalogValue", &source, &TransformContext::default());
        assert_eq!(out.target_type, "AnalogValue");
        assert!(out.fields.contains_key("byPass"));
    }

    #[test]
    fn chooses_apparent_power_limit_when_rating_context() {
        let mut source = BTreeMap::new();
        source.insert("value".to_string(), "12.3".to_string());

        let ctx = TransformContext {
            limit_set_is_rating: true,
            der_category_hint: None,
        };
        let out = transform_record("AnalogLimit", &source, &ctx);
        assert_eq!(out.target_type, "ApparentPowerLimit");
    }

    #[test]
    fn applies_default_override_from_notes() {
        let source = BTreeMap::new();
        let out = transform_record("Contingency", &source, &TransformContext::default());
        assert_eq!(
            out.fields.get("generationLoss").map(String::as_str),
            Some("99999")
        );
    }
}
