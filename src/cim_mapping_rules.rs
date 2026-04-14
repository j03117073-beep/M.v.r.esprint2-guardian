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
use crate::cim_mapping_data::{MappingEntry, MAPPING_DATA_WITH_NOTES};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingRuleKind {
    AttributeMoved,
    TypeConditionalRemap,
    UnitNormalization,
    DefaultValueOverride,
    EnumTranslation,
    NotMappedInSource,
    DerModelingHint,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerModelCategory {
    DgrOrDesrStyle,
    SodgStyle,
    UdgLoadReductionStyle,
    GenericDer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingAction {
    MoveAttribute,
    RemapTypeConditionally,
    SplitUnitAndMultiplier,
    ApplyDefaultValue,
    TranslateEnum,
    SkipUnmappedField,
    ManualReview,
}

pub fn classify_mapping_rule(entry: &MappingEntry) -> Option<MappingRuleKind> {
    let notes = entry.mapping_notes?;
    let n = notes.to_ascii_lowercase();

    if n.contains("attribute moved") {
        return Some(MappingRuleKind::AttributeMoved);
    }
    if n.contains("changed to") || n.contains("if associated limitset") {
        return Some(MappingRuleKind::TypeConditionalRemap);
    }
    if n.contains("unit is changed") || n.contains("multiplier") {
        return Some(MappingRuleKind::UnitNormalization);
    }
    if n.contains("all set to") {
        return Some(MappingRuleKind::DefaultValueOverride);
    }
    if n.contains("enumeration") || n.contains("enum") {
        return Some(MappingRuleKind::EnumTranslation);
    }
    if n.contains("not mapped") {
        return Some(MappingRuleKind::NotMappedInSource);
    }
    if n.contains("der") || n.contains("distributed") {
        return Some(MappingRuleKind::DerModelingHint);
    }
    Some(MappingRuleKind::Other)
}

pub fn derive_migration_actions(entry: &MappingEntry) -> Vec<MappingAction> {
    let mut actions = Vec::new();
    if let Some(kind) = classify_mapping_rule(entry) {
        match kind {
            MappingRuleKind::AttributeMoved => actions.push(MappingAction::MoveAttribute),
            MappingRuleKind::TypeConditionalRemap => {
                actions.push(MappingAction::RemapTypeConditionally)
            }
            MappingRuleKind::UnitNormalization => {
                actions.push(MappingAction::SplitUnitAndMultiplier)
            }
            MappingRuleKind::DefaultValueOverride => actions.push(MappingAction::ApplyDefaultValue),
            MappingRuleKind::EnumTranslation => actions.push(MappingAction::TranslateEnum),
            MappingRuleKind::NotMappedInSource => actions.push(MappingAction::SkipUnmappedField),
            MappingRuleKind::DerModelingHint | MappingRuleKind::Other => {
                actions.push(MappingAction::ManualReview)
            }
        }
    }

    if classify_der_model(entry).is_some() && !actions.contains(&MappingAction::ManualReview) {
        actions.push(MappingAction::ManualReview);
    }

    actions
}

pub fn classify_der_model(entry: &MappingEntry) -> Option<DerModelCategory> {
    let c10 = entry.instance_type_cim10.unwrap_or_default();
    let c16 = entry.instance_type_cim16.unwrap_or_default();
    let s10 = entry.attr_source_cim10.unwrap_or_default();
    let s16 = entry.attr_source_cim16.unwrap_or_default();
    let a10 = entry.attr_name_cim10.unwrap_or_default();

    let has_distribution_resource = c10 == "DistributionResource" || c16 == "DistributionResource";
    let has_distribution_generation =
        c10 == "DistributionGeneration" || c16 == "DistributionGeneration";
    let has_resource_meter =
        c10 == "EnergyProducerResourceMeter" || c16 == "EnergyProducerResourceMeter";
    let has_customer_load = c10 == "CustomerLoad"
        || c16 == "CustomerLoad"
        || s10 == "CustomerLoad"
        || s16 == "CustomerLoad";

    if has_distribution_resource {
        return Some(DerModelCategory::DgrOrDesrStyle);
    }
    if has_distribution_generation && has_resource_meter {
        return Some(DerModelCategory::SodgStyle);
    }
    if has_customer_load && a10.eq_ignore_ascii_case("loadReduction") {
        return Some(DerModelCategory::UdgLoadReductionStyle);
    }
    if has_distribution_generation || has_resource_meter || has_customer_load {
        return Some(DerModelCategory::GenericDer);
    }
    None
}

pub fn notes_entries_by_kind(kind: MappingRuleKind) -> Vec<&'static MappingEntry> {
    MAPPING_DATA_WITH_NOTES
        .iter()
        .filter(|entry| classify_mapping_rule(entry) == Some(kind))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cim_mapping_data::MAPPING_DATA;

    #[test]
    fn classifies_attribute_move_note() {
        let entry = MAPPING_DATA_WITH_NOTES
            .iter()
            .find(|e| {
                e.instance_type_cim10 == Some("AnalogValue")
                    && e.instance_type_cim16 == Some("Analog")
                    && e.attr_name_cim10 == Some("byPass")
            })
            .expect("expected known AnalogValue->Analog mapping");

        assert_eq!(
            classify_mapping_rule(entry),
            Some(MappingRuleKind::AttributeMoved)
        );
    }

    #[test]
    fn finds_some_unit_normalization_rules() {
        let units = notes_entries_by_kind(MappingRuleKind::UnitNormalization);
        assert!(!units.is_empty());
    }

    #[test]
    fn classifies_der_distribution_resource() {
        let der_entry = MAPPING_DATA
            .iter()
            .find(|e| e.instance_type_cim10 == Some("DistributionResource"))
            .expect("expected DistributionResource mapping");

        assert_eq!(
            classify_der_model(der_entry),
            Some(DerModelCategory::DgrOrDesrStyle)
        );
    }

    #[test]
    fn derives_actions_for_attribute_move() {
        let entry = MAPPING_DATA_WITH_NOTES
            .iter()
            .find(|e| {
                e.instance_type_cim10 == Some("AnalogValue")
                    && e.instance_type_cim16 == Some("Analog")
                    && e.attr_name_cim10 == Some("byPass")
            })
            .expect("expected known AnalogValue->Analog mapping");

        let actions = derive_migration_actions(entry);
        assert!(actions.contains(&MappingAction::MoveAttribute));
    }
}

